//! Basic Echo Client for MCP Subprocess Transport
//!
//! This client demonstrates how to spawn and communicate with an MCP server
//! as a subprocess using STDIO transport.

use serde_json::json;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{info, warn};
use ultrafast_mcp::{
    ClientCapabilities, ClientInfo, ListToolsRequest, ToolCall, ToolContent, ToolResult,
    UltraFastClient, StdioTransport,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("🚀 Starting Basic Echo MCP Client (Subprocess)");

    // Create client info
    let client_info = ClientInfo {
        name: "basic-echo-client".to_string(),
        version: "1.0.0".to_string(),
        description: Some("Basic echo client for MCP subprocess transport".to_string()),
        authors: None,
        homepage: None,
        license: None,
        repository: None,
    };

    // Create client capabilities
    let capabilities = ClientCapabilities::default();

    // Create client
    let client = UltraFastClient::new(client_info, capabilities);

    info!("🔧 Spawning echo server as subprocess...");

    // Spawn the server as a subprocess
    let mut server_process = Command::new("cargo")
        .args(&["run", "--release", "--bin", "basic-echo-server"])
        .current_dir(".")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to spawn server: {}", e))?;

    info!("✅ Server process spawned (PID: {:?})", server_process.id());

    // Wait a moment for the server to start
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    info!("🔌 Creating STDIO transport...");

    // Create STDIO transport (this will use the current process's stdin/stdout)
    let transport = StdioTransport::new().await
        .map_err(|e| anyhow::anyhow!("Failed to create STDIO transport: {}", e))?;

    info!("🔌 Connecting client to transport...");

    // Connect to server
    client.connect(Box::new(transport)).await
        .map_err(|e| anyhow::anyhow!("Failed to connect to server: {}", e))?;

    info!("✅ Connected to server");

    // List available tools
    info!("📋 Listing available tools...");
    let tools_response = client.list_tools(ListToolsRequest { cursor: None }).await
        .map_err(|e| anyhow::anyhow!("Failed to list tools: {}", e))?;

    println!("Found {} tools:", tools_response.tools.len());
    for tool in &tools_response.tools {
        println!("  - {}: {}", tool.name, tool.description);
    }

    // Test the echo tool multiple times
    for i in 1..=3 {
        info!("🔧 Calling echo tool (attempt {})...", i);

        let tool_call = ToolCall {
            name: "echo".to_string(),
            arguments: Some(json!({
                "message": format!("Hello from UltraFast MCP Client! (attempt {})", i)
            })),
        };

        let result: ToolResult = client.call_tool(tool_call).await
            .map_err(|e| anyhow::anyhow!("Failed to call echo tool: {}", e))?;

        // Extract and display the text content
        if let Some(content) = result.content.first() {
            match content {
                ToolContent::Text { text } => {
                    println!("📤 Echo response {}:", i);
                    println!("{}", text);
                    println!();
                }
                _ => {
                    warn!("Unexpected content type from echo tool");
                }
            }
        }

        // Small delay between calls
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    info!("🔧 Testing echo tool with default message...");

    // Test with default message (no arguments)
    let tool_call = ToolCall {
        name: "echo".to_string(),
        arguments: None,
    };

    let result: ToolResult = client.call_tool(tool_call).await
        .map_err(|e| anyhow::anyhow!("Failed to call echo tool with default: {}", e))?;

    if let Some(content) = result.content.first() {
        match content {
            ToolContent::Text { text } => {
                println!("📤 Echo response (default):");
                println!("{}", text);
                println!();
            }
            _ => {
                warn!("Unexpected content type from echo tool");
            }
        }
    }

    info!("🛑 Shutting down client...");

    // Shutdown the connection
    client.shutdown(Some("Test completed".to_string())).await
        .map_err(|e| anyhow::anyhow!("Failed to shutdown client: {}", e))?;

    info!("✅ Client shutdown complete");

    // Terminate the server process
    if let Err(e) = server_process.kill().await {
        warn!("Failed to kill server process: {}", e);
    }

    // Wait for server process to exit
    let exit_status = server_process.wait().await
        .map_err(|e| anyhow::anyhow!("Failed to wait for server: {}", e))?;

    info!("✅ Server process exited with status: {}", exit_status);

    println!("🎉 Basic echo subprocess transport test completed successfully!");
    Ok(())
} 