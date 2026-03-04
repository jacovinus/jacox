use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryUsageEntry {
    pub tag: String,
    pub usage_bytes: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStats {
    pub total_sessions: i64,
    pub total_messages: i64,
    pub total_tokens: i64,
    pub db_size_bytes: u64,
    pub memory_usage: Vec<MemoryUsageEntry>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub name: String,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct CreateMessageRequest {
    pub role: String,
    pub content: String,
    pub model: Option<String>,
    pub token_count: Option<i32>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default = "default_offset")]
    pub offset: usize,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSessionRequest {
    pub name: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

fn default_limit() -> usize {
    50
}

fn default_offset() -> usize {
    0
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ActiveProviderRequest {
    pub provider_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ActiveModelRequest {
    pub model_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProviderInfo {
    pub id: String,
    pub active: bool,
    pub supported_models: Vec<String>,
    pub status: String, // "online", "offline", "unverified"
}
