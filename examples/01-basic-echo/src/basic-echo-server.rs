//! Basic Echo Server for MCP with Transport Choice
//!
//! This server demonstrates a simple echo tool that can run with either:
//! - STDIO transport (for subprocess communication)
//! - Streamable HTTP transport (for network communication)
//!
//! Usage:
//!   cargo run --bin basic-echo-server -- stdio
//!   cargo run --bin basic-echo-server -- http --host 127.0.0.1 --port 8080

use clap::Parser;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};
use ultrafast_mcp::{
    ListToolsRequest, ListToolsResponse, MCPError, MCPResult, ServerCapabilities, ServerInfo, Tool,
    ToolCall, ToolContent, ToolHandler, ToolResult, ToolsCapability, UltraFastServer,
    HttpTransportConfig,
};

#[derive(Parser)]
#[command(name = "basic-echo-server")]
#[command(about = "Basic Echo MCP Server with transport choice")]
struct Args {
    /// Transport type to use
    #[arg(value_enum)]
    transport: TransportType,
    
    /// Host for HTTP transport (default: 127.0.0.1)
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    
    /// Port for HTTP transport (default: 8080)
    #[arg(long, default_value = "8080")]
    port: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, clap::ValueEnum)]
enum TransportType {
    /// Use STDIO transport (subprocess mode)
    Stdio,
    /// Use Streamable HTTP transport (network mode)
    Http,
}

#[derive(Debug, Serialize, Deserialize)]
struct EchoRequest {
    #[serde(default = "default_message")]
    message: String,
}

fn default_message() -> String {
    "Hello, World!".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
struct EchoResponse {
    message: String,
    timestamp: String,
    echo_count: u32,
    server_id: String,
    transport: String,
}

struct EchoToolHandler {
    echo_count: std::sync::atomic::AtomicU32,
    transport_type: String,
}

impl EchoToolHandler {
    fn new(transport_type: &str) -> Self {
        Self {
            echo_count: std::sync::atomic::AtomicU32::new(0),
            transport_type: transport_type.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl ToolHandler for EchoToolHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult> {
        info!("Handling tool call: {} (transport: {})", call.name, self.transport_type);

        // Validate tool name
        if call.name != "echo" {
            return Err(MCPError::method_not_found(format!(
                "Unknown tool: {}",
                call.name
            )));
        }

        // Parse and validate request
        let arguments = call
            .arguments
            .ok_or_else(|| MCPError::invalid_params("Missing arguments".to_string()))?;

        let request: EchoRequest = serde_json::from_value(arguments).map_err(|e| {
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

        // Increment echo counter
        let echo_count = self.echo_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;

        // Process the request
        let response = EchoResponse {
            message: request.message,
            timestamp: chrono::Utc::now().to_rfc3339(),
            echo_count,
            server_id: format!("echo-server-{}", std::process::id()),
            transport: self.transport_type.clone(),
        };

        let response_text = serde_json::to_string_pretty(&response).map_err(|e| {
            error!("Failed to serialize echo response: {}", e);
            MCPError::serialization_error(e.to_string())
        })?;

        info!("Echo tool completed successfully (count: {}, transport: {})", echo_count, self.transport_type);
        Ok(ToolResult {
            content: vec![ToolContent::text(response_text)],
            is_error: None,
        })
    }

    async fn list_tools(&self, _request: ListToolsRequest) -> MCPResult<ListToolsResponse> {
        info!("Listing available tools (transport: {})", self.transport_type);
        Ok(ListToolsResponse {
            tools: vec![Tool {
                name: "echo".to_string(),
                description: format!("Echo back a message with timestamp and metadata (transport: {})", self.transport_type),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "message": {
                            "type": "string",
                            "description": "Message to echo back (max 1000 characters, optional - defaults to 'Hello, World!')",
                            "maxLength": 1000,
                            "default": "Hello, World!"
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize tracing
    match args.transport {
        TransportType::Stdio => {
            // For STDIO, write to stderr to avoid interfering with protocol
            tracing_subscriber::fmt()
                .with_writer(std::io::stderr)
                .with_env_filter("info,ultrafast_mcp=debug")
                .with_target(false)
                .init();
        }
        TransportType::Http => {
            // For HTTP, we can use stdout
            tracing_subscriber::fmt()
                .with_env_filter("info,ultrafast_mcp=debug")
                .init();
        }
    }

    info!("🚀 Starting Basic Echo MCP Server");
    info!("📡 Transport: {:?}", args.transport);

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
        description: Some(format!("A simple echo server for MCP with {:?} transport", args.transport)),
        authors: Some(vec!["ULTRAFAST_MCP Team".to_string()]),
        homepage: Some("https://github.com/ultrafast-mcp/ultrafast-mcp".to_string()),
        license: Some("MIT OR Apache-2.0".to_string()),
        repository: Some("https://github.com/ultrafast-mcp/ultrafast-mcp".to_string()),
    };

    // Create server with tool handler
    let transport_name = format!("{:?}", args.transport);
    let server = UltraFastServer::new(server_info, capabilities)
        .with_tool_handler(Arc::new(EchoToolHandler::new(&transport_name)));

    // Run the server with the chosen transport
    match args.transport {
        TransportType::Stdio => {
            info!("✅ Starting STDIO transport (subprocess mode)");
            server.run_stdio().await?;
        }
        TransportType::Http => {
            info!("✅ Starting HTTP transport on {}:{}", args.host, args.port);
            let config = HttpTransportConfig {
                host: args.host,
                port: args.port,
                cors_enabled: true,
                protocol_version: "2025-06-18".to_string(),
                allow_origin: Some("*".to_string()),
                monitoring_enabled: true,
                enable_sse_resumability: true,
            };
            server.run_streamable_http_with_config(config).await?;
        }
    }

    info!("Server shutdown completed");
    Ok(())
}
