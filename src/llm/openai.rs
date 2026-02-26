use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use tokio::sync::mpsc::Sender;

use crate::llm::{models::{ChatOptions, ChatResponse, Message, Usage}, LlmError, LlmProvider};

pub struct OpenAiProvider {
    client: Client,
    api_key: String,
    base_url: String,
    default_model: String,
}

impl OpenAiProvider {
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
impl LlmProvider for OpenAiProvider {
    fn name(&self) -> &str {
        "openai"
    }

    async fn chat(&self, messages: &[Message], options: ChatOptions) -> Result<ChatResponse, LlmError> {
        let model = options.model.as_deref().unwrap_or(&self.default_model);

        let mut final_messages: Vec<Message> = messages.to_vec();
        if let Some(system) = &options.system_prompt {
            final_messages.insert(0, Message {
                role: "system".to_string(),
                content: system.clone(),
            });
        }

        let body = json!({
            "model": model,
            "messages": final_messages,
            "temperature": options.temperature.unwrap_or(0.7),
            "max_tokens": options.max_tokens.unwrap_or(4096),
        });

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
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
            return Err(LlmError::Api(format!("OpenAI Error {}: {}", status, text)));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LlmError::Network(e.to_string()))?;

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or(LlmError::InvalidRequest)?
            .to_string();

        let usage = if let Some(u) = json.get("usage") {
            Some(Usage {
                input_tokens: u["prompt_tokens"].as_u64().unwrap_or(0) as u32,
                output_tokens: u["completion_tokens"].as_u64().unwrap_or(0) as u32,
            })
        } else {
            None
        };

        Ok(ChatResponse {
            content,
            model: model.to_string(),
            usage,
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
            final_messages.insert(0, Message {
                role: "system".to_string(),
                content: system.clone(),
            });
        }

        let body = json!({
            "model": model,
            "messages": final_messages,
            "stream": true,
            "temperature": options.temperature.unwrap_or(0.7),
            "max_tokens": options.max_tokens.unwrap_or(4096),
        });

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
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
            return Err(LlmError::Api(format!("OpenAI Stream Error {}: {}", status, text)));
        }

        let mut stream = response.bytes_stream();
        use futures_util::StreamExt;

        while let Some(chunk) = stream.next().await {
            let bytes = chunk.map_err(|e| LlmError::Network(e.to_string()))?;
            if let Ok(text) = String::from_utf8(bytes.to_vec()) {
                for line in text.lines() {
                    let line = line.trim();
                    if line.is_empty() || line == "data: [DONE]" {
                        continue;
                    }
                    if let Some(data) = line.strip_prefix("data: ") {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                            if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
                                let _ = tx.send(content.to_string()).await;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn supported_models(&self) -> Vec<&str> {
        vec!["gpt-4o", "gpt-4-turbo", "gpt-3.5-turbo"]
    }
}
