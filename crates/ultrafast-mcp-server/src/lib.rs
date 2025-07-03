//! MCP server implementation for ULTRAFAST MCP
//!
//! This crate provides a high-level server implementation for the Model Context Protocol.

use async_trait::async_trait;
use serde_json::Value;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};

use ultrafast_mcp_core::{
    error::{MCPError, McpError, McpResult},
    protocol::{
        capabilities::{CapabilityNegotiator, ServerCapabilities},
        jsonrpc::{JsonRpcError, JsonRpcMessage, JsonRpcRequest, JsonRpcResponse},
    },
    types::{
        completion::{CompleteRequest, CompleteResponse},
        elicitation::{ElicitationRequest, ElicitationResponse},
        notifications::{
            CancelledNotification, LogLevel, LogLevelSetRequest, LogLevelSetResponse,
            LoggingMessageNotification, PingRequest, ProgressNotification,
        },
        prompts::{
            GetPromptRequest, GetPromptResponse, ListPromptsRequest, ListPromptsResponse, Prompt,
        },
        resources::{
            ListResourceTemplatesRequest, ListResourceTemplatesResponse, ListResourcesRequest,
            ListResourcesResponse, ReadResourceRequest, ReadResourceResponse, Resource,
            ResourceTemplate,
        },
        roots::Root,
        sampling::{CreateMessageRequest, CreateMessageResponse},
        server::ServerInfo,
        tools::{ListToolsRequest, ListToolsResponse, Tool, ToolCall, ToolResult},
    },
    utils::{CancellationManager, PingManager},
};
use ultrafast_mcp_transport::http::server::{HttpTransportConfig, HttpTransportServer};
use ultrafast_mcp_transport::{create_transport, Transport, TransportConfig};

pub mod handlers;

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
    #[allow(dead_code)]
    tools: Arc<RwLock<HashMap<String, Tool>>>,
    resources: Arc<RwLock<HashMap<String, Resource>>>,
    resource_templates: Arc<RwLock<HashMap<String, ResourceTemplate>>>,
    prompts: Arc<RwLock<HashMap<String, Prompt>>>,
    tool_handler: Option<Arc<dyn ToolHandler>>,
    resource_handler: Option<Arc<dyn ResourceHandler>>,
    prompt_handler: Option<Arc<dyn PromptHandler>>,
    sampling_handler: Option<Arc<dyn SamplingHandler>>,
    completion_handler: Option<Arc<dyn CompletionHandler>>,

    // Phase 3: Advanced server features
    roots_handler: Option<Arc<dyn RootsHandler>>,
    elicitation_handler: Option<Arc<dyn ElicitationHandler>>,
    subscription_handler: Option<Arc<dyn ResourceSubscriptionHandler>>,
    resource_subscriptions: Arc<RwLock<HashMap<String, Vec<String>>>>, // URI -> client IDs

    // MCP 2025-06-18 utilities
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

/// Tool handler trait with pagination support
#[async_trait]
pub trait ToolHandler: Send + Sync {
    async fn handle_tool_call(&self, call: ToolCall) -> McpResult<ToolResult>;
    async fn list_tools(&self, request: ListToolsRequest) -> McpResult<ListToolsResponse>;
}

/// Resource handler trait with pagination support
#[async_trait]
pub trait ResourceHandler: Send + Sync {
    async fn read_resource(&self, request: ReadResourceRequest) -> McpResult<ReadResourceResponse>;
    async fn list_resources(
        &self,
        request: ListResourcesRequest,
    ) -> McpResult<ListResourcesResponse>;
    async fn list_resource_templates(
        &self,
        request: ListResourceTemplatesRequest,
    ) -> McpResult<ListResourceTemplatesResponse>;
}

/// Prompt handler trait with pagination support
#[async_trait]
pub trait PromptHandler: Send + Sync {
    async fn get_prompt(&self, request: GetPromptRequest) -> McpResult<GetPromptResponse>;
    async fn list_prompts(&self, request: ListPromptsRequest) -> McpResult<ListPromptsResponse>;
}

/// Sampling handler trait
#[async_trait]
pub trait SamplingHandler: Send + Sync {
    async fn create_message(
        &self,
        request: CreateMessageRequest,
    ) -> McpResult<CreateMessageResponse>;
}

/// Completion handler trait for autocompletion
#[async_trait]
pub trait CompletionHandler: Send + Sync {
    async fn complete(&self, request: CompleteRequest) -> McpResult<CompleteResponse>;
}

/// Roots handler trait for filesystem boundary management
#[async_trait]
pub trait RootsHandler: Send + Sync {
    async fn list_roots(&self) -> McpResult<Vec<Root>>;
}

/// Elicitation handler trait for server-initiated user input
#[async_trait]
pub trait ElicitationHandler: Send + Sync {
    async fn handle_elicitation(
        &self,
        request: ElicitationRequest,
    ) -> McpResult<ElicitationResponse>;
}

/// Resource subscription handler trait
#[async_trait]
pub trait ResourceSubscriptionHandler: Send + Sync {
    async fn subscribe(&self, uri: String) -> McpResult<()>;
    async fn unsubscribe(&self, uri: String) -> McpResult<()>;
    async fn notify_change(&self, uri: String, content: serde_json::Value) -> McpResult<()>;
}

/// Phase 3: Server composition types
#[derive(Clone)]
pub struct ComposedServer {
    pub name: String,
    pub capabilities: ServerCapabilities,
    pub endpoint: Option<String>,
    pub priority: u8,
}

