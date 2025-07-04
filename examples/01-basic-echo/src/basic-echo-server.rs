use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use ultrafast_mcp_core::{
    error::MCPResult,
    protocol::capabilities::{ServerCapabilities, ToolsCapability},
    types::{
        server::ServerInfo,
        tools::{Tool, ToolCall, ToolResult},
    },
};
use ultrafast_mcp_server::{handlers::ToolHandler, UltraFastServer};
use ultrafast_mcp_transport::stdio::StdioTransport;

/// Simple echo tool handler
struct EchoToolHandler;

#[async_trait::async_trait]
impl ToolHandler for EchoToolHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult> {
        info!("Echo tool called with: {:?}", call);
        
        let message = if let Some(args) = call.arguments {
            if let Some(text) = args.get("message").and_then(|v| v.as_str()) {
                text.to_string()
            } else {
                "No message provided".to_string()
            }
        } else {
            "No arguments provided".to_string()
        };

        Ok(ToolResult {
            content: vec![ultrafast_mcp_core::types::tools::ToolContent::text(
                format!("Echo: {}", message)
            )],
            is_error: None,
        })
    }

    async fn list_tools(&self, _request: ultrafast_mcp_core::types::tools::ListToolsRequest) -> MCPResult<ultrafast_mcp_core::types::tools::ListToolsResponse> {
        let echo_tool = Tool::new(
            "echo".to_string(),
            "Echoes back the provided message".to_string(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "Message to echo back"
                    }
                },
                "required": ["message"]
            }),
        );

        Ok(ultrafast_mcp_core::types::tools::ListToolsResponse {
            tools: vec![echo_tool],
            next_cursor: None,
        })
    }
}

#[tokio::main]
async fn main() -> MCPResult<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting UltraFast MCP Echo Server");

    // Create server info
    let server_info = ServerInfo::new(
        "ultrafast-echo-server".to_string(),
        "1.0.0".to_string(),
    );

    // Create server capabilities
    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability {
            list_changed: None,
        }),
        ..Default::default()
    };

    // Create server
    let server = UltraFastServer::new(server_info, capabilities)
        .with_tool_handler(Arc::new(EchoToolHandler));

    info!("Server created, starting STDIO transport");

    // Create STDIO transport
    let transport = StdioTransport::new();
    
    // Run server with transport
    server.run_with_transport(Box::new(transport)).await?;

    info!("Server shutdown complete");
    Ok(())
} 