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
        if DbService::get_session(&conn, id).unwrap_or(None).is_none() {
            return Ok(HttpResponse::NotFound().body("Session not found"));
        }
    }

    info!("WebSocket connection established for session {:?}", id);

    // web::Data<T> is effectively Arc<T>.
    let llm_arc = llm.get_ref().clone(); // Arc<dyn LlmProvider>
    let pool_arc = pool.get_ref().clone(); // Arc<Mutex<Connection>>

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
                                let mut session_clone_err = session.clone();
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
                                        &mut session_clone_err,
                                    )
                                    .await;
                                }));
                            }
                            "cancel" => {
                                info!("Received cancel request for session {:?}", id);
                                if let Some(handle) = active_task.take() {
                                    let llm_cancel = llm_arc.clone();
                                    let sid_cancel = id.to_string();
                                    tokio::spawn(async move {
                                        let _ = llm_cancel.cancel(&sid_cancel).await;
                                    });
                                    handle.abort();
                                    info!("Chat task for session {:?} aborted", id);

                                    let status_msg = WsServerMessage {
                                        r#type: "status".to_string(),
                                        content: "Process cancelled".to_string(),
                                    };
                                    let _ = session
                                        .text(serde_json::to_string(&status_msg).unwrap())
                                        .await;

                                    let done_msg = WsServerMessage {
                                        r#type: "done".to_string(),
                                        content: "".to_string(),
                                    };
                                    let _ = session
                                        .text(serde_json::to_string(&done_msg).unwrap())
                                        .await;
                                } else {
                                    info!("No active task to cancel for session {:?}", id);
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
                        let llm_cancel = llm.clone();
                        let sid_cancel = id.to_string();
                        tokio::spawn(async move {
                            let _ = llm_cancel.cancel(&sid_cancel).await;
                        });
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
    session_err: &mut actix_ws::Session,
) {
    info!("Starting handle_chat_message for session {:?}", session_id);

    // 0. Send initial status
    let start_status = WsServerMessage {
        r#type: "status".to_string(),
        content: "Thinking...".to_string(),
    };
    let _ = session
        .text(serde_json::to_string(&start_status).unwrap())
        .await;
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
    info!(
        "Fetched history and session metadata for session {:?}",
        session_id
    );

    let mut system_prompt = config.chat.system_prompt.clone();
    if let Some(s) = session_db {
        if let Some(prompt) = s.metadata.get("system_prompt").and_then(|v| v.as_str()) {
            system_prompt = prompt.to_string();
        }
    }

    let mut llm_messages: Vec<LlmMessage> = history
        .into_iter()
        .map(|m| {
            let tool_calls = m
                .metadata
                .get("tool_calls")
                .and_then(|tc| serde_json::from_value(tc.clone()).ok());
            let tool_call_id = m
                .metadata
                .get("tool_call_id")
                .and_then(|tid| tid.as_str().map(|s| s.to_string()));
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
    let mut final_prompt = format!(
        "Current Date: {}.\n\n{}",
        current_date, grounded_system_prompt
    );

    if reason {
        final_prompt.push_str("\n\nIMPORTANT: Please reason step-by-step before providing your final answer. Externalize your internal monologue if possible.");
    }

    let mut tool_definitions = tools.get_definitions();
    if !search {
        // Filter out search tools if search is disabled
        tool_definitions.retain(|t| {
            t.function.name != "internet_search" 
            && t.function.name != "read_full_content" 
            && t.function.name != "read_url"
        });
    }

    let current_options = ChatOptions {
        system_prompt: Some(final_prompt),
        tools: Some(tool_definitions),
        user: Some(session_id.to_string()),
        max_tokens: Some(4096), // Increase default to prevent cut-off
        ..Default::default()
    };

    info!("Starting chat loop for session {:?}", session_id);
    let mut loop_count = 0;
    let max_loops = 5;

    while loop_count < max_loops {
        // Use streaming for the primary interaction to ensure responsiveness
        info!("Calling LLM chat_streaming for session {:?}", session_id);

        let (tx_stream, mut rx_stream) = tokio::sync::mpsc::channel(100);
        let llm_clone = llm.clone();
        let messages_clone = llm_messages.clone();
        let options_clone = current_options.clone();
        let mut session_err_clone = session_err.clone();

        let stream_handle = tokio::spawn(async move {
            let res = llm_clone
                .chat_streaming(&messages_clone, options_clone, tx_stream)
                .await;
            if let Err(ref e) = res {
                error!("Stream error in chat loop: {:?}", e);
                let err_resp = WsServerMessage {
                    r#type: "error".to_string(),
                    content: format!("LLM Error: {}", e),
                };
                let _ = session_err_clone
                    .text(serde_json::to_string(&err_resp).unwrap())
                    .await;
            }
            res
        });

        let mut turn_content = String::new();
        while let Some(chunk) = rx_stream.recv().await {
            turn_content.push_str(&chunk);
            let resp = WsServerMessage {
                r#type: "chunk".to_string(),
                content: chunk,
            };
            if let Ok(json) = serde_json::to_string(&resp) {
                let _ = session.text(json).await;
            }
        }

        // PERSIST FIRST
        {
            let conn = pool.lock().unwrap();
            let _ = crate::db::service::DbService::insert_message(
                &conn,
                session_id,
                "assistant",
                &turn_content,
                Some(llm.name()),
                None,
                serde_json::json!({}),
            );
        }

        // THEN SEND DONE signal to unlock UI
        let done_msg = WsServerMessage {
            r#type: "done".to_string(),
            content: "".to_string(),
        };
        info!(
            "Sending 'done' message to session {:?} after stream finished and persisted",
            session_id
        );
        let _ = session
            .text(serde_json::to_string(&done_msg).unwrap())
            .await;

        let stream_result = stream_handle.await;
        info!("stepbit-core stream worker joined for session {:?}", session_id);

        let mut next_loop = false;

        match stream_result {
            Ok(Ok(Some(tool_calls))) => {
                info!("Extracted {} tool calls from stream", tool_calls.len());
                
                // Strip the trailing JSON array from turn_content so it doesn't pollute the context
                if let Some((_, clean_text)) = crate::llm::extract_streaming_tool_call(&turn_content) {
                    turn_content = clean_text;
                }
                
                // Add assistant message with tool calls to history
                llm_messages.push(LlmMessage {
                    role: "assistant".to_string(),
                    content: turn_content.clone(),
                    tool_calls: Some(tool_calls.clone()),
                    tool_call_id: None,
                });

                // Update DB with the tool calls
                {
                    let conn = pool.lock().unwrap();
                    let _ = crate::db::service::DbService::insert_message(
                        &conn,
                        session_id,
                        "assistant",
                        &turn_content,
                        Some(llm.name()),
                        None,
                        serde_json::json!({ "tool_calls": tool_calls }),
                    );
                }

                // Execute tools
                for tool_call in tool_calls {
                    let tool_name = tool_call.function.name.clone();
                    
                    // Send status to UI
                    let status_msg = WsServerMessage {
                        r#type: "status".to_string(),
                        content: format!("Running tool: {}...", tool_name),
                    };
                    let _ = session.text(serde_json::to_string(&status_msg).unwrap()).await;

                    let tool_id = tool_call.id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
                    let result = tools.call_tool(&tool_name, &tool_call.function.arguments, session_id, pool.clone()).await;
                    
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
                next_loop = true;
            }
            Ok(Ok(None)) => {
                // No tools, just a normal answer
                llm_messages.push(LlmMessage {
                    role: "assistant".to_string(),
                    content: turn_content.clone(),
                    tool_calls: None,
                    tool_call_id: None,
                });
            }
            Ok(Err(e)) => {
                error!("Stream execution failed: {}", e);
            }
            Err(e) => {
                error!("Task execution failed: {}", e);
            }
        }

        if !next_loop {
            break;
        }
        loop_count += 1;
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(ws_chat);
}
