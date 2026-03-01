use actix_web::{delete, get, patch, post, web, HttpResponse, Result as WebResult};
use uuid::Uuid;
use std::sync::Arc;

use crate::api::models::{CreateMessageRequest, CreateSessionRequest, UpdateSessionRequest, PaginationQuery};
use crate::db::{service::DbService, DbPool};
use crate::llm::{LlmProvider, models::{Message as LlmMessage, ChatOptions}};

// --- Sessions ---

#[post("")]
pub async fn create_session(
    pool: web::Data<DbPool>,
    req: web::Json<CreateSessionRequest>,
) -> WebResult<HttpResponse> {
    let conn = pool.lock().unwrap();
    let req = req.into_inner();
    
    match DbService::insert_session(&conn, &req.name, req.metadata) {
        Ok(session) => Ok(HttpResponse::Created().json(session)),
        Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
    }
}

#[get("")]
pub async fn list_sessions(
    pool: web::Data<DbPool>,
    query: web::Query<PaginationQuery>,
) -> WebResult<HttpResponse> {
    let conn = pool.lock().unwrap();
    
    match DbService::list_sessions(&conn, query.limit, query.offset) {
        Ok(sessions) => Ok(HttpResponse::Ok().json(sessions)),
        Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
    }
}

#[get("/{id}")]
pub async fn get_session(
    pool: web::Data<DbPool>,
    id: web::Path<Uuid>,
) -> WebResult<HttpResponse> {
    let conn = pool.lock().unwrap();
    
    match DbService::get_session(&conn, id.into_inner()) {
        Ok(Some(session)) => Ok(HttpResponse::Ok().json(session)),
        Ok(None) => Ok(HttpResponse::NotFound().finish()),
        Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
    }
}

#[delete("/{id}")]
pub async fn delete_session(
    pool: web::Data<DbPool>,
    id: web::Path<Uuid>,
) -> WebResult<HttpResponse> {
    let conn = pool.lock().unwrap();
    match DbService::delete_session(&conn, id.into_inner()) {
        Ok(_) => Ok(HttpResponse::NoContent().finish()),
        Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
    }
}

#[patch("/{id}")]
pub async fn update_session(
    pool: web::Data<DbPool>,
    id: web::Path<Uuid>,
    req: web::Json<UpdateSessionRequest>,
) -> WebResult<HttpResponse> {
    let conn = pool.lock().unwrap();
    match DbService::update_session(&conn, id.into_inner(), req.name.clone(), req.metadata.clone()) {
        Ok(Some(session)) => Ok(HttpResponse::Ok().json(session)),
        Ok(None) => Ok(HttpResponse::NotFound().finish()),
        Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
    }
}

// --- Messages ---

#[post("/{id}/messages")]
pub async fn add_message(
    pool: web::Data<DbPool>,
    llm: web::Data<Arc<dyn LlmProvider>>,
    config: web::Data<crate::config::AppConfig>,
    id: web::Path<Uuid>,
    req: web::Json<CreateMessageRequest>,
) -> WebResult<HttpResponse> {
    let conn = pool.lock().unwrap();
    let id = id.into_inner();
    let req = req.into_inner();
    
    // Check if session exists first
    if DbService::get_session(&conn, id).unwrap_or(None).is_none() {
        return Ok(HttpResponse::NotFound().body("Session not found"));
    }

    let user_msg = match DbService::insert_message(
        &conn, 
        id, 
        &req.role, 
        &req.content, 
        req.model.as_deref(), 
        req.token_count, 
        req.metadata.clone()
    ) {
        Ok(message) => message,
        Err(e) => return Ok(HttpResponse::InternalServerError().body(e.to_string())),
    };

    // If it's not a user message, we don't trigger the LLM completion
    if req.role != "user" {
        return Ok(HttpResponse::Created().json(user_msg));
    }

    // Fetch history for LLM Context
    let history = match DbService::get_messages(&conn, id, 50, 0) {
        Ok(msgs) => msgs,
        Err(e) => return Ok(HttpResponse::InternalServerError().body(e.to_string())),
    };

    let mut llm_messages: Vec<LlmMessage> = history.into_iter().map(|m| {
        let tool_calls = m.metadata.get("tool_calls").and_then(|tc| serde_json::from_value(tc.clone()).ok());
        let tool_call_id = m.metadata.get("tool_call_id").and_then(|tid| tid.as_str().map(|s| s.to_string()));
        LlmMessage {
            role: m.role,
            content: m.content,
            tool_calls,
            tool_call_id,
        }
    }).collect();

    // Drop the DuckDB connection lock
    drop(conn);

    let tools = crate::tools::ToolRegistry::new();
    let current_date = chrono::Local::now().format("%A, %B %d, %Y").to_string();
    let system_prompt = config.chat.system_prompt.replace("{current_date}", &current_date);
    let grounded_prompt = format!("Current Date: {}.\n\n{}", current_date, system_prompt);
    
    let current_options = ChatOptions {
        model: req.model,
        system_prompt: Some(grounded_prompt),
        tools: Some(tools.get_definitions()),
        ..Default::default()
    };

    let mut loop_count = 0;
    let max_loops = 5;

    while loop_count < max_loops {
        let response = match llm.chat(&llm_messages, current_options.clone()).await {
            Ok(res) => res,
            Err(e) => return Ok(HttpResponse::InternalServerError().body(format!("LLM Error: {}", e))),
        };

        // If no tool calls, we're done
        if response.tool_calls.as_ref().map(|tc| tc.is_empty()).unwrap_or(true) {
            // Re-lock the DB pool to insert the assistant's context
            let conn = pool.lock().unwrap();
            let token_count = response.usage.as_ref().map(|u| u.input_tokens + u.output_tokens).map(|t| t as i32);

            return match DbService::insert_message(
                &conn,
                id,
                "assistant",
                &response.content,
                Some(&response.model),
                token_count,
                serde_json::json!({}),
            ) {
                Ok(assistant_msg) => Ok(HttpResponse::Created().json(assistant_msg)),
                Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
            };
        }

        // Handle tool calls
        let mut assistant_tool_calls = Vec::new();
        let tool_calls = response.tool_calls.unwrap();
        
        for tool_call in &tool_calls {
            assistant_tool_calls.push(tool_call.clone());
        }

        // 1. Add assistant message with tool calls to history
        llm_messages.push(LlmMessage {
            role: "assistant".to_string(),
            content: response.content.clone(),
            tool_calls: Some(assistant_tool_calls.clone()),
            tool_call_id: None,
        });

        // 2. Persist assistant message (optional but good for history)
        {
            let conn = pool.lock().unwrap();
            let _ = DbService::insert_message(
                &conn,
                id,
                "assistant",
                &response.content,
                Some(&response.model),
                None, // We'll count tokens at the end or skip here
                serde_json::json!({ "tool_calls": assistant_tool_calls }),
            );
        }

        // 3. Execute tools
        for tool_call in tool_calls {
            let tool_id = tool_call.id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
            let result = tools.call_tool(&tool_call.function.name, &tool_call.function.arguments).await;
            
            llm_messages.push(LlmMessage {
                role: "tool".to_string(),
                content: result.clone(),
                tool_calls: None,
                tool_call_id: Some(tool_id.clone()),
            });

            // Re-lock the DB pool to insert the tool result
            {
                let conn = pool.lock().unwrap();
                let _ = DbService::insert_message(
                    &conn,
                    id,
                    "tool",
                    &result,
                    None,
                    None,
                    serde_json::json!({ "tool_call_id": tool_id }),
                );
            }
        }

        loop_count += 1;
    }

    Ok(HttpResponse::InternalServerError().body("Max tool call loops reached"))
}

