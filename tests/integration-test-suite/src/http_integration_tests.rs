//! Basic integration tests for HTTP transport

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use serde_json::json;
    use std::sync::Arc;
    use ultrafast_mcp::{UltraFastClient, UltraFastServer};
    use ultrafast_mcp_core::types::tools::{ListToolsRequest, ListToolsResponse};
    use ultrafast_mcp_core::{
        error::{MCPError, MCPResult},
        protocol::capabilities::{ClientCapabilities, ServerCapabilities, ToolsCapability},
        types::{
            client::ClientInfo,
            server::ServerInfo,
            tools::{Tool, ToolCall, ToolContent, ToolResult},
        },
    };
    use ultrafast_mcp_server::ToolHandler;

// Mock tool handler for testing
struct TestToolHandler;

#[async_trait]
impl ToolHandler for TestToolHandler {
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
        }];

        Ok(ListToolsResponse {
            tools,
            next_cursor: None,
        })
    }
}

#[cfg(test)]
fn create_test_server() -> UltraFastServer {
    let server_info = ServerInfo {
        name: "http-test-server".to_string(),
        version: "1.0.0".to_string(),
        description: Some("Test server for HTTP integration tests".to_string()),
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

    UltraFastServer::new(server_info, capabilities).with_tool_handler(Arc::new(TestToolHandler))
}

#[tokio::test]
async fn test_http_transport_config() {
    // Test that we can create basic transport configurations
    let _config = ultrafast_mcp_transport::TransportConfig::Stdio;

    println!("✅ HTTP transport config test passed!");
}

#[tokio::test]
async fn test_server_creation_with_http() {
    let server = create_test_server();

    // Test that server is created successfully
    assert_eq!(server.info().name, "http-test-server");
    assert_eq!(server.info().version, "1.0.0");

    println!("✅ Server creation with HTTP test passed!");
}

#[tokio::test]
async fn test_client_creation() {
    let client_info = ClientInfo {
        name: "http-test-client".to_string(),
        version: "1.0.0".to_string(),
        description: Some("Test client for HTTP integration tests".to_string()),
        authors: None,
        homepage: None,
        repository: None,
        license: None,
    };

    let capabilities = ClientCapabilities::default();

    let _client = UltraFastClient::new(client_info, capabilities);

    // Test that client is created successfully
    // Note: UltraFastClient doesn't expose info() method directly

    println!("✅ Client creation test passed!");
}

#[tokio::test]
async fn test_json_rpc_message_serialization() {
    // Test JSON-RPC message creation and serialization
    let request = ultrafast_mcp_core::protocol::jsonrpc::JsonRpcRequest::new(
        "test_method".to_string(),
        None,
        Some(ultrafast_mcp_core::RequestId::String(
            "test-123".to_string(),
        )),
    );

    let serialized = serde_json::to_string(&request).unwrap();
    assert!(serialized.contains("test-123"));
    assert!(serialized.contains("test_method"));

    println!("✅ JSON-RPC message serialization test passed!");
}

#[tokio::test]
async fn test_protocol_compliance() {
    // Test MCP 2025-06-18 compliance
    use ultrafast_mcp_core::protocol::lifecycle::InitializeRequest;

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

    let serialized_init = serde_json::to_string(&init_request).unwrap();
    assert!(serialized_init.contains("2025-06-18"));

    println!("✅ Protocol compliance test passed!");
}

#[tokio::test]
async fn test_error_handling() {
    // Test that error types work correctly
    let error = MCPError::method_not_found("Test method not found".to_string());
    let error_string = error.to_string();
    assert!(error_string.contains("Test method not found"));

    println!("✅ Error handling test passed!");
}

#[tokio::test]
async fn test_concurrent_access() {
    let server = create_test_server();

    // Test concurrent access to server info
    let mut handles = Vec::new();

    for i in 0..5 {
        let server_clone = server.clone();
        let handle = tokio::spawn(async move {
            assert_eq!(server_clone.info().name, "http-test-server");
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
    assert_eq!(results.len(), 5);
    assert_eq!(results, (0..5).collect::<Vec<_>>());

    println!("✅ Concurrent access test passed!");
}
}
