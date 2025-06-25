//! Basic Echo Server Example with Streamable HTTP
//! 
//! This example demonstrates the new UltraFastServer API with a simple echo tool
//! using Streamable HTTP transport for high-performance communication.

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use ultrafast_mcp::{UltraFastServer, ToolHandler, ListToolsRequest, ListToolsResponse, ToolCall, ToolResult, ToolContent, Tool, ServerInfo, ServerCapabilities, ToolsCapability, MCPError, McpResult};
use tracing::info;

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
    async fn handle_tool_call(&self, call: ToolCall) -> McpResult<ToolResult> {
        info!("Handling tool call: {}", call.name);
        
        let request: EchoRequest = serde_json::from_value(call.arguments.unwrap_or_default())
            .map_err(|e| MCPError::serialization_error(e.to_string()))?;
        
        let response = EchoResponse {
            message: request.message,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        let response_text = serde_json::to_string_pretty(&response)
            .map_err(|e| MCPError::serialization_error(e.to_string()))?;
        
        Ok(ToolResult {
            content: vec![ToolContent::text(response_text)],
            is_error: None,
        })
    }
    
    async fn list_tools(&self, _request: ListToolsRequest) -> McpResult<ListToolsResponse> {
        Ok(ListToolsResponse {
            tools: vec![Tool {
                name: "echo".to_string(),
                description: "Echo back a message with timestamp".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "message": {
                            "type": "string",
                            "description": "Message to echo back"
                        }
                    },
                    "required": ["message"]
                }),
                output_schema: None,
            }],
            next_cursor: None,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("🚀 Starting Basic Echo MCP Server with Streamable HTTP");
    
    // Create server capabilities
    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability {
            list_changed: Some(true),
        }),
        ..Default::default()
    };
    
    // Create server
    let server = UltraFastServer::new(
        ServerInfo {
            name: "basic-echo-server".to_string(),
            version: "1.0.0".to_string(),
            description: Some("A simple echo server demonstrating UltraFastServer with Streamable HTTP".to_string()),
            authors: None,
            homepage: None,
            license: None,
            repository: None,
        },
        capabilities,
    )
    .with_tool_handler(Arc::new(EchoToolHandler));
    
    info!("Server created, starting Streamable HTTP transport on 127.0.0.1:8080");
    
    // Run the server with Streamable HTTP transport
    // This provides high-performance HTTP communication with session management
    server.run_streamable_http("127.0.0.1", 8080).await?;
    
    Ok(())
} 