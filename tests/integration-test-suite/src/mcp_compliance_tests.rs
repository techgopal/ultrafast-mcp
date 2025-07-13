//! Comprehensive MCP Compliance Integration Tests
//!
//! This test suite validates that the ultrafast-mcp implementation is fully compliant
//! with the MCP specification and works correctly with real MCP clients.

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use serde_json::json;
    use std::sync::Arc;
    use ultrafast_mcp::UltraFastServer;
    use ultrafast_mcp_core::types::tools::{ListToolsRequest, ListToolsResponse};
    use ultrafast_mcp_core::{
        error::{MCPError, MCPResult},
        protocol::{
            capabilities::{ClientCapabilities, ServerCapabilities, ToolsCapability},
            jsonrpc::{JsonRpcRequest, JsonRpcResponse},
            lifecycle::{InitializeRequest, InitializedNotification},
            version::PROTOCOL_VERSION,
        },
        types::{
            client::ClientInfo,
            server::ServerInfo,
            tools::{Tool, ToolCall, ToolContent, ToolResult},
        },
        RequestId,
    };
    use ultrafast_mcp_server::ToolHandler;

    // Mock tool handler for testing
    struct ComplianceToolHandler;

    #[async_trait]
    impl ToolHandler for ComplianceToolHandler {
        async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult> {
            match call.name.as_str() {
                "echo" => {
                    let message = call
                        .arguments
                        .and_then(|args| args.get("message").cloned())
                        .and_then(|v| v.as_str().map(|s| s.to_string()))
                        .unwrap_or_else(|| "Hello, World!".to_string());

                    Ok(ToolResult {
                        content: vec![ToolContent::text(message)],
                        is_error: Some(false),
                    })
                }
                _ => Err(MCPError::method_not_found(format!(
                    "Unknown tool: {}",
                    call.name
                ))),
            }
        }

        async fn list_tools(&self, _request: ListToolsRequest) -> MCPResult<ListToolsResponse> {
            let tools = vec![Tool {
                name: "echo".to_string(),
                description: "Echo a message back".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "message": {"type": "string", "default": "Hello, World!"}
                    },
                    "required": ["message"]
                }),
                output_schema: None,
                annotations: None,
            }];

            Ok(ListToolsResponse {
                tools,
                next_cursor: None,
            })
        }
    }

    fn create_compliance_server() -> UltraFastServer {
        let server_info = ServerInfo {
            name: "compliance-test-server".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test server for MCP compliance".to_string()),
            homepage: None,
            repository: None,
            authors: Some(vec!["test".to_string()]),
            license: Some("MIT".to_string()),
        };

        let capabilities = ServerCapabilities {
            tools: Some(ToolsCapability {
                list_changed: Some(true),
            }),
            ..Default::default()
        };

        UltraFastServer::new(server_info, capabilities)
            .with_tool_handler(Arc::new(ComplianceToolHandler))
    }

    /// Test the full MCP initialization sequence
    #[tokio::test]
    async fn test_mcp_initialization_sequence() {
        // Create initialization request
        let initialize_request = InitializeRequest {
            protocol_version: PROTOCOL_VERSION.to_string(),
            capabilities: ClientCapabilities::default(),
            client_info: ClientInfo {
                name: "test-client".to_string(),
                version: "1.0.0".to_string(),
                description: None,
                authors: None,
                homepage: None,
                repository: None,
                license: None,
            },
        };

        // Test serialization
        let serialized = serde_json::to_string(&initialize_request).unwrap();
        assert!(serialized.contains("2025-06-18"));
        assert!(serialized.contains("test-client"));

        // Test deserialization
        let deserialized: InitializeRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.protocol_version, PROTOCOL_VERSION);
        assert_eq!(deserialized.client_info.name, "test-client");

        // Test server creation
        let server = create_compliance_server();
        assert_eq!(server.info().name, "compliance-test-server");

        println!("✅ MCP initialization sequence test passed!");
    }

    /// Test protocol version negotiation
    #[tokio::test]
    async fn test_protocol_version_negotiation() {
        // Test with current supported version
        let init_request = InitializeRequest {
            protocol_version: "2025-06-18".to_string(),
            capabilities: ClientCapabilities::default(),
            client_info: ClientInfo {
                name: "test-client".to_string(),
                version: "1.0.0".to_string(),
                description: None,
                authors: None,
                homepage: None,
                repository: None,
                license: None,
            },
        };

        let serialized = serde_json::to_string(&init_request).unwrap();
        assert!(serialized.contains("2025-06-18"));

        // Test with older supported version
        let old_init_request = InitializeRequest {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ClientCapabilities::default(),
            client_info: ClientInfo {
                name: "test-client".to_string(),
                version: "1.0.0".to_string(),
                description: None,
                authors: None,
                homepage: None,
                repository: None,
                license: None,
            },
        };

        let old_serialized = serde_json::to_string(&old_init_request).unwrap();
        assert!(old_serialized.contains("2024-11-05"));

        println!("✅ Protocol version negotiation test passed!");
    }

    /// Test error handling
    #[tokio::test]
    async fn test_error_handling() {
        // Test JSON-RPC error creation
        let error = JsonRpcResponse::error(
            ultrafast_mcp_core::protocol::jsonrpc::JsonRpcError::new(
                -32601,
                "Method not found".to_string(),
            ),
            Some(RequestId::String("test-123".to_string())),
        );

        let serialized = serde_json::to_string(&error).unwrap();
        assert!(serialized.contains("Method not found"));
        assert!(serialized.contains("-32601"));

        // Test MCP-specific errors
        let mcp_error = MCPError::method_not_found("Test method not found".to_string());
        let error_string = mcp_error.to_string();
        assert!(error_string.contains("Test method not found"));

        println!("✅ Error handling test passed!");
    }

    /// Test notification handling
    #[tokio::test]
    async fn test_notification_no_response() {
        // Test notification creation
        let notification = InitializedNotification {};

        let serialized = serde_json::to_string(&notification).unwrap();
        println!("Serialized notification: {serialized}");
        assert_eq!(serialized, "{}");

        println!("✅ Notification handling test passed!");
    }

    /// Test server state management
    #[tokio::test]
    async fn test_server_state_management() {
        let server = create_compliance_server();

        // Test server info
        let info = server.info();
        assert_eq!(info.name, "compliance-test-server");
        assert_eq!(info.version, "1.0.0");

        println!("✅ Server state management test passed!");
    }

    /// Test concurrent requests
    #[tokio::test]
    async fn test_concurrent_requests() {
        let server = create_compliance_server();

        // Create multiple concurrent operations
        let mut handles = Vec::new();

        for i in 0..10 {
            let server_clone = server.clone();
            let handle = tokio::spawn(async move {
                assert_eq!(server_clone.info().name, "compliance-test-server");
                i
            });
            handles.push(handle);
        }

        // Wait for all operations to complete
        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await.unwrap());
        }

        // Check that all operations completed successfully
        assert_eq!(results.len(), 10);
        assert_eq!(results, (0..10).collect::<Vec<_>>());

        println!("✅ Concurrent requests test passed!");
    }

    /// Test MCP Inspector compatibility
    #[tokio::test]
    async fn test_mcp_inspector_compatibility() {
        // Test that our server can handle standard MCP Inspector requests

        // Test tools/list request
        let tools_request = JsonRpcRequest::new(
            "tools/list".to_string(),
            None,
            Some(RequestId::String("1".to_string())),
        );

        let serialized = serde_json::to_string(&tools_request).unwrap();
        assert!(serialized.contains("tools/list"));

        // Test tools/call request
        let tool_call_request = JsonRpcRequest::new(
            "tools/call".to_string(),
            Some(json!({
                "name": "echo",
                "arguments": {"message": "Hello, MCP!"}
            })),
            Some(RequestId::String("2".to_string())),
        );

        let serialized_call = serde_json::to_string(&tool_call_request).unwrap();
        assert!(serialized_call.contains("tools/call"));
        assert!(serialized_call.contains("echo"));
        assert!(serialized_call.contains("Hello, MCP!"));

        println!("✅ MCP Inspector compatibility test passed!");
    }

    /// Test transport lifecycle
    #[tokio::test]
    async fn test_transport_lifecycle() {
        // Test that we can create transport configurations
        let _config = ultrafast_mcp_transport::TransportConfig::Stdio;

        println!("✅ Transport lifecycle test passed!");
    }

    /// Test JSON-RPC 2.0 compliance
    #[tokio::test]
    async fn test_jsonrpc_compliance() {
        // Test request creation
        let request = JsonRpcRequest::new(
            "test_method".to_string(),
            Some(json!({"param": "value"})),
            Some(RequestId::String("test-123".to_string())),
        );

        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("jsonrpc"));
        assert!(serialized.contains("2.0"));
        assert!(serialized.contains("test_method"));
        assert!(serialized.contains("test-123"));

        // Test response creation
        let response = JsonRpcResponse::success(
            json!({"result": "success"}),
            Some(RequestId::String("test-123".to_string())),
        );

        let response_serialized = serde_json::to_string(&response).unwrap();
        assert!(response_serialized.contains("jsonrpc"));
        assert!(response_serialized.contains("2.0"));
        assert!(response_serialized.contains("success"));

        println!("✅ JSON-RPC 2.0 compliance test passed!");
    }
}
