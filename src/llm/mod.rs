pub mod anthropic;
pub mod copilot;
pub mod llmos;
pub mod models;
pub mod ollama;
pub mod openai;

use anthropic::AnthropicProvider;
use copilot::CopilotProvider;
use llmos::LlmosProvider;
use ollama::OllamaProvider;
use openai::OpenAiProvider;

use async_trait::async_trait;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc::Sender;

use crate::config::AppConfig;
use models::{ChatOptions, ChatResponse, Message};

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("Network Error: {0}")]
    Network(String),
    #[error("API Error: {0}")]
    Api(String),
    #[error("Invalid Request")]
    InvalidRequest,
    #[error("Rate Limited")]
    RateLimited,
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn name(&self) -> &str;

    async fn chat(
        &self,
        messages: &[Message],
        options: ChatOptions,
    ) -> Result<ChatResponse, LlmError>;

    async fn chat_streaming(
        &self,
        messages: &[Message],
        options: ChatOptions,
        tx: Sender<String>,
    ) -> Result<Option<Vec<models::ToolCall>>, LlmError>;

    fn supported_models(&self) -> Vec<String>;

    async fn discover_models(&self) -> Result<Vec<String>, LlmError> {
        Ok(self.supported_models())
    }

    async fn verify_connection(&self) -> Result<(), LlmError> {
        Ok(())
    }

    async fn cancel(&self, _session_id: &str) -> Result<(), LlmError> {
        Ok(())
    }

    fn default_model(&self) -> String;

    fn as_any(&self) -> &dyn std::any::Any;

    fn tools(&self) -> Vec<models::ToolDefinition> {
        vec![]
    }

    async fn get_mcp_tools(&self) -> Result<Vec<models::McpToolDefinition>, LlmError> {
        Ok(vec![])
    }

    async fn execute_reasoning(
        &self,
        _graph: models::ReasoningGraph,
    ) -> Result<HashMap<String, serde_json::Value>, LlmError> {
        Err(LlmError::Api("Reasoning not supported by this provider".to_string()))
    }

    async fn execute_reasoning_streaming(
        &self,
        _graph: models::ReasoningGraph,
        _tx: Sender<serde_json::Value>,
    ) -> Result<(), LlmError> {
        Err(LlmError::Api("Reasoning streaming not supported by this provider".to_string()))
    }

    async fn execute_pipeline(
        &self,
        _pipeline: serde_json::Value,
        _question: String,
    ) -> Result<models::PipelineExecuteResult, LlmError> {
        Err(LlmError::Api("Pipelines not supported by this provider".to_string()))
    }
}


/// A manager that holds all available providers and handles dynamic switching.
pub struct ProviderManager {
    providers: HashMap<String, Arc<dyn LlmProvider>>,
    active_provider_id: RwLock<String>,
    active_model_id: RwLock<Option<String>>,
}

impl ProviderManager {
    pub fn new(providers: HashMap<String, Arc<dyn LlmProvider>>, default_id: String) -> Self {
        Self {
            providers,
            active_provider_id: RwLock::new(default_id),
            active_model_id: RwLock::new(None),
        }
    }

    pub fn set_active_provider(&self, id: &str) -> Result<(), String> {
        if self.providers.contains_key(id) {
            let mut active_id = self.active_provider_id.write();
            *active_id = id.to_string();
            Ok(())
        } else {
            Err(format!("Provider '{}' not found in registry", id))
        }
    }

    pub fn get_active_provider_id(&self) -> String {
        self.active_provider_id.read().clone()
    }

    pub fn get_active_model_id(&self) -> Option<String> {
        self.active_model_id.read().clone()
    }

    pub fn set_active_model(&self, model_id: Option<String>) {
        let mut active_model = self.active_model_id.write();
        *active_model = model_id;
    }

    pub fn list_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    pub fn get_provider(&self, id: &str) -> Option<Arc<dyn LlmProvider>> {
        self.providers.get(id).cloned()
    }

