use std::sync::Arc;
use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::Client;
use serde_json::json;
use tokio::sync::mpsc::Sender;
use tracing::{debug, error, info};

use crate::llm::{
    models::{ChatOptions, ChatResponse, Message},
    LlmError, LlmProvider,
};

pub struct LlmosProvider {
    client: Client,
    base_url: String,
    default_model: String,
    api_key: Option<String>,
    rotating_token: Arc<std::sync::Mutex<Option<String>>>,
}

impl LlmosProvider {
    pub fn new(base_url: String, default_model: String, api_key: Option<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(300))
                .build()
                .unwrap_or_else(|_| Client::new()),
            base_url,
            default_model,
            api_key,
            rotating_token: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    async fn authenticated_request(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<reqwest::Response, LlmError> {
        let url = format!("{}{}", self.base_url, path);

        let rotating_token = {
            let guard = self.rotating_token.lock().unwrap();
            guard.clone()
        };

        // 1. Attempt with rotating token if available
        if let Some(token) = rotating_token {
            let mut req = self.client.request(method.clone(), &url).bearer_auth(&token);
            if let Some(b) = &body {
                req = req.json(b);
            }
            let res = req
                .send()
                .await
                .map_err(|e| LlmError::Network(e.to_string()))?;

            if res.status().is_success() {
                self.handle_token_rotation(&res);
                return Ok(res);
            }

            if res.status() == reqwest::StatusCode::UNAUTHORIZED {
                debug!("Rotating token unauthorized, falling back to master key");
                {
                    let mut guard = self.rotating_token.lock().unwrap();
                    *guard = None;
                }
            } else {
                // Return other statuses normally (404, 500, etc)
                return Ok(res);
            }
        }

        // 2. Fallback or Initial attempt with master key
        let mut req = self.client.request(method, &url);
        if let Some(master) = &self.api_key {
            req = req.bearer_auth(master);
        }
        if let Some(b) = &body {
            req = req.json(b);
        }

        let res = req
            .send()
            .await
            .map_err(|e| LlmError::Network(e.to_string()))?;
        if res.status().is_success() {
            self.handle_token_rotation(&res);
        }
        Ok(res)
    }

    fn handle_token_rotation(&self, res: &reqwest::Response) {
        if let Some(next_token) = res.headers().get("X-Next-Token") {
            if let Ok(token_str) = next_token.to_str() {
                let mut guard = self.rotating_token.lock().unwrap();
                *guard = Some(token_str.to_string());
                debug!("Token rotated successfully");
            }
        }
    }
}

#[async_trait]
impl LlmProvider for LlmosProvider {
    fn name(&self) -> &str {
        "llmos"
    }

    async fn chat(
        &self,
        messages: &[Message],
        options: ChatOptions,
    ) -> Result<ChatResponse, LlmError> {
        let model = options.model.as_deref().unwrap_or(&self.default_model);

        let mut final_messages: Vec<Message> = messages.to_vec();
        if let Some(system) = &options.system_prompt {
            final_messages.insert(
                0,
                Message {
                    role: "system".to_string(),
                    content: system.clone(),
                    tool_calls: None,
                    tool_call_id: None,
                },
            );
        }

        let body = json!({
            "model": model,
            "messages": final_messages,
            "stream": false,
            "max_tokens": options.max_tokens.unwrap_or(4096),
            "temperature": options.temperature.unwrap_or(0.7),
        });

        let response = self
            .authenticated_request(reqwest::Method::POST, "/v1/chat/completions", Some(body))
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::Api(format!("LLMOS Error {}: {}", status, text)));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LlmError::Network(e.to_string()))?;

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or_default()
            .to_string();