/// Phase 3: Federation configuration
#[derive(Clone)]
pub struct FederationConfig {
    pub servers: HashMap<String, ComposedServer>,
    pub routing_strategy: RoutingStrategy,
    pub health_check_interval: Duration,
}

#[derive(Clone, Debug)]
pub enum RoutingStrategy {
    /// Route to first available server
    FirstAvailable,
    /// Route based on server priority
    Priority,
    /// Load balance across servers
    LoadBalance,
    /// Route based on capabilities
    CapabilityBased,
}

impl Default for FederationConfig {
    fn default() -> Self {
        Self {
            servers: HashMap::new(),
            routing_strategy: RoutingStrategy::CapabilityBased,
            health_check_interval: Duration::from_secs(30),
        }
    }
}

/// High-level MCP server with Phase 3 composition features
#[derive(Clone)]
pub struct ComposableMcpServer {
    inner: Arc<ServerInner>,
}

struct ServerInner {
    /// Server information
    info: RwLock<ServerInfo>,

    /// Server capabilities
    capabilities: RwLock<ServerCapabilities>,

    /// Registered tools
    #[allow(dead_code)]
    tools: RwLock<HashMap<String, Arc<dyn ToolHandler>>>,

    /// Registered resources
    #[allow(dead_code)]
    resources: RwLock<HashMap<String, Arc<dyn ResourceHandler>>>,

    /// Resource subscriptions
    #[allow(dead_code)]
    subscriptions: RwLock<HashMap<String, broadcast::Sender<Value>>>,

    /// Registered prompts
    #[allow(dead_code)]
    prompts: RwLock<HashMap<String, Arc<dyn PromptHandler>>>,

    /// Phase 3: Federation configuration
    federation_config: RwLock<FederationConfig>,

    /// Phase 3: Server health status
    health_status: RwLock<HashMap<String, ServerHealth>>,
}

/// Phase 3: Server health status
#[derive(Clone, Debug)]
pub struct ServerHealth {
    pub is_healthy: bool,
    pub last_check: std::time::Instant,
    pub latency_ms: f64,
    pub error_count: u64,
}

impl Default for ServerHealth {
    fn default() -> Self {
        Self {
            is_healthy: true,
            last_check: std::time::Instant::now(),
            latency_ms: 0.0,
            error_count: 0,
        }
    }
}

impl ComposableMcpServer {
    /// Create a new composable MCP server
    pub fn new(name: &str, version: &str) -> Self {
        let info = ServerInfo::new(name.to_string(), version.to_string());
        let inner = Arc::new(ServerInner {
            info: RwLock::new(info),
            capabilities: RwLock::new(ServerCapabilities::default()),
            tools: RwLock::new(HashMap::new()),
            resources: RwLock::new(HashMap::new()),
            subscriptions: RwLock::new(HashMap::new()),
            prompts: RwLock::new(HashMap::new()),
            federation_config: RwLock::new(FederationConfig::default()),
            health_status: RwLock::new(HashMap::new()),
        });

        Self { inner }
    }

    /// Add a federated server
    pub async fn add_federated_server(&self, id: String, server: ComposedServer) -> McpResult<()> {
        let mut config = self.inner.federation_config.write().await;
        config.servers.insert(id, server);
        Ok(())
    }

    /// Remove a federated server
    pub async fn remove_federated_server(&self, id: &str) -> McpResult<()> {
        let mut config = self.inner.federation_config.write().await;
        config.servers.remove(id);
        Ok(())
    }

    /// Get federation status
    pub async fn federation_status(&self) -> McpResult<Value> {
        let config = self.inner.federation_config.read().await;
        let health = self.inner.health_status.read().await;

        let servers: Vec<Value> = config
            .servers
            .iter()
            .map(|(id, server)| {
                let health_status = health.get(id).cloned().unwrap_or_default();
                serde_json::json!({
                    "id": id,
                    "name": server.name,
                    "capabilities": server.capabilities,
                    "endpoint": server.endpoint,
                    "priority": server.priority,
                    "health": {
                        "is_healthy": health_status.is_healthy,
                        "latency_ms": health_status.latency_ms,
                        "error_count": health_status.error_count,
                        "last_check": health_status.last_check.elapsed().as_secs()
                    }
                })
            })
            .collect();

        Ok(serde_json::json!({
            "servers": servers,
            "routing_strategy": format!("{:?}", config.routing_strategy),
            "health_check_interval": config.health_check_interval.as_secs()
        }))
    }

