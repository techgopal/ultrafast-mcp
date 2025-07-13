//! Basic Echo Demo - Comprehensive Transport Showcase
//!
//! This demo showcases both STDIO and Streamable HTTP transports with the basic echo server.
//! It demonstrates the flexibility of UltraFast MCP's transport layer.
//!
//! Usage:
//!   cargo run --bin basic-echo-demo
//!   cargo run --bin basic-echo-demo -- stdio
//!   cargo run --bin basic-echo-demo -- http

use clap::Parser;
use serde_json::json;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{info, warn};
use ultrafast_mcp::{
    ClientCapabilities, ClientInfo, HttpTransportConfig, ListToolsRequest, MCPError, MCPResult,
    ServerCapabilities, ServerInfo, Tool, ToolCall, ToolContent, ToolHandler,
    ToolResult as ServerToolResult, ToolsCapability, UltraFastClient, UltraFastServer,
};

#[derive(Parser)]
#[command(name = "basic-echo-demo")]
#[command(about = "Comprehensive demo of UltraFast MCP transport options")]
struct Args {
    /// Transport type to demo (default: both)
    #[arg(value_enum)]
    transport: Option<TransportType>,

    /// Host for HTTP transport (default: 127.0.0.1)
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Port for HTTP transport (default: 8080)
    #[arg(long, default_value = "8080")]
    port: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, clap::ValueEnum)]
enum TransportType {
    /// Demo STDIO transport only
    Stdio,
    /// Demo HTTP transport only
    Http,
    /// Demo both transports
    Both,
}

// Simple echo tool handler for the demo
struct DemoEchoHandler {
    transport_type: String,
}

