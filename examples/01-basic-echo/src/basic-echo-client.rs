//! Basic Echo Client for MCP with Transport Choice
//!
//! This client demonstrates how to connect to an MCP server using either:
//! - STDIO transport (for subprocess communication)
//! - Streamable HTTP transport (for network communication)
//!
//! Usage:
//!   cargo run --bin basic-echo-client -- stdio
//!   cargo run --bin basic-echo-client -- http --url http://127.0.0.1:8080

use clap::Parser;
use serde_json::json;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{info, warn};
use ultrafast_mcp::{
    ClientCapabilities, ClientInfo, ListToolsRequest, ToolCall, ToolContent, ToolResult,
    UltraFastClient,
};

#[derive(Parser)]
#[command(name = "basic-echo-client")]
#[command(about = "Basic Echo MCP Client with transport choice")]
struct Args {
    /// Transport type to use
    #[arg(value_enum)]
    transport: TransportType,
    
    /// URL for HTTP transport (default: http://127.0.0.1:8080)
    #[arg(long, default_value = "http://127.0.0.1:8080")]
    url: String,
    
    /// Spawn server as subprocess (only for STDIO transport)
    #[arg(long)]
    spawn_server: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, clap::ValueEnum)]
enum TransportType {
    /// Use STDIO transport (subprocess mode)
    Stdio,
    /// Use Streamable HTTP transport (network mode)
    Http,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("🚀 Starting Basic Echo MCP Client");
    info!("📡 Transport: {:?}", args.transport);

    // Create client info
    let client_info = ClientInfo {
        name: "basic-echo-client".to_string(),
        version: "1.0.0".to_string(),
        description: Some(format!("Basic echo client for MCP with {:?} transport", args.transport)),
        authors: None,
        homepage: None,
        license: None,
        repository: None,
    };

    // Create client capabilities
    let capabilities = ClientCapabilities::default();

    // Create client
    let client = UltraFastClient::new(client_info, capabilities);

    // Handle server spawning for STDIO transport
    let server_process = if args.transport == TransportType::Stdio && args.spawn_server {
        info!("🔧 Spawning echo server as subprocess...");
        
        let process = Command::new("cargo")
            .args(&["run", "--release", "--bin", "basic-echo-server", "--", "stdio"])
            .current_dir(".")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to spawn server: {}", e))?;

        info!("✅ Server process spawned (PID: {:?})", process.id());
        
        // Wait a moment for the server to start
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        
        Some(process)
    } else {
        None
    };

    // Connect to server based on transport type
    match args.transport {
        TransportType::Stdio => {
            info!("🔌 Connecting via STDIO transport...");
            
            if args.spawn_server {
                // Use the convenience method for STDIO
                client.connect_stdio().await
                    .map_err(|e| anyhow::anyhow!("Failed to connect via STDIO: {}", e))?;
            } else {
                // For manual subprocess management, use the transport directly
                use ultrafast_mcp::StdioTransport;
                
                let transport = StdioTransport::new().await
                    .map_err(|e| anyhow::anyhow!("Failed to create STDIO transport: {}", e))?;
                
                client.connect(Box::new(transport)).await
                    .map_err(|e| anyhow::anyhow!("Failed to connect to server: {}", e))?;
            }
        }
        TransportType::Http => {
            info!("🔌 Connecting via HTTP transport to {}...", args.url);
            
            // Use the convenience method for Streamable HTTP
            client.connect_streamable_http(&args.url).await
                .map_err(|e| anyhow::anyhow!("Failed to connect via HTTP: {}", e))?;
        }
    }

    info!("✅ Connected to server");

    // Initialize the connection
    info!("🔧 Initializing MCP connection...");
    client.initialize().await
        .map_err(|e| anyhow::anyhow!("Failed to initialize connection: {}", e))?;

    info!("✅ Connection initialized");

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
                "message": format!("Hello from UltraFast MCP Client via {:?}! (attempt {})", args.transport, i)
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
    if let Err(e) = client.shutdown(Some("Test completed".to_string())).await {
        warn!("Failed to shutdown client gracefully: {}", e);
    }

    info!("✅ Client shutdown complete");

    // Clean up server process if we spawned one
    if let Some(mut process) = server_process {
        info!("🛑 Terminating server process...");
        
        if let Err(e) = process.kill().await {
            warn!("Failed to kill server process: {}", e);
        }

        // Wait for server process to exit
        let exit_status = process.wait().await
            .map_err(|e| anyhow::anyhow!("Failed to wait for server: {}", e))?;

        info!("✅ Server process exited with status: {}", exit_status);
    }

    println!("🎉 Basic echo transport test completed successfully!");
    println!("📊 Summary:");
    println!("  - Transport: {:?}", args.transport);
    println!("  - Server spawned: {}", args.spawn_server);
    if args.transport == TransportType::Http {
        println!("  - Server URL: {}", args.url);
    }
    
    Ok(())
} 