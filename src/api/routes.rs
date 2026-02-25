use actix_web::{delete, get, post, web, HttpResponse, Result as WebResult};
use uuid::Uuid;
use std::sync::Arc;

use crate::api::models::{CreateMessageRequest, CreateSessionRequest, PaginationQuery};
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
    let id = id.into_inner();
    
    // Check if exists first for better 404 handling
    if DbService::get_session(&conn, id).unwrap_or(None).is_none() {
        return Ok(HttpResponse::NotFound().finish());
    }

    match DbService::delete_session(&conn, id) {
        Ok(_) => Ok(HttpResponse::NoContent().finish()),
        Err(e) => Ok(HttpResponse::InternalServerError().body(e.to_string())),
    }
}

// --- Messages ---

#[post("/{id}/messages")]
pub async fn add_message(
    pool: web::Data<DbPool>,
    llm: web::Data<Arc<dyn LlmProvider>>,
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

    let llm_messages: Vec<LlmMessage> = history.into_iter().map(|m| LlmMessage {
        role: m.role,
        content: m.content,
    }).collect();

    // Drop the DuckDB connection lock so we don't block other threads during the slow LLM network boundary
    drop(conn);

    let chat_options = ChatOptions {
        model: req.model,
        ..Default::default()
    };

    let response = match llm.chat(&llm_messages, chat_options).await {
        Ok(res) => res,
        Err(e) => return Ok(HttpResponse::InternalServerError().body(format!("LLM Error: {}", e))),
    };

    // Re-lock the DB pool to insert the assistant's context
    let conn = pool.lock().unwrap();
    let token_count = response.usage.map(|u| u.input_tokens + u.output_tokens).map(|t| t as i32);

    match DbService::insert_message(
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
    }
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

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/sessions")
            .service(create_session)
            .service(list_sessions)
            .service(get_session)
            .service(delete_session)
            .service(add_message)
            .service(get_messages)
            .service(export_session)
            .service(import_session)
    );
}
