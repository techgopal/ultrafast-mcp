use std::sync::Arc;
use tokio::time::{sleep, Duration};
use ultrafast_mcp_client::UltraFastClient;
use ultrafast_mcp_core::{
    error::{MCPError, MCPResult},
    protocol::capabilities::{ClientCapabilities, ServerCapabilities, ToolsCapability},
    types::{
        client::ClientInfo,
        server::ServerInfo,
        tools::{Tool, ToolCall, ToolResult},
    },
};
use ultrafast_mcp_server::{UltraFastServer, ToolHandler};
use async_trait::async_trait;
use serde_json::json;

// Mock tool handler for testing
struct TestToolHandler;

#[async_trait]
impl ToolHandler for TestToolHandler {
    async fn handle_tool_call(
        &self,
        call: ultrafast_mcp_core::types::tools::ToolCall,
    ) -> MCPResult<ultrafast_mcp_core::types::tools::ToolResult> {
        match call.name.as_str() {
            "echo" => {
                let message = call.arguments
                    .and_then(|args| args.get("message").cloned())
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                    .unwrap_or_else(|| "Hello, World!".to_string());
                
                Ok(ultrafast_mcp_core::types::tools::ToolResult {
                    content: vec![ultrafast_mcp_core::types::tools::ToolContent::text(message)],
                    is_error: Some(false),
                })
            }
            "calculator" => {
                let expression = call.arguments
                    .and_then(|args| args.get("expression").cloned())
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                    .unwrap_or_else(|| "2 + 2".to_string());
                
                // Simple calculator (for testing purposes)
                let result = if expression.contains("+") {
                    let parts: Vec<&str> = expression.split('+').collect();
                    if parts.len() == 2 {
                        let a: i32 = parts[0].trim().parse().unwrap_or(0);
                        let b: i32 = parts[1].trim().parse().unwrap_or(0);
                        format!("{}", a + b)
                    } else {
                        "Invalid expression".to_string()
                    }
                } else {
                    "Unsupported operation".to_string()
                };
                
                Ok(ultrafast_mcp_core::types::tools::ToolResult {
                    content: vec![ultrafast_mcp_core::types::tools::ToolContent::text(result)],
                    is_error: Some(false),
                })
            }
            _ => Err(MCPError::method_not_found(
                format!("Unknown tool: {}", call.name)
            )),
        }
    }

    async fn list_tools(
        &self,
        _request: ultrafast_mcp_core::types::tools::ListToolsRequest,
    ) -> MCPResult<ultrafast_mcp_core::types::tools::ListToolsResponse> {
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
                output_schema: Some(json!({
                    "type": "object",
                    "properties": {
                        "output": {"type": "string"}
                    }
                })),
            },
            Tool {
                name: "calculator".to_string(),
                description: "Simple calculator for testing".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "expression": {"type": "string", "default": "2 + 2"}
                    },
                    "required": ["expression"]
                }),
                output_schema: Some(json!({
                    "type": "object",
                    "properties": {
                        "result": {"type": "string"}
                    }
                })),
            },
        ];

        Ok(ultrafast_mcp_core::types::tools::ListToolsResponse {
            tools,
            next_cursor: None,
        })
    }
}

fn create_test_server() -> UltraFastServer {
    let server_info = ServerInfo {
        name: "test-server".to_string(),
        version: "1.0.0".to_string(),
        description: Some("Test server for integration tests".to_string()),
        homepage: None,
        repository: None,
        authors: Some(vec!["test".to_string()]),
        license: Some("MIT".to_string()),
    };
    
    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability { list_changed: Some(true) }),
        ..Default::default()
    };
    
    UltraFastServer::new(server_info, capabilities)
        .with_tool_handler(Arc::new(TestToolHandler))
}

fn create_test_client() -> UltraFastClient {
    let client_info = ClientInfo {
        name: "test-client".to_string(),
        version: "1.0.0".to_string(),
        authors: None,
        description: Some("Test client for integration tests".to_string()),
        homepage: None,
        repository: None,
        license: None,
    };
    
    let capabilities = ClientCapabilities::default();
    
    UltraFastClient::new(client_info, capabilities)
}

#[tokio::test]
async fn test_client_server_connection() {
    let server = create_test_server();
    let client = create_test_client();
    
    // Start server in background
    let server_handle = tokio::spawn(async move {
        server.run_stdio().await
    });
    
    // Give server time to start
    sleep(Duration::from_millis(100)).await;
    
    // Connect client
    let result = client.connect_stdio().await;
    assert!(result.is_ok(), "Client should connect successfully: {:?}", result);
    
    // Verify client state
    assert_eq!(*client.state.read().await, ultrafast_mcp_client::ClientState::Initialized);
    
    // Verify server info was received
    let server_info = client.server_info.read().await;
    assert!(server_info.is_some(), "Server info should be available");
    assert_eq!(server_info.as_ref().unwrap().name, "test-server");
    
    // Disconnect
    client.disconnect().await.unwrap();
    
    // Cancel server
    server_handle.abort();
}