    fn get_active_provider(&self) -> Arc<dyn LlmProvider> {
        let id = self.active_provider_id.read();
        self.providers
            .get(&*id)
            .cloned()
            .expect("Active provider must exist")
    }
}

#[async_trait]
impl LlmProvider for ProviderManager {
    fn name(&self) -> &str {
        // Technically this is a proxy, but we can return the active one's name
        // Or "provider-manager"
        "provider-manager"
    }

    async fn chat(
        &self,
        messages: &[Message],
        mut options: ChatOptions,
    ) -> Result<ChatResponse, LlmError> {
        if let Some(model) = self.get_active_model_id() {
            options.model = Some(model);
        }
        self.get_active_provider().chat(messages, options).await
    }

    async fn chat_streaming(
        &self,
        messages: &[Message],
        mut options: ChatOptions,
        tx: Sender<String>,
    ) -> Result<Option<Vec<models::ToolCall>>, LlmError> {
        if let Some(model) = self.get_active_model_id() {
            options.model = Some(model);
        }
        self.get_active_provider()
            .chat_streaming(messages, options, tx)
            .await
    }

    fn supported_models(&self) -> Vec<String> {
        self.get_active_provider().supported_models()
    }

    async fn discover_models(&self) -> Result<Vec<String>, LlmError> {
        self.get_active_provider().discover_models().await
    }

    async fn verify_connection(&self) -> Result<(), LlmError> {
        self.get_active_provider().verify_connection().await
    }

    async fn cancel(&self, session_id: &str) -> Result<(), LlmError> {
        self.get_active_provider().cancel(session_id).await
    }

    fn tools(&self) -> Vec<models::ToolDefinition> {
        self.get_active_provider().tools()
    }

    async fn get_mcp_tools(&self) -> Result<Vec<models::McpToolDefinition>, LlmError> {
        self.get_active_provider().get_mcp_tools().await
    }

    async fn execute_reasoning(
        &self,
        graph: models::ReasoningGraph,
    ) -> Result<HashMap<String, serde_json::Value>, LlmError> {
        self.get_active_provider().execute_reasoning(graph).await
    }

    async fn execute_reasoning_streaming(
        &self,
        graph: models::ReasoningGraph,
        tx: Sender<serde_json::Value>,
    ) -> Result<(), LlmError> {
        self.get_active_provider()
            .execute_reasoning_streaming(graph, tx)
            .await
    }

    async fn execute_pipeline(
        &self,
        pipeline: serde_json::Value,
        question: String,
    ) -> Result<models::PipelineExecuteResult, LlmError> {
        self.get_active_provider().execute_pipeline(pipeline, question).await
    }

