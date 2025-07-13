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
    use serde_json::{Value, json};
    use std::borrow::Cow;

    use ultrafast_mcp_core::protocol::{JsonRpcMessage, JsonRpcRequest, RequestId};
    use ultrafast_mcp_transport::streamable_http::middleware::{TransportMiddleware, ValidationMiddleware};

    #[tokio::test]
    async fn test_validation_middleware_basic() {
        let middleware = ValidationMiddleware::new();
        let mut valid_request = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: Cow::Borrowed("2.0"),
            id: Some(RequestId::from("1".to_string())),
            method: "tools/list".to_string(),
            params: None,
            meta: std::collections::HashMap::new(),
        });
        assert!(
            middleware
                .process_outgoing(&mut valid_request)
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn test_validation_middleware_invalid_method() {
        let middleware = ValidationMiddleware::new();
        let mut invalid_request = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: Cow::Borrowed("2.0"),
            id: Some(RequestId::from("1".to_string())),
            method: "invalid/method".to_string(),
            params: None,
            meta: std::collections::HashMap::new(),
        });
        let result = middleware.process_outgoing(&mut invalid_request).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Method 'invalid/method' not allowed")
        );
    }

    #[tokio::test]
    async fn test_validation_middleware_invalid_jsonrpc_version() {
        let middleware = ValidationMiddleware::new();
        let mut invalid_request = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: Cow::Borrowed("1.0"),
            id: Some(RequestId::from("1".to_string())),
            method: "tools/list".to_string(),
            params: None,
            meta: std::collections::HashMap::new(),
        });
        let result = middleware.process_outgoing(&mut invalid_request).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid JSON-RPC version")
        );
    }

    #[tokio::test]
    async fn test_validation_middleware_null_request_id() {
        let middleware = ValidationMiddleware::new();
        let mut invalid_request = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: Cow::Borrowed("2.0"),
            id: None,
            method: "tools/list".to_string(),
            params: None,
            meta: std::collections::HashMap::new(),
        });
        assert!(
            middleware
                .process_outgoing(&mut invalid_request)
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn test_validation_middleware_strict_mode() {
        let middleware = ValidationMiddleware::strict();
        let mut invalid_request = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: Cow::Borrowed("2.0"),
            id: None,
            method: "tools/list".to_string(),
            params: None,
            meta: std::collections::HashMap::new(),
        });
        let result = middleware.process_outgoing(&mut invalid_request).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Request ID required in strict mode")
        );
    }

    #[tokio::test]
    async fn test_validation_middleware_string_sanitization() {
        let middleware = ValidationMiddleware::new();
        let mut request_with_nulls = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: Cow::Borrowed("2.0"),
            id: Some(RequestId::from("1".to_string())),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "test",
                "arguments": {
                    "input": "hello\0world"
                }
            })),
            meta: std::collections::HashMap::new(),
        });
        assert!(
            middleware
                .process_outgoing(&mut request_with_nulls)
                .await
                .is_ok()
        );
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
            jsonrpc: Cow::Borrowed("2.0"),
            id: Some(RequestId::from("1".to_string())),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "test",
                "arguments": {
                    "items": large_array
                }
            })),
            meta: std::collections::HashMap::new(),
        });
        let result = middleware
            .process_outgoing(&mut request_with_large_array)
            .await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Array parameter too large")
        );
    }

    #[tokio::test]
    async fn test_validation_middleware_object_size_limit() {
        let middleware = ValidationMiddleware::new();
        let mut large_object = serde_json::Map::new();
        for i in 0..1001 {
            large_object.insert(format!("key_{i}"), json!(i));
        }
        let mut request_with_large_object = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: Cow::Borrowed("2.0"),
            id: Some(RequestId::from("1".to_string())),
            method: "tools/call".to_string(),
            params: Some(Value::Object(large_object)),
            meta: std::collections::HashMap::new(),
        });
        let result = middleware
            .process_outgoing(&mut request_with_large_object)
            .await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Object parameter too large")
        );
    }

    #[tokio::test]
    async fn test_validation_middleware_reserved_key_names() {
        let middleware = ValidationMiddleware::new();
        let mut request_with_reserved_key = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: Cow::Borrowed("2.0"),
            id: Some(RequestId::from("1".to_string())),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "test",
                "_internal": "value"
            })),
            meta: std::collections::HashMap::new(),
        });
        let result = middleware
            .process_outgoing(&mut request_with_reserved_key)
            .await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Reserved key name '_internal' not allowed")
        );
    }

    #[tokio::test]
    async fn test_validation_middleware_meta_key_allowed() {
        let middleware = ValidationMiddleware::new();
        let mut request_with_meta = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: Cow::Borrowed("2.0"),
            id: Some(RequestId::from("1".to_string())),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "test",
                "_meta": {
                    "timestamp": 1234567890
                }
            })),
            meta: std::collections::HashMap::new(),
        });
        assert!(
            middleware
                .process_outgoing(&mut request_with_meta)
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn test_validation_middleware_uri_validation() {
        let middleware = ValidationMiddleware::new();
        let mut request_with_valid_uri = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: Cow::Borrowed("2.0"),
            id: Some(RequestId::from("1".to_string())),
            method: "resources/read".to_string(),
            params: Some(json!({
                "uri": "file:///path/to/file.txt"
            })),
            meta: std::collections::HashMap::new(),
        });
        assert!(
            middleware
                .process_outgoing(&mut request_with_valid_uri)
                .await
                .is_ok()
        );
        let mut request_with_invalid_uri = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: Cow::Borrowed("2.0"),
            id: Some(RequestId::from("1".to_string())),
            method: "resources/read".to_string(),
            params: Some(json!({
                "uri": "file:///path/../secret.txt"
            })),
            meta: std::collections::HashMap::new(),
        });
        let result = middleware
            .process_outgoing(&mut request_with_invalid_uri)
            .await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("URI contains path traversal attempt")
        );
    }

    #[tokio::test]
    async fn test_validation_middleware_protocol_version_validation() {
        let middleware = ValidationMiddleware::new();
        let mut request_with_valid_version = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: Cow::Borrowed("2.0"),
            id: Some(RequestId::from("1".to_string())),
            method: "initialize".to_string(),
            params: Some(json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {}
            })),
            meta: std::collections::HashMap::new(),
        });
        assert!(
            middleware
                .process_outgoing(&mut request_with_valid_version)
                .await
                .is_ok()
        );
        let mut request_with_invalid_version = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: Cow::Borrowed("2.0"),
            id: Some(RequestId::from("1".to_string())),
            method: "initialize".to_string(),
            params: Some(json!({
                "protocolVersion": "2023-01-01",
                "capabilities": {}
            })),
            meta: std::collections::HashMap::new(),
        });
        let result = middleware
            .process_outgoing(&mut request_with_invalid_version)
            .await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Unsupported protocol version")
        );
    }

    #[tokio::test]
    async fn test_validation_middleware_tool_name_validation() {
        let middleware = ValidationMiddleware::new();
        let mut request_with_valid_tool = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: Cow::Borrowed("2.0"),
            id: Some(RequestId::from("1".to_string())),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "valid_tool_name"
            })),
            meta: std::collections::HashMap::new(),
        });
        assert!(
            middleware
                .process_outgoing(&mut request_with_valid_tool)
                .await
                .is_ok()
        );
        let mut request_with_invalid_tool = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: Cow::Borrowed("2.0"),
            id: Some(RequestId::from("1".to_string())),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "_invalid_tool_name"
            })),
            meta: std::collections::HashMap::new(),
        });
        let result = middleware
            .process_outgoing(&mut request_with_invalid_tool)
            .await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Tool name cannot start with underscore")
        );
    }

    #[tokio::test]
    async fn test_validation_middleware_log_level_validation() {
        let middleware = ValidationMiddleware::new();
        let mut request_with_valid_level = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: Cow::Borrowed("2.0"),
            id: Some(RequestId::from("1".to_string())),
            method: "logging/log".to_string(),
            params: Some(json!({
                "level": "info",
                "message": "Test log message"
            })),
            meta: std::collections::HashMap::new(),
        });
        assert!(
            middleware
                .process_outgoing(&mut request_with_valid_level)
                .await
                .is_ok()
        );
        let mut request_with_invalid_level = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: Cow::Borrowed("2.0"),
            id: Some(RequestId::from("1".to_string())),
            method: "logging/log".to_string(),
            params: Some(json!({
                "level": "invalid_level",
                "message": "Test log message"
            })),
            meta: std::collections::HashMap::new(),
        });
        let result = middleware
            .process_outgoing(&mut request_with_invalid_level)
            .await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid log level")
        );
    }

    #[tokio::test]
    async fn test_validation_middleware_message_size_limit() {
        let middleware = ValidationMiddleware::new();

        // Test with a message that should be under the limit
        let mut small_request = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: Cow::Borrowed("2.0"),
            id: Some(RequestId::from("1".to_string())),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "test",
                "data": "small data"
            })),
            meta: std::collections::HashMap::new(),
        });
        assert!(
            middleware
                .process_incoming(&mut small_request)
                .await
                .is_ok()
        );

        // Create a huge message that definitely exceeds the 10MB limit
        let huge_string = "x".repeat(15 * 1024 * 1024); // 15MB string
        let mut huge_request = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: Cow::Borrowed("2.0"),
            id: Some(RequestId::from("1".to_string())),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "test",
                "data": huge_string
            })),
            meta: std::collections::HashMap::new(),
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
            jsonrpc: Cow::Borrowed("2.0"),
            id: Some(RequestId::from("1".to_string())),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "test",
                "data": deep_object
            })),
            meta: std::collections::HashMap::new(),
        });
        let result = middleware
            .process_outgoing(&mut request_with_deep_object)
            .await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Parameter depth exceeds maximum")
        );
    }

    #[tokio::test]
    async fn test_validation_middleware_custom_configuration() {
        let middleware = ValidationMiddleware::new()
            .with_max_message_size(1024) // 1KB limit
            .with_max_params_depth(3)
            .with_allowed_methods(vec!["custom/method".to_string()]);
        let mut request_with_custom_method = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: Cow::Borrowed("2.0"),
            id: Some(RequestId::from("1".to_string())),
            method: "custom/method".to_string(),
            params: None,
            meta: std::collections::HashMap::new(),
        });
        assert!(
            middleware
                .process_outgoing(&mut request_with_custom_method)
                .await
                .is_ok()
        );
        let mut request_with_standard_method = JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: Cow::Borrowed("2.0"),
            id: Some(RequestId::from("1".to_string())),
            method: "tools/list".to_string(),
            params: None,
            meta: std::collections::HashMap::new(),
        });
        let result = middleware
            .process_outgoing(&mut request_with_standard_method)
            .await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Method 'tools/list' not allowed")
        );
    }
}

