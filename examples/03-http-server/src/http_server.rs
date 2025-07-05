//! HTTP Server Example with Streamable HTTP Transport
//!
//! This example demonstrates the UltraFastServer API with Streamable HTTP transport.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;
use ultrafast_mcp::{
    ListToolsRequest, ListToolsResponse, MCPError, MCPResult, ServerCapabilities, ServerInfo, Tool,
    ToolCall, ToolContent, ToolHandler, ToolResult, ToolsCapability, UltraFastServer,
};

#[derive(Debug, Serialize, Deserialize)]
struct EchoRequest {
    message: String,
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

        if call.name != "echo" {
            return Err(MCPError::method_not_found(format!(
                "Unknown tool: {}",
                call.name
            )));
        }

        let request: EchoRequest = serde_json::from_value(call.arguments.unwrap_or_default())
            .map_err(|e| MCPError::invalid_params(format!("Invalid request format: {}", e)))?;

        if request.message.is_empty() {
            return Err(MCPError::invalid_params("Message cannot be empty".to_string()));
        }

        let response = EchoResponse {
            message: request.message,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let response_text = serde_json::to_string_pretty(&response)
            .map_err(|e| MCPError::serialization_error(e.to_string()))?;

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
                            "description": "Message to echo back (max 1000 characters)",
                            "maxLength": 1000
                        }
                    },
                    "required": ["message"]
                }),
                output_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "message": {
                            "type": "string",
                            "description": "Echoed message"
                        },
                        "timestamp": {
                            "type": "string",
                            "description": "ISO 8601 timestamp"
                        }
                    },
                    "required": ["message", "timestamp"]
                })),
            }],
            next_cursor: None,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("🚀 Starting HTTP Server with Streamable HTTP Transport");

    // Create server capabilities
    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability {
            list_changed: Some(true),
        }),
        ..Default::default()
    };

    // Create server info
    let server_info = ServerInfo {
        name: "http-server-streamable".to_string(),
        version: "1.0.0".to_string(),
        description: Some(
            "An HTTP server with Streamable HTTP transport for MCP Inspector".to_string(),
        ),
        authors: Some(vec!["ULTRAFAST_MCP Team".to_string()]),
        homepage: Some("https://github.com/ultrafast-mcp/ultrafast-mcp".to_string()),
        license: Some("MIT OR Apache-2.0".to_string()),
        repository: Some("https://github.com/ultrafast-mcp/ultrafast-mcp".to_string()),
    };

    // Create server with tool handler
    let server = UltraFastServer::new(server_info, capabilities)
        .with_tool_handler(Arc::new(EchoToolHandler));

    info!("✅ Server created, starting Streamable HTTP transport on 127.0.0.1:8080");

    // Run the server with Streamable HTTP transport
    server.run_streamable_http("127.0.0.1", 8080).await?;

    info!("Server shutdown completed");
    Ok(())
} 