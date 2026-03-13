use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use tokio::sync::mpsc::Sender;

use crate::llm::{models::{ChatOptions, ChatResponse, Message, Usage, ToolCall}, LlmError, LlmProvider};

pub struct OpenAiProvider {
    client: Client,
    api_key: String,
    base_url: String,
    default_model: String,
}

impl OpenAiProvider {
    pub fn new(api_key: String, base_url: String, default_model: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .unwrap_or_else(|_| Client::new()),
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
                tool_calls: None,
                tool_call_id: None,
            });
        }

        let mut body = json!({
            "model": model,
            "messages": final_messages,
            "temperature": options.temperature.unwrap_or(0.7),
            "max_tokens": options.max_tokens.unwrap_or(4096),
        });

        if let Some(tools) = &options.tools {
            body["tools"] = json!(tools);
        }
        if let Some(choice) = &options.tool_choice {
            body["tool_choice"] = json!(choice);
        }

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

        let message = &json["choices"][0]["message"];
        let content = message["content"]
            .as_str()
            .unwrap_or_default()
            .to_string();

        let tool_calls: Option<Vec<ToolCall>> = message.get("tool_calls")
            .and_then(|tc| serde_json::from_value(tc.clone()).ok());

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
            tool_calls,
        })
    }

    async fn chat_streaming(
        &self,
        messages: &[Message],
        options: ChatOptions,
        tx: Sender<String>,
    ) -> Result<Option<Vec<ToolCall>>, LlmError> {
        let model = options.model.as_deref().unwrap_or(&self.default_model);

        let mut final_messages: Vec<Message> = messages.to_vec();
        if let Some(system) = &options.system_prompt {
            final_messages.insert(0, Message {
                role: "system".to_string(),
                content: system.clone(),
                tool_calls: None,
                tool_call_id: None,
            });
        }

        let mut body = json!({
            "model": model,
            "messages": final_messages,
            "stream": true,
            "temperature": options.temperature.unwrap_or(0.7),
            "max_tokens": options.max_tokens.unwrap_or(4096),
        });

        if let Some(tools) = &options.tools {
            body["tools"] = json!(tools);
        }
        if let Some(choice) = &options.tool_choice {
            body["tool_choice"] = json!(choice);
        }

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

        Ok(None)
    }

    fn supported_models(&self) -> Vec<String> {
        vec!["gpt-4o", "gpt-4-turbo", "gpt-3.5-turbo"].into_iter().map(|s| s.to_string()).collect()
    }

    async fn discover_models(&self) -> Result<Vec<String>, LlmError> {
        let response = self
            .client
            .get(format!("{}/models", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| LlmError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Ok(self.supported_models());
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LlmError::Network(e.to_string()))?;

        if let Some(data) = json["data"].as_array() {
            let model_names: Vec<String> = data
                .iter()
                .filter_map(|m| m["id"].as_str().map(|s| s.to_string()))
                .filter(|id| id.contains("gpt"))
                .collect();
            if !model_names.is_empty() {
                return Ok(model_names);
            }
        }

        Ok(self.supported_models())
    }

    async fn verify_connection(&self) -> Result<(), LlmError> {
        let response = self
            .client
            .get(format!("{}/models", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| LlmError::Network(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let text = response.text().await.unwrap_or_default();
            Err(LlmError::Api(format!("OpenAI connection failed: {}", text)))
        }
    }

    fn default_model(&self) -> String {
        self.default_model.clone()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
