//! Basic Echo Client Example with Streamable HTTP
//!
//! This example demonstrates the new UltraFastClient API by connecting to the echo server
//! using Streamable HTTP transport for high-performance communication.

use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{error, info, warn};
use ultrafast_mcp::{ClientCapabilities, ClientInfo, ToolCall, ToolContent, UltraFastClient};

#[derive(Debug, Serialize, Deserialize)]
struct EchoRequest {
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct EchoResponse {
    message: String,
    timestamp: String,
}

/// Retry configuration
const MAX_RETRIES: u32 = 3;
const RETRY_DELAY: Duration = Duration::from_secs(1);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging with better configuration
    tracing_subscriber::fmt()
        .with_env_filter("info,ultrafast_mcp=debug")
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();

    info!("🚀 Starting Basic Echo Client with Streamable HTTP");

    // Create client info and capabilities
    let client_info = ClientInfo {
        name: "basic-echo-client".to_string(),
        version: "1.0.0".to_string(),
        description: Some(
            "A simple echo client demonstrating UltraFastClient with Streamable HTTP".to_string(),
        ),
        authors: Some(vec!["ULTRAFAST_MCP Team".to_string()]),
        homepage: Some("https://github.com/ultrafast-mcp/ultrafast-mcp".to_string()),
        license: Some("MIT OR Apache-2.0".to_string()),
        repository: Some("https://github.com/ultrafast-mcp/ultrafast-mcp".to_string()),
    };

    let client_capabilities = ClientCapabilities {
        ..Default::default()
    };

    // Create client with timeout configuration
    let client = UltraFastClient::new(client_info, client_capabilities);

    info!("Connecting to server via Streamable HTTP at http://127.0.0.1:8080");

    // Connect to server with retry logic
    let mut retry_count = 0;
    let connection_result: Result<(), Box<dyn std::error::Error>> = loop {
        // Use Streamable HTTP transport for connection
        let config = ultrafast_mcp::StreamableHttpClientConfig {
            base_url: "http://127.0.0.1:8080".to_string(),
            session_id: Some("basic-client-session".to_string()),
            protocol_version: "2025-06-18".to_string(),
            timeout: std::time::Duration::from_secs(30),
            max_retries: 3,
            auth_token: None,
            oauth_config: None,
        };

        let mut transport = ultrafast_mcp::StreamableHttpClient::new(config)
        .map_err(|e| anyhow::anyhow!("Failed to create transport: {}", e))?;
        // Connect the transport before passing to client
        match transport.connect().await {
            Ok(_) => match client.connect(Box::new(transport)).await {
                Ok(_) => {
                    info!("✅ Connected successfully!");
                    break Ok(());
                }
                Err(e) => {
                    retry_count += 1;
                    if retry_count >= MAX_RETRIES {
                        error!("Failed to connect after {} retries: {}", MAX_RETRIES, e);
                        break Err(e.into());
                    }
                    warn!(
                        "Client connection attempt {} failed: {}. Retrying in {:?}...",
                        retry_count, e, RETRY_DELAY
                    );
                    tokio::time::sleep(RETRY_DELAY).await;
                }
            },
            Err(e) => {
                retry_count += 1;
                if retry_count >= MAX_RETRIES {
                    error!(
                        "Failed to connect transport after {} retries: {}",
                        MAX_RETRIES, e
                    );
                    break Err(e.into());
                }
                warn!(
                    "Transport connection attempt {} failed: {}. Retrying in {:?}...",
                    retry_count, e, RETRY_DELAY
                );
                tokio::time::sleep(RETRY_DELAY).await;
            }
        }
    };

    connection_result?;

    info!("✅ Connected! Listing available tools");

    // List available tools with error handling
    let tools = match client.list_tools_default().await {
        Ok(tools) => {
            info!("Available tools: {:?}", tools);
            tools
        }
        Err(e) => {
            error!("Failed to list tools: {}", e);
            return Err(e.into());
        }
    };

    if tools.tools.is_empty() {
        warn!("No tools available on the server");
        return Ok(());
    }

    // Call the echo tool with proper error handling
    let echo_request = EchoRequest {
        message: "Hello, UltraFast MCP with Streamable HTTP!".to_string(),
    };

    info!("Calling echo tool with message: {}", echo_request.message);

    let tool_call = ToolCall {
        name: "echo".to_string(),
        arguments: Some(serde_json::to_value(echo_request)?),
    };

    let result = match client.call_tool(tool_call).await {
        Ok(result) => {
            info!("✅ Tool call completed successfully");
            result
        }
        Err(e) => {
            error!("❌ Tool call failed: {}", e);
            return Err(e.into());
        }
    };

    // Process the result with error handling
    for (i, content) in result.content.iter().enumerate() {
        match content {
            ToolContent::Text { text } => {
                info!("Received response {}: {}", i + 1, text);
                match serde_json::from_str::<EchoResponse>(text) {
                    Ok(response) => {
                        println!("🎯 Echoed message: {}", response.message);
                        println!("⏰ Timestamp: {}", response.timestamp);
                    }
                    Err(e) => {
                        warn!("Failed to parse response as EchoResponse: {}", e);
                        println!("📄 Raw response: {}", text);
                    }
                }
            }
            ToolContent::Image { .. } => {
                info!("Received image content {}", i + 1);
            }
            ToolContent::Resource { .. } => {
                info!("Received resource content {}", i + 1);
            }
        }
    }

    // Test error handling with invalid tool call
    info!("Testing error handling with invalid tool call...");
    let invalid_call = ToolCall {
        name: "nonexistent_tool".to_string(),
        arguments: Some(serde_json::json!({"test": "data"})),
    };

    match client.call_tool(invalid_call).await {
        Ok(_) => {
            warn!("Unexpected success for invalid tool call");
        }
        Err(e) => {
            info!("✅ Expected error for invalid tool: {}", e);
        }
    }

    // Test error handling with invalid parameters
    info!("Testing error handling with invalid parameters...");
    let invalid_params_call = ToolCall {
        name: "echo".to_string(),
        arguments: Some(serde_json::json!({
            "invalid_field": "this should fail"
        })),
    };

    match client.call_tool(invalid_params_call).await {
        Ok(_) => {
            warn!("Unexpected success for invalid parameters");
        }
        Err(e) => {
            info!("✅ Expected error for invalid parameters: {}", e);
        }
    }

    // info!("Disconnecting from server");
    // match client.disconnect().await {
    //     Ok(_) => {
    //         info!("✅ Disconnected successfully");
    //     }
    //     Err(e) => {
    //         warn!("Warning: Failed to disconnect cleanly: {}", e);
    //     }
    // }

    info!("✅ Example completed successfully!");

    Ok(())
}
