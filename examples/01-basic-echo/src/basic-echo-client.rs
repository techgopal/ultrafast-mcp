use serde_json::json;
use tracing::{info, warn};
use ultrafast_mcp::{
    UltraFastClient, MCPResult, ClientCapabilities, ClientInfo, ToolCall, ToolResult, ToolContent,
};
use ultrafast_mcp::StdioTransport;

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
    let mut transport = ultrafast_mcp::StreamableHttpClient::new(config)
        .map_err(|e| anyhow::anyhow!("Failed to create transport: {}", e))?;
    
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
            ToolContent::Text { text } => {
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