#[tokio::test]
async fn test_tool_listing() {
    let server = create_test_server();
    let client = create_test_client();
    
    // Start server in background
    let server_handle = tokio::spawn(async move {
        server.run_stdio().await
    });
    
    // Give server time to start
    sleep(Duration::from_millis(100)).await;
    
    // Connect client
    client.connect_stdio().await.unwrap();
    
    // List tools
    let tools = client.list_tools().await.unwrap();
    assert_eq!(tools.len(), 2, "Should return 2 tools");
    
    // Verify tool names
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    assert!(tool_names.contains(&"echo"), "Should include echo tool");
    assert!(tool_names.contains(&"calculator"), "Should include calculator tool");
    
    // Verify tool schemas
    let echo_tool = tools.iter().find(|t| t.name == "echo").unwrap();
    assert_eq!(echo_tool.description, "Echo a message back");
    assert!(echo_tool.input_schema.get("properties").is_some());
    assert!(echo_tool.output_schema.is_some());
    
    // Disconnect
    client.disconnect().await.unwrap();
    server_handle.abort();
}

#[tokio::test]
async fn test_tool_calling() {
    let server = create_test_server();
    let client = create_test_client();
    
    // Start server in background
    let server_handle = tokio::spawn(async move {
        server.run_stdio().await
    });
    
    // Give server time to start
    sleep(Duration::from_millis(100)).await;
    
    // Connect client
    client.connect_stdio().await.unwrap();
    
    // Test echo tool
    let echo_call = ToolCall {
        name: "echo".to_string(),
        arguments: Some(json!({
            "message": "Hello from client!"
        })),
    };
    
    let echo_result = client.call_tool(echo_call).await.unwrap();
    assert_eq!(echo_result.content.len(), 1);
    let echo_content = &echo_result.content[0];
    assert!(echo_content.text().unwrap().contains("Hello from client!"));
    assert_eq!(echo_result.is_error, Some(false));
    
    // Test calculator tool
    let calc_call = ToolCall {
        name: "calculator".to_string(),
        arguments: Some(json!({
            "expression": "5 + 3"
        })),
    };
    
    let calc_result = client.call_tool(calc_call).await.unwrap();
    assert_eq!(calc_result.content.len(), 1);
    let calc_content = &calc_result.content[0];
    assert_eq!(calc_content.text().unwrap(), "8");
    assert_eq!(calc_result.is_error, Some(false));
    
    // Disconnect
    client.disconnect().await.unwrap();
    server_handle.abort();
}

#[tokio::test]
async fn test_tool_calling_with_defaults() {
    let server = create_test_server();
    let client = create_test_client();
    
    // Start server in background
    let server_handle = tokio::spawn(async move {
        server.run_stdio().await
    });
    
    // Give server time to start
    sleep(Duration::from_millis(100)).await;
    
    // Connect client
    client.connect_stdio().await.unwrap();
    
    // Test echo tool with no arguments (should use defaults)
    let echo_call = ToolCall {
        name: "echo".to_string(),
        arguments: None,
    };
    
    let echo_result = client.call_tool(echo_call).await.unwrap();
    assert_eq!(echo_result.content.len(), 1);
    let echo_content = &echo_result.content[0];
    assert!(echo_content.text().unwrap().contains("Hello, World!"));
    
    // Disconnect
    client.disconnect().await.unwrap();
    server_handle.abort();
}

#[tokio::test]
async fn test_tool_calling_unknown_tool() {
    let server = create_test_server();
    let client = create_test_client();
    
    // Start server in background
    let server_handle = tokio::spawn(async move {
        server.run_stdio().await
    });
    
    // Give server time to start
    sleep(Duration::from_millis(100)).await;
    
    // Connect client
    client.connect_stdio().await.unwrap();
    
    // Test calling unknown tool
    let unknown_call = ToolCall {
        name: "unknown_tool".to_string(),
        arguments: Some(json!({
            "param": "value"
        })),
    };
    
    let result = client.call_tool(unknown_call).await;
    assert!(result.is_err(), "Should fail for unknown tool");
    
    if let Err(MCPError::MethodNotFound(_)) = result {
        // Expected error type
    } else {
        panic!("Expected MethodNotFound error, got: {:?}", result);
    }
    
    // Disconnect
    client.disconnect().await.unwrap();
    server_handle.abort();
}

#[tokio::test]
async fn test_tool_calling_invalid_arguments() {
    let server = create_test_server();
    let client = create_test_client();
    
    // Start server in background
    let server_handle = tokio::spawn(async move {
        server.run_stdio().await
    });
    
    // Give server time to start
    sleep(Duration::from_millis(100)).await;
    
    // Connect client
    client.connect_stdio().await.unwrap();
    
    // Test calculator with invalid expression
    let calc_call = ToolCall {
        name: "calculator".to_string(),
        arguments: Some(json!({
            "expression": "invalid expression"
        })),
    };
    
    let calc_result = client.call_tool(calc_call).await.unwrap();
    assert_eq!(calc_result.content.len(), 1);
    let calc_content = &calc_result.content[0];
    assert!(calc_content.text().unwrap().contains("Invalid expression"));
    
    // Disconnect
    client.disconnect().await.unwrap();
    server_handle.abort();
}