impl DemoEchoHandler {
    fn new(transport_type: &str) -> Self {
        Self {
            transport_type: transport_type.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl ToolHandler for DemoEchoHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ServerToolResult> {
        if call.name != "echo" {
            return Err(MCPError::method_not_found(format!(
                "Unknown tool: {}",
                call.name
            )));
        }

        let message = if let Some(args) = call.arguments {
            args.get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Hello from demo!")
                .to_string()
        } else {
            "Hello from demo!".to_string()
        };

        let response = json!({
            "message": message,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "transport": self.transport_type,
            "demo": true
        });

        Ok(ServerToolResult {
            content: vec![ultrafast_mcp::ToolContent::text(response.to_string())],
            is_error: None,
        })
    }

    async fn list_tools(
        &self,
        _request: ListToolsRequest,
    ) -> MCPResult<ultrafast_mcp::ListToolsResponse> {
        Ok(ultrafast_mcp::ListToolsResponse {
            tools: vec![Tool {
                name: "echo".to_string(),
                description: format!("Demo echo tool (transport: {})", self.transport_type),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "message": {
                            "type": "string",
                            "description": "Message to echo back",
                            "default": "Hello from demo!"
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

async fn run_stdio_demo() -> anyhow::Result<()> {
    info!("🚀 Starting STDIO Transport Demo");

    // Create client
    let client = UltraFastClient::new(
        ClientInfo {
            name: "stdio-demo-client".to_string(),
            version: "1.0.0".to_string(),
            description: Some("STDIO demo client".to_string()),
            authors: None,
            homepage: None,
            license: None,
            repository: None,
        },
        ClientCapabilities::default(),
    );

    // Spawn server process
    info!("🔧 Spawning STDIO server...");
    let mut server_process = Command::new("cargo")
        .args([
            "run",
            "--release",
            "--bin",
            "basic-echo-server",
            "--",
            "stdio",
        ])
        .current_dir(".")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to spawn server: {}", e))?;

    info!("✅ Server spawned (PID: {:?})", server_process.id());
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // Connect via STDIO
    client
        .connect_stdio()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;

    client
        .initialize()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to initialize: {}", e))?;

    // Test echo
    let result = client
        .call_tool(ToolCall {
            name: "echo".to_string(),
            arguments: Some(json!({ "message": "Hello from STDIO demo!" })),
        })
        .await?;

    if let Some(ToolContent::Text { text }) = result.content.first() {
        println!("📤 STDIO Response: {text}");
    }

    // Cleanup
    if let Err(e) = client.shutdown(None).await {
        warn!("Failed to shutdown client gracefully: {}", e);
    }
    server_process.kill().await?;
    server_process.wait().await?;

    info!("✅ STDIO demo completed");
    Ok(())
}

async fn run_http_demo(host: &str, port: u16) -> anyhow::Result<()> {
    info!("🚀 Starting HTTP Transport Demo");

    // Create client
    let client = UltraFastClient::new(
        ClientInfo {
            name: "http-demo-client".to_string(),
            version: "1.0.0".to_string(),
            description: Some("HTTP demo client".to_string()),
            authors: None,
            homepage: None,
            license: None,
            repository: None,
        },
        ClientCapabilities::default(),
    );

    // Start server in background
    info!("🔧 Starting HTTP server on {}:{}...", host, port);
    let mut server_process = Command::new("cargo")
        .args([
            "run",
            "--release",
            "--bin",
            "basic-echo-server",
            "--",
            "http",
            "--host",
            host,
            "--port",
            &port.to_string(),
        ])
        .current_dir(".")
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to spawn server: {}", e))?;

    info!("✅ Server started (PID: {:?})", server_process.id());
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

    // Connect via HTTP
    let url = format!("http://{host}:{port}");
    client
        .connect_streamable_http(&url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;

    client
        .initialize()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to initialize: {}", e))?;

    // Test echo
    let result = client
        .call_tool(ToolCall {
            name: "echo".to_string(),
            arguments: Some(json!({ "message": "Hello from HTTP demo!" })),
        })
        .await?;

    if let Some(ToolContent::Text { text }) = result.content.first() {
        println!("📤 HTTP Response: {text}");
    }

    // Cleanup
    if let Err(e) = client.shutdown(None).await {
        warn!("Failed to shutdown client gracefully: {}", e);
    }
    server_process.kill().await?;
    server_process.wait().await?;

    info!("✅ HTTP demo completed");
    Ok(())
}

async fn run_integrated_demo(host: &str, port: u16) -> anyhow::Result<()> {
    info!("🚀 Starting Integrated Demo (HTTP Server + STDIO Client)");

    let host_clone = host.to_string();

    // Create server
    let server = UltraFastServer::new(
        ServerInfo {
            name: "integrated-demo-server".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Integrated demo server".to_string()),
            authors: None,
            homepage: None,
            license: None,
            repository: None,
        },
        ServerCapabilities {
            tools: Some(ToolsCapability {
                list_changed: Some(true),
            }),
            ..Default::default()
        },
    )
    .with_tool_handler(std::sync::Arc::new(DemoEchoHandler::new("integrated")));

    // Start server in background
    let server_handle = tokio::spawn(async move {
        let config = HttpTransportConfig {
            host: host_clone,
            port,
            cors_enabled: true,
            protocol_version: "2025-06-18".to_string(),
            allow_origin: Some("*".to_string()),
            monitoring_enabled: true,
            enable_sse_resumability: true,
        };
        server.run_streamable_http_with_config(config).await
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // Create client
    let client = UltraFastClient::new(
        ClientInfo {
            name: "integrated-demo-client".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Integrated demo client".to_string()),
            authors: None,
            homepage: None,
            license: None,
            repository: None,
        },
        ClientCapabilities::default(),
    );

    // Connect via HTTP
    let url = format!("http://{host}:{port}");
    client
        .connect_streamable_http(&url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;

    client
        .initialize()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to initialize: {}", e))?;

    // Test echo
    let result = client
        .call_tool(ToolCall {
            name: "echo".to_string(),
            arguments: Some(json!({ "message": "Hello from integrated demo!" })),
        })
        .await?;

    if let Some(ToolContent::Text { text }) = result.content.first() {
        println!("📤 Integrated Response: {text}");
    }

    // Cleanup
    if let Err(e) = client.shutdown(None).await {
        warn!("Failed to shutdown client gracefully: {}", e);
    }
    server_handle.abort();

    info!("✅ Integrated demo completed");
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse arguments
    let args = Args::parse();
    let transport = args.transport.unwrap_or(TransportType::Both);

    println!("🎯 UltraFast MCP Basic Echo Demo");
    println!("=================================");

    match transport {
        TransportType::Stdio => {
            println!("📡 Demo: STDIO Transport Only");
            run_stdio_demo().await?;
        }
        TransportType::Http => {
            println!("📡 Demo: HTTP Transport Only");
            run_http_demo(&args.host, args.port).await?;
        }
        TransportType::Both => {
            println!("📡 Demo: Both Transports");
            println!();

            println!("1️⃣ STDIO Transport Demo");
            run_stdio_demo().await?;
            println!();

            println!("2️⃣ HTTP Transport Demo");
            run_http_demo(&args.host, args.port).await?;
            println!();

            println!("3️⃣ Integrated Demo (HTTP Server + Client)");
            run_integrated_demo(&args.host, args.port + 1).await?;
        }
    }

    println!();
    println!("🎉 All demos completed successfully!");
    println!();
    println!("💡 Try running individual demos:");
    println!("   cargo run --bin basic-echo-demo -- stdio");
    println!("   cargo run --bin basic-echo-demo -- http");
    println!("   cargo run --bin basic-echo-server -- stdio");
    println!("   cargo run --bin basic-echo-server -- http");
    println!("   cargo run --bin basic-echo-client -- stdio --spawn-server");
    println!("   cargo run --bin basic-echo-client -- http --url http://127.0.0.1:8080");

    Ok(())
}
