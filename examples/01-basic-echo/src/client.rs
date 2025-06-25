//! Basic Echo Client Example with Streamable HTTP
//! 
//! This example demonstrates the new UltraFastClient API by connecting to the echo server
//! using Streamable HTTP transport for high-performance communication.

use serde::{Deserialize, Serialize};
use ultrafast_mcp::{UltraFastClient, ClientInfo, ClientCapabilities, ToolCall, ToolContent};
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("🚀 Starting Basic Echo Client with Streamable HTTP");
    
    // Create client info and capabilities
    let client_info = ClientInfo {
        name: "basic-echo-client".to_string(),
        version: "1.0.0".to_string(),
        description: Some("A simple echo client demonstrating UltraFastClient with Streamable HTTP".to_string()),
        authors: None,
        homepage: None,
        license: None,
        repository: None,
    };
    
    let client_capabilities = ClientCapabilities {
        ..Default::default()
    };
    
    // Create client
    let client = UltraFastClient::new(client_info, client_capabilities);
    
    info!("Connecting to server via Streamable HTTP at http://127.0.0.1:8080");
    
    // Connect to server using Streamable HTTP transport
    client.connect_streamable_http("http://127.0.0.1:8080").await?;
    
    info!("✅ Connected! Listing available tools");
    
    // List available tools
    let tools = client.list_tools().await?;
    info!("Available tools: {:?}", tools);
    
    // Call the echo tool
    let echo_request = EchoRequest {
        message: "Hello, UltraFast MCP with Streamable HTTP!".to_string(),
    };
    
    info!("Calling echo tool with message: {}", echo_request.message);
    
    let tool_call = ToolCall {
        name: "echo".to_string(),
        arguments: Some(serde_json::to_value(echo_request)?),
    };
    
    let result = client.call_tool(tool_call).await?;
    
    // Process the result
    for content in result.content {
        match content {
            ToolContent::Text { text } => {
                info!("Received response: {}", text);
                
                // Parse the response
                let response: EchoResponse = serde_json::from_str(&text)?;
                println!("🎯 Echoed message: {}", response.message);
                println!("⏰ Timestamp: {}", response.timestamp);
            }
            _ => {
                info!("Received non-text content: {:?}", content);
            }
        }
    }
    
    info!("Disconnecting from server");
    client.disconnect().await?;
    
    info!("✅ Example completed successfully!");
    
    Ok(())
} 