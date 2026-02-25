use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub api_keys: Vec<String>,
    pub token_expiry_hours: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OpenAiConfig {
    pub api_base: String,
    pub api_key: String,
    pub default_model: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AnthropicConfig {
    pub api_base: String,
    pub api_key: String,
    pub default_model: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OllamaConfig {
    pub base_url: String,
    pub default_model: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LlmConfig {
    pub provider: String,
    pub model: String,
    pub openai: Option<OpenAiConfig>,
    pub anthropic: Option<AnthropicConfig>,
    pub ollama: Option<OllamaConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChatConfig {
    pub max_history_messages: u32,
    pub system_prompt: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub llm: LlmConfig,
    pub chat: ChatConfig,
}

impl AppConfig {
    pub fn load(path: &str) -> Result<Self, config::ConfigError> {
        dotenv::dotenv().ok();
        
        let settings = config::Config::builder()
            .add_source(config::File::with_name(path).required(false))
            .add_source(config::Environment::with_prefix("JACOX").separator("__"))
            .build()?;
            
        let mut app_config: AppConfig = settings.try_deserialize()?;
        
        // Expand environment variables if present like ${OPENAI_API_KEY}
        app_config.server.host = expand_env(&app_config.server.host);
        app_config.database.path = expand_env(&app_config.database.path);
        
        if let Some(ref mut openai) = app_config.llm.openai {
            openai.api_key = expand_env(&openai.api_key);
        }
        if let Some(ref mut anthropic) = app_config.llm.anthropic {
            anthropic.api_key = expand_env(&anthropic.api_key);
        }
        
        Ok(app_config)
    }
}

fn expand_env(val: &str) -> String {
    if val.starts_with("${") && val.ends_with('}') {
        let var_name = &val[2..val.len()-1];
        std::env::var(var_name).unwrap_or_else(|_| "".to_string())
    } else {
        val.to_string()
    }
}
