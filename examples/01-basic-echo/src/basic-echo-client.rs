use serde_json::json;
use tracing::{info, warn};
use ultrafast_mcp_client::UltraFastClient;
use ultrafast_mcp_core::{
    error::MCPResult,
    protocol::capabilities::ClientCapabilities,
    types::{
        client::ClientInfo,
        tools::{ToolCall, ToolResult},
    },
};
use ultrafast_mcp_transport::stdio::StdioTransport;

#[tokio::main]
async fn main() -> MCPResult<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting UltraFast MCP Echo Client");

    // Create client info
    let client_info = ClientInfo::new(
        "ultrafast-echo-client".to_string(),
        "1.0.0".to_string(),
    );

    // Create client capabilities
    let capabilities = ClientCapabilities::default();

    // Create client
    let client = UltraFastClient::new(client_info, capabilities);

    info!("Client created, connecting to server");

    // Create STDIO transport
    let transport = StdioTransport::new();
    
    // Connect to server
    client.connect(Box::new(transport)).await?;

    info!("Connected to server, testing echo tool");

    // Test the echo tool
    let tool_call = ToolCall {
        name: "echo".to_string(),
        arguments: Some(json!({
            "message": "Hello from UltraFast MCP!"
        })),
    };

    let result: ToolResult = client.call_tool(tool_call).await?;
    
    info!("Echo tool result: {:?}", result);

    // Extract the text content
    if let Some(content) = result.content.first() {
        match content {
            ultrafast_mcp_core::types::tools::ToolContent::Text { text } => {
                println!("Server response: {}", text);
            }
            _ => {
                warn!("Unexpected content type from echo tool");
            }
        }
    }

    info!("Test completed, shutting down");

    // Shutdown the connection
    client.shutdown(Some("Test completed".to_string())).await?;

    info!("Client shutdown complete");
    Ok(())
} 