    fn default_model(&self) -> String {

        self.get_active_provider().default_model()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// A registry or factory trait to initialize providers from config.
pub struct ProviderFactory;

impl ProviderFactory {
    pub fn create_all(config: &AppConfig) -> Arc<dyn LlmProvider> {
        let mut providers: HashMap<String, Arc<dyn LlmProvider>> = HashMap::new();

        if let Some(cfg) = &config.llm.openai {
            providers.insert(
                "openai".to_string(),
                Arc::new(OpenAiProvider::new(
                    cfg.api_key.clone(),
                    cfg.api_base.clone(),
                    cfg.default_model.clone(),
                )),
            );
        }

        if let Some(cfg) = &config.llm.anthropic {
            providers.insert(
                "anthropic".to_string(),
                Arc::new(AnthropicProvider::new(
                    cfg.api_key.clone(),
                    cfg.api_base.clone(),
                    cfg.default_model.clone(),
                )),
            );
        }

        if let Some(cfg) = &config.llm.ollama {
            providers.insert(
                "ollama".to_string(),
                Arc::new(OllamaProvider::new(
                    cfg.base_url.clone(),
                    cfg.default_model.clone(),
                )),
            );
        }

        if let Some(cfg) = &config.llm.copilot {
            providers.insert(
                "copilot".to_string(),
                Arc::new(CopilotProvider::new(
                    cfg.api_key.clone(),
                    cfg.api_base.clone(),
                    cfg.default_model.clone(),
                )),
            );
        }

        if let Some(cfg) = &config.llm.llmos {
            providers.insert(
                "llmos".to_string(),
                Arc::new(LlmosProvider::new(
                    cfg.base_url.clone(),
                    cfg.default_model.clone(),
                    cfg.api_key.clone(),
                )),
            );
        }

        let default_id = config.llm.provider.clone();
        Arc::new(ProviderManager::new(providers, default_id))
    }

    pub fn create_default(config: &AppConfig) -> Option<Arc<dyn LlmProvider>> {
        Some(Self::create_all(config))
    }
}

/// Helper method to extract a JSON tool call array from a raw text stream buffer.
/// It looks for `[{"name": "...", "arguments": ...}]` and returns the parsed ToolCall
/// along with the remaining text before the JSON started.
pub fn extract_streaming_tool_call(buffer: &str) -> Option<(Vec<models::ToolCall>, String)> {
    let mut search_start = 0;
    while let Some(start_pos) = buffer[search_start..].find('[') {
        let absolute_start = search_start + start_pos;
        let json_part = &buffer[absolute_start..];
        
        let mut bracket_count = 0;
        let mut end_pos = None;
        let mut in_string = false;
        let mut escape_next = false;

        for (i, c) in json_part.chars().enumerate() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match c {
                '\\' => escape_next = true,
                '"' => in_string = !in_string,
                '[' if !in_string => bracket_count += 1,
                ']' if !in_string => {
                    bracket_count -= 1;
                    if bracket_count == 0 {
                        end_pos = Some(i + 1);
                        break;
                    }
                }
                _ => {}
            }
        }

        if let Some(len) = end_pos {
            let potential_json = &json_part[..len];
            if let Ok(parsed_functions) = serde_json::from_str::<Vec<serde_json::Value>>(potential_json) {
                let mapped_tools: Vec<models::ToolCall> = parsed_functions.into_iter().filter_map(|f| {
                    let func_obj = if f.get("function").is_some() { &f["function"] } else { &f };
                    
                    let name = func_obj.get("name")?.as_str()?.to_string();
                    let args = if let Some(a) = func_obj.get("arguments") {
                        if a.is_string() {
                            a.as_str()?.to_string()
                        } else {
                            a.to_string()
                        }
                    } else {
                        "{}".to_string()
                    };
                    
                    Some(models::ToolCall {
                        id: f.get("id").and_then(|id| id.as_str()).map(|s| s.to_string()),
                        r#type: Some("function".to_string()),
                        function: models::FunctionCall { name, arguments: args },
                    })
                }).collect();
                
                if !mapped_tools.is_empty() {
                    let text_before = buffer[..absolute_start].trim().to_string();
                    return Some((mapped_tools, text_before));
                }
            }
        }
        
        // If we didn't find a valid tool call here, advance the search offset
        search_start = absolute_start + 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_streaming_tool_call_simple() {
        let text = "Here is the search result.\n\n[{\"name\": \"internet_search\", \"arguments\": {\"query\": \"rust actix\"}}]";
        let (tools, text_before) = extract_streaming_tool_call(text).unwrap();
        
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].function.name, "internet_search");
        assert_eq!(tools[0].function.arguments, "{\"query\":\"rust actix\"}");
        assert_eq!(text_before, "Here is the search result.");
    }

    #[test]
    fn test_extract_streaming_tool_call_with_wrapper() {
        let text = "Searching now...\n[{\"type\": \"function\", \"function\": {\"name\": \"test_tool\", \"arguments\": \"{}\"}}]";
        let (tools, text_before) = extract_streaming_tool_call(text).unwrap();
        
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].function.name, "test_tool");
        assert_eq!(tools[0].function.arguments, "{}");
        assert_eq!(text_before, "Searching now...");
    }

    #[test]
    fn test_extract_streaming_tool_call_incomplete() {
        let text = "Searching...\n[{\"name\": \"test_tool\", \"arg";
        let result = extract_streaming_tool_call(text);
        assert!(result.is_none());
    }
}
