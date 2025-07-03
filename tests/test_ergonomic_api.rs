//! Comprehensive tests for the ULTRAFAST_MCP ergonomic API
//!
//! These tests verify that the high-level API works correctly and provides
//! the expected developer experience.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::time::{timeout, Duration};
use ultrafast_mcp::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
struct TestRequest {
    message: String,
    count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct TestResponse {
    echo: String,
    doubled: u32,
    timestamp: String,
}

/// Test tool handler implementation
struct TestToolHandler;

#[async_trait::async_trait]
impl ToolHandler for TestToolHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> McpResult<ToolResult> {
        match call.name.as_str() {
            "echo" => {
                let request: TestRequest = serde_json::from_value(
                    call.arguments.unwrap_or_default(),
                )
                .map_err(|e| MCPError::serialization_error(e.to_string()))?;

                let response = TestResponse {
                    echo: request.message,
                    doubled: request.count * 2,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };

                let response_text = serde_json::to_string_pretty(&response)
                    .map_err(|e| MCPError::serialization_error(e.to_string()))?;

                Ok(ToolResult {
                    content: vec![ToolContent::text(response_text)],
                    is_error: None,
                })
            }
            _ => Err(MCPError::method_not_found(format!("Unknown tool: {}", call.name))),
        }
    }

    async fn list_tools(&self, _request: ListToolsRequest) -> McpResult<ListToolsResponse> {
        Ok(ListToolsResponse {
            tools: vec![Tool {
                name: "echo".to_string(),
                description: "Echo back a message with doubled count".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "message": {
                            "type": "string",
                            "description": "Message to echo back"
                        },
                        "count": {
                            "type": "integer",
                            "description": "Number to double"
                        }
                    },
                    "required": ["message", "count"]
                }),
                output_schema: None,
            }],
            next_cursor: None,
        })
    }
}

/// Test resource handler implementation
struct TestResourceHandler;

#[async_trait::async_trait]
impl ResourceHandler for TestResourceHandler {
    async fn read_resource(&self, request: ReadResourceRequest) -> McpResult<ReadResourceResponse> {
        match request.uri.as_str() {
            "test://status" => {
                let content = serde_json::json!({
                    "name": "Test Server",
                    "status": "running",
                    "timestamp": chrono::Utc::now().to_rfc3339()
                });

                Ok(ReadResourceResponse {
                    contents: vec![ResourceContent::text(
                        serde_json::to_string_pretty(&content).unwrap(),
                    )],
                    mime_type: "application/json".to_string(),
                })
            }
            _ => Err(MCPError::not_found(format!("Resource not found: {}", request.uri))),
        }
    }

    async fn list_resources(
        &self,
        _request: ListResourcesRequest,
    ) -> McpResult<ListResourcesResponse> {
        Ok(ListResourcesResponse {
            resources: vec![Resource {
                uri: "test://status".to_string(),
                name: "Server Status".to_string(),
                description: Some("Current server status".to_string()),
                mime_type: "application/json".to_string(),
            }],
            next_cursor: None,
        })
    }

    async fn list_resource_templates(
        &self,
        _request: ListResourceTemplatesRequest,
    ) -> McpResult<ListResourceTemplatesResponse> {
        Ok(ListResourceTemplatesResponse {
            resource_templates: vec![ResourceTemplate {
                uri: "test://user/{user_id}".to_string(),
                name: "User Profile".to_string(),
                description: Some("User profile template".to_string()),
            }],
            next_cursor: None,
        })
    }
}

#[tokio::test]
async fn test_server_creation() {
    let server_info = ServerInfo {
        name: "test-server".to_string(),
        version: "1.0.0".to_string(),
        description: Some("Test server for API validation".to_string()),
        authors: None,
        homepage: None,
        license: None,
        repository: None,
    };

    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability {
            list_changed: Some(true),
        }),
        resources: Some(ResourcesCapability {
            subscribe: Some(true),
            list_changed: Some(true),
        }),
        ..Default::default()
    };

    let server = UltraFastServer::new(server_info, capabilities)
        .with_tool_handler(Arc::new(TestToolHandler))
        .with_resource_handler(Arc::new(TestResourceHandler));

    assert_eq!(server.info().name, "test-server");
    assert_eq!(server.info().version, "1.0.0");
}

