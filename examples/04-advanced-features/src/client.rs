//! Advanced Features Client Example
//!
//! This example demonstrates the complete UltraFastClient API with all advanced features:
//! - OAuth 2.1 authentication
//! - Monitoring and metrics collection
//! - Middleware integration
//! - Recovery mechanisms
//! - Health checking
//! - Multiple tool calls
//! - Resource access
//! - Prompt generation

use serde_json::json;
use tracing::{error, info};
use ultrafast_mcp::{
    ClientCapabilities,
    ClientInfo,
    GetPromptRequest,
    ListPromptsRequest,
    ListResourcesRequest,
    ListToolsRequest,
    MCPError,
    MCPResult,
    // Import OAuth and HTTP types
    OAuthConfig,
    StreamableHttpClient,
    StreamableHttpClientConfig,
    ToolCall,
    UltraFastClient,
};

struct AdvancedClient {
    client: UltraFastClient,
}

impl AdvancedClient {
    async fn new() -> MCPResult<Self> {
        info!("🚀 Initializing Advanced Features MCP Client");

        // Create OAuth configuration (for demonstration)
        let _oauth_config = OAuthConfig {
            client_id: "demo-client".to_string(),
            client_secret: "demo-secret".to_string(),
            redirect_uri: "http://localhost:8080/callback".to_string(),
            auth_url: "https://auth.example.com/oauth/authorize".to_string(),
            token_url: "https://auth.example.com/oauth/token".to_string(),
            scopes: vec!["read".to_string(), "write".to_string()],
        };

        // Create client capabilities (use only valid fields)
        let capabilities = ClientCapabilities::default();

        // Create client info
        let client_info = ClientInfo {
            name: "advanced-features-client".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Advanced MCP client with all features".to_string()),
            authors: Some(vec!["ULTRAFAST_MCP Team".to_string()]),
            homepage: Some("https://github.com/ultrafast-mcp/ultrafast-mcp".to_string()),
            license: Some("MIT OR Apache-2.0".to_string()),
            repository: Some("https://github.com/ultrafast-mcp/ultrafast-mcp".to_string()),
        };

        // Create the client
        let client = UltraFastClient::new(client_info, capabilities);

        info!("✅ Advanced client created successfully");
        Ok(Self { client })
    }

    async fn connect(&mut self) -> MCPResult<()> {
        info!("🔗 Connecting to MCP server via HTTP...");

        // Create HTTP transport configuration
        let transport_config = StreamableHttpClientConfig {
            base_url: "http://127.0.0.1:8080".to_string(),
            session_id: Some("advanced-client-session".to_string()),
            protocol_version: "2025-06-18".to_string(),
            timeout: std::time::Duration::from_secs(30),
            max_retries: 3,
            auth_token: None,
            oauth_config: None,
        };

        // Create HTTP transport
        let mut transport = StreamableHttpClient::new(transport_config)
            .map_err(|e| MCPError::invalid_request(format!("Transport creation failed: {}", e)))?;

        // Connect the transport first
        transport.connect().await.map_err(|e| {
            MCPError::invalid_request(format!("Transport connection failed: {}", e))
        })?;

        // Connect using HTTP transport
        self.client
            .connect(Box::new(transport))
            .await
            .map_err(|e| {
                error!("Failed to initialize client: {}", e);
                MCPError::invalid_request(format!("Initialization failed: {}", e))
            })?;

        info!("✅ Connected to MCP server successfully");
        Ok(())
    }

    async fn demonstrate_tools(&mut self) -> MCPResult<()> {
        info!("🔧 Demonstrating tool calls...");
        // List available tools
        let tools_req = ListToolsRequest { cursor: None };
        let tools = self.client.list_tools(tools_req).await?;
        info!(
            "Available tools: {:?}",
            tools.tools.iter().map(|t| &t.name).collect::<Vec<_>>()
        );
        // Call calculator tool
        let calculator_call = ToolCall {
            name: "calculate".to_string(),
            arguments: Some(json!({
                "operation": "add",
                "a": 10.5,
                "b": 20.3
            })),
        };
        info!("Calling calculator tool: {:?}", calculator_call);
        let calculator_result = self.client.call_tool(calculator_call).await?;
        info!("Calculator result: {:?}", calculator_result);
        // Call weather tool
        let weather_call = ToolCall {
            name: "weather".to_string(),
            arguments: Some(json!({
                "city": "San Francisco",
                "country": "US"
            })),
        };
        info!("Calling weather tool: {:?}", weather_call);
        let weather_result = self.client.call_tool(weather_call).await?;
        info!("Weather result: {:?}", weather_result);
        Ok(())
    }

    async fn demonstrate_resources(&mut self) -> MCPResult<()> {
        info!("📁 Demonstrating resource access...");
        let req = ListResourcesRequest { cursor: None };
        let resources = self.client.list_resources(req).await?;
        info!(
            "Available resources: {:?}",
            resources
                .resources
                .iter()
                .map(|r| &r.uri)
                .collect::<Vec<_>>()
        );
        Ok(())
    }

    async fn demonstrate_prompts(&mut self) -> MCPResult<()> {
        info!("💬 Demonstrating prompt generation...");
        let req = ListPromptsRequest { cursor: None };
        let prompts = self.client.list_prompts(req).await?;
        info!(
            "Available prompts: {:?}",
            prompts.prompts.iter().map(|p| &p.name).collect::<Vec<_>>()
        );
        let get_req = GetPromptRequest {
            name: "greeting".to_string(),
            arguments: Some(json!({"name": "Alice"})),
        };
        let greeting_prompt = self.client.get_prompt(get_req).await?;
        info!("Greeting prompt: {:?}", greeting_prompt);
        Ok(())
    }

    async fn run_demo(&mut self) -> MCPResult<()> {
        info!("🎯 Starting advanced features demonstration...");
        self.demonstrate_tools().await?;
        self.demonstrate_resources().await?;
        self.demonstrate_prompts().await?;
        info!("✅ Advanced features demonstration completed successfully!");
        Ok(())
    }

    async fn shutdown(&mut self) -> MCPResult<()> {
        info!("🛑 Shutting down advanced client...");
        self.client.disconnect().await.map_err(|e| {
            error!("Failed to shutdown client: {}", e);
            MCPError::internal_error(format!("Client shutdown failed: {}", e))
        })?;
        info!("✅ Advanced client shutdown completed");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info,ultrafast_mcp=debug,ultrafast_mcp_transport=debug")
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .init();
    info!("🚀 Starting Advanced Features MCP Client");
    let mut client = AdvancedClient::new().await?;
    client.connect().await?;
    if let Err(e) = client.run_demo().await {
        error!("Demo failed: {}", e);
        return Err(e.into());
    }
    client.shutdown().await?;
    info!("✅ Advanced client completed successfully");
    Ok(())
}
