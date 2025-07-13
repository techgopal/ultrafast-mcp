//! Common test fixtures
//!
//! Consolidates test setup functions from:
//! - tests/integration-test-suite/src/integration_tests.rs
//! - tests/integration-test-suite/src/client_integration_tests.rs
//! - tests/integration-test-suite/src/completion_tests.rs
//! - crates/ultrafast-mcp-server/src/server.rs

use serde_json::json;
use std::sync::Arc;
use ultrafast_mcp_client::UltraFastClient;
use ultrafast_mcp_core::{
    protocol::capabilities::ToolsCapability,
    types::{
        client::{ClientCapabilities, ClientInfo},
        server::{ServerCapabilities, ServerInfo},
        tools::{ListToolsRequest, ListToolsResponse, Tool, ToolCall, ToolContent, ToolResult},
    },
    MCPResult,
};
use ultrafast_mcp_server::{ToolHandler, UltraFastServer};

/// Create a test server with standard configuration
///
/// Consolidates implementations from multiple test files
pub fn create_test_server() -> UltraFastServer {
    let server_info = ServerInfo {
        name: "test-server".to_string(),
        version: "1.0.0".to_string(),
        description: Some("A test MCP server".to_string()),
        authors: Some(vec!["Test Author".to_string()]),
        homepage: Some("https://test.example.com".to_string()),
        license: Some("MIT".to_string()),
        repository: Some("https://github.com/test/test-server".to_string()),
    };

    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability {
            list_changed: Some(true),
        }),
        ..Default::default()
    };

    UltraFastServer::new(server_info, capabilities)
}

/// Create a test server with custom name
pub fn create_test_server_with_name(name: &str) -> UltraFastServer {
    let server_info = ServerInfo {
        name: name.to_string(),
        version: "1.0.0".to_string(),
        description: Some(format!("A test MCP server: {name}")),
        authors: Some(vec!["Test Author".to_string()]),
        homepage: Some("https://test.example.com".to_string()),
        license: Some("MIT".to_string()),
        repository: Some("https://github.com/test/test-server".to_string()),
    };

    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability {
            list_changed: Some(true),
        }),
        ..Default::default()
    };

    UltraFastServer::new(server_info, capabilities)
}

/// Create a test client with standard configuration
///
/// Consolidates implementations from multiple test files
pub fn create_test_client() -> UltraFastClient {
    let client_info = ClientInfo {
        name: "test-client".to_string(),
        version: "1.0.0".to_string(),
        description: Some("A test MCP client".to_string()),
        authors: Some(vec!["Test Author".to_string()]),
        homepage: Some("https://test.example.com".to_string()),
        license: Some("MIT".to_string()),
        repository: Some("https://github.com/test/test-client".to_string()),
    };

    let capabilities = ClientCapabilities::default();

    UltraFastClient::new(client_info, capabilities)
}

/// Create a test client with custom name
pub fn create_test_client_with_name(name: &str) -> UltraFastClient {
    let client_info = ClientInfo {
        name: name.to_string(),
        version: "1.0.0".to_string(),
        description: Some(format!("A test MCP client: {name}")),
        authors: Some(vec!["Test Author".to_string()]),
        homepage: Some("https://test.example.com".to_string()),
        license: Some("MIT".to_string()),
        repository: Some("https://github.com/test/test-client".to_string()),
    };

    let capabilities = ClientCapabilities::default();

    UltraFastClient::new(client_info, capabilities)
}

/// Test tool handler for echo functionality
pub struct TestEchoToolHandler;

#[async_trait::async_trait]
impl ToolHandler for TestEchoToolHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult> {
        match call.name.as_str() {
            "echo" => {
                let message = call
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("message"))
                    .and_then(|msg| msg.as_str())
                    .unwrap_or("Hello, World!");

                Ok(ToolResult {
                    content: vec![ToolContent::text(message.to_string())],
                    is_error: Some(false),
                })
            }
            "error" => Ok(ToolResult {
                content: vec![ToolContent::text("This is a test error".to_string())],
                is_error: Some(true),
            }),
            _ => Ok(ToolResult {
                content: vec![ToolContent::text(format!("Unknown tool: {}", call.name))],
                is_error: Some(true),
            }),
        }
    }

    async fn list_tools(&self, _request: ListToolsRequest) -> MCPResult<ListToolsResponse> {
        Ok(ListToolsResponse {
            tools: vec![
                Tool {
                    name: "echo".to_string(),
                    description: "Echo back the input message".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "message": {
                                "type": "string",
                                "description": "The message to echo back"
                            }
                        },
                        "required": ["message"]
                    }),
                    output_schema: None,
                    annotations: None,
                },
                Tool {
                    name: "error".to_string(),
                    description: "Always returns an error for testing".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {}
                    }),
                    output_schema: None,
                    annotations: None,
                },
            ],
            next_cursor: None,
        })
    }
}

/// Create a test server with echo tool handler
pub fn create_test_server_with_echo_handler() -> UltraFastServer {
    create_test_server().with_tool_handler(Arc::new(TestEchoToolHandler))
}

/// Create standard test tool call for echo
pub fn create_test_echo_call(message: &str) -> ToolCall {
    ToolCall {
        name: "echo".to_string(),
        arguments: Some(json!({
            "message": message
        })),
    }
}

/// Create test tool call for error testing
pub fn create_test_error_call() -> ToolCall {
    ToolCall {
        name: "error".to_string(),
        arguments: Some(json!({})),
    }
}

/// Create test server info
pub fn create_test_server_info(name: &str) -> ServerInfo {
    ServerInfo {
        name: name.to_string(),
        version: "1.0.0".to_string(),
        description: Some(format!("Test server: {name}")),
        authors: Some(vec!["Test Author".to_string()]),
        homepage: Some("https://test.example.com".to_string()),
        license: Some("MIT".to_string()),
        repository: Some("https://github.com/test/test".to_string()),
    }
}

/// Create test client info
pub fn create_test_client_info(name: &str) -> ClientInfo {
    ClientInfo {
        name: name.to_string(),
        version: "1.0.0".to_string(),
        description: Some(format!("Test client: {name}")),
        authors: Some(vec!["Test Author".to_string()]),
        homepage: Some("https://test.example.com".to_string()),
        license: Some("MIT".to_string()),
        repository: Some("https://github.com/test/test".to_string()),
    }
}

/// Create test server capabilities with all features enabled
pub fn create_test_server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        tools: Some(ToolsCapability {
            list_changed: Some(true),
        }),
        ..Default::default()
    }
}

/// Create test client capabilities with all features enabled
pub fn create_test_client_capabilities() -> ClientCapabilities {
    ClientCapabilities::default()
}
