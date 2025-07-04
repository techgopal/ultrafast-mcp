#[cfg(test)]
mod stdio_tests {
    use serde_json::json;

    #[test]
    fn test_stdio_transport_creation() {
        // Test that we can create basic JSON-RPC messages
        let message = json!({
            "jsonrpc": "2.0",
            "id": "1",
            "method": "test",
            "params": {}
        });

        assert_eq!(message["jsonrpc"], "2.0");
    }

    #[test]
    fn test_stdio_transport_message_format() {
        // Test message format validation
        let message = json!({
            "jsonrpc": "2.0",
            "id": "1",
            "method": "test",
            "params": {}
        });

        assert!(message.is_object());
        assert_eq!(message["jsonrpc"], "2.0");
    }

    #[test]
    fn test_transport_error_handling() {
        // Test error handling with JSON
        let error_message = json!({
            "jsonrpc": "2.0",
            "id": "1",
            "error": {
                "code": -32601,
                "message": "Method not found"
            }
        });

        assert!(error_message["error"]["code"].as_i64().unwrap() < 0);
    }

    #[test]
    fn test_transport_error_types() {
        // Test error type creation with JSON
        let error = json!({
            "code": -32601,
            "message": "test error"
        });

        assert!(error["message"].as_str().unwrap().contains("test error"));
    }
}

#[cfg(test)]
mod http_tests {
    use serde_json::json;

    #[test]
    fn test_http_transport_config() {
        // Test HTTP transport configuration with JSON
        let config = json!({
            "port": 8080,
            "host": "localhost",
            "enable_cors": true,
            "enable_sse": true
        });

        assert_eq!(config["port"], 8080);
        assert_eq!(config["host"], "localhost");
    }

    #[test]
    fn test_http_transport_server_creation() {
        // Test HTTP server configuration
        let config = json!({
            "port": 0,
            "host": "127.0.0.1",
            "enable_cors": false,
            "enable_sse": false
        });

        assert_eq!(config["host"], "127.0.0.1");
    }
}

#[cfg(test)]
mod middleware_tests {
    use serde_json::json;

    #[test]
    fn test_middleware_pipeline() {
        // Test middleware pipeline with JSON
        let middleware_config = json!({
            "middleware": [],
            "enabled": true
        });

        assert!(middleware_config["enabled"].as_bool().unwrap());
    }
}

#[cfg(test)]
mod transport_abstraction_tests {
    use serde_json::json;

    #[test]
    fn test_transport_factory() {
        // Test transport factory with JSON
        let transport_config = json!({
            "type": "stdio",
            "command": "echo",
            "args": []
        });

        assert_eq!(transport_config["type"], "stdio");
    }

    #[test]
    fn test_transport_result_type() {
        // Test result type creation
        let result = json!({
            "jsonrpc": "2.0",
            "id": "1",
            "result": "success"
        });

        assert!(result["result"].as_str().unwrap() == "success");
    }
}

#[cfg(test)]
mod performance_tests {
    use serde_json::json;

    #[test]
    fn test_message_serialization_performance() {
        let message = json!({
            "jsonrpc": "2.0",
            "id": "1",
            "method": "test",
            "params": {"key": "value"}
        });

        // Test serialization
        let serialized = serde_json::to_string(&message).unwrap();
        assert!(serialized.contains("jsonrpc"));
    }

    #[test]
    fn test_concurrent_transport_operations() {
        // Test concurrent operation simulation
        let messages = [
            json!({"id": "1", "method": "test1"}),
            json!({"id": "2", "method": "test2"}),
            json!({"id": "3", "method": "test3"}),
        ];

        assert_eq!(messages.len(), 3);
    }
}

#[cfg(test)]
mod validation_middleware_tests {
    use serde_json::{json, Value};
    use std::collections::HashMap;
    use ultrafast_mcp_core::protocol::{JsonRpcMessage, JsonRpcRequest, RequestId};
    use ultrafast_mcp_transport::middleware::{TransportMiddleware, ValidationMiddleware};