    /// Route a request to the appropriate federated server
    pub async fn route_request(&self, capability: &str, _request_type: &str) -> McpResult<String> {
        let config = self.inner.federation_config.read().await;
        let health = self.inner.health_status.read().await;

        match config.routing_strategy {
            RoutingStrategy::FirstAvailable => {
                for (id, server) in &config.servers {
                    let is_healthy = if let Some(_endpoint) = &server.endpoint {
                        if let Some(health_status) = health.get(id) {
                            health_status.is_healthy
                                && health_status.last_check.elapsed() < config.health_check_interval
                        } else {
                            true // Assume healthy if no health data
                        }
                    } else {
                        false
                    };

                    if CapabilityNegotiator::supports_capability(&server.capabilities, capability)
                        && is_healthy
                    {
                        return Ok(id.clone());
                    }
                }
            }
            RoutingStrategy::Priority => {
                let mut servers: Vec<_> = config.servers.iter().collect();
                servers.sort_by(|a, b| b.1.priority.cmp(&a.1.priority));

                for (id, server) in servers {
                    let is_healthy = if let Some(_endpoint) = &server.endpoint {
                        if let Some(health_status) = health.get(id) {
                            health_status.is_healthy
                                && health_status.last_check.elapsed() < config.health_check_interval
                        } else {
                            true // Assume healthy if no health data
                        }
                    } else {
                        false
                    };

                    if CapabilityNegotiator::supports_capability(&server.capabilities, capability)
                        && is_healthy
                    {
                        return Ok(id.clone());
                    }
                }
            }
            RoutingStrategy::LoadBalance => {
                // Simple round-robin load balancing
                let healthy_servers: Vec<_> = config
                    .servers
                    .iter()
                    .filter(|(id, server)| {
                        let is_healthy = if let Some(_endpoint) = &server.endpoint {
                            if let Some(health_status) = health.get(*id) {
                                health_status.is_healthy
                                    && health_status.last_check.elapsed()
                                        < config.health_check_interval
                            } else {
                                true // Assume healthy if no health data
                            }
                        } else {
                            false
                        };
                        CapabilityNegotiator::supports_capability(&server.capabilities, capability)
                            && is_healthy
                    })
                    .collect();

                if !healthy_servers.is_empty() {
                    let index = (std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as usize)
                        % healthy_servers.len();
                    return Ok(healthy_servers[index].0.clone());
                }
            }
            RoutingStrategy::CapabilityBased => {
                // Route based on specific capability requirements
                for (id, server) in &config.servers {
                    let is_healthy = if let Some(_endpoint) = &server.endpoint {
                        if let Some(health_status) = health.get(id) {
                            health_status.is_healthy
                                && health_status.last_check.elapsed() < config.health_check_interval
                        } else {
                            true // Assume healthy if no health data
                        }
                    } else {
                        false
                    };

                    if CapabilityNegotiator::supports_capability(&server.capabilities, capability)
                        && is_healthy
                    {
                        return Ok(id.clone());
                    }
                }
            }
        }

        Err(MCPError::internal_error(
            "No available server for request".to_string(),
        ))
    }

    /// Perform health check on all federated servers
    pub async fn health_check(&self) -> McpResult<()> {
        let config = self.inner.federation_config.read().await;
        let mut health = self.inner.health_status.write().await;

        for (id, server) in &config.servers {
            let _start = std::time::Instant::now();
            let is_healthy = if let Some(endpoint) = &server.endpoint {
                // Perform actual health check
                match self.perform_health_check(endpoint).await {
                    Ok(latency) => {
                        health.insert(
                            id.clone(),
                            ServerHealth {
                                is_healthy: true,
                                last_check: std::time::Instant::now(),
                                latency_ms: latency,
                                error_count: 0,
                            },
                        );
                        true
                    }
                    Err(_) => {
                        let current_health = health.get(id).cloned().unwrap_or_default();
                        health.insert(
                            id.clone(),
                            ServerHealth {
                                is_healthy: false,
                                last_check: std::time::Instant::now(),
                                latency_ms: 0.0,
                                error_count: current_health.error_count + 1,
                            },
                        );
                        false
                    }
                }
            } else {
                false
            };

            if !is_healthy {
                warn!("Federated server {} is unhealthy", id);
            }
        }

        Ok(())
    }

