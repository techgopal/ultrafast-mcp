//! UltraFastServer implementation module
//!
//! This module contains the main server implementation with all the core functionality.

use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use ultrafast_mcp_core::{
    error::{MCPError, MCPResult},
    protocol::{
        capabilities::ServerCapabilities,
        jsonrpc::{JsonRpcError, JsonRpcMessage, JsonRpcRequest, JsonRpcResponse},
    },
    types::{
        prompts::Prompt,
        resources::{Resource, ResourceTemplate},
        server::ServerInfo,
        tools::Tool,
    },
    utils::{CancellationManager, PingManager},
};
#[cfg(feature = "http")]
use ultrafast_mcp_transport::http::server::{HttpTransportConfig, HttpTransportServer};
use ultrafast_mcp_transport::{create_transport, Transport, TransportConfig};

use crate::handlers::*;

/// MCP Server state
#[derive(Debug, Clone)]
pub enum ServerState {
    Uninitialized,
    Initializing,
    Initialized,
    Shutdown,
}

/// MCP Server implementation
#[derive(Clone)]
pub struct UltraFastServer {
    info: ServerInfo,
    capabilities: ServerCapabilities,
    state: Arc<RwLock<ServerState>>,
    tools: Arc<RwLock<HashMap<String, Tool>>>,
    resources: Arc<RwLock<HashMap<String, Resource>>>,
    resource_templates: Arc<RwLock<HashMap<String, ResourceTemplate>>>,
    prompts: Arc<RwLock<HashMap<String, Prompt>>>,
    tool_handler: Option<Arc<dyn ToolHandler>>,
    resource_handler: Option<Arc<dyn ResourceHandler>>,
    prompt_handler: Option<Arc<dyn PromptHandler>>,
    sampling_handler: Option<Arc<dyn SamplingHandler>>,
    completion_handler: Option<Arc<dyn CompletionHandler>>,
    roots_handler: Option<Arc<dyn RootsHandler>>,
    elicitation_handler: Option<Arc<dyn ElicitationHandler>>,
    subscription_handler: Option<Arc<dyn ResourceSubscriptionHandler>>,
    resource_subscriptions: Arc<RwLock<HashMap<String, Vec<String>>>>,
    cancellation_manager: Arc<CancellationManager>,
    ping_manager: Arc<PingManager>,

    #[cfg(feature = "monitoring")]
    monitoring_system: Option<Arc<ultrafast_mcp_monitoring::MonitoringSystem>>,
}

impl std::fmt::Debug for UltraFastServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UltraFastServer")
            .field("info", &self.info)
            .field("capabilities", &self.capabilities)
            .finish()
    }
}

impl UltraFastServer {
    /// Create a new UltraFastServer with the given info and capabilities
    pub fn new(info: ServerInfo, capabilities: ServerCapabilities) -> Self {
        Self {
            info,
            capabilities,
            state: Arc::new(RwLock::new(ServerState::Uninitialized)),
            tools: Arc::new(RwLock::new(HashMap::new())),
            resources: Arc::new(RwLock::new(HashMap::new())),
            resource_templates: Arc::new(RwLock::new(HashMap::new())),
            prompts: Arc::new(RwLock::new(HashMap::new())),
            tool_handler: None,
            resource_handler: None,
            prompt_handler: None,
            sampling_handler: None,
            completion_handler: None,
            roots_handler: None,
            elicitation_handler: None,
            subscription_handler: None,
            resource_subscriptions: Arc::new(RwLock::new(HashMap::new())),
            cancellation_manager: Arc::new(CancellationManager::new()),
            ping_manager: Arc::new(PingManager::default()),

            #[cfg(feature = "monitoring")]
            monitoring_system: None,
        }
    }

    /// Add a tool handler to the server
    pub fn with_tool_handler(mut self, handler: Arc<dyn ToolHandler>) -> Self {
        self.tool_handler = Some(handler);
        self
    }

    /// Add a resource handler to the server
    pub fn with_resource_handler(mut self, handler: Arc<dyn ResourceHandler>) -> Self {
        self.resource_handler = Some(handler);
        self
    }

    /// Add a prompt handler to the server
    pub fn with_prompt_handler(mut self, handler: Arc<dyn PromptHandler>) -> Self {
        self.prompt_handler = Some(handler);
        self
    }

