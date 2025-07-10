//! Everything MCP Client Example (STDIO)
//! Connects to the everything server via stdio and demonstrates tool calls.

use std::sync::Arc;
use tracing::info;
use ultrafast_mcp::{
    types::elicitation::{ElicitationAction, ElicitationRequest, ElicitationResponse},
    ClientCapabilities, ClientElicitationHandler, ClientInfo, ListToolsRequest, ToolCall,
    ToolContent, ToolResult, UltraFastClient,
};

/// Example client-side elicitation handler
struct ExampleElicitationHandler;

#[async_trait::async_trait]
impl ClientElicitationHandler for ExampleElicitationHandler {
    async fn handle_elicitation_request(
        &self,
        request: ElicitationRequest,
    ) -> ultrafast_mcp::MCPResult<ElicitationResponse> {
        info!("Client received elicitation request: {}", request.message);
        info!("Requested schema: {:?}", request.requested_schema);

        // In a real implementation, this would present the request to the user
        // For demonstration, we'll simulate different responses based on the message content

        if request.message.contains("username") {
            // Simulate user providing a username
            Ok(ElicitationResponse {
                action: ElicitationAction::Accept,
                content: Some(serde_json::json!({
                    "username": "demo_user"
                })),
            })
        } else if request.message.contains("confirm") {
            // Simulate user declining a confirmation
            Ok(ElicitationResponse {
                action: ElicitationAction::Decline,
                content: None,
            })
        } else {
            // Simulate user cancelling for other requests
            Ok(ElicitationResponse {
                action: ElicitationAction::Cancel,
                content: None,
            })
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting UltraFast MCP Everything Client");

    // Create client info
    let client_info = ClientInfo::new(
        "ultrafast-everything-client".to_string(),
        "1.0.0".to_string(),
    );

    // Create client capabilities
    let capabilities = ClientCapabilities::default();

    // Create client with elicitation handler
    let client = UltraFastClient::new(client_info, capabilities)
        .with_elicitation_handler(Arc::new(ExampleElicitationHandler));

    info!("Client created, connecting to server");

    // Connect to server using STDIO
    client.connect_stdio().await?;

    info!("Connected to server, testing tools");

    // Test listing tools
    let tools_request = ListToolsRequest { cursor: None };
    let tools_response = client.list_tools(tools_request).await?;
    info!("Available tools: {:?}", tools_response.tools);

    // Test calling a tool
    let tool_call = ToolCall {
        name: "echo".to_string(),
        arguments: Some(serde_json::json!({
            "message": "Hello from UltraFast MCP!"
        })),
    };

    let result: ToolResult = client.call_tool(tool_call).await?;

    info!("Tool result: {:?}", result);

    // Extract the text content
    if let Some(content) = result.content.first() {
        match content {
            ToolContent::Text { text } => {
                println!("Server response: {}", text);
            }
            _ => {
                println!("Unexpected content type from tool");
            }
        }
    }

    info!("Test completed, shutting down");

    // Shutdown the connection
    client.shutdown(Some("Test completed".to_string())).await?;

    info!("Client shutdown complete");
    Ok(())
}
