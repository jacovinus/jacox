#[cfg(test)]
mod tests {
    use stepbit::tools::search::SearchTool;
    use stepbit::tools::Tool;

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
        let session_id = uuid::Uuid::new_v4();
        
        // Create a temporary in-memory DuckDB for testing
        let conn = duckdb::Connection::open_in_memory().unwrap();
        // Initialize schema
        conn.execute_batch(r#"
            CREATE SEQUENCE seq_tool_results_id;
            CREATE TABLE tool_results (
                id BIGINT PRIMARY KEY DEFAULT nextval('seq_tool_results_id'),
                session_id UUID,
                source_url VARCHAR,
                content TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
        "#).unwrap();
        let pool = std::sync::Arc::new(std::sync::Mutex::new(conn));

        let result = tool.call(r#"{"query": "rust programming language"}"#, session_id, pool).await;
        
        println!("Search Result Snippet: {}", result.chars().take(200).collect::<String>());
        assert!(!result.contains("Error"));
        assert!(!result.is_empty());
        assert!(result.contains("Source ID:"));
    }
}