#[tokio::test]
async fn test_ping_functionality() {
    let server = create_test_server();
    let client = create_test_client();
    
    // Start server in background
    let server_handle = tokio::spawn(async move {
        server.run_stdio().await
    });
    
    // Give server time to start
    sleep(Duration::from_millis(100)).await;
    
    // Connect client
    client.connect_stdio().await.unwrap();
    
    // Test ping without data
    let ping_result = client.ping(None).await.unwrap();
    assert!(ping_result.data.is_some());
    
    // Test ping with data
    let ping_data = json!({"test": "data"});
    let ping_result = client.ping(Some(ping_data.clone())).await.unwrap();
    assert_eq!(ping_result.data, Some(ping_data));
    
    // Disconnect
    client.disconnect().await.unwrap();
    server_handle.abort();
}

#[tokio::test]
async fn test_cancellation_functionality() {
    let server = create_test_server();
    let client = create_test_client();
    
    // Start server in background
    let server_handle = tokio::spawn(async move {
        server.run_stdio().await
    });
    
    // Give server time to start
    sleep(Duration::from_millis(100)).await;
    
    // Connect client
    client.connect_stdio().await.unwrap();
    
    // Test cancellation manager
    let cancellation_manager = client.cancellation_manager();
    let request_id = json!("test-request-123");
    
    // Register a request
    client.register_request(request_id.clone(), "test_method".to_string()).await.unwrap();
    
    // Check if request is not cancelled initially
    assert!(!client.is_request_cancelled(&request_id).await);
    
    // Cancel the request
    client.cancel_request(request_id.clone(), Some("Test cancellation".to_string())).await.unwrap();
    
    // Check if request is now cancelled
    assert!(client.is_request_cancelled(&request_id).await);
    
    // Disconnect
    client.disconnect().await.unwrap();
    server_handle.abort();
}

#[tokio::test]
async fn test_client_capabilities_negotiation() {
    let server = create_test_server();
    let client = create_test_client();
    
    // Start server in background
    let server_handle = tokio::spawn(async move {
        server.run_stdio().await
    });
    
    // Give server time to start
    sleep(Duration::from_millis(100)).await;
    
    // Connect client
    client.connect_stdio().await.unwrap();
    
    // Verify server capabilities were received
    let server_capabilities = client.server_capabilities.read().await;
    assert!(server_capabilities.is_some(), "Server capabilities should be available");
    
    let capabilities = server_capabilities.as_ref().unwrap();
    assert!(capabilities.tools.is_some(), "Server should support tools");
    assert!(capabilities.tools.as_ref().unwrap().list_changed.unwrap_or(false));
    
    // Disconnect
    client.disconnect().await.unwrap();
    server_handle.abort();
}

#[tokio::test]
async fn test_client_reconnection() {
    let server = create_test_server();
    let client = create_test_client();
    
    // Start server in background
    let server_handle = tokio::spawn(async move {
        server.run_stdio().await
    });
    
    // Give server time to start
    sleep(Duration::from_millis(100)).await;
    
    // Connect client
    client.connect_stdio().await.unwrap();
    
    // Verify connected
    assert_eq!(*client.state.read().await, ultrafast_mcp_client::ClientState::Initialized);
    
    // Disconnect
    client.disconnect().await.unwrap();
    
    // Verify disconnected
    assert_eq!(*client.state.read().await, ultrafast_mcp_client::ClientState::Disconnected);
    
    // Reconnect
    client.connect_stdio().await.unwrap();
    
    // Verify reconnected
    assert_eq!(*client.state.read().await, ultrafast_mcp_client::ClientState::Initialized);
    
    // Test tool listing after reconnection
    let tools = client.list_tools().await.unwrap();
    assert_eq!(tools.len(), 2);
    
    // Disconnect
    client.disconnect().await.unwrap();
    server_handle.abort();
}

#[tokio::test]
async fn test_concurrent_tool_calls() {
    let server = create_test_server();
    let client = create_test_client();
    
    // Start server in background
    let server_handle = tokio::spawn(async move {
        server.run_stdio().await
    });
    
    // Give server time to start
    sleep(Duration::from_millis(100)).await;
    
    // Connect client
    client.connect_stdio().await.unwrap();
    
    // Make concurrent tool calls
    let client_arc = Arc::new(client);
    let mut handles = vec![];
    
    for i in 0..5 {
        let client_clone = client_arc.clone();
        let handle = tokio::spawn(async move {
            let call = ToolCall {
                name: "echo".to_string(),
                arguments: Some(json!({
                    "message": format!("Message {}", i)
                })),
            };
            client_clone.call_tool(call).await
        });
        handles.push(handle);
    }
    
    // Wait for all calls to complete
    let results = futures::future::join_all(handles).await;
    
    // Verify all calls succeeded
    for (i, result) in results.into_iter().enumerate() {
        let tool_result = result.unwrap().unwrap();
        assert_eq!(tool_result.content.len(), 1);
        let content = &tool_result.content[0];
        assert!(content.text().unwrap().contains(&format!("Message {}", i)));
    }
    
    // Disconnect
    client_arc.disconnect().await.unwrap();
    server_handle.abort();
} 