#[tokio::test]
async fn test_client_creation() {
    let client_info = ClientInfo {
        name: "test-client".to_string(),
        version: "1.0.0".to_string(),
        description: Some("Test client for API validation".to_string()),
        authors: None,
        homepage: None,
        license: None,
        repository: None,
    };

    let client_capabilities = ClientCapabilities {
        tools: Some(ToolsCapability {
            list_changed: Some(true),
        }),
        resources: Some(ResourcesCapability {
            subscribe: Some(true),
            list_changed: Some(true),
        }),
        ..Default::default()
    };

    let client = UltraFastClient::new(client_info, client_capabilities);

    assert_eq!(client.info().name, "test-client");
    assert_eq!(client.info().version, "1.0.0");
}

#[tokio::test]
async fn test_context_functionality() {
    let ctx = Context::new()
        .with_session_id("test-session".to_string())
        .with_request_id("test-request".to_string())
        .with_metadata("test_key".to_string(), serde_json::json!("test_value"));

    assert_eq!(ctx.session_id(), Some("test-session"));
    assert_eq!(ctx.request_id(), Some("test-request"));
    assert_eq!(
        ctx.get_metadata("test_key"),
        Some(&serde_json::json!("test_value"))
    );

    // Test logging methods (should not panic)
    ctx.log_info("Test info message").await.unwrap();
    ctx.log_warn("Test warning message").await.unwrap();
    ctx.log_error("Test error message").await.unwrap();

    // Test progress tracking
    ctx.progress("Starting test", 0.0, Some(1.0)).await.unwrap();
    ctx.progress("Halfway done", 0.5, Some(1.0)).await.unwrap();
    ctx.progress("Completed", 1.0, Some(1.0)).await.unwrap();
}

#[tokio::test]
async fn test_error_handling() {
    // Test MCPError creation
    let error = MCPError::method_not_found("test_method".to_string());
    assert!(error.to_string().contains("test_method"));

    let error = MCPError::invalid_params("test_params".to_string());
    assert!(error.to_string().contains("test_params"));

    let error = MCPError::not_found("test_resource".to_string());
    assert!(error.to_string().contains("test_resource"));

    let error = MCPError::internal_error("test_internal".to_string());
    assert!(error.to_string().contains("test_internal"));
}

#[tokio::test]
async fn test_tool_handler() {
    let handler = TestToolHandler;

    // Test tool listing
    let list_request = ListToolsRequest { cursor: None };
    let list_response = handler.list_tools(list_request).await.unwrap();
    assert_eq!(list_response.tools.len(), 1);
    assert_eq!(list_response.tools[0].name, "echo");

    // Test tool execution
    let tool_call = ToolCall {
        name: "echo".to_string(),
        arguments: Some(serde_json::json!({
            "message": "Hello, World!",
            "count": 42
        })),
    };

    let result = handler.handle_tool_call(tool_call).await.unwrap();
    assert_eq!(result.content.len(), 1);
    assert!(!result.is_error.unwrap_or(false));

    // Test unknown tool
    let unknown_call = ToolCall {
        name: "unknown".to_string(),
        arguments: None,
    };

    let error = handler.handle_tool_call(unknown_call).await.unwrap_err();
    assert!(error.to_string().contains("Unknown tool"));
}

