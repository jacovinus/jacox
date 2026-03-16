#[cfg(test)]
mod tests {
    use stepbit::llm::stepbit_core::StepbitCoreProvider;
    use stepbit::llm::{
        models::{ChatOptions, Message},
        LlmProvider,
    };
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_llmos_chat() {
        let mock_server = MockServer::start().await;
        let token = "test-token".to_string();
        let provider = StepbitCoreProvider::new(mock_server.uri(), "phi-4".to_string(), Some(token.clone()));

        let messages = vec![Message {
            role: "user".to_string(),
            content: "Hello".to_string(),
            tool_calls: None,
            tool_call_id: None,
        }];

        // Verify request body if needed
        let _expected_body = json!({
            "model": "phi-4",
            "messages": messages,
            "stream": false,
            "max_tokens": 4096,
            "temperature": 0.7,
        });

        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .and(wiremock::matchers::header("Authorization", format!("Bearer {}", token).as_str()))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "chatcmpl-123",
                "object": "chat.completion",
                "created": 123456789,
                "model": "phi-4",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Hi there!"
                    },
                    "finish_reason": "stop"
                }],
                "usage": {
                    "prompt_tokens": 10,
                    "completion_tokens": 5,
                    "total_tokens": 15
                }
            })))
            .mount(&mock_server)
            .await;

        let response = provider
            .chat(&messages, ChatOptions::default())
            .await
            .unwrap();
        assert_eq!(response.content, "Hi there!");
        assert_eq!(response.model, "phi-4");
    }

    #[tokio::test]
    async fn test_llmos_token_rotation() {
        let mock_server = MockServer::start().await;
        let master_token = "master-key".to_string();
        let provider = StepbitCoreProvider::new(mock_server.uri(), "phi-4".to_string(), Some(master_token.clone()));

        // 1. Initial request with master token, returns a rotating token
        let next_token = "rotating-123";
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .and(wiremock::matchers::header("Authorization", format!("Bearer {}", master_token).as_str()))
            .respond_with(ResponseTemplate::new(200)
                .insert_header("X-Next-Token", next_token)
                .set_body_json(json!({
                    "choices": [{"message": {"content": "First response"}}]
                })))
            .mount(&mock_server)
            .await;

        let messages = vec![Message {
            role: "user".to_string(),
            content: "Hello".to_string(),
            tool_calls: None,
            tool_call_id: None,
        }];
        
        provider.chat(&messages, ChatOptions::default()).await.unwrap();

        // 2. Second request should now use the rotating token
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .and(wiremock::matchers::header("Authorization", format!("Bearer {}", next_token).as_str()))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json!({
                    "choices": [{"message": {"content": "Second response"}}]
                })))
            .mount(&mock_server)
            .await;

        let response = provider.chat(&messages, ChatOptions::default()).await.unwrap();
        assert_eq!(response.content, "Second response");
    }

    #[tokio::test]
    async fn test_llmos_get_mcp_tools() {
        let mock_server = MockServer::start().await;
        let provider = StepbitCoreProvider::new(mock_server.uri(), "phi-4".to_string(), None);

        Mock::given(method("GET"))
            .and(path("/v1/mcp/tools"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "tools": [
                    {
                        "name": "calc",
                        "description": "Calculator",
                        "input_schema": {}
                    }
                ]
            })))
            .mount(&mock_server)
            .await;

        let tools = provider.get_mcp_tools().await.unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "calc");
    }

    #[tokio::test]
    async fn test_llmos_execute_reasoning() {
        let mock_server = MockServer::start().await;
        let provider = StepbitCoreProvider::new(mock_server.uri(), "phi-4".to_string(), None);

        Mock::given(method("POST"))
            .and(path("/v1/reasoning/execute"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "results": {
                    "node1": {"value": 42}
                }
            })))
            .mount(&mock_server)
            .await;

        use stepbit::llm::models::ReasoningGraph;
        let graph = ReasoningGraph::default();
        let results = provider.execute_reasoning(graph).await.unwrap();
        
        assert_eq!(results["node1"]["value"], 42);
    }
}
