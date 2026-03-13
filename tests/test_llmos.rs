#[cfg(test)]
mod tests {
    use jacox::llm::llmos::LlmosProvider;
    use jacox::llm::{
        models::{ChatOptions, Message},
        LlmProvider,
    };
    use serde_json::json;
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_llmos_chat() {
        let mock_server = MockServer::start().await;
        let token = "test-token".to_string();
        let provider = LlmosProvider::new(mock_server.uri(), "phi-4".to_string(), Some(token.clone()));

        let messages = vec![Message {
            role: "user".to_string(),
            content: "Hello".to_string(),
            tool_calls: None,
            tool_call_id: None,
        }];

        let expected_body = json!({
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
}
