use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct WsClientMessage {
    pub r#type: String, // Expected: "message", "cancel"
    pub content: String,
    pub stream: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct WsServerMessage {
    pub r#type: String, // Expected: "chunk", "done", "error", "status"
    pub content: String,
}