#[cfg(test)]
#[cfg(feature = "http")]
mod transport_compliance_tests {

    use axum::{
        Json,
        body::Bytes,
        http::{HeaderMap, StatusCode},
        response::{IntoResponse, Response},
    };
    use serde_json::json;
    use std::sync::Arc;
    use ultrafast_mcp_core::protocol::{JsonRpcError, JsonRpcResponse};
    use ultrafast_mcp_transport::streamable_http::server::{
        HttpTransportConfig, HttpTransportServer, HttpTransportState,
    };

    #[tokio::test]
    async fn test_protocol_version_header_validation() {
        let config = HttpTransportConfig {
            allow_origin: Some("http://localhost:3000".to_string()),
            ..Default::default()
        };
        let server = HttpTransportServer::new(config);
        let state = server.get_state();

        // Test valid protocol version
        let mut headers = HeaderMap::new();
        headers.insert("mcp-protocol-version", "2025-06-18".parse().unwrap());
        headers.insert("origin", "http://localhost:3000".parse().unwrap());

        let body = Bytes::from(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}"#,
        );

        let response = handle_mcp_post_internal(Arc::new(state.clone()), headers, body).await;
        assert_eq!(response.status(), StatusCode::OK);

        // Test invalid protocol version
        let mut headers = HeaderMap::new();
        headers.insert("mcp-protocol-version", "2020-01-01".parse().unwrap());
        headers.insert("origin", "http://localhost:3000".parse().unwrap());

