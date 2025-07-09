//! Basic Echo Server for MCP Inspector
//!
//! This is a STDIO-based MCP server that can be used with the MCP Inspector.
//! It implements a simple echo tool that returns the input message with a timestamp.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};
use ultrafast_mcp::{
    ListToolsRequest, ListToolsResponse, MCPError, MCPResult, ServerCapabilities, ServerInfo, Tool,
    ToolCall, ToolContent, ToolHandler, ToolResult, ToolsCapability, UltraFastServer,
};

#[derive(Debug, Serialize, Deserialize)]
struct EchoRequest {
    #[serde(default = "default_message")]
    message: String,
}

fn default_message() -> String {
    "Hello, World!".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
struct EchoResponse {
    message: String,
    timestamp: String,
}

struct EchoToolHandler;

#[async_trait::async_trait]
impl ToolHandler for EchoToolHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult> {
        info!("Handling tool call: {}", call.name);

        // Validate tool name
        if call.name != "echo" {
            return Err(MCPError::method_not_found(format!(
                "Unknown tool: {}",
                call.name
            )));
        }

        // Parse and validate request
        let arguments = call
            .arguments
            .ok_or_else(|| MCPError::invalid_params("Missing arguments".to_string()))?;

        let request: EchoRequest = serde_json::from_value(arguments).map_err(|e| {
            error!("Failed to parse echo request: {}", e);
            MCPError::invalid_params(format!("Invalid request format: {}", e))
        })?;

        // Validate input
        if request.message.is_empty() {
            return Err(MCPError::invalid_params(
                "Message cannot be empty".to_string(),
            ));
        }

        if request.message.len() > 1000 {
            return Err(MCPError::invalid_params(
                "Message too long (max 1000 characters)".to_string(),
            ));
        }

        // Process the request
        let response = EchoResponse {
            message: request.message,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let response_text = serde_json::to_string_pretty(&response).map_err(|e| {
            error!("Failed to serialize echo response: {}", e);
            MCPError::serialization_error(e.to_string())
        })?;

        info!("Echo tool completed successfully");
        Ok(ToolResult {
            content: vec![ToolContent::text(response_text)],
            is_error: None,
        })
    }

    async fn list_tools(&self, _request: ListToolsRequest) -> MCPResult<ListToolsResponse> {
        info!("Listing available tools");
        Ok(ListToolsResponse {
            tools: vec![Tool {
                name: "echo".to_string(),
                description: "Echo back a message with timestamp".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "message": {
                            "type": "string",
                            "description": "Message to echo back (max 1000 characters, optional - defaults to 'Hello, World!')",
                            "maxLength": 1000,
                            "default": "Hello, World!"
                        }
                    }
                }),
                output_schema: None,
                annotations: None,
            }],
            next_cursor: None,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing to stderr so it doesn't interfere with STDIO protocol
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter("info,ultrafast_mcp=debug")
        .with_target(false)
        .init();

    info!("🚀 Starting Basic Echo MCP Server (STDIO)");

    // Create server capabilities
    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability {
            list_changed: Some(true),
        }),
        ..Default::default()
    };

    // Create server info
    let server_info = ServerInfo {
        name: "basic-echo-server".to_string(),
        version: "1.0.0".to_string(),
        description: Some("A simple echo server for MCP Inspector testing".to_string()),
        authors: Some(vec!["ULTRAFAST_MCP Team".to_string()]),
        homepage: Some("https://github.com/ultrafast-mcp/ultrafast-mcp".to_string()),
        license: Some("MIT OR Apache-2.0".to_string()),
        repository: Some("https://github.com/ultrafast-mcp/ultrafast-mcp".to_string()),
    };

    // Create server with tool handler
    let server = UltraFastServer::new(server_info, capabilities)
        .with_tool_handler(Arc::new(EchoToolHandler));

    info!("✅ Server created, starting STDIO transport");

    // Run the server with STDIO transport (for MCP Inspector)
    server.run_stdio().await?;

    info!("Server shutdown completed");
    Ok(())
}
