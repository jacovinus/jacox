use async_trait::async_trait;
use crate::llm::models::{FunctionDefinition, ToolDefinition};
use crate::tools::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub struct ReadFullContentTool;

#[derive(Serialize, Deserialize)]
struct ReadArguments {
    source_id: i64,
}

impl ReadFullContentTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for ReadFullContentTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            r#type: "function".to_string(),
            function: FunctionDefinition {
                name: "read_full_content".to_string(),
                description: "Read the full content of a previously cached search result or document.".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "source_id": {
                            "type": "integer",
                            "description": "The ID of the source to read, as provided in the search snippets."
                        }
                    },
                    "required": ["source_id"]
                }),
            },
        }
    }

    async fn call(&self, arguments: &str, _session_id: uuid::Uuid, pool: crate::db::DbPool) -> String {
        let args: ReadArguments = match serde_json::from_str(arguments) {
            Ok(a) => a,
            Err(e) => return format!("Error parsing arguments: {}", e),
        };

        let conn = pool.lock().unwrap();
        match crate::db::service::DbService::get_tool_result(&conn, args.source_id) {
            Ok(Some(res)) => {
                format!("Source URL: {}\nFull Content:\n{}", res.source_url, res.content)
            }
            Ok(None) => format!("Error: Source with ID {} not found in cache.", args.source_id),
            Err(e) => format!("Error retrieving source: {}", e),
        }
    }
}
