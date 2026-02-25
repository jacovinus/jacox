use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use actix_ws::Message;
use futures_util::StreamExt as _;
use std::sync::Arc;
use tokio::sync::mpsc;
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

    // web::Data<T> behaves like an Arc<T>. To get the inner Arc out of Data<Arc<T>> we deref and clone.
    let llm_arc = llm.as_ref().clone();
    let pool_arc = pool.as_ref().clone();

    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Ping(bytes) => {
                    if session.pong(&bytes).await.is_err() {
                        return;
                    }
                }
                Message::Text(text) => {
                    let client_msg: Result<WsClientMessage, _> = serde_json::from_str(&text);
                    if let Ok(msg) = client_msg {
                        if msg.r#type == "message" {
                            handle_chat_message(
                                msg.content,
                                id,
                                pool_arc.clone(),
                                llm_arc.clone(),
                                &mut session,
                            )
                            .await;
                        }
                    }
                }
                Message::Close(reason) => {
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
    session_id: Uuid,
    pool: DbPool,
    llm: Arc<dyn LlmProvider>,
    session: &mut actix_ws::Session,
) {
    // 1. Save user message to database
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
        let err_resp = WsServerMessage {
            r#type: "error".to_string(),
            content: "Database error".to_string(),
        };
        let _ = session
            .text(serde_json::to_string(&err_resp).unwrap())
            .await;
        return;
    }

    // 2. Fetch History
    let history = match DbService::get_messages(&conn, session_id, 50, 0) {
        Ok(msgs) => msgs,
        Err(e) => {
            error!("Failed to fetch history: {}", e);
            return;
        }
    };

    let llm_messages: Vec<LlmMessage> = history
        .into_iter()
        .map(|m| LlmMessage {
            role: m.role,
            content: m.content,
        })
        .collect();

    drop(conn);

    // 3. Trigger Streaming LLM Provider Request
    let (tx, mut rx) = mpsc::channel::<String>(100);

    let llm_clone = llm.clone();
    
    // Spawn the network request payload in background so we can listen to the chunk rx channel
    tokio::spawn(async move {
        let opts = ChatOptions::default();
        if let Err(e) = llm_clone.chat_streaming(&llm_messages, opts, tx).await {
            error!("LLM Streaming Error: {:?}", e);
        }
    });

    let mut full_assistant_response = String::new();

    // 4. Stream tokens to WS client
    while let Some(chunk) = rx.recv().await {
        full_assistant_response.push_str(&chunk);
        let resp = WsServerMessage {
            r#type: "chunk".to_string(),
            content: chunk.clone(),
        };
        if session
            .text(serde_json::to_string(&resp).unwrap())
            .await
            .is_err()
        {
            // Client likely disconnected
            break;
        }
    }

    // 5. Send 'done' message
    let done_msg = WsServerMessage {
        r#type: "done".to_string(),
        content: "".to_string(),
    };
    let _ = session.text(serde_json::to_string(&done_msg).unwrap()).await;

    // 6. Save final assistant message to database
    let conn = pool.lock().unwrap();
    let _ = DbService::insert_message(
        &conn,
        session_id,
        "assistant",
        &full_assistant_response,
        Some(llm.name()),
        None,
        serde_json::json!({}),
    );
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(ws_chat);
}
