use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use actix_ws::Message;
use futures_util::StreamExt as _;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

use crate::api::models_ws::{WsClientMessage, WsServerMessage};
use crate::db::{service::DbService, DbPool};
use crate::llm::{
    models::{ChatOptions, Message as LlmMessage},
    LlmProvider,
};

#[get("/ws/chat/{session_id}")]
pub async fn ws_chat(
    req: HttpRequest,
    body: web::Payload,
    pool: web::Data<DbPool>,
    llm: web::Data<Arc<dyn LlmProvider>>,
    config: web::Data<crate::config::AppConfig>,
    session_id: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;
    let id = session_id.into_inner();

    // Verify session exists before accepting connection
    {
        let conn = pool.lock().unwrap();
        if DbService::get_session(&conn, id)
            .unwrap_or(None)
            .is_none()
        {
            return Ok(HttpResponse::NotFound().body("Session not found"));
        }
    }

    info!("WebSocket connection established for session {:?}", id);

    // web::Data<T> is effectively Arc<T>.
    let llm_arc = llm.get_ref().clone();    // Arc<dyn LlmProvider>
    let pool_arc = pool.get_ref().clone();  // Arc<Mutex<Connection>>
    
    // config is web::Data<AppConfig>, which is Arc<AppConfig>
    // We can get the inner Arc by cloning the Data and calling into_inner()
    let config_arc = config.clone().into_inner(); 

    actix_web::rt::spawn(async move {
        let mut active_task: Option<actix_web::rt::task::JoinHandle<()>> = None;

        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Ping(bytes) => {
                    if session.pong(&bytes).await.is_err() {
                        return;
                    }
                }
                Message::Text(text) => {
                    info!("Received WebSocket message: {}", text);
                    let client_msg: Result<WsClientMessage, _> = serde_json::from_str(&text);
                    if let Ok(msg) = client_msg {
                        match msg.r#type.as_str() {
                            "message" => {
                                // If there's an active task, abort it before starting a new one
                                if let Some(handle) = active_task.take() {
                                    handle.abort();
                                }

                                let mut session_clone = session.clone();
                                let pool_clone = pool_arc.clone();
                                let llm_clone = llm_arc.clone();
                                let config_clone = config_arc.clone();
                                let content = msg.content;
                                let search = msg.search.unwrap_or(false);
                                let reason = msg.reason.unwrap_or(false);

                                active_task = Some(actix_web::rt::spawn(async move {
                                    handle_chat_message(
                                        content,
                                        search,
                                        reason,
                                        id,
                                        pool_clone,
                                        llm_clone,
                                        config_clone,
                                        &mut session_clone,
                                    )
                                    .await;
                                }));
                            }
                            "cancel" => {
                                if let Some(handle) = active_task.take() {
                                    handle.abort();
                                    info!("Chat task for session {:?} cancelled by user", id);
                                    
                                    let status_msg = WsServerMessage {
                                        r#type: "status".to_string(),
                                        content: "Process cancelled".to_string(),
                                    };
                                    let _ = session.text(serde_json::to_string(&status_msg).unwrap()).await;
                                    
                                    let done_msg = WsServerMessage {
                                        r#type: "done".to_string(),
                                        content: "".to_string(),
                                    };
                                    let _ = session.text(serde_json::to_string(&done_msg).unwrap()).await;
                                }
                            }
                            _ => {
                                error!("Unknown message type: {}", msg.r#type);
                            }
                        }
                    }
                }
                Message::Close(reason) => {
                    if let Some(handle) = active_task.take() {
                        handle.abort();
                    }
                    let _ = session.close(reason).await;
                    break;
                }
                _ => {}
            }
        }
        info!("WebSocket connection closed for session {:?}", id);
    });

    Ok(response)
}

