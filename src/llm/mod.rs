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
    ) -> Result<(), LlmError>;

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
    ) -> Result<(), LlmError> {
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
