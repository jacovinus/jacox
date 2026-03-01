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

    let current_date = chrono::Local::now().format("%A, %B %d, %Y").to_string();
    let grounding_str = format!("Current Date: {}.\n\n", current_date);
    
    let mut current_llm_messages: Vec<LlmMessage> = req
        .messages
        .iter()
        .map(|m| LlmMessage {
            role: m.role.clone(),
            content: if m.role == "system" {
                format!("{}{}", grounding_str, m.content.replace("{current_date}", &current_date))
            } else {
                m.content.replace("{current_date}", &current_date)
            },
            tool_calls: m.tool_calls.clone(),
            tool_call_id: m.tool_call_id.clone(),
        })
        .collect();
        
    // If no system message was present, prepend one
    if !current_llm_messages.iter().any(|m| m.role == "system") {
        current_llm_messages.insert(0, LlmMessage {
            role: "system".to_string(),
            content: grounding_str,
            tool_calls: None,
            tool_call_id: None,
        });
    }

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

    let mut chat_options = ChatOptions {
        model: Some(req.model.clone()),
        temperature: req.temperature,
        max_tokens: req.max_tokens,
        system_prompt: None,
        tools: req.tools,
        tool_choice: req.tool_choice,
    };

    // If no tools provided in request, offer the default ones from the registry
    if chat_options.tools.is_none() {
        let registry = crate::tools::ToolRegistry::new();
        chat_options.tools = Some(registry.get_definitions());
    }

    let is_streaming = req.stream.unwrap_or(false);

    if is_streaming {
        // For streaming, we'll keep the existing logic but pass the options
        let (tx, mut rx) = mpsc::channel(100);
        let llm_clone = llm.into_inner();
        let model_name = req.model.clone();
        let pool_clone = pool.as_ref().clone();

        tokio::spawn(async move {
            if let Err(e) = llm_clone
                .chat_streaming(&current_llm_messages, chat_options, tx)
                .await
            {
                tracing::error!("OpenAI Adapter Streaming Error: {:?}", e);
            }
        });
        
        // ... (rest of streaming logic remains largely same, just uses current_llm_messages)

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
        let tools = crate::tools::ToolRegistry::new();
        let mut loop_count = 0;
        let max_loops = 5;

        while loop_count < max_loops {
            let response = match llm.chat(&current_llm_messages, chat_options.clone()).await {
                Ok(res) => res,
                Err(e) => {
                    return Ok(HttpResponse::InternalServerError()
                        .json(serde_json::json!({ "error": e.to_string() })))
                }
            };

            if response.tool_calls.as_ref().map(|tc| tc.is_empty()).unwrap_or(true) {
                // Final response
                let openai_resp = OpenAIChatResponse {
                    id: format!("chatcmpl-{}", Uuid::new_v4()),
                    object: "chat.completion".to_string(),
                    created: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                    model: response.model.clone(),
                    choices: vec![OpenAIChoice {
                        index: 0,
                        message: OpenAIMessage {
                            role: "assistant".to_string(),
                            content: response.content.clone(),
                            tool_calls: None,
                            tool_call_id: None,
                        },
                        finish_reason: Some("stop".to_string()),
                    }],
                    usage: response.usage.as_ref().map(|u| OpenAIUsage {
                        prompt_tokens: u.input_tokens,
                        completion_tokens: u.output_tokens,
                        total_tokens: u.input_tokens + u.output_tokens,
                    }),
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
                        response.usage.as_ref().map(|u| (u.input_tokens + u.output_tokens) as i32),
                        serde_json::json!({ "source": "openai_adapter" }),
                    );
                }

                return Ok(HttpResponse::Ok().json(openai_resp));
            }

            // Handle tool calls
            let mut assistant_tool_calls = Vec::new();
            let tool_calls = response.tool_calls.unwrap();
            
            for tool_call in &tool_calls {
                assistant_tool_calls.push(tool_call.clone());
            }

            // Add assistant message with tool calls to history
            current_llm_messages.push(LlmMessage {
                role: "assistant".to_string(),
                content: response.content.clone(),
                tool_calls: Some(assistant_tool_calls.clone()),
                tool_call_id: None,
            });

            // Persist assistant message
            if let Some(sid) = session_id {
                let conn = pool.lock().unwrap();
                let _ = DbService::insert_message(
                    &conn,
                    sid,
                    "assistant",
                    &response.content,
                    Some(&response.model),
                    None,
                    serde_json::json!({ "source": "openai_adapter", "tool_calls": assistant_tool_calls }),
                );
            }

            // Execute tools
            for tool_call in tool_calls {
                let tool_id = tool_call.id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
                let result = tools.call_tool(&tool_call.function.name, &tool_call.function.arguments).await;
                
                current_llm_messages.push(LlmMessage {
                    role: "tool".to_string(),
                    content: result.clone(),
                    tool_calls: None,
                    tool_call_id: Some(tool_id.clone()),
                });

                // Persist tool result
                if let Some(sid) = session_id {
                    let conn = pool.lock().unwrap();
                    let _ = DbService::insert_message(
                        &conn,
                        sid,
                        "tool",
                        &result,
                        None,
                        None,
                        serde_json::json!({ "source": "openai_adapter", "tool_call_id": tool_id }),
                    );
                }
            }

            loop_count += 1;
        }

        Ok(HttpResponse::InternalServerError().body("Max tool call loops reached"))
    }
}