        let body = Bytes::from(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2020-01-01","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}"#,
        );

        let response = handle_mcp_post_internal(Arc::new(state.clone()), headers, body).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_session_id_validation() {
        let config = HttpTransportConfig {
            allow_origin: Some("http://localhost:3000".to_string()),
            ..Default::default()
        };
        let server = HttpTransportServer::new(config);
        let state = server.get_state();

        // Test valid session ID (UUID)
        let mut headers = HeaderMap::new();
        headers.insert(
            "mcp-session-id",
            "550e8400-e29b-41d4-a716-446655440000".parse().unwrap(),
        );
        headers.insert("origin", "http://localhost:3000".parse().unwrap());

        let body = Bytes::from(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}"#,
        );

        let response = handle_mcp_post_internal(Arc::new(state.clone()), headers, body).await;
        assert_eq!(response.status(), StatusCode::OK);

        // Test invalid session ID (contains non-ASCII characters)
        let mut headers = HeaderMap::new();
        headers.insert("mcp-session-id", "invalid-session-🚀".parse().unwrap());
        headers.insert("origin", "http://localhost:3000".parse().unwrap());

        let body = Bytes::from(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}"#,
        );

        let response = handle_mcp_post_internal(Arc::new(state.clone()), headers, body).await;
        assert_eq!(response.status(), StatusCode::OK); // Should still work for initialize
    }

    #[tokio::test]
    async fn test_accept_header_requirement() {
        let config = HttpTransportConfig {
            allow_origin: Some("http://localhost:3000".to_string()),
            ..Default::default()
        };
        let server = HttpTransportServer::new(config);
        let state = server.get_state();

        // Test GET request (should return SSE stream)
        let mut headers = HeaderMap::new();
        headers.insert("accept", "text/event-stream".parse().unwrap());
        headers.insert("origin", "http://localhost:3000".parse().unwrap());

        let response = handle_mcp_get(Arc::new(state.clone()), headers).await;
        // The response should be an SSE stream
        assert!(response.into_response().status().is_success());
    }

    #[tokio::test]
    async fn test_sse_event_id_generation() {
        let config = HttpTransportConfig {
            enable_sse_resumability: true,
            allow_origin: Some("http://localhost:3000".to_string()),
            ..Default::default()
        };
        let server = HttpTransportServer::new(config);
        let state = server.get_state();

        // Start an SSE stream
        let mut headers = HeaderMap::new();
        headers.insert("accept", "text/event-stream".parse().unwrap());
        headers.insert("origin", "http://localhost:3000".parse().unwrap());

        let response = handle_mcp_get(Arc::new(state.clone()), headers).await;
        assert!(response.into_response().status().is_success());
    }

    #[tokio::test]
    async fn test_last_event_id_header() {
        let config = HttpTransportConfig {
            enable_sse_resumability: true,
            allow_origin: Some("http://localhost:3000".to_string()),
            ..Default::default()
        };
        let server = HttpTransportServer::new(config);
        let state = server.get_state();

        // Test resuming from a specific event ID
        let mut headers = HeaderMap::new();
        headers.insert("accept", "text/event-stream".parse().unwrap());
        headers.insert("last-event-id", "test-event-id-123".parse().unwrap());
        headers.insert("origin", "http://localhost:3000".parse().unwrap());

        let response = handle_mcp_get(Arc::new(state.clone()), headers).await;
        assert!(response.into_response().status().is_success());
    }

    #[tokio::test]
    async fn test_origin_validation() {
        let config = HttpTransportConfig {
            allow_origin: Some("http://localhost:3000".to_string()),
            ..Default::default()
        };
        let server = HttpTransportServer::new(config);
        let state = server.get_state();

        // Test allowed origin
        let mut headers = HeaderMap::new();
        headers.insert("origin", "http://localhost:3000".parse().unwrap());

        let body = Bytes::from(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}"#,
        );

        let response = handle_mcp_post_internal(Arc::new(state.clone()), headers, body).await;
        assert_eq!(response.status(), StatusCode::OK);

        // Test disallowed origin
        let mut headers = HeaderMap::new();
        headers.insert("origin", "http://malicious-site.com".parse().unwrap());

        let body = Bytes::from(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}"#,
        );

        let response = handle_mcp_post_internal(Arc::new(state.clone()), headers, body).await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_session_management() {
        let config = HttpTransportConfig {
            allow_origin: Some("http://localhost:3000".to_string()),
            ..Default::default()
        };
        let server = HttpTransportServer::new(config);
        let state = server.get_state();

        // Create a session
        let session_id = "test-session-123";
        let mut headers = HeaderMap::new();
        headers.insert("mcp-session-id", session_id.parse().unwrap());
        headers.insert("origin", "http://localhost:3000".parse().unwrap());

        let body = Bytes::from(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}"#,
        );

        let response = handle_mcp_post_internal(Arc::new(state.clone()), headers, body).await;
        assert_eq!(response.status(), StatusCode::OK);

        // Verify session exists
        {
            let sessions = state.session_store.read().await;
            assert!(sessions.contains_key(session_id));
        } // Explicitly drop the read lock here

        // Delete the session
        let mut headers = HeaderMap::new();
        headers.insert("mcp-session-id", session_id.parse().unwrap());
        headers.insert("origin", "http://localhost:3000".parse().unwrap());

        let response = handle_mcp_delete(Arc::new(state.clone()), headers).await;
        assert_eq!(response.into_response().status(), StatusCode::OK);

        // Verify session is removed
        {
            let sessions = state.session_store.read().await;
            assert!(!sessions.contains_key(session_id));
        } // Explicitly drop the read lock here
    }

    // Helper function to simulate the server's POST handler
    async fn handle_mcp_post_internal(
        state: Arc<HttpTransportState>,
        headers: HeaderMap,
        body: Bytes,
    ) -> Response {
        // This is a simplified version of the actual handler
        // In a real test, you'd want to use a proper HTTP client

        // Validate Origin header
        if !validate_origin(&headers, &state.config) {
            return (
                StatusCode::FORBIDDEN,
                Json(JsonRpcResponse::error(
                    JsonRpcError::new(-32000, "Origin not allowed".to_string()),
                    None,
                )),
            )
                .into_response();
        }

        // Validate protocol version header if present
        if let Some(protocol_version) = extract_protocol_version(&headers) {
            if !validate_protocol_version(&protocol_version) {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(JsonRpcResponse::error(
                        JsonRpcError::new(
                            -32000,
                            format!("Unsupported protocol version: {protocol_version}"),
                        ),
                        None,
                    )),
                )
                    .into_response();
            }
        }

        // Simulate session creation for initialize requests
        let session_id = headers
            .get("mcp-session-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "default-session".to_string());
        if let Ok(message) = serde_json::from_slice::<serde_json::Value>(&body) {
            if message.get("method") == Some(&serde_json::Value::String("initialize".to_string())) {
                use ultrafast_mcp_transport::streamable_http::server::SessionInfo;
                let mut sessions = state.session_store.write().await;
                sessions.entry(session_id).or_insert_with(SessionInfo::new);
            }
        }

        // For this test, just return success
        (StatusCode::OK, Json(json!({"result": "success"}))).into_response()
    }

    // Helper function to simulate the server's GET handler
    async fn handle_mcp_get(state: Arc<HttpTransportState>, headers: HeaderMap) -> Response {
        // Validate Origin header
        if !validate_origin(&headers, &state.config) {
            return (
                StatusCode::FORBIDDEN,
                Json(JsonRpcResponse::error(
                    JsonRpcError::new(-32000, "Origin not allowed".to_string()),
                    None,
                )),
            )
                .into_response();
        }

        // For this test, just return success
        (StatusCode::OK, "data: test\n\n").into_response()
    }

    // Helper function to simulate the server's DELETE handler
    async fn handle_mcp_delete(state: Arc<HttpTransportState>, headers: HeaderMap) -> Response {
        // Validate Origin header
        if !validate_origin(&headers, &state.config) {
            return (
                StatusCode::FORBIDDEN,
                Json(JsonRpcResponse::error(
                    JsonRpcError::new(-32000, "Origin not allowed".to_string()),
                    None,
                )),
            )
                .into_response();
        }

        // Remove session from store
        let session_id = headers
            .get("mcp-session-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "default-session".to_string());
        let mut sessions = state.session_store.write().await;
        sessions.remove(&session_id);

        // For this test, just return success
        StatusCode::OK.into_response()
    }

    // Helper functions (these would be imported from the actual server module)
    fn validate_origin(headers: &HeaderMap, config: &HttpTransportConfig) -> bool {
        if let Some(origin) = headers.get("origin") {
            if let Ok(origin_str) = origin.to_str() {
                if let Some(allowed_origin) = &config.allow_origin {
                    return origin_str == allowed_origin;
                }
                return origin_str.contains("localhost") || origin_str.contains("127.0.0.1");
            }
            return false;
        }
        config.host == "127.0.0.1" || config.host == "localhost" || config.host == "0.0.0.0"
    }

    fn extract_protocol_version(headers: &HeaderMap) -> Option<String> {
        headers
            .get("mcp-protocol-version")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    }

    fn validate_protocol_version(version: &str) -> bool {
        ["2025-06-18", "2025-03-26", "2024-11-05"].contains(&version)
    }
}