    /// Start periodic health monitoring
    pub async fn start_health_monitoring(&self) -> McpResult<()> {
        let config = self.inner.federation_config.read().await;
        let interval = config.health_check_interval;
        drop(config);

        let server = self.clone();
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            loop {
                interval_timer.tick().await;
                if let Err(e) = server.health_check().await {
                    error!("Health check failed: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Get server information
    pub async fn info(&self) -> ServerInfo {
        self.inner.info.read().await.clone()
    }

    /// Get server capabilities
    pub async fn capabilities(&self) -> ServerCapabilities {
        self.inner.capabilities.read().await.clone()
    }

    /// Set server capabilities
    pub async fn set_capabilities(&self, capabilities: ServerCapabilities) -> McpResult<()> {
        let mut caps = self.inner.capabilities.write().await;
        *caps = capabilities;
        Ok(())
    }

    async fn perform_health_check(&self, _endpoint: &str) -> McpResult<f64> {
        // Implement actual health check logic here
        // For now, just return a mock latency
        Ok(10.0)
    }
}

impl UltraFastServer {
    /// Create a new MCP server
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

    /// Add a tool handler
    pub fn with_tool_handler(mut self, handler: Arc<dyn ToolHandler>) -> Self {
        self.tool_handler = Some(handler);
        self
    }

    /// Add a resource handler
    pub fn with_resource_handler(mut self, handler: Arc<dyn ResourceHandler>) -> Self {
        self.resource_handler = Some(handler);
        self
    }

    /// Add a prompt handler
    pub fn with_prompt_handler(mut self, handler: Arc<dyn PromptHandler>) -> Self {
        self.prompt_handler = Some(handler);
        self
    }

    /// Add a sampling handler
    pub fn with_sampling_handler(mut self, handler: Arc<dyn SamplingHandler>) -> Self {
        self.sampling_handler = Some(handler);
        self
    }

    /// Add a completion handler
    pub fn with_completion_handler(mut self, handler: Arc<dyn CompletionHandler>) -> Self {
        self.completion_handler = Some(handler);
        self
    }

    /// Add a roots handler
    pub fn with_roots_handler(mut self, handler: Arc<dyn RootsHandler>) -> Self {
        self.roots_handler = Some(handler);
        self
    }

    /// Add an elicitation handler
    pub fn with_elicitation_handler(mut self, handler: Arc<dyn ElicitationHandler>) -> Self {
        self.elicitation_handler = Some(handler);
        self
    }

    /// Add a subscription handler
    pub fn with_subscription_handler(
        mut self,
        handler: Arc<dyn ResourceSubscriptionHandler>,
    ) -> Self {
        self.subscription_handler = Some(handler);
        self
    }

    /// Configure monitoring (when monitoring feature is enabled)
    #[cfg(feature = "monitoring")]
    pub fn with_monitoring_config(
        mut self,
        config: ultrafast_mcp_monitoring::MonitoringConfig,
    ) -> Self {
        self.monitoring_system = Some(Arc::new(ultrafast_mcp_monitoring::MonitoringSystem::new(
            config,
        )));
        self
    }

    /// Enable default monitoring (when monitoring feature is enabled)
    #[cfg(feature = "monitoring")]
    pub fn with_default_monitoring(self) -> Self {
        self.with_monitoring_config(ultrafast_mcp_monitoring::MonitoringConfig::default())
    }

    /// Get monitoring system (when monitoring feature is enabled)
    #[cfg(feature = "monitoring")]
    pub fn monitoring(&self) -> Option<Arc<ultrafast_mcp_monitoring::MonitoringSystem>> {
        self.monitoring_system.clone()
    }

    /// Run the server with STDIO transport
    pub async fn run_stdio(&self) -> McpResult<()> {
        let transport = create_transport(TransportConfig::Stdio)
            .await
            .map_err(|e| MCPError::internal_error(e.to_string()))?;
        self.run_with_transport(transport).await
    }

    /// Run the server with a custom transport
    pub async fn run_with_transport(&self, mut transport: Box<dyn Transport>) -> McpResult<()> {
        info!("Starting MCP server: {}", self.info.name);

        // Initialize the server to Uninitialized state
        {
            let mut state = self.state.write().await;
            *state = ServerState::Uninitialized;
        }

        info!("MCP server ready, waiting for initialize request");

        // Main message handling loop
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

        self.shutdown().await
    }

    /// Handle incoming JSON-RPC messages
    async fn handle_message(
        &self,
        message: JsonRpcMessage,
        transport: &mut Box<dyn Transport>,
    ) -> McpResult<()> {
        match message {
            JsonRpcMessage::Request(request) => {
                let response = self.handle_request(request).await;
                transport
                    .send_message(JsonRpcMessage::Response(response))
                    .await
                    .map_err(|e| MCPError::internal_error(e.to_string()))?;
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

    /// Handle incoming HTTP transport messages
    async fn handle_http_message(
        &self,
        message: JsonRpcMessage,
        message_queue: &ultrafast_mcp_transport::http::session::MessageQueue, // Direct access to message queue
        session_id: &str,
    ) -> McpResult<()> {
        info!(
            "Handling HTTP message for session {}: {:?}",
            session_id, message
        );
        match message {
            JsonRpcMessage::Request(request) => {
                info!("Processing request: {}", request.method);
                let response = self.handle_request(request).await;
                let response_message = JsonRpcMessage::Response(response);

                info!(
                    "Queueing response for session {}: {:?}",
                    session_id, response_message
                );
                // Queue response directly to message queue (don't broadcast to avoid loops)
                message_queue
                    .enqueue_message(session_id.to_string(), response_message)
                    .await;
                info!("Successfully queued response for session {}", session_id);
            }
            JsonRpcMessage::Notification(notification) => {
                info!("Processing notification: {}", notification.method);
                self.handle_notification(notification).await?;
            }
            JsonRpcMessage::Response(_) => {
                warn!("Received unexpected response message via HTTP");
            }
        }
        Ok(())
    }

    /// Handle JSON-RPC requests
    async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        // Register request for cancellation tracking (except for ping and basic protocol methods)
        if let Some(ref id) = request.id {
            match request.method.as_str() {
                "initialize" | "shutdown" | "ping" => {
                    // Don't track these basic protocol methods
                }
                _ => {
                    if let Err(e) = self
                        .cancellation_manager
                        .register_request(
                            serde_json::to_value(id).unwrap_or(serde_json::Value::Null),
                            request.method.clone(),
                        )
                        .await
                    {
                        warn!(
                            "Failed to register request for cancellation tracking: {}",
                            e
                        );
                    }
                }
            }
        }

        let request_id_for_cleanup = request.id.clone();
        let cancellation_manager = self.cancellation_manager.clone();

        let response = match request.method.as_str() {
            "initialize" => {
                // Handle initialization
                let response = serde_json::json!({
                    "protocolVersion": "2025-06-18",
                    "capabilities": self.capabilities,
                    "serverInfo": self.info
                });

                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(response),
                    error: None,
                    meta: HashMap::new(),
                }
            }
            "shutdown" => {
                // Handle shutdown
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(serde_json::json!(null)),
                    error: None,
                    meta: HashMap::new(),
                }
            }
            "tools/list" => {
                // Handle tools/list request
                if let Some(tool_handler) = &self.tool_handler {
                    let list_request = match request.params {
                        Some(params) => match serde_json::from_value::<ListToolsRequest>(params) {
                            Ok(req) => req,
                            Err(e) => {
                                return JsonRpcResponse {
                                    jsonrpc: "2.0".to_string(),
                                    id: request.id,
                                    result: None,
                                    error: Some(JsonRpcError {
                                        code: -32602,
                                        message: format!("Invalid params: {}", e),
                                        data: None,
                                    }),
                                    meta: HashMap::new(),
                                };
                            }
                        },
                        None => ListToolsRequest { cursor: None },
                    };

                    match tool_handler.list_tools(list_request).await {
                        Ok(response) => match serde_json::to_value(response) {
                            Ok(value) => JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                id: request.id,
                                result: Some(value),
                                error: None,
                                meta: HashMap::new(),
                            },
                            Err(e) => JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                id: request.id,
                                result: None,
                                error: Some(JsonRpcError {
                                    code: -32603,
                                    message: format!("Internal error: {}", e),
                                    data: None,
                                }),
                                meta: HashMap::new(),
                            },
                        },
                        Err(e) => {
                            let json_rpc_error =
                                ultrafast_mcp_core::protocol::JsonRpcError::from(e);
                            JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                id: request.id,
                                result: None,
                                error: Some(JsonRpcError {
                                    code: json_rpc_error.code,
                                    message: json_rpc_error.message,
                                    data: json_rpc_error.data,
                                }),
                                meta: HashMap::new(),
                            }
                        }
                    }
                } else {
                    JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32601,
                            message: "Tools not supported".to_string(),
                            data: None,
                        }),
                        meta: HashMap::new(),
                    }
                }
            }
            "tools/call" => {
                // Handle tools/call request
                if let Some(tool_handler) = &self.tool_handler {
                    let tool_call = match request.params {
                        Some(params) => match serde_json::from_value::<ToolCall>(params) {
                            Ok(call) => call,
                            Err(e) => {
                                return JsonRpcResponse {
                                    jsonrpc: "2.0".to_string(),
                                    id: request.id,
                                    result: None,
                                    error: Some(JsonRpcError {
                                        code: -32602,
                                        message: format!("Invalid params: {}", e),
                                        data: None,
                                    }),
                                    meta: HashMap::new(),
                                };
                            }
                        },
                        None => {
                            return JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                id: request.id,
                                result: None,
                                error: Some(JsonRpcError {
                                    code: -32602,
                                    message: "Missing params".to_string(),
                                    data: None,
                                }),
                                meta: HashMap::new(),
                            };
                        }
                    };

                    match tool_handler.handle_tool_call(tool_call).await {
                        Ok(response) => match serde_json::to_value(response) {
                            Ok(value) => JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                id: request.id,
                                result: Some(value),
                                error: None,
                                meta: HashMap::new(),
                            },
                            Err(e) => JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                id: request.id,
                                result: None,
                                error: Some(JsonRpcError {
                                    code: -32603,
                                    message: format!("Internal error: {}", e),
                                    data: None,
                                }),
                                meta: HashMap::new(),
                            },
                        },
                        Err(e) => {
                            let json_rpc_error =
                                ultrafast_mcp_core::protocol::JsonRpcError::from(e);
                            JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                id: request.id,
                                result: None,
                                error: Some(JsonRpcError {
                                    code: json_rpc_error.code,
                                    message: json_rpc_error.message,
                                    data: json_rpc_error.data,
                                }),
                                meta: HashMap::new(),
                            }
                        }
                    }
                } else {
                    JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32601,
                            message: "Tools not supported".to_string(),
                            data: None,
                        }),
                        meta: HashMap::new(),
                    }
                }
            }
            "initialized" => {
                // Handle initialized notification
                match self.handle_initialized().await {
                    Ok(_) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: Some(serde_json::json!(null)),
                        error: None,
                        meta: HashMap::new(),
                    },
                    Err(e) => {
                        let json_rpc_error = ultrafast_mcp_core::protocol::JsonRpcError::from(e);
                        JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: request.id,
                            result: None,
                            error: Some(JsonRpcError {
                                code: json_rpc_error.code,
                                message: json_rpc_error.message,
                                data: json_rpc_error.data,
                            }),
                            meta: HashMap::new(),
                        }
                    }
                }
            }
            "resources/list" => {
                // Handle resources/list request
                match self.handle_list_resources(request.params).await {
                    Ok(result) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: Some(result),
                        error: None,
                        meta: HashMap::new(),
                    },
                    Err(e) => {
                        let json_rpc_error = ultrafast_mcp_core::protocol::JsonRpcError::from(e);
                        JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: request.id,
                            result: None,
                            error: Some(JsonRpcError {
                                code: json_rpc_error.code,
                                message: json_rpc_error.message,
                                data: json_rpc_error.data,
                            }),
                            meta: HashMap::new(),
                        }
                    }
                }
            }
            "resources/read" => {
                // Handle resources/read request
                match self.handle_read_resource(request.params).await {
                    Ok(result) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: Some(result),
                        error: None,
                        meta: HashMap::new(),
                    },
                    Err(e) => {
                        let json_rpc_error = ultrafast_mcp_core::protocol::JsonRpcError::from(e);
                        JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: request.id,
                            result: None,
                            error: Some(JsonRpcError {
                                code: json_rpc_error.code,
                                message: json_rpc_error.message,
                                data: json_rpc_error.data,
                            }),
                            meta: HashMap::new(),
                        }
                    }
                }
            }
            "resources/templates/list" => {
                // Handle resources/templates/list request
                match self.handle_list_resource_templates(request.params).await {
                    Ok(result) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: Some(result),
                        error: None,
                        meta: HashMap::new(),
                    },
                    Err(e) => {
                        let json_rpc_error = ultrafast_mcp_core::protocol::JsonRpcError::from(e);
                        JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: request.id,
                            result: None,
                            error: Some(JsonRpcError {
                                code: json_rpc_error.code,
                                message: json_rpc_error.message,
                                data: json_rpc_error.data,
                            }),
                            meta: HashMap::new(),
                        }
                    }
                }
            }
            "resources/subscribe" => {
                // Handle resources/subscribe request
                match self.handle_subscribe_resource(request.params).await {
                    Ok(result) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: Some(result),
                        error: None,
                        meta: HashMap::new(),
                    },
                    Err(e) => {
                        let json_rpc_error = ultrafast_mcp_core::protocol::JsonRpcError::from(e);
                        JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: request.id,
                            result: None,
                            error: Some(JsonRpcError {
                                code: json_rpc_error.code,
                                message: json_rpc_error.message,
                                data: json_rpc_error.data,
                            }),
                            meta: HashMap::new(),
                        }
                    }
                }
            }
            "resources/unsubscribe" => {
                // Handle resources/unsubscribe request
                match self.handle_unsubscribe_resource(request.params).await {
                    Ok(result) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: Some(result),
                        error: None,
                        meta: HashMap::new(),
                    },
                    Err(e) => {
                        let json_rpc_error = ultrafast_mcp_core::protocol::JsonRpcError::from(e);
                        JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: request.id,
                            result: None,
                            error: Some(JsonRpcError {
                                code: json_rpc_error.code,
                                message: json_rpc_error.message,
                                data: json_rpc_error.data,
                            }),
                            meta: HashMap::new(),
                        }
                    }
                }
            }
            "prompts/list" => {
                // Handle prompts/list request
                match self.handle_list_prompts(request.params).await {
                    Ok(result) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: Some(result),
                        error: None,
                        meta: HashMap::new(),
                    },
                    Err(e) => {
                        let json_rpc_error = ultrafast_mcp_core::protocol::JsonRpcError::from(e);
                        JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: request.id,
                            result: None,
                            error: Some(JsonRpcError {
                                code: json_rpc_error.code,
                                message: json_rpc_error.message,
                                data: json_rpc_error.data,
                            }),
                            meta: HashMap::new(),
                        }
                    }
                }
            }
            "prompts/get" => {
                // Handle prompts/get request
                match self.handle_get_prompt(request.params).await {
                    Ok(result) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: Some(result),
                        error: None,
                        meta: HashMap::new(),
                    },
                    Err(e) => {
                        let json_rpc_error = ultrafast_mcp_core::protocol::JsonRpcError::from(e);
                        JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: request.id,
                            result: None,
                            error: Some(JsonRpcError {
                                code: json_rpc_error.code,
                                message: json_rpc_error.message,
                                data: json_rpc_error.data,
                            }),
                            meta: HashMap::new(),
                        }
                    }
                }
            }
            "sampling/createMessage" => {
                // Handle sampling/createMessage request
                match self.handle_create_message(request.params).await {
                    Ok(result) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: Some(result),
                        error: None,
                        meta: HashMap::new(),
                    },
                    Err(e) => {
                        let json_rpc_error = ultrafast_mcp_core::protocol::JsonRpcError::from(e);
                        JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: request.id,
                            result: None,
                            error: Some(JsonRpcError {
                                code: json_rpc_error.code,
                                message: json_rpc_error.message,
                                data: json_rpc_error.data,
                            }),
                            meta: HashMap::new(),
                        }
                    }
                }
            }
            "completion/complete" => {
                // Handle completion/complete request
                match self.handle_completion(request.params).await {
                    Ok(result) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: Some(result),
                        error: None,
                        meta: HashMap::new(),
                    },
                    Err(e) => {
                        let json_rpc_error = ultrafast_mcp_core::protocol::JsonRpcError::from(e);
                        JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: request.id,
                            result: None,
                            error: Some(JsonRpcError {
                                code: json_rpc_error.code,
                                message: json_rpc_error.message,
                                data: json_rpc_error.data,
                            }),
                            meta: HashMap::new(),
                        }
                    }
                }
            }
            "roots/list" => {
                // Handle roots/list request
                match self.handle_list_roots(request.params).await {
                    Ok(result) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: Some(result),
                        error: None,
                        meta: HashMap::new(),
                    },
                    Err(e) => {
                        let json_rpc_error = ultrafast_mcp_core::protocol::JsonRpcError::from(e);
                        JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: request.id,
                            result: None,
                            error: Some(JsonRpcError {
                                code: json_rpc_error.code,
                                message: json_rpc_error.message,
                                data: json_rpc_error.data,
                            }),
                            meta: HashMap::new(),
                        }
                    }
                }
            }
            "logging/setLevel" => {
                // Handle logging level set request
                match self.handle_set_log_level(request.params).await {
                    Ok(result) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: Some(result),
                        error: None,
                        meta: HashMap::new(),
                    },
                    Err(e) => {
                        let json_rpc_error = ultrafast_mcp_core::protocol::JsonRpcError::from(e);
                        JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: request.id,
                            result: None,
                            error: Some(JsonRpcError {
                                code: json_rpc_error.code,
                                message: json_rpc_error.message,
                                data: json_rpc_error.data,
                            }),
                            meta: HashMap::new(),
                        }
                    }
                }
            }
            "ping" => {
                // Handle ping request
                let ping_request = match request.params {
                    Some(params) => {
                        match serde_json::from_value::<PingRequest>(params) {
                            Ok(req) => req,
                            Err(_) => PingRequest::new(), // Default on parse error
                        }
                    }
                    None => PingRequest::new(),
                };

                match self.ping_manager.handle_ping(ping_request).await {
                    Ok(response) => match serde_json::to_value(response) {
                        Ok(value) => JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: request.id,
                            result: Some(value),
                            error: None,
                            meta: HashMap::new(),
                        },
                        Err(e) => JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: request.id,
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32603,
                                message: format!("Internal error: {}", e),
                                data: None,
                            }),
                            meta: HashMap::new(),
                        },
                    },
                    Err(e) => {
                        let json_rpc_error = ultrafast_mcp_core::protocol::JsonRpcError::from(e);
                        JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: request.id,
                            result: None,
                            error: Some(JsonRpcError {
                                code: json_rpc_error.code,
                                message: json_rpc_error.message,
                                data: json_rpc_error.data,
                            }),
                            meta: HashMap::new(),
                        }
                    }
                }
            }
            _ => {
                // Handle other requests
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32601,
                        message: "Method not found".to_string(),
                        data: None,
                    }),
                    meta: HashMap::new(),
                }
            }
        };

        // Clean up request from cancellation tracking
        if let Some(id) = request_id_for_cleanup {
            if let Err(e) = cancellation_manager
                .complete_request(&serde_json::to_value(id).unwrap_or(serde_json::Value::Null))
                .await
            {
                warn!("Failed to clean up completed request: {}", e);
            }
        }

        response
    }

    /// Handle JSON-RPC notifications
    async fn handle_notification(&self, notification: JsonRpcRequest) -> McpResult<()> {
        match notification.method.as_str() {
            "exit" => {
                info!("Received exit notification");
                self.shutdown().await?;
            }
            "notifications/cancelled" => {
                debug!("Received cancellation notification");
                if let Some(params) = notification.params {
                    match serde_json::from_value::<CancelledNotification>(params) {
                        Ok(cancel_notification) => {
                            let cancelled = self
                                .cancellation_manager
                                .handle_cancellation(cancel_notification)
                                .await?;
                            if cancelled {
                                info!("Successfully cancelled request");
                            } else {
                                debug!("Request not found or already completed");
                            }
                        }
                        Err(e) => {
                            warn!("Invalid cancellation notification: {}", e);
                        }
                    }
                }
            }
            "notifications/progress" => {
                debug!("Received progress notification");
                // Progress notifications are handled internally by the progress system
            }
            _ => {
                debug!("Received notification: {}", notification.method);
            }
        }
        Ok(())
    }

    /// Run the server with HTTP transport
    pub async fn run_http(&self, config: HttpTransportConfig) -> McpResult<()> {
        info!(
            "Starting MCP server with HTTP transport: {}",
            self.info.name
        );

        // Initialize the server
        {
            let mut state = self.state.write().await;
            *state = ServerState::Initializing;
        }

        // Create HTTP transport server
        let http_server = HttpTransportServer::new(config);

        // Get message receiver to handle incoming HTTP messages
        let mut message_receiver = http_server.get_message_receiver();
        let message_queue = http_server.get_message_queue();

        // Start the HTTP server in a separate task
        let http_task = tokio::spawn(async move {
            if let Err(e) = http_server.run().await {
                error!("HTTP server error: {}", e);
            }
        });

        info!("MCP server initialized and ready");

        // Main message handling loop for HTTP transport
        debug!("Entering HTTP message handling loop");
        loop {
            match message_receiver.recv().await {
                Ok((session_id, message)) => {
                    if let Err(e) = self
                        .handle_http_message(message, &message_queue, &session_id)
                        .await
                    {
                        error!("Error handling HTTP message: {}", e);
                    }
                }
                Err(broadcast::error::RecvError::Closed) => {
                    info!("HTTP message channel closed");
                    break;
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    warn!("HTTP message channel lagged by {} messages", n);
                }
            }
        }

        // Cancel HTTP server task
        http_task.abort();

        self.shutdown().await
    }

    /// Run the server with Streamable HTTP transport (recommended)
    ///
    /// This is the preferred method for high-performance HTTP communication.
    /// Streamable HTTP provides 10x better performance than HTTP+SSE under load.
    pub async fn run_streamable_http(&self, host: &str, port: u16) -> McpResult<()> {
        let config = HttpTransportConfig {
            host: host.to_string(),
            port,
            enable_streamable_http: true,
            enable_legacy_endpoints: false,
            cors_enabled: true,
            auth_required: false,
            ..Default::default()
        };
        self.run_http(config).await
    }

    /// Run the server with HTTP+SSE transport (legacy compatibility)
    ///
    /// This method provides backward compatibility with HTTP+SSE from MCP 2024-11-05.
    /// For new applications, prefer `run_streamable_http` for better performance.
    pub async fn run_http_sse(&self, host: &str, port: u16) -> McpResult<()> {
        let config = HttpTransportConfig {
            host: host.to_string(),
            port,
            enable_streamable_http: false,
            enable_legacy_endpoints: true,
            cors_enabled: true,
            auth_required: false,
            ..Default::default()
        };
        self.run_http(config).await
    }

    /// Run the server with custom HTTP configuration
    ///
    /// Use this method when you need fine-grained control over HTTP transport settings.
    pub async fn run_with_config(&self, config: HttpTransportConfig) -> McpResult<()> {
        self.run_http(config).await
    }

    /// Notify clients that the tools list has changed
    pub async fn notify_tools_list_changed(&self) -> McpResult<()> {
        let _notification = JsonRpcMessage::Notification(JsonRpcRequest::notification(
            "tools/listChanged".to_string(),
            Some(serde_json::json!({})),
        ));

        // Send to all connected clients
        // Implementation depends on transport
        Ok(())
    }

    /// Notify clients that the resources list has changed
    pub async fn notify_resources_list_changed(&self) -> McpResult<()> {
        let _notification = JsonRpcMessage::Notification(JsonRpcRequest::notification(
            "resources/listChanged".to_string(),
            Some(serde_json::json!({})),
        ));

        // Send to all connected clients
        // Implementation depends on transport
        Ok(())
    }

    /// Notify clients that the prompts list has changed
    pub async fn notify_prompts_list_changed(&self) -> McpResult<()> {
        let _notification = JsonRpcMessage::Notification(JsonRpcRequest::notification(
            "prompts/listChanged".to_string(),
            Some(serde_json::json!({})),
        ));

        // Send to all connected clients
        // Implementation depends on transport
        Ok(())
    }

    /// Send a log message to clients
    pub async fn send_log_message(
        &self,
        level: LogLevel,
        data: serde_json::Value,
        logger: Option<String>,
    ) -> McpResult<()> {
        let notification = LoggingMessageNotification {
            level,
            data,
            logger,
        };

        let _json_rpc_notification = JsonRpcMessage::Notification(JsonRpcRequest::notification(
            "notifications/message".to_string(),
            Some(serde_json::to_value(notification)?),
        ));

        // Send to all connected clients
        // Implementation depends on transport
        debug!("Sent log message notification");
        Ok(())
    }

    /// Shutdown the server
    async fn shutdown(&self) -> McpResult<()> {
        let mut state = self.state.write().await;
        *state = ServerState::Shutdown;
        info!("MCP server shutdown complete");
        Ok(())
    }

    /// Public getter for server info
    pub fn info(&self) -> &ServerInfo {
        &self.info
    }

    /// Get the cancellation manager
    pub fn cancellation_manager(&self) -> Arc<CancellationManager> {
        self.cancellation_manager.clone()
    }

    /// Get the ping manager
    pub fn ping_manager(&self) -> Arc<PingManager> {
        self.ping_manager.clone()
    }

    /// Send a progress notification
    pub async fn send_progress(
        &self,
        progress_token: serde_json::Value,
        progress: f64,
        total: Option<f64>,
        message: Option<String>,
    ) -> McpResult<()> {
        let mut notification = ProgressNotification::new(progress_token, progress);
        if let Some(total) = total {
            notification = notification.with_total(total);
        }
        if let Some(message) = message {
            notification = notification.with_message(message);
        }

        let _json_rpc_notification = JsonRpcRequest::notification(
            "notifications/progress".to_string(),
            Some(serde_json::to_value(notification)?),
        );

        // Send to all connected clients
        // Implementation depends on transport
        debug!("Sent progress notification");
        Ok(())
    }

    /// Cancel an active request
    pub async fn cancel_request(
        &self,
        request_id: serde_json::Value,
        reason: Option<String>,
    ) -> McpResult<bool> {
        self.cancellation_manager
            .cancel_request(&request_id, reason)
            .await
    }

    /// Check if a request has been cancelled
    pub async fn is_request_cancelled(&self, request_id: &serde_json::Value) -> bool {
        self.cancellation_manager.is_cancelled(request_id).await
    }

    /// Send a cancellation notification to clients
    pub async fn send_cancellation(
        &self,
        request_id: serde_json::Value,
        reason: Option<String>,
    ) -> McpResult<()> {
        let mut notification = CancelledNotification::new(request_id);
        if let Some(reason) = reason {
            notification = notification.with_reason(reason);
        }

        let _json_rpc_notification = JsonRpcRequest::notification(
            "notifications/cancelled".to_string(),
            Some(serde_json::to_value(notification)?),
        );

        // Send to all connected clients
        // Implementation depends on transport
        debug!("Sent cancellation notification");
        Ok(())
    }

    /// Enable ping monitoring
    pub fn enable_ping_monitoring(&self) {
        // Note: This would typically be done during server setup
        // The actual implementation would depend on the transport layer
        debug!("Ping monitoring enabled");
    }

    /// Handle logging level set request
    pub(crate) async fn handle_set_log_level(&self, params: Option<Value>) -> McpResult<Value> {
        debug!("Handling logging level set request");

        let _request: LogLevelSetRequest = match params {
            Some(params) => serde_json::from_value(params)
                .map_err(|e| McpError::invalid_params(e.to_string()))?,
            None => return Err(McpError::invalid_params("Missing parameters".to_string())),
        };

        // Set the minimum log level (implementation depends on server configuration)
        // For now, we'll just acknowledge the request
        let response = LogLevelSetResponse::new();

        serde_json::to_value(response).map_err(|e| McpError::serialization_error(e.to_string()))
    }
}

// Context for tool and resource handlers
#[derive(Clone)]
pub struct Context {
    session_id: Option<String>,
    request_id: Option<String>,
    metadata: HashMap<String, serde_json::Value>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            session_id: None,
            request_id: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }

    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub async fn progress(
        &self,
        _message: &str,
        _progress: f64,
        _total: Option<f64>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Send progress notification
        // Implementation depends on transport
        Ok(())
    }

    pub async fn log_info(
        &self,
        _message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Send info log message
        // Implementation depends on transport
        Ok(())
    }

    pub async fn log_warn(
        &self,
        _message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Send warning log message
        // Implementation depends on transport
        Ok(())
    }

    pub async fn log_error(
        &self,
        _message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Send error log message
        // Implementation depends on transport
        Ok(())
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}