async fn handle_chat_message(
    content: String,
    search: bool,
    reason: bool,
    session_id: Uuid,
    pool: DbPool,
    llm: Arc<dyn LlmProvider>,
    config: Arc<crate::config::AppConfig>,
    session: &mut actix_ws::Session,
) {
    info!("Starting handle_chat_message for session {:?}", session_id);
    // 1. Save user message to database
    {
        let conn = pool.lock().unwrap();
        if let Err(e) = DbService::insert_message(
            &conn,
            session_id,
            "user",
            &content,
            None,
            None,
            serde_json::json!({}),
        ) {
            error!("Failed to insert user message: {}", e);
            drop(conn); // Drop lock before await
            let err_resp = WsServerMessage {
                r#type: "error".to_string(),
                content: "Database error".to_string(),
            };
            let _ = session
                .text(serde_json::to_string(&err_resp).unwrap())
                .await;
            return;
        }
    }

    // 2. Fetch History & Session Metadata
    let (history, session_db) = {
        let conn = pool.lock().unwrap();
        let history = DbService::get_messages(&conn, session_id, 50, 0).unwrap_or_default();
        let session_db = DbService::get_session(&conn, session_id).unwrap_or(None);
        (history, session_db)
    };
    info!("Fetched history and session metadata for session {:?}", session_id);

    let mut system_prompt = config.chat.system_prompt.clone();
    if let Some(s) = session_db {
        if let Some(prompt) = s.metadata.get("system_prompt").and_then(|v| v.as_str()) {
            system_prompt = prompt.to_string();
        }
    }

    let mut llm_messages: Vec<LlmMessage> = history
        .into_iter()
        .map(|m| {
            let tool_calls = m.metadata.get("tool_calls").and_then(|tc| serde_json::from_value(tc.clone()).ok());
            let tool_call_id = m.metadata.get("tool_call_id").and_then(|tid| tid.as_str().map(|s| s.to_string()));
            LlmMessage {
                role: m.role,
                content: m.content,
                tool_calls,
                tool_call_id,
            }
        })
        .collect();

    let tools = crate::tools::ToolRegistry::new();
    let current_date = chrono::Local::now().format("%A, %B %d, %Y").to_string();
    let grounded_system_prompt = system_prompt.replace("{current_date}", &current_date);
    let mut final_prompt = format!("Current Date: {}.\n\n{}", current_date, grounded_system_prompt);

    if reason {
        final_prompt.push_str("\n\nIMPORTANT: Please reason step-by-step before providing your final answer. Externalize your internal monologue if possible.");
    }

    let mut tool_definitions = tools.get_definitions();
    if !search {
        // Filter out search tools if search is disabled
        tool_definitions.retain(|t| {
            t.function.name != "internet_search" && t.function.name != "read_full_content"
        });
    }

    let current_options = ChatOptions {
        system_prompt: Some(final_prompt),
        tools: Some(tool_definitions),
        ..Default::default()
    };

    info!("Starting chat loop for session {:?}", session_id);
    let mut loop_count = 0;
    let max_loops = 5;

    while loop_count < max_loops {
        // Since we want tool support and it's easier to manage in non-streaming for the loop,
        // we use chat() here but we can send updates to the client.
        info!("Calling LLM chat for session {:?}", session_id);
        let response = match llm.chat(&llm_messages, current_options.clone()).await {
            Ok(res) => res,
            Err(e) => {
                error!("LLM Error: {:?}", e);
                return;
            }
        };
        info!("LLM responded for session {:?}", session_id);

        if response.tool_calls.as_ref().map(|tc| tc.is_empty()).unwrap_or(true) {
            // Final response - stream it to client (we can simulate streaming or just send it)
            // For better UX, we'll send it as chunks if it's large, but here we'll just send it.
            let resp = WsServerMessage {
                r#type: "chunk".to_string(),
                content: response.content.clone(),
            };
            let _ = session.text(serde_json::to_string(&resp).unwrap()).await;

            // Save final assistant message to database
            let token_count = response.usage.map(|u| u.input_tokens + u.output_tokens).map(|t| t as i32);
            let conn = pool.lock().unwrap();
            let _ = DbService::insert_message(
                &conn,
                session_id,
                "assistant",
                &response.content,
                Some(llm.name()),
                token_count,
                serde_json::json!({}),
            );
            break;
        }

        // Handle tool calls
        let mut assistant_tool_calls = Vec::new();
        let tool_calls = response.tool_calls.unwrap();
        
        for tool_call in &tool_calls {
            let mut tc = tool_call.clone();
            if tc.id.is_none() {
                tc.id = Some(uuid::Uuid::new_v4().to_string());
            }
            assistant_tool_calls.push(tc);
            
            // Notify client about tool usage
            let status_msg = WsServerMessage {
                r#type: "status".to_string(),
                content: format!("Searching: {}...", tool_call.function.name),
            };
            let _ = session.text(serde_json::to_string(&status_msg).unwrap()).await;
        }

        // Add assistant message with tool calls to history
        llm_messages.push(LlmMessage {
            role: "assistant".to_string(),
            content: response.content.clone(),
            tool_calls: Some(assistant_tool_calls.clone()),
            tool_call_id: None,
        });

        // Persist assistant message
        {
            let conn = pool.lock().unwrap();
            let _ = crate::db::service::DbService::insert_message(
                &conn,
                session_id,
                "assistant",
                &response.content,
                Some(llm.name()),
                None,
                serde_json::json!({ "tool_calls": assistant_tool_calls }),
            );
        }

        // Execute tools
        for tool_call in assistant_tool_calls {
            let tool_id = tool_call.id.unwrap(); // Guaranteed to exist now
            info!("Executing tool: {} with args: {} for session {:?}", tool_call.function.name, tool_call.function.arguments, session_id);
            let result = tools.call_tool(&tool_call.function.name, &tool_call.function.arguments, session_id, pool.clone()).await;
            info!("Tool {} execution completed for session {:?}", tool_call.function.name, session_id);
            
            llm_messages.push(LlmMessage {
                role: "tool".to_string(),
                content: result.clone(),
                tool_calls: None,
                tool_call_id: Some(tool_id.clone()),
            });

            // Persist tool result
            {
                let conn = pool.lock().unwrap();
                let _ = crate::db::service::DbService::insert_message(
                    &conn,
                    session_id,
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

    // 5. Send 'done' message
    let done_msg = WsServerMessage {
        r#type: "done".to_string(),
        content: "".to_string(),
    };
    let _ = session.text(serde_json::to_string(&done_msg).unwrap()).await;
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(ws_chat);
}
