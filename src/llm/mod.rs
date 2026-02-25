pub mod anthropic;
pub mod models;
pub mod ollama;
pub mod openai;

use anthropic::AnthropicProvider;
use ollama::OllamaProvider;
use openai::OpenAiProvider;

use async_trait::async_trait;
use thiserror::Error;
use tokio::sync::mpsc::Sender;
use std::sync::Arc;

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
    
    async fn chat(&self, messages: &[Message], options: ChatOptions) -> Result<ChatResponse, LlmError>;
    
    async fn chat_streaming(
        &self, 
        messages: &[Message], 
        options: ChatOptions,
        tx: Sender<String>
    ) -> Result<(), LlmError>;
    
    fn supported_models(&self) -> Vec<&str>;
}

/// A registry or factory trait to initialize providers from config.
pub struct ProviderFactory;

impl ProviderFactory {
    pub fn create_default(config: &AppConfig) -> Option<Arc<dyn LlmProvider>> {
        let provider_name = config.llm.provider.as_str();
        
        match provider_name {
            "openai" => {
                let cfg = config.llm.openai.as_ref()?;
                Some(Arc::new(OpenAiProvider::new(
                    cfg.api_key.clone(),
                    cfg.api_base.clone(),
                    cfg.default_model.clone(),
                )))
            }
            "anthropic" => {
                let cfg = config.llm.anthropic.as_ref()?;
                Some(Arc::new(AnthropicProvider::new(
                    cfg.api_key.clone(),
                    cfg.api_base.clone(),
                    cfg.default_model.clone(),
                )))
            }
            "ollama" => {
                let cfg = config.llm.ollama.as_ref()?;
                Some(Arc::new(OllamaProvider::new(
                    cfg.base_url.clone(),
                    cfg.default_model.clone(),
                )))
            }
            _ => None,
        }
    }
}