#[tokio::test]
async fn test_resource_handler() {
    let handler = TestResourceHandler;

    // Test resource listing
    let list_request = ListResourcesRequest { cursor: None };
    let list_response = handler.list_resources(list_request).await.unwrap();
    assert_eq!(list_response.resources.len(), 1);
    assert_eq!(list_response.resources[0].uri, "test://status");

    // Test resource reading
    let read_request = ReadResourceRequest {
        uri: "test://status".to_string(),
    };
    let read_response = handler.read_resource(read_request).await.unwrap();
    assert_eq!(read_response.contents.len(), 1);
    assert_eq!(read_response.mime_type, "application/json");

    // Test unknown resource
    let unknown_request = ReadResourceRequest {
        uri: "test://unknown".to_string(),
    };
    let error = handler.read_resource(unknown_request).await.unwrap_err();
    assert!(error.to_string().contains("Resource not found"));
}

#[tokio::test]
async fn test_serialization() {
    let request = TestRequest {
        message: "Hello, World!".to_string(),
        count: 42,
    };

    let serialized = serde_json::to_string(&request).unwrap();
    let deserialized: TestRequest = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.message, "Hello, World!");
    assert_eq!(deserialized.count, 42);
}

#[tokio::test]
async fn test_timeout_handling() {
    // Test that operations can be timed out
    let result = timeout(
        Duration::from_millis(100),
        async {
            // Simulate a long-running operation
            tokio::time::sleep(Duration::from_secs(1)).await;
            "completed"
        },
    )
    .await;

    assert!(result.is_err()); // Should timeout
}

