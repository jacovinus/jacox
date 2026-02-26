use actix_web::{post, web, HttpResponse, HttpRequest, Result as WebResult};
use bytes::Bytes;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::api::models_openai::{
    OpenAIChatRequest, OpenAIChatResponse, OpenAIChoice, OpenAIMessage, OpenAIStreamChoice,
    OpenAIStreamChunk, OpenAIStreamDelta, OpenAIUsage,
};
use crate::llm::{
    models::{ChatOptions, Message as LlmMessage},
    LlmProvider,
};
use crate::db::{service::DbService, DbPool};

#[post("/v1/chat/completions")]
pub async fn openai_chat_completions(
    llm: web::Data<Arc<dyn LlmProvider>>,
    pool: web::Data<DbPool>,
    req_http: HttpRequest,
    req: web::Json<OpenAIChatRequest>,
) -> WebResult<HttpResponse> {
    let req = req.into_inner();
    
    // Check for session persistence header
    let session_id = req_http
        .headers()
        .get("X-Jacox-Session-Id")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok());

    let llm_messages: Vec<LlmMessage> = req
        .messages
        .iter()
        .map(|m| LlmMessage {
            role: m.role.clone(),
            content: m.content.clone(),
        })
        .collect();

    // If session_id is provided, persist the last user message
    if let Some(sid) = session_id {
        let conn = pool.lock().unwrap();
        // Just a safety check if session exists
        if DbService::get_session(&conn, sid).unwrap_or(None).is_some() {
            if let Some(last_msg) = req.messages.last() {
                if last_msg.role == "user" {
                    let _ = DbService::insert_message(
                        &conn,
                        sid,
                        "user",
                        &last_msg.content,
                        Some(&req.model),
                        None,
                        serde_json::json!({ "source": "openai_adapter" }),
                    );
                }
            }
        }
    }

    let chat_options = ChatOptions {
        model: Some(req.model.clone()),
        temperature: req.temperature,
        max_tokens: req.max_tokens,
        system_prompt: None,
    };

    let is_streaming = req.stream.unwrap_or(false);

    if is_streaming {
        let (tx, mut rx) = mpsc::channel(100);
        let llm_clone = llm.into_inner();
        let model_name = req.model.clone();
        let pool_clone = pool.as_ref().clone();

        tokio::spawn(async move {
            if let Err(e) = llm_clone
                .chat_streaming(&llm_messages, chat_options, tx)
                .await
            {
                tracing::error!("OpenAI Adapter Streaming Error: {:?}", e);
            }
        });

        let stream = async_stream::stream! {
            let id = format!("chatcmpl-{}", Uuid::new_v4());
            let mut full_content = String::new();
            
            // OpenAI requires an empty role delta to start
            let initial_chunk = OpenAIStreamChunk {
                id: id.clone(),
                object: "chat.completion.chunk".to_string(),
                created: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                model: model_name.clone(),
                choices: vec![OpenAIStreamChoice {
                    index: 0,
                    delta: OpenAIStreamDelta { role: Some("assistant".to_string()), content: None },
                    finish_reason: None,
                }],
            };
            
            yield Ok::<Bytes, actix_web::Error>(Bytes::from(format!("data: {}\n\n", serde_json::to_string(&initial_chunk).unwrap())));

            while let Some(chunk_text) = rx.recv().await {
                full_content.push_str(&chunk_text);
                let chunk = OpenAIStreamChunk {
                    id: id.clone(),
                    object: "chat.completion.chunk".to_string(),
                    created: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                    model: model_name.clone(),
                    choices: vec![OpenAIStreamChoice {
                        index: 0,
                        delta: OpenAIStreamDelta { role: None, content: Some(chunk_text) },
                        finish_reason: None,
                    }],
                };
                yield Ok::<Bytes, actix_web::Error>(Bytes::from(format!("data: {}\n\n", serde_json::to_string(&chunk).unwrap())));
            }
            
            // Persist the assistant message if session_id was present
            if let Some(sid) = session_id {
                let conn = pool_clone.lock().unwrap();
                let _ = DbService::insert_message(
                    &conn,
                    sid,
                    "assistant",
                    &full_content,
                    Some(&model_name),
                    None,
                    serde_json::json!({ "source": "openai_adapter" }),
                );
            }

            // Final chunk
            let final_chunk = OpenAIStreamChunk {
                id: id.clone(),
                object: "chat.completion.chunk".to_string(),
                created: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                model: model_name.clone(),
                choices: vec![OpenAIStreamChoice {
                    index: 0,
                    delta: OpenAIStreamDelta { role: None, content: None },
                    finish_reason: Some("stop".to_string()),
                }],
            };
            yield Ok::<Bytes, actix_web::Error>(Bytes::from(format!("data: {}\n\n", serde_json::to_string(&final_chunk).unwrap())));
            yield Ok::<Bytes, actix_web::Error>(Bytes::from("data: [DONE]\n\n"));
        };

        Ok(HttpResponse::Ok()
            .content_type("text/event-stream")
            .streaming(stream))
    } else {
        // Synchronous non-streaming
        let response = match llm.chat(&llm_messages, chat_options).await {
            Ok(res) => res,
            Err(e) => {
                return Ok(HttpResponse::InternalServerError()
                    .json(serde_json::json!({ "error": e.to_string() })))
            }
        };

        // Persist the assistant message if session_id was present
        if let Some(sid) = session_id {
            let conn = pool.lock().unwrap();
            let _ = DbService::insert_message(
                &conn,
                sid,
                "assistant",
                &response.content,
                Some(&response.model),
                None,
                serde_json::json!({ "source": "openai_adapter" }),
            );
        }

        let usage = response.usage.map(|u| OpenAIUsage {
            prompt_tokens: u.input_tokens,
            completion_tokens: u.output_tokens,
            total_tokens: u.input_tokens + u.output_tokens,
        });

        let resp = OpenAIChatResponse {
            id: format!("chatcmpl-{}", Uuid::new_v4()),
            object: "chat.completion".to_string(),
            created: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            model: response.model,
            choices: vec![OpenAIChoice {
                index: 0,
                message: OpenAIMessage {
                    role: "assistant".to_string(),
                    content: response.content,
                },
                finish_reason: "stop".to_string(),
            }],
            usage,
        };

        Ok(HttpResponse::Ok().json(resp))
    }
}