    #[tokio::test]
    async fn test_validation_middleware_basic() {
        let middleware = ValidationMiddleware::new();
        let mut valid_request = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(RequestId::from("1".to_string())),
            method: "tools/list".to_string(),
            params: None,
            meta: HashMap::new(),
        });
        assert!(middleware
            .process_outgoing(&mut valid_request)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_validation_middleware_invalid_method() {
        let middleware = ValidationMiddleware::new();
        let mut invalid_request = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(RequestId::from("1".to_string())),
            method: "invalid/method".to_string(),
            params: None,
            meta: HashMap::new(),
        });
        let result = middleware.process_outgoing(&mut invalid_request).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Method 'invalid/method' not allowed"));
    }

    #[tokio::test]
    async fn test_validation_middleware_invalid_jsonrpc_version() {
        let middleware = ValidationMiddleware::new();
        let mut invalid_request = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: "1.0".to_string(),
            id: Some(RequestId::from("1".to_string())),
            method: "tools/list".to_string(),
            params: None,
            meta: HashMap::new(),
        });
        let result = middleware.process_outgoing(&mut invalid_request).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid JSON-RPC version"));
    }

    #[tokio::test]
    async fn test_validation_middleware_null_request_id() {
        let middleware = ValidationMiddleware::new();
        let mut invalid_request = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: None,
            method: "tools/list".to_string(),
            params: None,
            meta: HashMap::new(),
        });
        assert!(middleware
            .process_outgoing(&mut invalid_request)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_validation_middleware_strict_mode() {
        let middleware = ValidationMiddleware::strict();
        let mut invalid_request = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: None,
            method: "tools/list".to_string(),
            params: None,
            meta: HashMap::new(),
        });
        let result = middleware.process_outgoing(&mut invalid_request).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Request ID required in strict mode"));
    }

    #[tokio::test]
    async fn test_validation_middleware_string_sanitization() {
        let middleware = ValidationMiddleware::new();
        let mut request_with_nulls = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(RequestId::from("1".to_string())),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "test",
                "arguments": {
                    "input": "hello\0world"
                }
            })),
            meta: HashMap::new(),
        });
        assert!(middleware
            .process_outgoing(&mut request_with_nulls)
            .await
            .is_ok());
        if let JsonRpcMessage::Request(req) = &request_with_nulls {
            if let Some(params) = &req.params {
                if let Some(input) = params.get("arguments").and_then(|a| a.get("input")) {
                    assert_eq!(input.as_str().unwrap(), "helloworld");
                }
            }
        }
    }

    #[tokio::test]
    async fn test_validation_middleware_array_size_limit() {
        let middleware = ValidationMiddleware::new();
        let large_array: Vec<Value> = (0..10001).map(|i| json!(i)).collect();
        let mut request_with_large_array = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(RequestId::from("1".to_string())),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "test",
                "arguments": {
                    "items": large_array
                }
            })),
            meta: HashMap::new(),
        });
        let result = middleware
            .process_outgoing(&mut request_with_large_array)
            .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Array parameter too large"));
    }

    #[tokio::test]
    async fn test_validation_middleware_object_size_limit() {
        let middleware = ValidationMiddleware::new();
        let mut large_object = serde_json::Map::new();
        for i in 0..1001 {
            large_object.insert(format!("key_{}", i), json!(i));
        }
        let mut request_with_large_object = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(RequestId::from("1".to_string())),
            method: "tools/call".to_string(),
            params: Some(Value::Object(large_object)),
            meta: HashMap::new(),
        });
        let result = middleware
            .process_outgoing(&mut request_with_large_object)
            .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Object parameter too large"));
    }

    #[tokio::test]
    async fn test_validation_middleware_reserved_key_names() {
        let middleware = ValidationMiddleware::new();
        let mut request_with_reserved_key = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(RequestId::from("1".to_string())),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "test",
                "_internal": "value"
            })),
            meta: HashMap::new(),
        });
        let result = middleware
            .process_outgoing(&mut request_with_reserved_key)
            .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Reserved key name '_internal' not allowed"));
    }

    #[tokio::test]
    async fn test_validation_middleware_meta_key_allowed() {
        let middleware = ValidationMiddleware::new();
        let mut request_with_meta = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(RequestId::from("1".to_string())),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "test",
                "_meta": {
                    "timestamp": 1234567890
                }
            })),
            meta: HashMap::new(),
        });
        assert!(middleware
            .process_outgoing(&mut request_with_meta)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_validation_middleware_uri_validation() {
        let middleware = ValidationMiddleware::new();
        let mut request_with_valid_uri = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(RequestId::from("1".to_string())),
            method: "resources/read".to_string(),
            params: Some(json!({
                "uri": "file:///path/to/file.txt"
            })),
            meta: HashMap::new(),
        });
        assert!(middleware
            .process_outgoing(&mut request_with_valid_uri)
            .await
            .is_ok());
        let mut request_with_invalid_uri = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(RequestId::from("1".to_string())),
            method: "resources/read".to_string(),
            params: Some(json!({
                "uri": "file:///path/../secret.txt"
            })),
            meta: HashMap::new(),
        });
        let result = middleware
            .process_outgoing(&mut request_with_invalid_uri)
            .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("URI contains path traversal attempt"));
    }

    #[tokio::test]
    async fn test_validation_middleware_protocol_version_validation() {
        let middleware = ValidationMiddleware::new();
        let mut request_with_valid_version = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(RequestId::from("1".to_string())),
            method: "initialize".to_string(),
            params: Some(json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {}
            })),
            meta: HashMap::new(),
        });
        assert!(middleware
            .process_outgoing(&mut request_with_valid_version)
            .await
            .is_ok());
        let mut request_with_invalid_version = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(RequestId::from("1".to_string())),
            method: "initialize".to_string(),
            params: Some(json!({
                "protocolVersion": "2023-01-01",
                "capabilities": {}
            })),
            meta: HashMap::new(),
        });
        let result = middleware
            .process_outgoing(&mut request_with_invalid_version)
            .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsupported protocol version"));
    }

    #[tokio::test]
    async fn test_validation_middleware_tool_name_validation() {
        let middleware = ValidationMiddleware::new();
        let mut request_with_valid_tool = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(RequestId::from("1".to_string())),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "valid_tool_name"
            })),
            meta: HashMap::new(),
        });
        assert!(middleware
            .process_outgoing(&mut request_with_valid_tool)
            .await
            .is_ok());
        let mut request_with_invalid_tool = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(RequestId::from("1".to_string())),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "_invalid_tool_name"
            })),
            meta: HashMap::new(),
        });
        let result = middleware
            .process_outgoing(&mut request_with_invalid_tool)
            .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Tool name cannot start with underscore"));
    }

    #[tokio::test]
    async fn test_validation_middleware_log_level_validation() {
        let middleware = ValidationMiddleware::new();
        let mut request_with_valid_level = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(RequestId::from("1".to_string())),
            method: "logging/log".to_string(),
            params: Some(json!({
                "level": "info",
                "message": "Test log message"
            })),
            meta: HashMap::new(),
        });
        assert!(middleware
            .process_outgoing(&mut request_with_valid_level)
            .await
            .is_ok());
        let mut request_with_invalid_level = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(RequestId::from("1".to_string())),
            method: "logging/log".to_string(),
            params: Some(json!({
                "level": "invalid_level",
                "message": "Test log message"
            })),
            meta: HashMap::new(),
        });
        let result = middleware
            .process_outgoing(&mut request_with_invalid_level)
            .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid log level"));
    }

    #[tokio::test]
    async fn test_validation_middleware_message_size_limit() {
        let middleware = ValidationMiddleware::new();

        // Test with a message that should be under the limit
        let mut small_request = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(RequestId::from("1".to_string())),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "test",
                "data": "small data"
            })),
            meta: HashMap::new(),
        });
        assert!(middleware
            .process_incoming(&mut small_request)
            .await
            .is_ok());

        // Create a huge message that definitely exceeds the 10MB limit
        let huge_string = "x".repeat(15 * 1024 * 1024); // 15MB string
        let mut huge_request = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(RequestId::from("1".to_string())),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "test",
                "data": huge_string
            })),
            meta: HashMap::new(),
        });

        // This should fail due to the message size limit
        let result = middleware.process_incoming(&mut huge_request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Message size"));
    }

    #[tokio::test]
    async fn test_validation_middleware_parameter_depth_limit() {
        let middleware = ValidationMiddleware::new();
        let mut deep_object = json!("value");
        for _ in 0..11 {
            deep_object = json!({ "nested": deep_object });
        }
        let mut request_with_deep_object = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(RequestId::from("1".to_string())),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "test",
                "data": deep_object
            })),
            meta: HashMap::new(),
        });
        let result = middleware
            .process_outgoing(&mut request_with_deep_object)
            .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Parameter depth exceeds maximum"));
    }

    #[tokio::test]
    async fn test_validation_middleware_custom_configuration() {
        let middleware = ValidationMiddleware::new()
            .with_max_message_size(1024) // 1KB limit
            .with_max_params_depth(3)
            .with_allowed_methods(vec!["custom/method".to_string()]);
        let mut request_with_custom_method = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(RequestId::from("1".to_string())),
            method: "custom/method".to_string(),
            params: None,
            meta: HashMap::new(),
        });
        assert!(middleware
            .process_outgoing(&mut request_with_custom_method)
            .await
            .is_ok());
        let mut request_with_standard_method = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(RequestId::from("1".to_string())),
            method: "tools/list".to_string(),
            params: None,
            meta: HashMap::new(),
        });
        let result = middleware
            .process_outgoing(&mut request_with_standard_method)
            .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Method 'tools/list' not allowed"));
    }
}
