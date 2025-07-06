//! Everything MCP Client Example (Streamable HTTP)
//! Connects to the everything server via HTTP and demonstrates tool calls.

use ultrafast_mcp::{
    ClientCapabilities, ClientInfo, ListPromptsRequest, ListResourcesRequest, ListToolsRequest,
    StreamableHttpClient, StreamableHttpClientConfig, ToolCall, ToolContent, UltraFastClient,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 Starting Everything MCP Client (Streamable HTTP)");

    // Create client info
    let client_info = ClientInfo {
        name: "everything-client-http".to_string(),
        version: "1.0.0".to_string(),
        description: Some("MCP Everything Example Client (Streamable HTTP)".to_string()),
        authors: None,
        homepage: None,
        license: None,
        repository: None,
    };

    // Create client capabilities
    let capabilities = ClientCapabilities::default();

    // Create the client
    let mut client = UltraFastClient::new(client_info, capabilities);

    println!("🔗 Connecting to MCP server via HTTP...");

    // Create HTTP transport configuration
    let transport_config = StreamableHttpClientConfig {
        base_url: "http://127.0.0.1:8080".to_string(),
        session_id: Some("everything-client-session".to_string()),
        protocol_version: "2025-06-18".to_string(),
        timeout: std::time::Duration::from_secs(30),
        max_retries: 3,
        auth_token: None,
        oauth_config: None,
    };

    // Create HTTP transport
    let mut transport = StreamableHttpClient::new(transport_config)
        .map_err(|e| anyhow::anyhow!("Transport creation failed: {}", e))?;

    // Connect the transport first
    transport
        .connect()
        .await
        .map_err(|e| anyhow::anyhow!("Transport connection failed: {}", e))?;

    // Connect using HTTP transport
    client.connect(Box::new(transport)).await?;

    println!("✅ Connected to MCP server successfully");

    // List available tools
    println!("📋 Listing available tools...");
    let tools_response = client.list_tools(ListToolsRequest { cursor: None }).await?;
    println!("Found {} tools:", tools_response.tools.len());
    for tool in &tools_response.tools {
        println!("  - {}: {}", tool.name, tool.description);
    }

    // Call the echo tool
    if let Some(_echo_tool) = tools_response.tools.iter().find(|t| t.name == "echo") {
        println!("🔧 Calling echo tool...");

        let tool_call = ToolCall {
            name: "echo".to_string(),
            arguments: Some(serde_json::json!({
                "message": "Hello from Everything HTTP Client!"
            })),
        };

        let result = client.call_tool(tool_call).await?;

        println!("📤 Echo tool result:");
        for content in &result.content {
            match content {
                ToolContent::Text { text } => {
                    println!("  Text: {}", text);
                }
                _ => {
                    println!("  Other content: {:?}", content);
                }
            }
        }
    } else {
        println!("❌ Echo tool not found");
    }

    // List available resources
    println!("📁 Listing available resources...");
    let resources_response = client
        .list_resources(ListResourcesRequest { cursor: None })
        .await?;
    println!("Found {} resources:", resources_response.resources.len());
    for resource in &resources_response.resources {
        println!(
            "  - {}: {}",
            resource.name,
            resource.description.as_deref().unwrap_or("No description")
        );
    }

    // List available prompts
    println!("💬 Listing available prompts...");
    let prompts_response = client
        .list_prompts(ListPromptsRequest { cursor: None })
        .await?;
    println!("Found {} prompts:", prompts_response.prompts.len());
    for prompt in &prompts_response.prompts {
        println!(
            "  - {}: {}",
            prompt.name,
            prompt.description.as_deref().unwrap_or("No description")
        );
    }

    println!("✅ Everything HTTP client completed successfully");
    Ok(())
}
