use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use tokio::sync::mpsc::Sender;

use crate::llm::{models::{ChatOptions, ChatResponse, Message, ToolCall, FunctionCall}, LlmError, LlmProvider};

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
            "stream": false,
            "options": {
                "temperature": options.temperature.unwrap_or(0.7),
                "num_predict": options.max_tokens.unwrap_or(4096)
            }
        });

        if let Some(tools) = &options.tools {
            body["tools"] = json!(tools);
        }

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

        let message = &json["message"];
        let mut content = message["content"]
            .as_str()
            .unwrap_or_default()
            .to_string();

        let mut tool_calls: Option<Vec<ToolCall>> = message.get("tool_calls")
            .and_then(|tc| serde_json::from_value(tc.clone()).ok());

        // Fallback: If no tool_calls, try to find JSON tool calls in the content string
        if tool_calls.as_ref().map(|tc| tc.is_empty()).unwrap_or(true) {
            // Find the start of a potential JSON array
            if let Some(start_pos) = content.find('[') {
                let json_part = &content[start_pos..];
                
                // Try to find the matching closing bracket for the array
                let mut bracket_count = 0;
                let mut end_pos = None;
                for (i, c) in json_part.chars().enumerate() {
                    if c == '[' { bracket_count += 1; }
                    else if c == ']' {
                        bracket_count -= 1;
                        if bracket_count == 0 {
                            end_pos = Some(i + 1);
                            break;
                        }
                    }
                }

                if let Some(len) = end_pos {
                    let potential_json = &json_part[..len];
                    if let Ok(parsed_functions) = serde_json::from_str::<Vec<serde_json::Value>>(potential_json) {
                        let mapped_tools: Vec<ToolCall> = parsed_functions.into_iter().filter_map(|f| {
                            // High flexibility: handle both ToolCall wrapper and direct function object
                            let func_obj = if f.get("function").is_some() { &f["function"] } else { &f };
                            
                            let name = func_obj["name"].as_str()?.to_string();
                            let args = if func_obj["arguments"].is_string() {
                                func_obj["arguments"].as_str()?.to_string()
                            } else {
                                func_obj["arguments"].to_string()
                            };
                            
                            Some(ToolCall {
                                id: f["id"].as_str().map(|s| s.to_string()),
                                r#type: Some("function".to_string()),
                                function: FunctionCall { name, arguments: args },
                            })
                        }).collect();
                        
                        if !mapped_tools.is_empty() {
                            tool_calls = Some(mapped_tools);
                            // Keep text before the JSON, discarding the JSON and anything after it
                            content = content[..start_pos].trim().to_string();
                        }
                    }
                }
            }
        }

        Ok(ChatResponse {
            content,
            model: model.to_string(),
            usage: None,
            tool_calls,
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
                tool_calls: None,
                tool_call_id: None,
            });
        }

        let mut body = json!({
            "model": model,
            "messages": final_messages,
            "stream": true,
            "options": {
                "temperature": options.temperature.unwrap_or(0.7),
                "num_predict": options.max_tokens.unwrap_or(4096)
            }
        });

        if let Some(tools) = &options.tools {
            body["tools"] = json!(tools);
        }

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
