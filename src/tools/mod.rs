pub mod search;
pub mod read_full_content;

use async_trait::async_trait;
use crate::llm::models::ToolDefinition;

#[async_trait]
pub trait Tool: Send + Sync {
    fn definition(&self) -> ToolDefinition;
    async fn call(&self, arguments: &str, session_id: uuid::Uuid, pool: crate::db::DbPool) -> String;
}

pub struct ToolRegistry {
    pub tools: Vec<Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: vec![
                Box::new(search::SearchTool::new()),
                Box::new(read_full_content::ReadFullContentTool::new()),
            ],
        }
    }

    pub fn get_definitions(&self) -> Vec<ToolDefinition> {
        self.tools.iter().map(|t| t.definition()).collect()
    }

    pub async fn call_tool(&self, name: &str, arguments: &str, session_id: uuid::Uuid, pool: crate::db::DbPool) -> String {
        for tool in &self.tools {
            if tool.definition().function.name == name {
                return tool.call(arguments, session_id, pool).await;
            }
        }
        format!("Error: Tool '{}' not found", name)
    }
}
