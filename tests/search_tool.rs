#[cfg(test)]
mod tests {
    use jacox::tools::search::SearchTool;
    use jacox::tools::Tool;

    #[tokio::test]
    async fn test_search_tool_definition() {
        let tool = SearchTool::new();
        let def = tool.definition();
        assert_eq!(def.function.name, "internet_search");
    }

    #[tokio::test]
    async fn test_search_tool_execution() {
        // This test requires internet access. 
        // We'll just check if it returns something instead of an error for a basic query.
        let tool = SearchTool::new();
        let result = tool.call(r#"{"query": "rust programming language"}"#).await;
        
        println!("Search Result Snippet: {}", result.chars().take(200).collect::<String>());
        assert!(!result.contains("Error"));
        assert!(!result.is_empty());
    }
}
