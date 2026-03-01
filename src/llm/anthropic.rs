use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use tokio::sync::mpsc::Sender;

use crate::llm::{models::{ChatOptions, ChatResponse, Message, Usage}, LlmError, LlmProvider};

pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    base_url: String,
    default_model: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String, base_url: String, default_model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url,
            default_model,
        }
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    async fn chat(&self, messages: &[Message], options: ChatOptions) -> Result<ChatResponse, LlmError> {
        let model = options.model.as_deref().unwrap_or(&self.default_model);

        // Anthropic requires the 'system' prompt as a separate field, and 'messages' handles role/content only for 'user'/'assistant'
        let mut system = String::new();
        let filtered_messages: Vec<Message> = messages
            .iter()
            .filter_map(|m| {
                if m.role == "system" {
                    system.push_str(&m.content);
                    system.push('\n');
                    None
                } else {
                    Some(m.clone())
                }
            })
            .collect();

        if let Some(opts_system) = &options.system_prompt {
            system.push_str(opts_system);
        }

        let body = json!({
            "model": model,
            "messages": filtered_messages,
            "system": system.trim(),
            "temperature": options.temperature.unwrap_or(0.7),
            "max_tokens": options.max_tokens.unwrap_or(4096),
        });

        let response = self
            .client
            .post(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                return Err(LlmError::RateLimited);
            }
            return Err(LlmError::Api(format!("Anthropic Error {}: {}", status, text)));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LlmError::Network(e.to_string()))?;

        let content = json["content"][0]["text"]
            .as_str()
            .ok_or(LlmError::InvalidRequest)?
            .to_string();

        let usage = if let Some(u) = json.get("usage") {
            Some(Usage {
                input_tokens: u["input_tokens"].as_u64().unwrap_or(0) as u32,
                output_tokens: u["output_tokens"].as_u64().unwrap_or(0) as u32,
            })
        } else {
            None
        };

        Ok(ChatResponse {
            content,
            model: model.to_string(),
            usage,
            tool_calls: None,
        })
    }

    async fn chat_streaming(
        &self,
        messages: &[Message],
        options: ChatOptions,
        tx: Sender<String>,
    ) -> Result<(), LlmError> {
        let model = options.model.as_deref().unwrap_or(&self.default_model);

        // Anthropic requires the 'system' prompt as a separate field
        let mut system = String::new();
        let filtered_messages: Vec<Message> = messages
            .iter()
            .filter_map(|m| {
                if m.role == "system" {
                    system.push_str(&m.content);
                    system.push('\n');
                    None
                } else {
                    Some(m.clone())
                }
            })
            .collect();

        if let Some(opts_system) = &options.system_prompt {
            system.push_str(opts_system);
        }

        let body = json!({
            "model": model,
            "messages": filtered_messages,
            "system": system.trim(),
            "stream": true,
            "temperature": options.temperature.unwrap_or(0.7),
            "max_tokens": options.max_tokens.unwrap_or(4096),
        });

        let response = self
            .client
            .post(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                return Err(LlmError::RateLimited);
            }
            return Err(LlmError::Api(format!("Anthropic Stream Error {}: {}", status, text)));
        }

        let mut stream = response.bytes_stream();
        use futures_util::StreamExt;

        while let Some(chunk) = stream.next().await {
            let bytes = chunk.map_err(|e| LlmError::Network(e.to_string()))?;
            if let Ok(text) = String::from_utf8(bytes.to_vec()) {
                for line in text.lines() {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    if let Some(data) = line.strip_prefix("data: ") {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                            // Anthropic streams delta objects differently than OpenAI
                            if json["type"].as_str() == Some("content_block_delta") {
                                if let Some(content) = json["delta"]["text"].as_str() {
                                    let _ = tx.send(content.to_string()).await;
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn supported_models(&self) -> Vec<&str> {
        vec!["claude-3-5-sonnet-20241022", "claude-3-opus-20240229"]
    }
}