    /// Add a sampling handler to the server
    pub fn with_sampling_handler(mut self, handler: Arc<dyn SamplingHandler>) -> Self {
        self.sampling_handler = Some(handler);
        self
    }

    /// Add a completion handler to the server
    pub fn with_completion_handler(mut self, handler: Arc<dyn CompletionHandler>) -> Self {
        self.completion_handler = Some(handler);
        self
    }

    /// Add a roots handler to the server
    pub fn with_roots_handler(mut self, handler: Arc<dyn RootsHandler>) -> Self {
        self.roots_handler = Some(handler);
        self
    }

    /// Add an elicitation handler to the server
    pub fn with_elicitation_handler(mut self, handler: Arc<dyn ElicitationHandler>) -> Self {
        self.elicitation_handler = Some(handler);
        self
    }

    /// Add a subscription handler to the server
    pub fn with_subscription_handler(
        mut self,
        handler: Arc<dyn ResourceSubscriptionHandler>,
    ) -> Self {
        self.subscription_handler = Some(handler);
        self
    }

    /// Run the server with stdio transport
    pub async fn run_stdio(&self) -> MCPResult<()> {
        let transport = create_transport(TransportConfig::Stdio)
            .await
            .map_err(|e| MCPError::internal_error(format!("Transport creation failed: {}", e)))?;
        self.run_with_transport(transport).await
    }

    /// Run the server with a custom transport
    pub async fn run_with_transport(&self, mut transport: Box<dyn Transport>) -> MCPResult<()> {
        info!("Starting UltraFastServer with transport");

        // Initialize the server
        *self.state.write().await = ServerState::Initializing;

        // Start message handling loop
        loop {
            match transport.receive_message().await {
                Ok(message) => {
                    if let Err(e) = self.handle_message(message, &mut transport).await {
                        error!("Error handling message: {}", e);
                    }
                }
                Err(e) => {
                    error!("Transport error: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Run the server with Streamable HTTP transport
    #[cfg(feature = "http")]
    pub async fn run_streamable_http(&self, host: &str, port: u16) -> MCPResult<()> {
        info!(
            "Starting UltraFastServer with Streamable HTTP on {}:{}",
            host, port
        );

        let config = HttpTransportConfig {
            host: host.to_string(),
            port,
            ..Default::default()
        };

        self.run_http(config).await
    }

    /// Run the server with HTTP transport
    #[cfg(feature = "http")]
    pub async fn run_http(&self, config: HttpTransportConfig) -> MCPResult<()> {
        let server = HttpTransportServer::new(config);
        server
            .run()
            .await
            .map_err(|e| MCPError::internal_error(format!("HTTP server failed: {}", e)))?;
        Ok(())
    }

    /// Get server info
    pub fn info(&self) -> &ServerInfo {
        &self.info
    }

    /// Get cancellation manager
    pub fn cancellation_manager(&self) -> Arc<CancellationManager> {
        self.cancellation_manager.clone()
    }

    /// Get ping manager
    pub fn ping_manager(&self) -> Arc<PingManager> {
        self.ping_manager.clone()
    }

    /// Handle incoming messages
    async fn handle_message(
        &self,
        message: JsonRpcMessage,
        transport: &mut Box<dyn Transport>,
    ) -> MCPResult<()> {
        match message {
            JsonRpcMessage::Request(request) => {
                let response = self.handle_request(request).await;
                transport
                    .send_message(JsonRpcMessage::Response(response))
                    .await
                    .map_err(|e| {
                        MCPError::internal_error(format!("Failed to send message: {}", e))
                    })?;
            }
            JsonRpcMessage::Notification(notification) => {
                self.handle_notification(notification).await?;
            }
            JsonRpcMessage::Response(_) => {
                warn!("Received unexpected response message");
            }
        }
        Ok(())
    }

    /// Handle incoming requests
    async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        // Implementation will be moved to handlers module
        JsonRpcResponse::error(
            JsonRpcError::new(-32601, "Method not implemented".to_string()),
            request.id,
        )
    }

    /// Handle incoming notifications
    async fn handle_notification(&self, _notification: JsonRpcRequest) -> MCPResult<()> {
        // Implementation will be moved to handlers module
        Ok(())
    }
}
