use actix_web::{post, web, HttpResponse, Result as WebResult};
use bytes::Bytes;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::api::models_openai::{
    OpenAIChatRequest, OpenAIChatResponse, OpenAIChoice, OpenAIMessage, OpenAIStreamChoice,
    OpenAIStreamChunk, OpenAIStreamDelta, OpenAIUsage,
};
use crate::llm::{
    models::{ChatOptions, Message as LlmMessage},
    LlmProvider,
};

#[post("/v1/chat/completions")]
pub async fn openai_chat_completions(
    llm: web::Data<Arc<dyn LlmProvider>>,
    req: web::Json<OpenAIChatRequest>,
) -> WebResult<HttpResponse> {
    let req = req.into_inner();

    let llm_messages: Vec<LlmMessage> = req
        .messages
        .into_iter()
        .map(|m| LlmMessage {
            role: m.role,
            content: m.content,
        })
        .collect();

    let chat_options = ChatOptions {
        model: Some(req.model.clone()),
        temperature: req.temperature,
        max_tokens: req.max_tokens,
        system_prompt: None, // Assumed passed as role: "system" in the array directly
    };

    let is_streaming = req.stream.unwrap_or(false);

    if is_streaming {
        let (tx, mut rx) = mpsc::channel(100);
        let llm_clone = llm.into_inner();
        let model_name = req.model.clone();

        tokio::spawn(async move {
            if let Err(e) = llm_clone
                .chat_streaming(&llm_messages, chat_options, tx)
                .await
            {
                tracing::error!("OpenAI Adapter Streaming Error: {:?}", e);
            }
        });

        let stream = async_stream::stream! {
            let id = format!("chatcmpl-{}", uuid::Uuid::new_v4());
            
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

        let usage = response.usage.map(|u| OpenAIUsage {
            prompt_tokens: u.input_tokens,
            completion_tokens: u.output_tokens,
            total_tokens: u.input_tokens + u.output_tokens,
        });

        let resp = OpenAIChatResponse {
            id: format!("chatcmpl-{}", uuid::Uuid::new_v4()),
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
