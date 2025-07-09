//! Lifecycle Compliance Example
//!
//! This example demonstrates the MCP 2025-06-18 lifecycle compliance improvements
//! including proper state transitions and comprehensive timeout configuration.

use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{error, info, warn};
use ultrafast_mcp::{
    ClientCapabilities, ClientInfo, ServerCapabilities, ServerInfo, TimeoutConfig, ToolCall,
    ToolContent, ToolHandler, ToolResult, UltraFastServer, UltraFastClient,
    ListToolsRequest, ListToolsResponse, MCPResult, MCPError
};
use ultrafast_mcp_core::Tool;

#[derive(Deserialize)]
struct TimeoutTestRequest {
    duration_seconds: u64,
    should_timeout: bool,
}

#[derive(Serialize)]
struct TimeoutTestResponse {
    message: String,
    duration: u64,
    completed: bool,
}

/// Tool handler that demonstrates timeout behavior
struct TimeoutTestHandler;

#[async_trait::async_trait]
impl ToolHandler for TimeoutTestHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult> {
        match call.name.as_str() {
            "timeout_test" => {
                let args: TimeoutTestRequest = serde_json::from_value(
                    call.arguments.unwrap_or_default()
                )?;

                info!("Starting timeout test with duration: {}s", args.duration_seconds);

                // Simulate work for the specified duration
                tokio::time::sleep(Duration::from_secs(args.duration_seconds)).await;

                let response = TimeoutTestResponse {
                    message: if args.should_timeout {
                        "This should have timed out!".to_string()
                    } else {
                        "Timeout test completed successfully".to_string()
                    },
                    duration: args.duration_seconds,
                    completed: true,
                };

                Ok(ToolResult {
                    content: vec![ToolContent::text(serde_json::to_string(&response)?)],
                    is_error: Some(false),
                })
            }
            _ => Err(MCPError::method_not_found(
                format!("Unknown tool: {}", call.name)
            )),
        }
    }

    async fn list_tools(&self, _request: ListToolsRequest) -> MCPResult<ListToolsResponse> {
        Ok(ListToolsResponse {
            tools: vec![Tool {
                name: "timeout_test".to_string(),
                description: "Test timeout behavior with configurable duration".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "duration_seconds": {"type": "integer", "minimum": 1, "maximum": 300},
                        "should_timeout": {"type": "boolean", "default": false}
                    },
                    "required": ["duration_seconds"]
                }),
                output_schema: None,
            }],
            next_cursor: None,
        })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info,ultrafast_mcp=debug")
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();

    info!("🚀 Starting Lifecycle Compliance Example");

    // Create server with different timeout configurations
    let server_info = ServerInfo {
        name: "lifecycle-compliance-server".to_string(),
        version: "1.0.0".to_string(),
        description: Some("Demonstrates MCP 2025-06-18 lifecycle compliance".to_string()),
        authors: None,
        homepage: None,
        license: None,
        repository: None,
    };

    let capabilities = ServerCapabilities {
        tools: Some(ultrafast_mcp::ToolsCapability { list_changed: Some(true) }),
        ..Default::default()
    };

    // Create server with high-performance timeout configuration
    let server = UltraFastServer::new(server_info, capabilities)
        .with_tool_handler(std::sync::Arc::new(TimeoutTestHandler))
        .with_high_performance_timeouts(); // 30s tool call timeout

    info!("✅ Server created with high-performance timeout configuration");
    info!("📋 Tool call timeout: {:?}", server.get_operation_timeout("tools/call"));
    info!("📋 Operation timeout: {:?}", server.get_operation_timeout("unknown"));

    // Create client
    let client_info = ClientInfo {
        name: "lifecycle-compliance-client".to_string(),
        version: "1.0.0".to_string(),
        description: Some("Demonstrates MCP 2025-06-18 lifecycle compliance".to_string()),
        authors: None,
        homepage: None,
        license: None,
        repository: None,
    };

    let client_capabilities = ClientCapabilities::default();
    let client = UltraFastClient::new(client_info, client_capabilities);

    info!("🔗 Connecting to server via STDIO...");

    // Run server in background
    let server_handle = tokio::spawn(async move {
        server.run_stdio().await
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect client
    client.connect_stdio().await?;

    info!("✅ Client connected successfully");

    // Test 1: Normal operation (should complete)
    info!("🧪 Test 1: Normal operation (5 seconds)");
    let tool_call = ToolCall {
        name: "timeout_test".to_string(),
        arguments: Some(serde_json::json!({
            "duration_seconds": 5,
            "should_timeout": false
        })),
    };

    let result = client.call_tool(tool_call).await?;
    info!("✅ Test 1 completed: {:?}", result);

    // Test 2: Operation that should timeout (60 seconds > 30 second timeout)
    info!("🧪 Test 2: Timeout test (60 seconds, should timeout)");
    let tool_call = ToolCall {
        name: "timeout_test".to_string(),
        arguments: Some(serde_json::json!({
            "duration_seconds": 60,
            "should_timeout": true
        })),
    };

    let result = client.call_tool(tool_call).await;
    match result {
        Ok(_) => warn!("⚠️ Test 2 unexpectedly completed (should have timed out)"),
        Err(e) => {
            if e.to_string().contains("timeout") {
                info!("✅ Test 2 correctly timed out: {}", e);
            } else {
                error!("❌ Test 2 failed with unexpected error: {}", e);
            }
        }
    }

    // Test 3: Create server with long-running timeout configuration
    info!("🧪 Test 3: Long-running timeout configuration");
    let server_info = ServerInfo {
        name: "long-running-server".to_string(),
        version: "1.0.0".to_string(),
        description: Some("Server with long-running timeout configuration".to_string()),
        authors: None,
        homepage: None,
        license: None,
        repository: None,
    };

    let long_running_server = UltraFastServer::new(server_info, capabilities)
        .with_tool_handler(std::sync::Arc::new(TimeoutTestHandler))
        .with_long_running_timeouts(); // 5 minute tool call timeout

    info!("📋 Long-running tool call timeout: {:?}", long_running_server.get_operation_timeout("tools/call"));

    // Test 4: Custom timeout configuration
    info!("🧪 Test 4: Custom timeout configuration");
    let custom_timeout_config = TimeoutConfig::new(
        Duration::from_secs(60),   // initialize_timeout
        Duration::from_secs(600),  // operation_timeout
        Duration::from_secs(120),  // tool_call_timeout
        Duration::from_secs(60),   // resource_timeout
        Duration::from_secs(1800), // sampling_timeout
        Duration::from_secs(300),  // elicitation_timeout
        Duration::from_secs(60),   // completion_timeout
        Duration::from_secs(30),   // ping_timeout
        Duration::from_secs(60),   // shutdown_timeout
        Duration::from_secs(30),   // cancellation_timeout
        Duration::from_secs(10),   // progress_interval
    );

    let custom_server = UltraFastServer::new(server_info, capabilities)
        .with_tool_handler(std::sync::Arc::new(TimeoutTestHandler))
        .with_timeout_config(custom_timeout_config);

    info!("📋 Custom tool call timeout: {:?}", custom_server.get_operation_timeout("tools/call"));

    // Disconnect client
    client.disconnect().await?;
    info!("✅ Client disconnected");

    // Cancel server
    server_handle.abort();
    info!("✅ Server stopped");

    info!("🎉 Lifecycle compliance example completed successfully!");
    info!("");
    info!("📋 Summary of improvements:");
    info!("   ✅ Proper state transitions (Initialized → Operating)");
    info!("   ✅ Comprehensive timeout configuration");
    info!("   ✅ High-performance timeout presets");
    info!("   ✅ Long-running operation timeout presets");
    info!("   ✅ Custom timeout configuration support");
    info!("   ✅ Operation-specific timeout handling");
    info!("   ✅ Timeout error handling and cancellation notifications");

    Ok(())
} 