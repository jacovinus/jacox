use async_trait::async_trait;
use crate::llm::models::{FunctionDefinition, ToolDefinition};
use crate::tools::Tool;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{error, info};

pub struct SearchTool {
    client: Client,
}

#[derive(Serialize, Deserialize)]
struct SearchArguments {
    query: String,
}

impl SearchTool {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
                .build()
                .unwrap(),
        }
    }

    async fn scrape_search_results(&self, query: &str) -> Vec<String> {
        let url = format!("https://html.duckduckgo.com/html/?q={}", urlencoding::encode(query));
        let response = match self.client.get(url).send().await {
            Ok(res) => res,
            Err(e) => {
                error!("Failed to fetch search results: {}", e);
                return vec![];
            }
        };

        let html_content = response.text().await.unwrap_or_default();
        let document = Html::parse_document(&html_content);
        let selector = Selector::parse(".result__a").unwrap();

        document
            .select(&selector)
            .filter_map(|element| {
                let mut href = element.value().attr("href")?.to_string();
                
                // DuckDuckGo often uses redirects like /l/?uddg=URL
                if href.contains("uddg=") {
                    if let Some(pos) = href.find("uddg=") {
                        let encoded_url = &href[pos + 5..];
                        if let Some(end_pos) = encoded_url.find('&') {
                            href = urlencoding::decode(&encoded_url[..end_pos]).ok()?.to_string();
                        } else {
                            href = urlencoding::decode(encoded_url).ok()?.to_string();
                        }
                    }
                }

                if href.contains("duckduckgo.com") || href.starts_with('/') {
                    None
                } else {
                    Some(href)
                }
            })
            .take(3) // Top 3 results
            .collect()
    }

    async fn scrape_page_content(&self, url: &str) -> String {
        info!("Scraping page: {}", url);
        let response = match self.client.get(url).send().await {
            Ok(res) => res,
            Err(e) => return format!("Error fetching {}: {}", url, e),
        };

        if !response.status().is_success() {
            return format!("Error fetching {}: Status {}", url, response.status());
        }

        let html = response.text().await.unwrap_or_default();
        
        // Use llm_readability to extract main content
        let mut cursor = std::io::Cursor::new(html.clone());
        let base_url = match reqwest::Url::parse(url) {
            Ok(u) => u,
            Err(_) => return "Invalid URL".to_string(),
        };

        let result = llm_readability::extractor::extract(&mut cursor, &base_url);
        
        match result {
            Ok(product) => {
                // Convert extracted HTML to Markdown
                // Using html_to_markdown_rs::convert based on docs
                let markdown = html_to_markdown_rs::convert(&product.content, None).unwrap_or_else(|_| product.content.clone());
                format!("Source: {}\nContent:\n{}", url, markdown)
            }
            Err(_) => {
                // Fallback to simple text extraction if readability fails
                let document = Html::parse_document(&html);
                let body_selector = Selector::parse("body").unwrap();
                let text = document
                    .select(&body_selector)
                    .next()
                    .map(|e| e.text().collect::<Vec<_>>().join(" "))
                    .unwrap_or_default();
                format!("Source: {}\n(Readability failed, raw text follows)\n{}", url, text.chars().take(2000).collect::<String>())
            }
        }
    }
}

#[async_trait]
impl Tool for SearchTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            r#type: "function".to_string(),
            function: FunctionDefinition {
                name: "internet_search".to_string(),
                description: "Search the internet for real-time information or specific topics.".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "The search query to look up on the web."
                        }
                    },
                    "required": ["query"]
                }),
            },
        }
    }

    async fn call(&self, arguments: &str) -> String {
        let args: SearchArguments = match serde_json::from_str(arguments) {
            Ok(a) => a,
            Err(e) => return format!("Error parsing arguments: {}", e),
        };

        info!("Performing internet search for: {}", args.query);
        let urls = self.scrape_search_results(&args.query).await;

        if urls.is_empty() {
            return "No results found for that query.".to_string();
        }

        let mut combined_content = String::new();
        for url in urls {
            let content = self.scrape_page_content(&url).await;
            combined_content.push_str(&content);
            combined_content.push_str("\n\n---\n\n");
        }

        combined_content
    }
}