        Ok(ChatResponse {
            content,
            model: model.to_string(),
            usage: None,
            tool_calls: None, // jac_llmos doesn't support tools yet in its ChatCompletionResponse
        })
    }

    async fn chat_streaming(
        &self,
        messages: &[Message],
        options: ChatOptions,
        tx: Sender<String>,
    ) -> Result<(), LlmError> {
        let model = options.model.as_deref().unwrap_or(&self.default_model);

        let mut final_messages: Vec<Message> = messages.to_vec();
        if let Some(system) = &options.system_prompt {
            final_messages.insert(
                0,
                Message {
                    role: "system".to_string(),
                    content: system.clone(),
                    tool_calls: None,
                    tool_call_id: None,
                },
            );
        }

        let body = json!({
            "model": model,
            "messages": final_messages,
            "stream": true,
            "max_tokens": options.max_tokens.unwrap_or(4096),
            "temperature": options.temperature.unwrap_or(0.7),
        });

        let response = self
            .authenticated_request(reqwest::Method::POST, "/v1/chat/completions", Some(body))
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::Api(format!(
                "LLMOS Stream Error {}: {}",
                status, text
            )));
        }

        let mut stream = response.bytes_stream();
        let mut buffer = String::new();

        loop {
            let next_chunk =
                tokio::time::timeout(std::time::Duration::from_secs(30), stream.next()).await;

            let chunk = match next_chunk {
                Ok(Some(result)) => result.map_err(|e| LlmError::Network(e.to_string()))?,
                Ok(None) => break, // Stream ended naturally
                Err(_) => {
                    error!("LLMOS stream timed out after 30s");
                    return Err(LlmError::Network("Stream timeout".to_string()));
                }
            };

            let chunk_str = String::from_utf8_lossy(&chunk);
            debug!("Received chunk from LLMOS: {:?}", chunk_str);
            buffer.push_str(&chunk_str);

            while let Some(line_end) = buffer.find('\n') {
                let line = buffer[..line_end].trim().to_string();
                buffer.drain(..=line_end);

                if line.is_empty() {
                    continue;
                }

                if line == "data: [DONE]" || line.contains("[DONE]") {
                    info!(
                        "SSE Stream reached [DONE] signal for session {:?}",
                        options.user
                    );
                    return Ok(());
                }

                if line.starts_with("data: ") {
                    let data = &line[6..];
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                        // Check for the specific {type: 'done'} message that should REALLY end the stream
                        if json.get("type").and_then(|t| t.as_str()) == Some("done") {
                            info!(
                                "SSE Stream reached explicit done signal (type: done) for session {:?}",
                                options.user
                            );
                            return Ok(());
                        }

                        if let Some(choices) = json["choices"].as_array() {
                            if let Some(choice) = choices.get(0) {
                                if let Some(content) = choice["delta"]["content"].as_str() {
                                    let _ = tx.send(content.to_string()).await;
                                }
                                if let Some(reason) = choice["finish_reason"].as_str() {
                                    debug!("Stream chunk finish_reason: {}", reason);
                                    // Previously we might have hit 'done: true' and bailed.
                                    // LLMOS sends 'done: true' when a microbatch finishes or EOS.
                                    // We should only stop if it's EOS or explicitly Told to.
                                    // Actually, we'll let the loop continue until [DONE] or type: done.
                                }
                            }
                        }
                    }
                }
            }
        }

        // Handle trailing data if any
        let final_line = buffer.trim().to_string();
        if !final_line.is_empty() {
            if final_line == "data: [DONE]" || final_line.contains("[DONE]") {
                info!("SSE Stream reached [DONE] in trailing data");
                return Ok(());
            }
        }

        info!(
            "LlmosProvider::chat_streaming loop finished naturally for session {:?}",
            options.user
        );
        Ok(())
    }

    fn supported_models(&self) -> Vec<String> {
        vec![self.default_model.clone()]
    }

    async fn discover_models(&self) -> Result<Vec<String>, LlmError> {
        let response = self
            .authenticated_request(reqwest::Method::GET, "/v1/models", None)
            .await?;

        if !response.status().is_success() {
            return Ok(self.supported_models());
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LlmError::Network(e.to_string()))?;

        if let Some(models) = json["data"].as_array() {
            let model_names: Vec<String> = models
                .iter()
                .filter_map(|m| m["id"].as_str().map(|s| s.to_string()))
                .collect();
            if !model_names.is_empty() {
                return Ok(model_names);
            }
        }

        Ok(self.supported_models())
    }

    async fn verify_connection(&self) -> Result<(), LlmError> {
        let response = self
            .authenticated_request(reqwest::Method::GET, "/health", None)
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(LlmError::Api(format!(
                "LLMOS connection failed with status: {}",
                response.status()
            )))
        }
    }

    async fn cancel(&self, session_id: &str) -> Result<(), LlmError> {
        let path = format!("/v1/chat/cancel/{}", session_id);
        let _ = self.authenticated_request(reqwest::Method::DELETE, &path, None).await;
        Ok(())
    }

    fn default_model(&self) -> String {
        self.default_model.clone()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
