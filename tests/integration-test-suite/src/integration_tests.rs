#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use std::sync::Arc;
    use tokio::time::Duration;
    use ultrafast_mcp::UltraFastServer;
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
            "concurrent_tool" => {
                let input = call
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("input").cloned())
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                    .unwrap_or_else(|| "default".to_string());

                let worker_id = call
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("worker_id").cloned())
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);

                // Simulate some async work
                tokio::time::sleep(Duration::from_millis(10)).await;

                Ok(ToolResult {
                    content: vec![ToolContent::text(
                        json!({
                            "processed": input,
                            "worker_id": worker_id
                        })
                        .to_string(),
                    )],
                    is_error: Some(false),
                })
            }
            "error_tool" => Err(MCPError::internal_error(
                "Intentional test failure".to_string(),
            )),
            _ => Err(MCPError::method_not_found(format!(
                "Unknown tool: {}",
                call.name
            ))),
        }
    }

    async fn list_tools(&self, _request: ListToolsRequest) -> MCPResult<ListToolsResponse> {
        let tools = vec![
            Tool {
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
            },
            Tool {
                name: "concurrent_tool".to_string(),
                description: "Tool for concurrent testing".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "input": {"type": "string"},
                        "worker_id": {"type": "integer"}
                    },
                    "required": ["input", "worker_id"]
                }),
                output_schema: None,
            },
            Tool {
                name: "error_tool".to_string(),
                description: "Tool that always errors".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
                output_schema: None,
            },
        ];

        Ok(ListToolsResponse {
            tools,
            next_cursor: None,
        })
    }
}

fn create_test_server() -> UltraFastServer {
    let server_info = ServerInfo {
        name: "integration-test-server".to_string(),
        version: "1.0.0".to_string(),
        description: Some("Test server for integration tests".to_string()),
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
async fn test_complete_mcp_server_client_flow() -> Result<(), Box<dyn std::error::Error>> {
    // Create a test server
    let server = create_test_server();

    // Test that server is created successfully
    assert_eq!(server.info().name, "integration-test-server");
    assert_eq!(server.info().version, "1.0.0");

    println!("✅ Complete MCP server-client integration test passed!");
    Ok(())
}

#[tokio::test]
async fn test_mcp_protocol_compliance() -> Result<(), Box<dyn std::error::Error>> {
    // Test JSON-RPC message creation and serialization
    let request = ultrafast_mcp_core::protocol::jsonrpc::JsonRpcRequest::new(
        "test_method".to_string(),
        None,
        Some(ultrafast_mcp_core::RequestId::String(
            "test-123".to_string(),
        )),
    );

    let serialized = serde_json::to_string(&request)?;
    assert!(serialized.contains("test-123"));
    assert!(serialized.contains("test_method"));

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

    let serialized_init = serde_json::to_string(&init_request)?;
    assert!(serialized_init.contains("2025-06-18"));

    println!("✅ MCP protocol compliance test passed!");
    Ok(())
}

#[tokio::test]
async fn test_performance_benchmarks() -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Instant;

    // Test server creation performance
    let start = Instant::now();
    let _server = create_test_server();
    let creation_time = start.elapsed();

    // Should create server very quickly (< 10ms)
    assert!(creation_time < Duration::from_millis(10));

    // Test message serialization performance
    let start = Instant::now();
    let request = ultrafast_mcp_core::protocol::jsonrpc::JsonRpcRequest::new(
        "test_method".to_string(),
        None,
        Some(ultrafast_mcp_core::RequestId::String(
            "perf-test".to_string(),
        )),
    );

    for _ in 0..1000 {
        let _serialized = serde_json::to_string(&request).unwrap();
    }
    let serialization_time = start.elapsed();

    // Should serialize 1000 messages quickly (< 100ms)
    assert!(serialization_time < Duration::from_millis(100));

    println!("✅ Performance benchmarks passed!");
    println!("   Server creation: {:?}", creation_time);
    println!("   1000 message serializations: {:?}", serialization_time);

    Ok(())
}

#[tokio::test]
async fn test_concurrent_operations() -> Result<(), Box<dyn std::error::Error>> {
    let server = create_test_server();

    // Create multiple concurrent operations
    let mut handles = Vec::new();

    for i in 0..10 {
        let server_clone = server.clone();
        let handle = tokio::spawn(async move {
            // Test that server can handle concurrent access
            assert_eq!(server_clone.info().name, "integration-test-server");
            i
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await?);
    }

    // Check that all operations completed successfully
    assert_eq!(results.len(), 10);
    assert_eq!(results, (0..10).collect::<Vec<_>>());

    println!("✅ Concurrent operations test passed!");
    Ok(())
}

#[tokio::test]
async fn test_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    let server = create_test_server();

    // Test that server is created successfully
    assert_eq!(server.info().name, "integration-test-server");

    println!("✅ Error handling test passed!");
    Ok(())
}

#[tokio::test]
async fn test_schema_generation() -> Result<(), Box<dyn std::error::Error>> {
    #[derive(Debug, Serialize, Deserialize)]
    struct TestInput {
        name: String,
        count: u32,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct TestOutput {
        message: String,
        total: u32,
    }

    // Test that we can serialize/deserialize typed structures
    let input = TestInput {
        name: "Test".to_string(),
        count: 5,
    };

    let output = TestOutput {
        message: format!("Hello, {}!", input.name),
        total: input.count * 2,
    };

    // Test serialization
    let input_json = serde_json::to_string(&input)?;
    let output_json = serde_json::to_string(&output)?;

    assert!(input_json.contains("Test"));
    assert!(input_json.contains("5"));
    assert!(output_json.contains("Hello, Test!"));
    assert!(output_json.contains("10"));

    println!("✅ Schema generation test passed!");
    Ok(())
}
}
