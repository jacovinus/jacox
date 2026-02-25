use serde::Deserialize;

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

fn default_limit() -> usize {
    50
}

fn default_offset() -> usize {
    0
}