#[get("/{id}/messages")]
pub async fn get_messages(
    pool: web::Data<DbPool>,
    id: web::Path<Uuid>,
    query: web::Query<PaginationQuery>,
) -> WebResult<HttpResponse> {
    let conn = pool.lock().unwrap();
    
    match DbService::get_messages(&conn, id.into_inner(), query.limit, query.offset) {
        Ok(messages) => Ok(HttpResponse::Ok().json(messages)),
        Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
    }
}

#[get("/{id}/export")]
pub async fn export_session(
    pool: web::Data<DbPool>,
    id: web::Path<Uuid>,
) -> WebResult<HttpResponse> {
    let conn = pool.lock().unwrap();
    let id = id.into_inner();

    let session = match DbService::get_session(&conn, id) {
        Ok(Some(s)) => s,
        Ok(None) => return Ok(HttpResponse::NotFound().finish()),
        Err(e) => return Ok(HttpResponse::InternalServerError().body(e.to_string())),
    };

    let messages = DbService::get_messages(&conn, id, 1000, 0).unwrap_or_default();
    
    let mut export = String::new();
    export.push_str(&format!("Session: {}\n", session.name));
    export.push_str(&format!("ID: {}\n", session.id));
    export.push_str(&format!("Created At: {}\n", session.created_at));
    export.push_str("---\n");

    for m in messages {
        export.push_str(&format!("[{}]: {}\n", m.role.to_uppercase(), m.content));
        export.push_str("---\n");
    }

    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .insert_header(("Content-Disposition", format!("attachment; filename=\"session_{}.txt\"", id)))
        .body(export))
}

#[post("/import")]
pub async fn import_session(
    pool: web::Data<DbPool>,
    body: String,
) -> WebResult<HttpResponse> {
    let conn = pool.lock().unwrap();
    let mut lines = body.lines();
    
    let name = lines.next()
        .and_then(|l| l.strip_prefix("Session: "))
        .unwrap_or("Imported Session");
        
    match DbService::insert_session(&conn, name, serde_json::json!({})) {
        Ok(session) => {
            let mut current_role = String::new();
            let mut current_content = String::new();
            
            for line in lines {
                if line == "---" {
                    if !current_role.is_empty() && !current_content.is_empty() {
                        let _ = DbService::insert_message(
                            &conn, session.id, &current_role.to_lowercase(), 
                            &current_content.trim(), None, None, serde_json::json!({})
                        );
                        current_content.clear();
                    }
                } else if line.starts_with("[") && line.contains("]: ") {
                    if let (Some(start), Some(end)) = (line.find('['), line.find(']')) {
                        current_role = line[start+1..end].to_string();
                        current_content = line[end+2..].to_string();
                    }
                } else {
                    current_content.push_str("\n");
                    current_content.push_str(line);
                }
            }
            Ok(HttpResponse::Created().json(session))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
    }
}

#[get("/stats")]
pub async fn get_stats(
    pool: web::Data<DbPool>,
    config: web::Data<crate::config::AppConfig>,
) -> WebResult<HttpResponse> {
    let conn = pool.lock().unwrap();
    match DbService::get_stats(&conn, &config.database.path) {
        Ok(stats) => Ok(HttpResponse::Ok().json(stats)),
        Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/sessions")
            .service(create_session)
            .service(list_sessions)
            .service(get_stats)
            .service(get_session)
            .service(update_session)
            .service(delete_session)
            .service(add_message)
            .service(get_messages)
            .service(export_session)
            .service(import_session)
    );
}
