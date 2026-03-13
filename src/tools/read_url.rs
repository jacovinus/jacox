use async_trait::async_trait;
use crate::llm::models::{FunctionDefinition, ToolDefinition};
use crate::tools::Tool;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{error, info};

pub struct ReadUrlTool {
    client: Client,
}

#[derive(Serialize, Deserialize)]
struct ReadUrlArguments {
    url: String,
}

impl ReadUrlTool {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap(),
        }
    }
}

#[async_trait]
impl Tool for ReadUrlTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            r#type: "function".to_string(),
            function: FunctionDefinition {
                name: "read_url".to_string(),
                description: "Fetch and read the full markdown content of a webpage from a given URL.".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "The exact URL of the webpage to read."
                        }
                    },
                    "required": ["url"]
                }),
            },
        }
    }

    async fn call(&self, arguments: &str, session_id: uuid::Uuid, pool: crate::db::DbPool) -> String {
        let args: ReadUrlArguments = match serde_json::from_str(arguments) {
            Ok(a) => a,
            Err(e) => return format!("Error parsing arguments: {}", e),
        };

        let url = args.url.trim();
        info!("Scraping page from read_url tool: {}", url);
        
        // Validate URL
        if !url.starts_with("http") {
            return format!("Error: '{}' is not a valid HTTP/HTTPS URL.", url);
        }

        let response = match self.client.get(url).send().await {
            Ok(res) => res,
            Err(e) => return format!("Error fetching {}: {}", url, e),
        };

        if !response.status().is_success() {
            return format!("Error fetching {}: Status {}", url, response.status());
        }

        let html = response.text().await.unwrap_or_default();
        
        let mut cursor = std::io::Cursor::new(html.clone());
        let base_url = match reqwest::Url::parse(url) {
            Ok(u) => u,
            Err(_) => return "Invalid URL format".to_string(),
        };

        let result = llm_readability::extractor::extract(&mut cursor, &base_url);
        
        let full_content = match result {
            Ok(product) => {
                let markdown = html_to_markdown_rs::convert(&product.content, None).unwrap_or_else(|_| product.content.clone());
                format!("Source: {}\nContent:\n{}", url, markdown)
            }
            Err(_) => {
                let document = Html::parse_document(&html);
                let body_selector = Selector::parse("body").unwrap();
                let text = document
                    .select(&body_selector)
                    .next()
                    .map(|e| e.text().collect::<Vec<_>>().join(" "))
                    .unwrap_or_default();
                format!("Source: {}\n(Readability failed, raw text follows)\n{}", url, text.chars().take(4000).collect::<String>())
            }
        };

        // Cache full content in DuckDB so it's available via search history if needed
        let cache_result = {
            let conn = pool.lock().unwrap();
            crate::db::service::DbService::insert_tool_result(&conn, session_id, url, &full_content)
        };
        
        if let Err(e) = cache_result {
            error!("Failed to cache read_url tool result: {}", e);
        }

        full_content
    }
}
