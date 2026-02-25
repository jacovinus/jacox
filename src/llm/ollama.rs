use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use tokio::sync::mpsc::Sender;

use crate::llm::{models::{ChatOptions, ChatResponse, Message}, LlmError, LlmProvider};

pub struct OllamaProvider {
    client: Client,
    base_url: String,
    default_model: String,
}

impl OllamaProvider {
    pub fn new(base_url: String, default_model: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            default_model,
        }
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    async fn chat(&self, messages: &[Message], options: ChatOptions) -> Result<ChatResponse, LlmError> {
        let model = options.model.as_deref().unwrap_or(&self.default_model);

        let body = json!({
            "model": model,
            "messages": messages,
            "stream": false,
            "options": {
                "temperature": options.temperature.unwrap_or(0.7),
                "num_predict": options.max_tokens.unwrap_or(4096)
            }
        });

        let response = self
            .client
            .post(format!("{}/api/chat", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::Api(format!("Ollama Error {}: {}", status, text)));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LlmError::Network(e.to_string()))?;

        let content = json["message"]["content"]
            .as_str()
            .ok_or(LlmError::InvalidRequest)?
            .to_string();

        Ok(ChatResponse {
            content,
            model: model.to_string(),
            usage: None,
        })
    }

    async fn chat_streaming(
        &self,
        messages: &[Message],
        options: ChatOptions,
        tx: Sender<String>,
    ) -> Result<(), LlmError> {
        let model = options.model.as_deref().unwrap_or(&self.default_model);

        let body = json!({
            "model": model,
            "messages": messages,
            "stream": true,
            "options": {
                "temperature": options.temperature.unwrap_or(0.7),
                "num_predict": options.max_tokens.unwrap_or(4096)
            }
        });

        let response = self
            .client
            .post(format!("{}/api/chat", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::Api(format!("Ollama Stream Error {}: {}", status, text)));
        }

        let mut stream = response.bytes_stream();
        use futures_util::StreamExt;
        
        while let Some(chunk) = stream.next().await {
            let bytes = chunk.map_err(|e| LlmError::Network(e.to_string()))?;
            if let Ok(text) = String::from_utf8(bytes.to_vec()) {
                for line in text.lines() {
                    if line.trim().is_empty() {
                        continue;
                    }
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                        if let Some(content) = json["message"]["content"].as_str() {
                            let _ = tx.send(content.to_string()).await;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn supported_models(&self) -> Vec<&str> {
        vec!["llama3.2", "mistral"]
    }
}
