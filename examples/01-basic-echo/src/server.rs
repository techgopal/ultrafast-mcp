//! Basic Echo Server Example with Streamable HTTP
//!
//! This example demonstrates the new UltraFastServer API with a simple echo tool
//! using Streamable HTTP transport for high-performance communication.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};
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

        // Validate tool name
        if call.name != "echo" {
            return Err(MCPError::method_not_found(format!(
                "Unknown tool: {}",
                call.name
            )));
        }

        // Parse and validate request
        let request: EchoRequest = serde_json::from_value(call.arguments.unwrap_or_default())
            .map_err(|e| {
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
    // Initialize tracing with better configuration
    tracing_subscriber::fmt()
        .with_env_filter("info,ultrafast_mcp=debug")
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();

    info!("🚀 Starting Basic Echo MCP Server with Streamable HTTP");

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
        description: Some(
            "A simple echo server demonstrating UltraFastServer with Streamable HTTP".to_string(),
        ),
        authors: Some(vec!["ULTRAFAST_MCP Team".to_string()]),
        homepage: Some("https://github.com/ultrafast-mcp/ultrafast-mcp".to_string()),
        license: Some("MIT OR Apache-2.0".to_string()),
        repository: Some("https://github.com/ultrafast-mcp/ultrafast-mcp".to_string()),
    };

    // Create server with error handling
    let server = match UltraFastServer::new(server_info, capabilities)
        .with_tool_handler(Arc::new(EchoToolHandler))
    {
        server => {
            info!("✅ Server created successfully");
            server
        }
    };

    info!("Server created, starting Streamable HTTP transport on 127.0.0.1:8080");

    // Set up graceful shutdown
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for ctrl+c signal");
        info!("Received shutdown signal");
    };

    // Run the server with error handling and graceful shutdown
    let server_task = async {
        if let Err(e) = server.run_streamable_http("127.0.0.1", 8080).await {
            error!("Server error: {}", e);
            return Err(e);
        }
        Ok(())
    };

    // Wait for either shutdown signal or server error
    tokio::select! {
        _ = shutdown_signal => {
            info!("Shutting down server gracefully...");
            // TODO: Implement proper server shutdown
            Ok(())
        }
        result = server_task => {
            match result {
                Ok(_) => {
                    info!("Server completed successfully");
                    Ok(())
                }
                Err(e) => {
                    error!("Server failed: {}", e);
                    Err(e.into())
                }
            }
        }
    }
}