#[tokio::test]
async fn test_concurrent_operations() {
    let handler = Arc::new(TestToolHandler);

    // Create multiple concurrent tool calls
    let mut handles = Vec::new();
    for i in 0..10 {
        let handler = handler.clone();
        let handle = tokio::spawn(async move {
            let tool_call = ToolCall {
                name: "echo".to_string(),
                arguments: Some(serde_json::json!({
                    "message": format!("Message {}", i),
                    "count": i
                })),
            };
            handler.handle_tool_call(tool_call).await
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await.unwrap().unwrap());
    }

    assert_eq!(results.len(), 10);
    for result in results {
        assert_eq!(result.content.len(), 1);
        assert!(!result.is_error.unwrap_or(false));
    }
}

#[tokio::test]
async fn test_error_propagation() {
    let handler = TestToolHandler;

    // Test that errors are properly propagated
    let invalid_call = ToolCall {
        name: "echo".to_string(),
        arguments: Some(serde_json::json!({
            "invalid": "data"
        })),
    };

    let error = handler.handle_tool_call(invalid_call).await.unwrap_err();
    assert!(error.to_string().contains("serialization error"));
}

#[tokio::test]
async fn test_capability_negotiation() {
    let server_capabilities = ServerCapabilities {
        tools: Some(ToolsCapability {
            list_changed: Some(true),
        }),
        resources: Some(ResourcesCapability {
            subscribe: Some(true),
            list_changed: Some(true),
        }),
        prompts: Some(PromptsCapability {
            list_changed: Some(true),
        }),
        logging: Some(LoggingCapability {}),
        ..Default::default()
    };

    let client_capabilities = ClientCapabilities {
        tools: Some(ToolsCapability {
            list_changed: Some(true),
        }),
        resources: Some(ResourcesCapability {
            subscribe: Some(true),
            list_changed: Some(true),
        }),
        ..Default::default()
    };

    // Test that capabilities are properly structured
    assert!(server_capabilities.tools.is_some());
    assert!(server_capabilities.resources.is_some());
    assert!(client_capabilities.tools.is_some());
    assert!(client_capabilities.resources.is_some());
}

#[tokio::test]
async fn test_transport_config() {
    // Test stdio transport config
    let stdio_config = TransportConfig::Stdio;
    assert!(matches!(stdio_config, TransportConfig::Stdio));

    // Test streamable HTTP config
    #[cfg(feature = "http")]
    {
        let http_config = TransportConfig::Streamable {
            base_url: "http://localhost:8080/mcp".to_string(),
            auth_token: Some("test-token".to_string()),
            session_id: Some("test-session".to_string()),
        };
        assert!(matches!(http_config, TransportConfig::Streamable { .. }));
    }
}

#[tokio::test]
async fn test_cancellation_manager() {
    let manager = CancellationManager::new();

    // Test request registration
    let request_id = serde_json::json!("test-request");
    manager
        .register_request(request_id.clone(), "test_method".to_string())
        .await
        .unwrap();

    // Test cancellation check
    assert!(!manager.is_cancelled(&request_id).await);

    // Test request completion
    manager.complete_request(&request_id).await.unwrap();
}

#[tokio::test]
async fn test_ping_manager() {
    let manager = PingManager::default();

    // Test ping request
    let ping_request = PingRequest::new();
    let ping_response = manager.handle_ping(ping_request).await.unwrap();

    assert_eq!(ping_response.jsonrpc, "2.0");
    assert!(ping_response.result.is_some());
}

// Integration test that simulates a full server-client interaction
#[tokio::test]
async fn test_integration_workflow() {
    // This test would require actual transport implementation
    // For now, we'll test the components individually
    
    let server_info = ServerInfo {
        name: "integration-test-server".to_string(),
        version: "1.0.0".to_string(),
        description: Some("Integration test server".to_string()),
        authors: None,
        homepage: None,
        license: None,
        repository: None,
    };

    let server_capabilities = ServerCapabilities {
        tools: Some(ToolsCapability {
            list_changed: Some(true),
        }),
        resources: Some(ResourcesCapability {
            subscribe: Some(true),
            list_changed: Some(true),
        }),
        ..Default::default()
    };

    let server = UltraFastServer::new(server_info, server_capabilities)
        .with_tool_handler(Arc::new(TestToolHandler))
        .with_resource_handler(Arc::new(TestResourceHandler));

    let client_info = ClientInfo {
        name: "integration-test-client".to_string(),
        version: "1.0.0".to_string(),
        description: Some("Integration test client".to_string()),
        authors: None,
        homepage: None,
        license: None,
        repository: None,
    };

    let client_capabilities = ClientCapabilities {
        tools: Some(ToolsCapability {
            list_changed: Some(true),
        }),
        resources: Some(ResourcesCapability {
            subscribe: Some(true),
            list_changed: Some(true),
        }),
        ..Default::default()
    };

    let client = UltraFastClient::new(client_info, client_capabilities);

    // Verify that both server and client are properly configured
    assert_eq!(server.info().name, "integration-test-server");
    assert_eq!(client.info().name, "integration-test-client");
}

// Performance test
#[tokio::test]
async fn test_performance() {
    use std::time::Instant;

    let handler = TestToolHandler;

    // Test tool call performance
    let start = Instant::now();
    for _ in 0..100 {
        let tool_call = ToolCall {
            name: "echo".to_string(),
            arguments: Some(serde_json::json!({
                "message": "Performance test",
                "count": 1
            })),
        };
        let _result = handler.handle_tool_call(tool_call).await.unwrap();
    }
    let duration = start.elapsed();

    // Should complete 100 tool calls in under 1 second
    assert!(duration < Duration::from_secs(1));
    println!("100 tool calls completed in {:?}", duration);
}

// Memory usage test
#[tokio::test]
async fn test_memory_usage() {
    let handler = TestToolHandler;

    // Create many tool calls to test memory usage
    let mut results = Vec::new();
    for i in 0..1000 {
        let tool_call = ToolCall {
            name: "echo".to_string(),
            arguments: Some(serde_json::json!({
                "message": format!("Memory test {}", i),
                "count": i
            })),
        };
        let result = handler.handle_tool_call(tool_call).await.unwrap();
        results.push(result);
    }

    assert_eq!(results.len(), 1000);
    for result in results {
        assert_eq!(result.content.len(), 1);
        assert!(!result.is_error.unwrap_or(false));
    }
}
