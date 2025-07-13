//! Everything MCP Client Example (Streamable HTTP)
//! Connects to the everything server via HTTP and demonstrates tool calls.
//! This example showcases the new convenience methods for different authentication types.

use std::sync::Arc;
use ultrafast_mcp::types::ElicitationAction;
use ultrafast_mcp::{
    ClientCapabilities, ClientElicitationHandler, ClientInfo, ElicitationRequest,
    ElicitationResponse, ListPromptsRequest, ListResourcesRequest, ListToolsRequest,
    StreamableHttpClientConfig, ToolCall, ToolContent, UltraFastClient,
};

/// Simple elicitation handler for demonstration
struct DemoElicitationHandler;

#[async_trait::async_trait]
impl ClientElicitationHandler for DemoElicitationHandler {
    async fn handle_elicitation_request(
        &self,
        request: ElicitationRequest,
    ) -> ultrafast_mcp::MCPResult<ElicitationResponse> {
        println!("🎯 Received elicitation request: {}", request.message);
        println!("   Schema: {:?}", request.requested_schema);

        // For demo purposes, we'll accept the elicitation with some sample content
        println!("   Responding with Accept action and sample content");
        Ok(ElicitationResponse {
            action: ElicitationAction::Accept,
            content: Some(serde_json::json!({
                "favoriteColor": "blue",
                "favoriteNumber": 42,
                "favoritePets": ["dogs", "cats"]
            })),
        })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 Starting Everything MCP Client (Streamable HTTP)");
    println!(
        "📚 This example demonstrates various connection methods including new convenience APIs"
    );

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

    // Demonstrate different connection methods
    let url = "http://127.0.0.1:8080";

    println!("\n🔗 Connection Method 1: Simple Streamable HTTP (recommended)");
    println!("   Using: client.connect_streamable_http(url).await?");

    // Create the client
    let client = UltraFastClient::new(client_info.clone(), capabilities.clone());

    // Method 1: Simple connection (new convenience method)
    client.connect_streamable_http(url).await?;
    println!("✅ Connected successfully using simple method");

    // Test the connection with a quick tool call
    test_connection(&client, "Simple Connection").await?;

    // Disconnect for next demo
    client.disconnect().await?;

    println!("\n🔗 Connection Method 2: With Bearer Token Authentication");
    println!("   Using: client.connect_streamable_http_with_bearer(url, token).await?");

    // Method 2: Bearer token authentication
    #[cfg(all(feature = "http", feature = "oauth"))]
    {
        let client2 = UltraFastClient::new(client_info.clone(), capabilities.clone());
        let mock_token = "demo-bearer-token-12345";
        match client2
            .connect_streamable_http_with_bearer(url, mock_token.to_string())
            .await
        {
            Ok(_) => {
                println!("✅ Connected with Bearer token (server may not require auth)");
                test_connection(&client2, "Bearer Token").await?;
                client2.disconnect().await?;
            }
            Err(e) => {
                println!("⚠️  Bearer token connection failed (expected if server doesn't support auth): {}", e);
            }
        }
    }
    #[cfg(not(all(feature = "http", feature = "oauth")))]
    {
        println!("⚠️  Bearer token authentication not available (requires http+oauth features)");
    }

    println!("\n🔗 Connection Method 3: With API Key Authentication");
    println!("   Using: client.connect_streamable_http_with_api_key(url, api_key).await?");

    // Method 3: API key authentication (new convenience method)
    #[cfg(all(feature = "http", feature = "oauth"))]
    {
        let client3 = UltraFastClient::new(client_info.clone(), capabilities.clone());
        let mock_api_key = "demo-api-key-67890";
        match client3
            .connect_streamable_http_with_api_key(url, mock_api_key.to_string())
            .await
        {
            Ok(_) => {
                println!("✅ Connected with API key (server may not require auth)");
                test_connection(&client3, "API Key").await?;
                client3.disconnect().await?;
            }
            Err(e) => {
                println!(
                    "⚠️  API key connection failed (expected if server doesn't support auth): {}",
                    e
                );
            }
        }
    }
    #[cfg(not(all(feature = "http", feature = "oauth")))]
    {
        println!("⚠️  API key authentication not available (requires http+oauth features)");
    }

    println!("\n🔗 Connection Method 4: With Basic Authentication");
    println!("   Using: client.connect_streamable_http_with_basic(url, username, password).await?");

    // Method 4: Basic authentication (new convenience method)
    #[cfg(all(feature = "http", feature = "oauth"))]
    {
        let client4 = UltraFastClient::new(client_info.clone(), capabilities.clone());
        match client4
            .connect_streamable_http_with_basic(
                url,
                "demo_user".to_string(),
                "demo_pass".to_string(),
            )
            .await
        {
            Ok(_) => {
                println!("✅ Connected with Basic auth (server may not require auth)");
                test_connection(&client4, "Basic Auth").await?;
                client4.disconnect().await?;
            }
            Err(e) => {
                println!("⚠️  Basic auth connection failed (expected if server doesn't support auth): {}", e);
            }
        }
    }
    #[cfg(not(all(feature = "http", feature = "oauth")))]
    {
        println!("⚠️  Basic authentication not available (requires http+oauth features)");
    }

    println!("\n�� Connection Method 5: Client-Level Authentication Integration");
    println!("   Using: client.with_bearer_auth(token).connect_streamable_http(url).await?");

    // Method 5: Client-level auth that integrates automatically (new feature)
    #[cfg(feature = "oauth")]
    {
        let client5 = UltraFastClient::new(client_info.clone(), capabilities.clone())
            .with_bearer_auth("client-level-token-123".to_string());

        match client5.connect_streamable_http(url).await {
            Ok(_) => {
                println!("✅ Connected with client-level auth integration");
                test_connection(&client5, "Client-Level Auth").await?;
                client5.disconnect().await?;
            }
            Err(e) => {
                println!("⚠️  Client-level auth connection failed: {}", e);
            }
        }
    }
    #[cfg(not(feature = "oauth"))]
    {
        println!("⚠️  OAuth feature not enabled - skipping client-level auth demo");
    }

    println!("\n🔗 Connection Method 6: Custom Configuration (advanced)");
    println!("   Using: client.connect_streamable_http_with_config(custom_config).await?");

    // Method 6: Custom configuration (for advanced use cases)
    let client6 = UltraFastClient::new(client_info.clone(), capabilities.clone())
        .with_elicitation_handler(Arc::new(DemoElicitationHandler));

    let custom_config = StreamableHttpClientConfig {
        base_url: url.to_string(),
        session_id: Some("custom-session-id".to_string()),
        protocol_version: "2025-06-18".to_string(),
        timeout: std::time::Duration::from_secs(60),
        max_retries: 3,
        auth_token: None,
        oauth_config: None,
        auth_method: None,
    };

    client6
        .connect_streamable_http_with_config(custom_config)
        .await?;
    println!("✅ Connected with custom configuration");

    // Run the main demo with the custom configured client
    run_full_demo(&client6).await?;

    println!("\n🎉 All connection methods demonstrated successfully!");
    println!("💡 Key takeaways:");
    println!("   • Use connect_streamable_http() for simple connections");
    println!("   • Use connect_streamable_http_with_*() for specific auth types");
    println!("   • Use client.with_*_auth() for client-level auth integration");
    println!("   • Use connect_streamable_http_with_config() for advanced scenarios");

    Ok(())
}

/// Test a connection with a simple tool call
async fn test_connection(client: &UltraFastClient, method_name: &str) -> anyhow::Result<()> {
    // Quick test - just list tools to verify connection works
    match client.list_tools(ListToolsRequest { cursor: None }).await {
        Ok(response) => {
            println!(
                "   ✅ {} connection verified - found {} tools",
                method_name,
                response.tools.len()
            );
            // Show available tools for verification
            for tool in &response.tools {
                println!("     - {}: {}", tool.name, tool.description);
            }
        }
        Err(e) => {
            println!("   ❌ {method_name} connection test failed: {e}");
        }
    }
    Ok(())
}

/// Run the full demo with all features
async fn run_full_demo(client: &UltraFastClient) -> anyhow::Result<()> {
    println!("\n📋 Running full feature demonstration...");

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
                "message": "Hello from Everything HTTP Client with new convenience APIs!"
            })),
        };

        let result = client.call_tool(tool_call).await?;

        println!("📤 Echo tool result:");
        for content in &result.content {
            match content {
                ToolContent::Text { text } => {
                    println!("  Text: {text}");
                }
                _ => {
                    println!("  Other content: {content:?}");
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
    for resource in resources_response.resources.iter().take(3) {
        // Show first 3 only
        println!(
            "  - {}: {}",
            resource.name,
            resource.description.as_deref().unwrap_or("No description")
        );
    }
    if resources_response.resources.len() > 3 {
        println!(
            "  ... and {} more resources",
            resources_response.resources.len() - 3
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

    // Test the startElicitation tool
    println!("🎯 Testing startElicitation tool...");
    let tool_call = ToolCall {
        name: "startElicitation".to_string(),
        arguments: Some(serde_json::json!({
            "prompt": "What is your favorite color?",
            "description": "A simple elicitation example"
        })),
    };

    match client.call_tool(tool_call).await {
        Ok(result) => {
            println!("✅ Elicitation tool called successfully!");
            for content in &result.content {
                match content {
                    ToolContent::Text { text } => {
                        println!("   Response: {text}");
                    }
                    _ => {
                        println!("   Other content: {content:?}");
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to call elicitation tool: {e}");
        }
    }

    println!("✅ Full demo completed successfully");
    Ok(())
}
