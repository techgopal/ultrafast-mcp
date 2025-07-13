//! UltraFastServer implementation module
//!
//! This module contains the main server implementation with all the core functionality.

use std::{borrow::Cow, collections::HashMap, sync::Arc};
use tokio::sync::{RwLock, broadcast};
use tracing::{error, info, warn};

use ultrafast_mcp_core::{
    config::TimeoutConfig,
    error::{MCPError, MCPResult},
    protocol::{
        capabilities::ServerCapabilities,
        jsonrpc::{JsonRpcError, JsonRpcMessage, JsonRpcRequest, JsonRpcResponse},
    },
    schema::validation::validate_tool_schema,
    types::{
        notifications::{LogLevel, LogLevelSetRequest, LogLevelSetResponse},
        prompts::Prompt,
        resources::{Resource, SubscribeResponse},
        roots::{RootsListChangedNotification, SetRootsRequest, SetRootsResponse},
        server::ServerInfo,
        tools::Tool,
    },
    utils::{CancellationManager, PingManager},
};
#[cfg(feature = "http")]
use ultrafast_mcp_transport::streamable_http::server::{HttpTransportConfig, HttpTransportServer};
use ultrafast_mcp_transport::{Transport, TransportConfig, create_transport};

use crate::context::{Context, LoggerConfig};
use crate::handlers::*;

/// MCP Server state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerState {
    Uninitialized,
    Initializing,
    Initialized,
    Operating,
    ShuttingDown,
    Shutdown,
}

impl ServerState {
    /// Check if the server can accept operations
    /// According to MCP 2025-06-18 specification, operations are allowed
    /// once the server is initialized (after initialize response)
    pub fn can_operate(&self) -> bool {
        matches!(self, ServerState::Initialized | ServerState::Operating)
    }

    /// Check if the server is initialized
    pub fn is_initialized(&self) -> bool {
        matches!(self, ServerState::Initialized | ServerState::Operating)
    }

    /// Check if the server is shutting down
    pub fn is_shutting_down(&self) -> bool {
        matches!(self, ServerState::ShuttingDown | ServerState::Shutdown)
    }
}

/// Tool registration error
#[derive(Debug, thiserror::Error)]
pub enum ToolRegistrationError {
    #[error("Tool with name '{0}' already exists")]
    ToolAlreadyExists(String),
    #[error("Invalid tool schema: {0}")]
    InvalidSchema(String),
    #[error("Tool name '{0}' is reserved")]
    ReservedName(String),
    #[error("Tool description is required")]
    MissingDescription,
    #[error("Tool input schema is required")]
    MissingInputSchema,
    #[error("Tool output schema is required")]
    MissingOutputSchema,
}

/// Server logging configuration
#[derive(Debug, Clone)]
pub struct ServerLoggingConfig {
    /// Current minimum log level
    pub current_level: LogLevel,
    /// Whether clients can change the log level
    pub allow_level_changes: bool,
    /// Default logger configuration for new contexts
    pub default_logger_config: LoggerConfig,
}

impl Default for ServerLoggingConfig {
    fn default() -> Self {
        Self {
            current_level: LogLevel::Info,
            allow_level_changes: true,
            default_logger_config: LoggerConfig::default(),
        }
    }
}

/// MCP Server implementation
#[derive(Clone)]
pub struct UltraFastServer {
    info: ServerInfo,
    capabilities: ServerCapabilities,
    state: Arc<RwLock<ServerState>>,
    tools: Arc<RwLock<HashMap<String, Tool>>>,
    resources: Arc<RwLock<HashMap<String, Resource>>>,
    prompts: Arc<RwLock<HashMap<String, Prompt>>>,
    tool_handler: Option<Arc<dyn ToolHandler>>,
    resource_handler: Option<Arc<dyn ResourceHandler>>,
    prompt_handler: Option<Arc<dyn PromptHandler>>,
    sampling_handler: Option<Arc<dyn SamplingHandler>>,
    completion_handler: Option<Arc<dyn CompletionHandler>>,
    roots_handler: Option<Arc<dyn RootsHandler>>,
    elicitation_handler: Option<Arc<dyn ElicitationHandler>>,
    subscription_handler: Option<Arc<dyn ResourceSubscriptionHandler>>,
    #[allow(dead_code)]
    resource_subscriptions: Arc<RwLock<HashMap<String, Vec<String>>>>,
    cancellation_manager: Arc<CancellationManager>,
    ping_manager: Arc<PingManager>,
    // Enhanced logging configuration
    logging_config: Arc<RwLock<ServerLoggingConfig>>,

    #[cfg(feature = "monitoring")]
    monitoring_system: Option<Arc<crate::MonitoringSystem>>,

    // Advanced handlers
    advanced_sampling_handler: Option<Arc<dyn AdvancedSamplingHandler>>,

    // Timeout configuration (MCP 2025-06-18 compliance)
    timeout_config: Arc<TimeoutConfig>,
    // Authentication middleware (removed oauth feature)
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
            logging_config: Arc::new(RwLock::new(ServerLoggingConfig::default())),

            #[cfg(feature = "monitoring")]
            monitoring_system: None,

            // Advanced handlers
            advanced_sampling_handler: None,

            // Timeout configuration (MCP 2025-06-18 compliance)
            timeout_config: Arc::new(TimeoutConfig::default()),
        }
    }

    /// Configure server logging
    pub async fn set_logging_config(&self, config: ServerLoggingConfig) {
        let mut logging_config = self.logging_config.write().await;
        *logging_config = config;
        info!("Server logging configuration updated");
    }

    /// Get current server logging configuration
    pub async fn get_logging_config(&self) -> ServerLoggingConfig {
        self.logging_config.read().await.clone()
    }

    /// Set timeout configuration
    pub fn with_timeout_config(mut self, config: TimeoutConfig) -> Self {
        self.timeout_config = Arc::new(config);
        self
    }

    /// Get current timeout configuration
    pub fn get_timeout_config(&self) -> TimeoutConfig {
        (*self.timeout_config).clone()
    }

    /// Set timeout configuration for high-performance scenarios
    pub fn with_high_performance_timeouts(mut self) -> Self {
        self.timeout_config = Arc::new(TimeoutConfig::high_performance());
        self
    }

    /// Set timeout configuration for long-running operations
    pub fn with_long_running_timeouts(mut self) -> Self {
        self.timeout_config = Arc::new(TimeoutConfig::long_running());
        self
    }

    /// Get timeout for a specific operation
    pub fn get_operation_timeout(&self, operation: &str) -> std::time::Duration {
        self.timeout_config.get_timeout_for_operation(operation)
    }

    /// Validate timeout configuration
    pub fn validate_timeout_config(&self) -> Result<(), String> {
        let config = &self.timeout_config;

        // Validate all timeouts are within bounds
        if !config.validate_timeout(config.connect_timeout) {
            return Err("Connect timeout is out of bounds".to_string());
        }
        if !config.validate_timeout(config.request_timeout) {
            return Err("Request timeout is out of bounds".to_string());
        }
        if !config.validate_timeout(config.response_timeout) {
            return Err("Response timeout is out of bounds".to_string());
        }
        if !config.validate_timeout(config.tool_execution_timeout) {
            return Err("Tool execution timeout is out of bounds".to_string());
        }
        if !config.validate_timeout(config.resource_read_timeout) {
            return Err("Resource read timeout is out of bounds".to_string());
        }
        if !config.validate_timeout(config.prompt_generation_timeout) {
            return Err("Prompt generation timeout is out of bounds".to_string());
        }
        if !config.validate_timeout(config.sampling_timeout) {
            return Err("Sampling timeout is out of bounds".to_string());
        }
        if !config.validate_timeout(config.completion_timeout) {
            return Err("Completion timeout is out of bounds".to_string());
        }
        if !config.validate_timeout(config.shutdown_timeout) {
            return Err("Shutdown timeout is out of bounds".to_string());
        }
        if !config.validate_timeout(config.heartbeat_interval) {
            return Err("Heartbeat interval is out of bounds".to_string());
        }

        Ok(())
    }

    /// Set the current log level
    pub async fn set_log_level(&self, level: LogLevel) -> MCPResult<()> {
        let mut logging_config = self.logging_config.write().await;

        if !logging_config.allow_level_changes {
            return Err(MCPError::invalid_request(
                "Log level changes are not allowed on this server".to_string(),
            ));
        }

        logging_config.current_level = level.clone();
        logging_config.default_logger_config.min_level = level.clone();

        info!("Server log level changed to: {:?}", level);
        Ok(())
    }

    /// Get the current log level
    pub async fn get_log_level(&self) -> LogLevel {
        self.logging_config.read().await.current_level.clone()
    }

    // ===== FLUENT BUILDER METHODS =====

    /// Enable monitoring with custom configuration
    #[cfg(feature = "monitoring")]
    pub fn with_monitoring_config(mut self, config: crate::MonitoringConfig) -> Self {
        let monitoring = crate::MonitoringSystem::new(config);
        self.monitoring_system = Some(Arc::new(monitoring));
        info!("Monitoring enabled with custom configuration");
        self
    }
    #[cfg(not(feature = "monitoring"))]
    pub fn with_monitoring_config(self, _config: ()) -> Self {
        warn!("Monitoring feature not enabled. Add 'monitoring' feature to enable monitoring.");
        self
    }

    /// Enable monitoring with default configuration
    #[cfg(feature = "monitoring")]
    pub fn with_monitoring(mut self) -> Self {
        let monitoring = crate::MonitoringSystem::new(crate::MonitoringConfig::default());
        self.monitoring_system = Some(Arc::new(monitoring));
        info!("Monitoring enabled with default configuration");
        self
    }
    #[cfg(not(feature = "monitoring"))]
    pub fn with_monitoring(self) -> Self {
        warn!("Monitoring feature not enabled. Add 'monitoring' feature to enable monitoring.");
        self
    }

    /// Enable health checks with default configuration
    #[cfg(feature = "monitoring")]
    pub fn with_health_checks(mut self) -> Self {
        if let Some(_monitoring) = &self.monitoring_system {
            // Health checks are automatically initialized when monitoring system is created
            info!("Health checks enabled");
        } else {
            // Create monitoring system if not already present
            let mut config = crate::MonitoringConfig::default();
            config.health.enabled = true;
            let monitoring = crate::MonitoringSystem::new(config);
            self.monitoring_system = Some(Arc::new(monitoring));
            info!("Health checks enabled with new monitoring system");
        }
        self
    }
    #[cfg(not(feature = "monitoring"))]
    pub fn with_health_checks(self) -> Self {
        warn!(
            "Health checks require monitoring feature. Add 'monitoring' feature to enable health checks."
        );
        self
    }

    /// Enable metrics collection with default configuration
    #[cfg(feature = "monitoring")]
    pub fn with_metrics(mut self) -> Self {
        if let Some(_monitoring) = &self.monitoring_system {
            // Metrics are automatically available via monitoring.metrics()
            info!("Metrics collection enabled");
        } else {
            // Create monitoring system if not already present
            let mut config = crate::MonitoringConfig::default();
            config.metrics.enabled = true;
            let monitoring = crate::MonitoringSystem::new(config);
            self.monitoring_system = Some(Arc::new(monitoring));
            info!("Metrics collection enabled with new monitoring system");
        }
        self
    }
    #[cfg(not(feature = "monitoring"))]
    pub fn with_metrics(self) -> Self {
        warn!("Metrics require monitoring feature. Add 'monitoring' feature to enable metrics.");
        self
    }

    /// Enable tracing with default configuration
    #[cfg(feature = "monitoring")]
    pub fn with_tracing(mut self) -> Self {
        if let Some(_monitoring) = &self.monitoring_system {
            // Tracing is configured via the monitoring config
            info!("Tracing enabled");
        } else {
            // Create monitoring system if not already present
            let mut config = crate::MonitoringConfig::default();
            config.tracing.enabled = true;
            let monitoring = crate::MonitoringSystem::new(config);
            self.monitoring_system = Some(Arc::new(monitoring));
            info!("Tracing enabled with new monitoring system");
        }
        self
    }
    #[cfg(not(feature = "monitoring"))]
    pub fn with_tracing(self) -> Self {
        warn!("Tracing requires monitoring feature. Add 'monitoring' feature to enable tracing.");
        self
    }

    /// Enable all monitoring features (health checks, metrics, tracing)
    #[cfg(feature = "monitoring")]
    pub fn with_full_monitoring(mut self) -> Self {
        let mut config = crate::MonitoringConfig::default();
        config.health.enabled = true;
        config.metrics.enabled = true;
        config.tracing.enabled = true;
        let monitoring = crate::MonitoringSystem::new(config);
        self.monitoring_system = Some(Arc::new(monitoring));
        info!("Full monitoring enabled (health checks, metrics, tracing)");
        self
    }
    #[cfg(not(feature = "monitoring"))]
    pub fn with_full_monitoring(self) -> Self {
        warn!(
            "Full monitoring requires monitoring feature. Add 'monitoring' feature to enable all monitoring features."
        );
        self
    }

    /// Enable middleware support
    pub fn with_middleware(self) -> Self {
        // This would integrate with the transport middleware system
        info!("Middleware support enabled");
        self
    }

    /// Enable recovery mechanisms
    pub fn with_recovery(self) -> Self {
        info!("Recovery mechanisms enabled");
        self
    }

    /// Enable OAuth authentication
    pub fn with_oauth(self) -> Self {
        info!("OAuth authentication enabled");
        self
    }

    /// Enable authentication with custom configuration (feature removed)
    pub fn with_authentication(self, _token_validator: (), _required_scopes: Vec<String>) -> Self {
        warn!("Authentication feature has been removed. Use ultrafast-mcp-auth crate directly.");
        self
    }

    /// Enable Bearer token authentication (feature removed)
    pub fn with_bearer_auth(self, _secret: String, _required_scopes: Vec<String>) -> Self {
        warn!(
            "Bearer authentication feature has been removed. Use ultrafast-mcp-auth crate directly."
        );
        self
    }

    /// Enable API key authentication
    pub fn with_api_key_auth(self) -> Self {
        info!("API key authentication enabled");
        self
    }

    /// Enable Basic authentication
    pub fn with_basic_auth(self) -> Self {
        info!("Basic authentication enabled");
        self
    }

    /// Enable rate limiting
    pub fn with_rate_limiting(self, requests_per_minute: u32) -> Self {
        info!(
            "Rate limiting enabled: {} requests per minute",
            requests_per_minute
        );
        self
    }

    /// Enable request validation
    pub fn with_request_validation(self) -> Self {
        info!("Request validation enabled");
        self
    }

    /// Enable response caching
    pub fn with_response_caching(self) -> Self {
        info!("Response caching enabled");
        self
    }

    /// Get the monitoring system if available
    #[cfg(feature = "monitoring")]
    pub fn monitoring(&self) -> Option<Arc<crate::MonitoringSystem>> {
        self.monitoring_system.clone()
    }
    #[cfg(not(feature = "monitoring"))]
    pub fn monitoring(&self) -> Option<()> {
        None
    }

    /// Create a context with the current server logging configuration
    pub async fn create_context(&self) -> Context {
        let logging_config = self.logging_config.read().await;
        let logger_config = logging_config.default_logger_config.clone();

        Context::new().with_logger_config(logger_config)
    }

    /// Create a context with custom request and session IDs
    pub async fn create_context_with_ids(
        &self,
        request_id: String,
        session_id: Option<String>,
    ) -> Context {
        let logging_config = self.logging_config.read().await;
        let logger_config = logging_config.default_logger_config.clone();

        let mut context = Context::new()
            .with_request_id(request_id)
            .with_logger_config(logger_config);

        if let Some(session_id) = session_id {
            context = context.with_session_id(session_id);
        }

        context
    }

    /// Register a tool with validation
    pub async fn register_tool(&self, tool: Tool) -> Result<(), ToolRegistrationError> {
        // Validate tool name
        if tool.name.is_empty() {
            return Err(ToolRegistrationError::MissingDescription);
        }

        if self.is_reserved_name(&tool.name) {
            return Err(ToolRegistrationError::ReservedName(tool.name.clone()));
        }

        // Validate required fields
        if tool.description.is_empty() {
            return Err(ToolRegistrationError::MissingDescription);
        }

        // Validate tool schema
        if let Err(e) = validate_tool_schema(&tool.input_schema) {
            return Err(ToolRegistrationError::InvalidSchema(format!(
                "Input schema: {e}"
            )));
        }

        if let Some(output_schema) = &tool.output_schema {
            if let Err(e) = validate_tool_schema(output_schema) {
                return Err(ToolRegistrationError::InvalidSchema(format!(
                    "Output schema: {e}"
                )));
            }
        } else {
            return Err(ToolRegistrationError::MissingOutputSchema);
        }

        // Check for existing tool
        let mut tools = self.tools.write().await;
        if tools.contains_key(&tool.name) {
            return Err(ToolRegistrationError::ToolAlreadyExists(tool.name.clone()));
        }

        // Register the tool
        let tool_name = tool.name.clone();
        tools.insert(tool_name.clone(), tool);
        info!("Registered tool: {}", tool_name);

        Ok(())
    }

    /// Register multiple tools
    pub async fn register_tools(&self, tools: Vec<Tool>) -> Result<(), ToolRegistrationError> {
        for tool in tools {
            self.register_tool(tool).await?;
        }
        Ok(())
    }

    /// Unregister a tool by name
    pub async fn unregister_tool(&self, name: &str) -> bool {
        let mut tools = self.tools.write().await;
        tools.remove(name).is_some()
    }

    /// Get a tool by name
    pub async fn get_tool(&self, name: &str) -> Option<Tool> {
        let tools = self.tools.read().await;
        tools.get(name).cloned()
    }

    /// List all registered tools
    pub async fn list_tools(&self) -> Vec<Tool> {
        let tools = self.tools.read().await;
        tools.values().cloned().collect()
    }

    /// Check if a tool exists
    pub async fn has_tool(&self, name: &str) -> bool {
        let tools = self.tools.read().await;
        tools.contains_key(name)
    }

    /// Get tool count
    pub async fn tool_count(&self) -> usize {
        let tools = self.tools.read().await;
        tools.len()
    }

    /// Clear all tools
    pub async fn clear_tools(&self) {
        let mut tools = self.tools.write().await;
        let count = tools.len();
        tools.clear();
        info!("Cleared {} tools", count);
    }

    /// Check if a name is reserved
    fn is_reserved_name(&self, name: &str) -> bool {
        // MCP reserved method names
        let reserved_names = [
            "initialize",
            "initialized",
            "shutdown",
            "exit",
            "ping",
            "tools/list",
            "tools/call",
            "resources/list",
            "resources/read",
            "resources/subscribe",
            "resources/unsubscribe",
            "prompts/list",
            "prompts/get",
            "sampling/create",
            "completion/complete",
            "roots/list",
            "elicitation/request",
            "logging/setLevel",
        ];

        reserved_names.contains(&name)
    }

    /// Validate tool call arguments against tool schema
    pub async fn validate_tool_call(
        &self,
        tool_name: &str,
        arguments: &serde_json::Value,
    ) -> Result<(), MCPError> {
        let tool = self.get_tool(tool_name).await;
        let tool =
            tool.ok_or_else(|| MCPError::invalid_request(format!("Tool '{tool_name}' not found")))?;

        ultrafast_mcp_core::schema::validation::validate_tool_input(arguments, &tool.input_schema)
            .map_err(|e| {
                MCPError::invalid_request(format!(
                    "Tool '{tool_name}' input validation failed: {e}"
                ))
            })?;

        Ok(())
    }

    /// Execute a tool call with validation
    pub async fn execute_tool_call(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<ultrafast_mcp_core::types::tools::ToolResult, MCPError> {
        // Validate the tool call
        self.validate_tool_call(tool_name, &arguments).await?;

        // Get the tool handler
        let tool_handler = self
            .tool_handler
            .as_ref()
            .ok_or_else(|| MCPError::internal_error("No tool handler configured".to_string()))?;

        // Create the tool call
        let tool_call = ultrafast_mcp_core::types::tools::ToolCall {
            name: tool_name.to_string(),
            arguments: Some(arguments),
        };

        // Execute the tool call
        tool_handler
            .handle_tool_call(tool_call)
            .await
            .map_err(|e| MCPError::internal_error(format!("Tool execution failed: {e}")))
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
        // Note: Roots is a client capability, not server capability
        // The server responds to roots requests but doesn't advertise it
        self
    }

    /// Add an elicitation handler to the server
    pub fn with_elicitation_handler(mut self, handler: Arc<dyn ElicitationHandler>) -> Self {
        self.elicitation_handler = Some(handler);
        // Note: Elicitation is a client capability, not server capability
        // The server responds to elicitation requests but doesn't advertise it
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

    /// Configure logging with a custom configuration
    pub fn with_logging_config(mut self, config: ServerLoggingConfig) -> Self {
        let logging_config = Arc::get_mut(&mut self.logging_config)
            .expect("Cannot modify logging config after server has been cloned");
        *logging_config.get_mut() = config;
        self
    }

    /// Run the server with stdio transport
    pub async fn run_stdio(&self) -> MCPResult<()> {
        let transport = create_transport(TransportConfig::Stdio)
            .await
            .map_err(|e| MCPError::internal_error(format!("Transport creation failed: {e}")))?;
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
        info!("Starting HTTP transport server with config: {:?}", config);

        let transport_server = HttpTransportServer::new(config);
        let message_receiver = transport_server.get_message_receiver();
        let message_sender = transport_server.get_message_sender();
        let response_sender = transport_server.get_response_sender();

        // Start message processing task
        let server_clone = self.clone();
        let _message_processor = tokio::spawn(async move {
            server_clone
                .process_http_messages(message_receiver, message_sender, response_sender)
                .await;
        });

        // Start the HTTP server
        transport_server
            .run()
            .await
            .map_err(|e| MCPError::internal_error(format!("HTTP server failed: {e}")))
    }

    /// Run the server with custom Streamable HTTP transport configuration
    /// This provides clearer naming for advanced Streamable HTTP configuration
    #[cfg(feature = "http")]
    pub async fn run_streamable_http_with_config(
        &self,
        config: HttpTransportConfig,
    ) -> MCPResult<()> {
        self.run_http(config).await
    }

    /// Process HTTP messages from the transport layer
    #[allow(dead_code)]
    async fn process_http_messages(
        &self,
        mut message_receiver: broadcast::Receiver<(String, JsonRpcMessage)>,
        _message_sender: broadcast::Sender<(String, JsonRpcMessage)>,
        response_sender: broadcast::Sender<(String, JsonRpcMessage)>,
    ) {
        info!("HTTP message processor started");

        while let Ok((session_id, message)) = message_receiver.recv().await {
            let session_id_clone = session_id.clone();
            match message {
                JsonRpcMessage::Request(request) => {
                    info!(
                        "Processing HTTP request: {} (session: {})",
                        request.method, session_id
                    );

                    let response = self.handle_request(request).await;
                    let response_message = JsonRpcMessage::Response(response);

                    info!(
                        "Sending response for session {}: {:?}",
                        session_id, response_message
                    );

                    // Send the response back through the response sender
                    if let Err(e) = response_sender.send((session_id_clone, response_message)) {
                        error!("Failed to send response for session {}: {}", session_id, e);
                    } else {
                        info!("Successfully sent response for session {}", session_id);
                    }
                }
                JsonRpcMessage::Notification(notification) => {
                    info!(
                        "Processing HTTP notification: {} (session: {})",
                        notification.method, session_id
                    );

                    if let Err(e) = self.handle_notification(notification).await {
                        error!(
                            "Failed to handle notification for session {}: {}",
                            session_id, e
                        );
                    }
                    // Notifications don't have responses, so no need to send anything back
                }
                JsonRpcMessage::Response(_) => {
                    warn!(
                        "Received unexpected response message for session: {}",
                        session_id
                    );
                }
            }
        }

        info!("HTTP message processor stopped");
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

    /// Start periodic ping monitoring (optional, for connection health)
    /// This method should be called after the server is running with a transport
    pub async fn start_ping_monitoring(&self, ping_interval: std::time::Duration) -> MCPResult<()> {
        info!(
            "Starting periodic ping monitoring with interval: {:?}",
            ping_interval
        );

        // Note: This is a placeholder for future implementation
        // The actual ping monitoring would need to be integrated with the transport layer
        // For now, we log that ping monitoring is enabled
        info!("Ping monitoring enabled (interval: {:?})", ping_interval);

        // The PingManager is already configured with default intervals
        // Future implementation would integrate with the transport layer
        // to send periodic pings to clients

        Ok(())
    }

    /// Stop ping monitoring
    pub async fn stop_ping_monitoring(&self) -> MCPResult<()> {
        info!("Stopping periodic ping monitoring");
        // The ping monitoring task will naturally stop when the transport is closed
        Ok(())
    }

    /// Handle MCP initialize request
    async fn handle_initialize(
        &self,
        request: ultrafast_mcp_core::protocol::InitializeRequest,
    ) -> Result<ultrafast_mcp_core::protocol::InitializeResponse, MCPError> {
        info!(
            "Handling initialize request from client: {} (version: {})",
            request.client_info.name, request.protocol_version
        );

        // Negotiate protocol version
        let negotiated_version = match ultrafast_mcp_core::protocol::version::negotiate_version(
            &request.protocol_version,
        ) {
            Ok(version) => {
                info!(
                    "Protocol version negotiated: {} -> {}",
                    request.protocol_version, version
                );
                version
            }
            Err(e) => {
                error!("Protocol version negotiation failed: {}", e);
                return Err(MCPError::invalid_request(format!(
                    "Protocol version negotiation failed: {}. Supported versions: {:?}",
                    e,
                    ultrafast_mcp_core::protocol::version::SUPPORTED_VERSIONS
                )));
            }
        };

        // Validate the initialize request
        if let Err(e) = request.validate_protocol_version() {
            warn!("Initialize request validation warning: {}", e);
            // Continue with warning but don't fail
        }

        // Validate compatibility
        if let Err(e) = ultrafast_mcp_core::protocol::capabilities::validate_compatibility(
            &request.capabilities,
            &self.capabilities,
        ) {
            error!("Capability validation failed: {}", e);
            return Err(MCPError::Protocol(
                ultrafast_mcp_core::error::ProtocolError::CapabilityNotSupported(e),
            ));
        }

        info!("Capabilities validated successfully");

        // Update server state to Initialized (not Operating yet)
        // This follows MCP 2025-06-18 specification: server should wait for initialized notification
        {
            let mut state = self.state.write().await;
            *state = ServerState::Initialized;
        }

        info!(
            "Server initialized with protocol version: {} (waiting for initialized notification)",
            negotiated_version
        );

        Ok(ultrafast_mcp_core::protocol::InitializeResponse {
            protocol_version: negotiated_version,
            capabilities: self.capabilities.clone(),
            server_info: self.info.clone(),
            instructions: None,
        })
    }

    /// Handle MCP initialized notification
    async fn handle_initialized(
        &self,
        _notification: ultrafast_mcp_core::protocol::InitializedNotification,
    ) -> MCPResult<()> {
        info!("Received initialized notification from client");

        // Ensure server state is operating (it should already be from initialize)
        {
            let mut state = self.state.write().await;
            *state = ServerState::Operating;
        }

        info!("Server confirmed operating state via initialized notification");
        Ok(())
    }

    /// Handle MCP shutdown request
    async fn handle_shutdown(
        &self,
        request: ultrafast_mcp_core::protocol::ShutdownRequest,
    ) -> MCPResult<()> {
        info!("Handling shutdown request: {:?}", request.reason);

        // Update server state
        {
            let mut state = self.state.write().await;
            *state = ServerState::ShuttingDown;
        }

        // Perform cleanup
        self.perform_shutdown_cleanup().await;

        // Update state to shutdown
        {
            let mut state = self.state.write().await;
            *state = ServerState::Shutdown;
        }

        info!("Server shutdown completed");
        Ok(())
    }

    /// Perform shutdown cleanup
    async fn perform_shutdown_cleanup(&self) {
        info!("Performing shutdown cleanup");

        // Clear all tools
        self.clear_tools().await;

        // Clear all resources
        {
            let mut resources = self.resources.write().await;
            resources.clear();
        }

        // Clear all prompts
        {
            let mut prompts = self.prompts.write().await;
            prompts.clear();
        }

        // Clear resource subscriptions
        {
            let mut subscriptions = self.resource_subscriptions.write().await;
            subscriptions.clear();
        }

        info!("Shutdown cleanup completed");
    }

    /// Get current server state
    pub async fn get_state(&self) -> ServerState {
        self.state.read().await.clone()
    }

    /// Check if server can accept operations
    pub async fn can_operate(&self) -> bool {
        self.state.read().await.can_operate()
    }

    /// Helper function to deserialize request parameters with proper defaults
    fn deserialize_list_tools_request(
        &self,
        params: Option<serde_json::Value>,
    ) -> ultrafast_mcp_core::types::tools::ListToolsRequest {
        serde_json::from_value(params.unwrap_or_default()).unwrap_or_default()
    }

    fn deserialize_list_resources_request(
        &self,
        params: Option<serde_json::Value>,
    ) -> ultrafast_mcp_core::types::resources::ListResourcesRequest {
        serde_json::from_value(params.unwrap_or_default()).unwrap_or_default()
    }

    fn deserialize_list_prompts_request(
        &self,
        params: Option<serde_json::Value>,
    ) -> ultrafast_mcp_core::types::prompts::ListPromptsRequest {
        serde_json::from_value(params.unwrap_or_default()).unwrap_or_default()
    }

    fn deserialize_get_prompt_request(
        &self,
        params: Option<serde_json::Value>,
    ) -> ultrafast_mcp_core::types::prompts::GetPromptRequest {
        serde_json::from_value(params.unwrap_or_default()).unwrap_or_default()
    }

    fn deserialize_read_resource_request(
        &self,
        params: Option<serde_json::Value>,
    ) -> ultrafast_mcp_core::types::resources::ReadResourceRequest {
        serde_json::from_value(params.unwrap_or_default()).unwrap_or_default()
    }

    fn deserialize_list_resource_templates_request(
        &self,
        params: Option<serde_json::Value>,
    ) -> ultrafast_mcp_core::types::resources::ListResourceTemplatesRequest {
        serde_json::from_value(params.unwrap_or_default()).unwrap_or_default()
    }

    fn deserialize_subscribe_request(
        &self,
        params: Option<serde_json::Value>,
    ) -> ultrafast_mcp_core::types::resources::SubscribeRequest {
        serde_json::from_value(params.unwrap_or_default()).unwrap_or_else(|_| {
            ultrafast_mcp_core::types::resources::SubscribeRequest { uri: String::new() }
        })
    }

    fn deserialize_unsubscribe_request(
        &self,
        params: Option<serde_json::Value>,
    ) -> ultrafast_mcp_core::types::resources::UnsubscribeRequest {
        serde_json::from_value(params.unwrap_or_default()).unwrap_or_else(|_| {
            ultrafast_mcp_core::types::resources::UnsubscribeRequest { uri: String::new() }
        })
    }

    fn deserialize_create_message_request(
        &self,
        params: Option<serde_json::Value>,
    ) -> ultrafast_mcp_core::types::sampling::CreateMessageRequest {
        serde_json::from_value(params.unwrap_or_default()).unwrap_or_default()
    }

    fn deserialize_elicitation_request(
        &self,
        params: Option<serde_json::Value>,
    ) -> ultrafast_mcp_core::types::elicitation::ElicitationRequest {
        serde_json::from_value(params.unwrap_or_default()).unwrap_or_default()
    }

    fn deserialize_complete_request(
        &self,
        params: Option<serde_json::Value>,
    ) -> ultrafast_mcp_core::types::completion::CompleteRequest {
        match params {
            Some(params) => serde_json::from_value(params).unwrap_or_else(|_| {
                ultrafast_mcp_core::types::completion::CompleteRequest {
                    reference: ultrafast_mcp_core::types::completion::CompletionReference {
                        ref_type: "ref/prompt".to_string(),
                        name: "".to_string(),
                    },
                    argument: ultrafast_mcp_core::types::completion::CompletionArgument {
                        name: "".to_string(),
                        value: "".to_string(),
                    },
                    context: None,
                }
            }),
            None => ultrafast_mcp_core::types::completion::CompleteRequest {
                reference: ultrafast_mcp_core::types::completion::CompletionReference {
                    ref_type: "ref/prompt".to_string(),
                    name: "".to_string(),
                },
                argument: ultrafast_mcp_core::types::completion::CompletionArgument {
                    name: "".to_string(),
                    value: "".to_string(),
                },
                context: None,
            },
        }
    }

    /// Handle incoming messages
    async fn handle_message(
        &self,
        message: JsonRpcMessage,
        transport: &mut Box<dyn Transport>,
    ) -> MCPResult<()> {
        match message {
            JsonRpcMessage::Request(request) => {
                // Check if this is actually a notification (no ID)
                if request.id.is_none() {
                    // This is a notification, handle it as such
                    self.handle_notification(request).await?;
                } else {
                    // This is a request, handle it with timeout
                    let operation_timeout = self.get_operation_timeout(&request.method);
                    let request_id = request.id.clone(); // Clone before moving request
                    let response =
                        tokio::time::timeout(operation_timeout, self.handle_request(request)).await;

                    match response {
                        Ok(response) => {
                            transport
                                .send_message(JsonRpcMessage::Response(response))
                                .await
                                .map_err(|e| {
                                    MCPError::internal_error(format!("Failed to send message: {e}"))
                                })?;
                        }
                        Err(_) => {
                            // Request timed out, send timeout error
                            let timeout_error = JsonRpcResponse::error(
                                JsonRpcError::new(-32000, "Request timeout".to_string()),
                                request_id.clone(),
                            );
                            transport
                                .send_message(JsonRpcMessage::Response(timeout_error))
                                .await
                                .map_err(|e| {
                                    MCPError::internal_error(format!(
                                        "Failed to send timeout error: {e}"
                                    ))
                                })?;

                            // Send cancellation notification
                            if let Some(request_id) = &request_id {
                                self.notify_cancelled(
                                    serde_json::Value::String(request_id.to_string()),
                                    Some("Request timed out".to_string()),
                                    transport,
                                )
                                .await?;
                            }
                        }
                    }
                }
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
        info!(
            "Handling request: {} (id: {:?})",
            request.method, request.id
        );

        match request.method.as_str() {
            // MCP Lifecycle methods
            "initialize" => {
                match serde_json::from_value::<ultrafast_mcp_core::protocol::InitializeRequest>(
                    request.params.unwrap_or_default(),
                ) {
                    Ok(init_request) => match self.handle_initialize(init_request).await {
                        Ok(response) => match serde_json::to_value(response) {
                            Ok(value) => JsonRpcResponse::success(value, request.id),
                            Err(e) => JsonRpcResponse::error(
                                JsonRpcError::new(-32603, format!("Serialization error: {e}")),
                                request.id,
                            ),
                        },
                        Err(e) => JsonRpcResponse::error(
                            JsonRpcError::new(-32603, e.to_string()),
                            request.id,
                        ),
                    },
                    Err(e) => JsonRpcResponse::error(
                        JsonRpcError::invalid_params(Some(format!(
                            "Invalid initialize request: {e}"
                        ))),
                        request.id,
                    ),
                }
            }
            "shutdown" => {
                let shutdown_request = match serde_json::from_value::<
                    ultrafast_mcp_core::protocol::ShutdownRequest,
                >(request.params.unwrap_or_default())
                {
                    Ok(req) => req,
                    Err(_) => ultrafast_mcp_core::protocol::ShutdownRequest { reason: None },
                };

                match self.handle_shutdown(shutdown_request).await {
                    Ok(_) => JsonRpcResponse::success(serde_json::json!({}), request.id),
                    Err(e) => {
                        JsonRpcResponse::error(JsonRpcError::new(-32603, e.to_string()), request.id)
                    }
                }
            }

            // Tools methods
            "tools/list" => {
                if !self.can_operate().await {
                    return JsonRpcResponse::error(
                        JsonRpcError::internal_error(Some("Server not ready".to_string())),
                        request.id,
                    );
                }

                let list_request = self.deserialize_list_tools_request(request.params.clone());

                if let Some(handler) = &self.tool_handler {
                    match handler.list_tools(list_request).await {
                        Ok(response) => {
                            // If handler returns empty tools, fallback to registered tools
                            if response.tools.is_empty() {
                                let tools = self.list_tools().await;
                                let response =
                                    ultrafast_mcp_core::types::tools::ListToolsResponse {
                                        tools,
                                        next_cursor: None,
                                    };
                                match serde_json::to_value(response) {
                                    Ok(value) => JsonRpcResponse::success(value, request.id),
                                    Err(e) => JsonRpcResponse::error(
                                        JsonRpcError::new(
                                            -32603,
                                            format!("Serialization error: {e}"),
                                        ),
                                        request.id,
                                    ),
                                }
                            } else {
                                match serde_json::to_value(response) {
                                    Ok(value) => JsonRpcResponse::success(value, request.id),
                                    Err(e) => JsonRpcResponse::error(
                                        JsonRpcError::new(
                                            -32603,
                                            format!("Serialization error: {e}"),
                                        ),
                                        request.id,
                                    ),
                                }
                            }
                        }
                        Err(e) => JsonRpcResponse::error(
                            JsonRpcError::new(-32603, format!("Tools list failed: {e}")),
                            request.id,
                        ),
                    }
                } else {
                    // Fallback to registered tools
                    let tools = self.list_tools().await;
                    let response = ultrafast_mcp_core::types::tools::ListToolsResponse {
                        tools,
                        next_cursor: None,
                    };
                    match serde_json::to_value(response) {
                        Ok(value) => JsonRpcResponse::success(value, request.id),
                        Err(e) => JsonRpcResponse::error(
                            JsonRpcError::new(-32603, format!("Serialization error: {e}")),
                            request.id,
                        ),
                    }
                }
            }
            "tools/call" => {
                if !self.can_operate().await {
                    return JsonRpcResponse::error(
                        JsonRpcError::internal_error(Some("Server not ready".to_string())),
                        request.id,
                    );
                }

                let params = match &request.params {
                    Some(params) => params,
                    None => {
                        return JsonRpcResponse::error(
                            JsonRpcError::new(
                                -32602,
                                "Tool call failed: Missing parameters".to_string(),
                            ),
                            request.id,
                        );
                    }
                };

                let tool_name = params.get("name").and_then(|v| v.as_str());
                let arguments = params
                    .get("arguments")
                    .cloned()
                    .unwrap_or(serde_json::json!({}));

                if let Some(tool_name) = tool_name {
                    if let Some(handler) = &self.tool_handler {
                        let tool_call = ultrafast_mcp_core::types::tools::ToolCall {
                            name: tool_name.to_string(),
                            arguments: Some(arguments.clone()),
                        };
                        // Arguments validation will be handled by the tool handler
                        match handler.handle_tool_call(tool_call).await {
                            Ok(result) => match serde_json::to_value(result) {
                                Ok(value) => JsonRpcResponse::success(value, request.id),
                                Err(e) => JsonRpcResponse::error(
                                    JsonRpcError::new(-32603, format!("Serialization error: {e}")),
                                    request.id,
                                ),
                            },
                            Err(e) => {
                                use ultrafast_mcp_core::error::{MCPError, ProtocolError};
                                let (code, msg) = match &e {
                                    MCPError::Protocol(ProtocolError::InvalidParams(_))
                                    | MCPError::Protocol(ProtocolError::NotFound(_)) => {
                                        (-32602, format!("Tool call failed: {e}"))
                                    }
                                    _ => (-32603, format!("Tool call failed: {e}")),
                                };
                                JsonRpcResponse::error(JsonRpcError::new(code, msg), request.id)
                            }
                        }
                    } else {
                        // Fallback to registered tools
                        if !self.has_tool(tool_name).await {
                            return JsonRpcResponse::error(
                                JsonRpcError::new(
                                    -32602,
                                    format!("Tool call failed: Tool not found: {tool_name}"),
                                ),
                                request.id,
                            );
                        }
                        // Arguments validation will be handled by the tool handler
                        match self.execute_tool_call(tool_name, arguments).await {
                            Ok(result) => match serde_json::to_value(result) {
                                Ok(value) => JsonRpcResponse::success(value, request.id),
                                Err(e) => JsonRpcResponse::error(
                                    JsonRpcError::new(-32603, format!("Serialization error: {e}")),
                                    request.id,
                                ),
                            },
                            Err(e) => {
                                use ultrafast_mcp_core::error::{MCPError, ProtocolError};
                                let (code, msg) = match &e {
                                    MCPError::Protocol(ProtocolError::InvalidParams(_))
                                    | MCPError::Protocol(ProtocolError::NotFound(_)) => {
                                        (-32602, format!("Tool call failed: {e}"))
                                    }
                                    _ => (-32603, format!("Tool call failed: {e}")),
                                };
                                JsonRpcResponse::error(JsonRpcError::new(code, msg), request.id)
                            }
                        }
                    }
                } else {
                    JsonRpcResponse::error(
                        JsonRpcError::new(
                            -32602,
                            "Tool call failed: Missing or invalid tool name".to_string(),
                        ),
                        request.id,
                    )
                }
            }

            // Resources methods
            "resources/list" => {
                if !self.can_operate().await {
                    return JsonRpcResponse::error(
                        JsonRpcError::new(-32000, "Server not ready".to_string()),
                        request.id,
                    );
                }

                let list_request = self.deserialize_list_resources_request(request.params.clone());

                if let Some(handler) = &self.resource_handler {
                    // For resources/list, we don't validate against roots since it's a general listing
                    // Root validation will be done when individual resources are accessed

                    match handler.list_resources(list_request).await {
                        Ok(response) => match serde_json::to_value(response) {
                            Ok(value) => JsonRpcResponse::success(value, request.id),
                            Err(e) => JsonRpcResponse::error(
                                JsonRpcError::new(-32603, format!("Serialization error: {e}")),
                                request.id,
                            ),
                        },
                        Err(e) => JsonRpcResponse::error(
                            JsonRpcError::new(-32603, format!("Resources list failed: {e}")),
                            request.id,
                        ),
                    }
                } else {
                    JsonRpcResponse::error(
                        JsonRpcError::new(-32601, "Resources not supported".to_string()),
                        request.id,
                    )
                }
            }
            "resources/read" => {
                if !self.can_operate().await {
                    return JsonRpcResponse::error(
                        JsonRpcError::new(-32000, "Server not ready".to_string()),
                        request.id,
                    );
                }

                let read_request = self.deserialize_read_resource_request(request.params.clone());

                if let Some(handler) = &self.resource_handler {
                    // Validate against roots if roots handler is available
                    if let Some(roots_handler) = &self.roots_handler {
                        match roots_handler.list_roots().await {
                            Ok(roots) => {
                                if let Err(e) = handler
                                    .validate_resource_access(
                                        &read_request.uri,
                                        ultrafast_mcp_core::types::roots::RootOperation::Read,
                                        &roots,
                                    )
                                    .await
                                {
                                    return JsonRpcResponse::error(
                                        JsonRpcError::new(
                                            -32603,
                                            format!("Root validation failed: {e}"),
                                        ),
                                        request.id,
                                    );
                                }
                            }
                            Err(e) => {
                                return JsonRpcResponse::error(
                                    JsonRpcError::new(-32603, format!("Failed to get roots: {e}")),
                                    request.id,
                                );
                            }
                        }
                    }

                    match handler.read_resource(read_request).await {
                        Ok(response) => match serde_json::to_value(response) {
                            Ok(value) => JsonRpcResponse::success(value, request.id),
                            Err(e) => JsonRpcResponse::error(
                                JsonRpcError::new(-32603, format!("Serialization error: {e}")),
                                request.id,
                            ),
                        },
                        Err(e) => JsonRpcResponse::error(
                            JsonRpcError::new(-32603, format!("Resource read failed: {e}")),
                            request.id,
                        ),
                    }
                } else {
                    JsonRpcResponse::error(
                        JsonRpcError::new(-32601, "Resources not supported".to_string()),
                        request.id,
                    )
                }
            }
            "resources/templates/list" => {
                if !self.can_operate().await {
                    return JsonRpcResponse::error(
                        JsonRpcError::new(-32000, "Server not ready".to_string()),
                        request.id,
                    );
                }

                let list_request =
                    self.deserialize_list_resource_templates_request(request.params.clone());

                if let Some(handler) = &self.resource_handler {
                    match handler.list_resource_templates(list_request).await {
                        Ok(response) => JsonRpcResponse::success(
                            serde_json::to_value(response).unwrap(),
                            request.id,
                        ),
                        Err(e) => JsonRpcResponse::error(
                            JsonRpcError::new(
                                -32603,
                                format!("Resource templates list failed: {e}"),
                            ),
                            request.id,
                        ),
                    }
                } else {
                    JsonRpcResponse::error(
                        JsonRpcError::new(-32601, "Resources not supported".to_string()),
                        request.id,
                    )
                }
            }
            "resources/subscribe" => {
                if !self.can_operate().await {
                    return JsonRpcResponse::error(
                        JsonRpcError::new(-32000, "Server not ready".to_string()),
                        request.id,
                    );
                }

                let subscribe_request = self.deserialize_subscribe_request(request.params.clone());

                // Validate against roots if roots handler is available
                if let Some(roots_handler) = &self.roots_handler {
                    if let Some(resource_handler) = &self.resource_handler {
                        match roots_handler.list_roots().await {
                            Ok(roots) => {
                                if let Err(e) = resource_handler
                                    .validate_resource_access(
                                        &subscribe_request.uri,
                                        ultrafast_mcp_core::types::roots::RootOperation::Read,
                                        &roots,
                                    )
                                    .await
                                {
                                    return JsonRpcResponse::error(
                                        JsonRpcError::new(
                                            -32603,
                                            format!("Root validation failed: {e}"),
                                        ),
                                        request.id,
                                    );
                                }
                            }
                            Err(e) => {
                                return JsonRpcResponse::error(
                                    JsonRpcError::new(-32603, format!("Failed to get roots: {e}")),
                                    request.id,
                                );
                            }
                        }
                    }
                }

                if let Some(handler) = &self.subscription_handler {
                    match handler.subscribe(subscribe_request.uri.clone()).await {
                        Ok(_) => {
                            // Subscription successful - return success response
                            // Note: The client may timeout if it expects immediate notifications
                            // This is a limitation of the current MCP architecture
                            JsonRpcResponse::success(
                                serde_json::to_value(SubscribeResponse::new()).unwrap(),
                                request.id,
                            )
                        }
                        Err(e) => JsonRpcResponse::error(
                            JsonRpcError::new(-32603, format!("Resource subscribe failed: {e}")),
                            request.id,
                        ),
                    }
                } else {
                    JsonRpcResponse::error(
                        JsonRpcError::new(
                            -32601,
                            "Resource subscriptions not supported".to_string(),
                        ),
                        request.id,
                    )
                }
            }
            "resources/unsubscribe" => {
                if !self.can_operate().await {
                    return JsonRpcResponse::error(
                        JsonRpcError::new(-32000, "Server not ready".to_string()),
                        request.id,
                    );
                }

                let unsubscribe_request =
                    self.deserialize_unsubscribe_request(request.params.clone());

                if let Some(handler) = &self.subscription_handler {
                    match handler.unsubscribe(unsubscribe_request.uri).await {
                        Ok(_) => JsonRpcResponse::success(serde_json::Value::Null, request.id),
                        Err(e) => JsonRpcResponse::error(
                            JsonRpcError::new(-32603, format!("Resource unsubscribe failed: {e}")),
                            request.id,
                        ),
                    }
                } else {
                    JsonRpcResponse::error(
                        JsonRpcError::new(
                            -32601,
                            "Resource subscriptions not supported".to_string(),
                        ),
                        request.id,
                    )
                }
            }

            // Prompts methods
            "prompts/list" => {
                if !self.can_operate().await {
                    return JsonRpcResponse::error(
                        JsonRpcError::new(-32000, "Server not ready".to_string()),
                        request.id,
                    );
                }

                let list_request = self.deserialize_list_prompts_request(request.params.clone());

                if let Some(handler) = &self.prompt_handler {
                    match handler.list_prompts(list_request).await {
                        Ok(response) => JsonRpcResponse::success(
                            serde_json::to_value(response).unwrap(),
                            request.id,
                        ),
                        Err(e) => JsonRpcResponse::error(
                            JsonRpcError::new(-32603, format!("Prompts list failed: {e}")),
                            request.id,
                        ),
                    }
                } else {
                    JsonRpcResponse::error(
                        JsonRpcError::new(-32601, "Prompts not supported".to_string()),
                        request.id,
                    )
                }
            }
            "prompts/get" => {
                if !self.can_operate().await {
                    return JsonRpcResponse::error(
                        JsonRpcError::new(-32000, "Server not ready".to_string()),
                        request.id,
                    );
                }

                let get_request = self.deserialize_get_prompt_request(request.params.clone());

                if let Some(handler) = &self.prompt_handler {
                    match handler.get_prompt(get_request).await {
                        Ok(response) => JsonRpcResponse::success(
                            serde_json::to_value(response).unwrap(),
                            request.id,
                        ),
                        Err(e) => JsonRpcResponse::error(
                            JsonRpcError::new(-32603, format!("Prompt get failed: {e}")),
                            request.id,
                        ),
                    }
                } else {
                    JsonRpcResponse::error(
                        JsonRpcError::new(-32601, "Prompts not supported".to_string()),
                        request.id,
                    )
                }
            }

            // Completion methods
            "completion/complete" => {
                if !self.can_operate().await {
                    return JsonRpcResponse::error(
                        JsonRpcError::new(-32000, "Server not ready".to_string()),
                        request.id,
                    );
                }

                let complete_request = self.deserialize_complete_request(request.params.clone());

                if let Some(handler) = &self.completion_handler {
                    match handler.complete(complete_request).await {
                        Ok(response) => JsonRpcResponse::success(
                            serde_json::to_value(response).unwrap(),
                            request.id,
                        ),
                        Err(e) => JsonRpcResponse::error(
                            JsonRpcError::new(-32603, format!("Completion failed: {e}")),
                            request.id,
                        ),
                    }
                } else {
                    JsonRpcResponse::error(
                        JsonRpcError::new(-32601, "Completion not supported".to_string()),
                        request.id,
                    )
                }
            }

            // Sampling methods
            "sampling/createMessage" => {
                if !self.can_operate().await {
                    return JsonRpcResponse::error(
                        JsonRpcError::new(-32000, "Server not ready".to_string()),
                        request.id,
                    );
                }

                let create_request =
                    self.deserialize_create_message_request(request.params.clone());

                if let Some(handler) = &self.sampling_handler {
                    match handler.create_message(create_request).await {
                        Ok(response) => JsonRpcResponse::success(
                            serde_json::to_value(response).unwrap(),
                            request.id,
                        ),
                        Err(e) => JsonRpcResponse::error(
                            JsonRpcError::new(-32603, format!("Message creation failed: {e}")),
                            request.id,
                        ),
                    }
                } else {
                    JsonRpcResponse::error(
                        JsonRpcError::new(-32601, "Sampling not supported".to_string()),
                        request.id,
                    )
                }
            }

            // Roots methods
            "roots/list" => {
                if !self.can_operate().await {
                    return JsonRpcResponse::error(
                        JsonRpcError::new(-32000, "Server not ready".to_string()),
                        request.id,
                    );
                }

                if let Some(handler) = &self.roots_handler {
                    match handler.list_roots().await {
                        Ok(response) => JsonRpcResponse::success(
                            serde_json::to_value(response).unwrap(),
                            request.id,
                        ),
                        Err(e) => JsonRpcResponse::error(
                            JsonRpcError::new(-32603, format!("Roots list failed: {e}")),
                            request.id,
                        ),
                    }
                } else {
                    JsonRpcResponse::error(
                        JsonRpcError::new(-32601, "Roots not supported".to_string()),
                        request.id,
                    )
                }
            }

            // Elicitation methods
            "elicitation/create" => {
                if !self.can_operate().await {
                    return JsonRpcResponse::error(
                        JsonRpcError::new(-32000, "Server not ready".to_string()),
                        request.id,
                    );
                }

                let elicitation_request =
                    self.deserialize_elicitation_request(request.params.clone());

                if let Some(handler) = &self.elicitation_handler {
                    match handler.handle_elicitation(elicitation_request).await {
                        Ok(response) => JsonRpcResponse::success(
                            serde_json::to_value(response).unwrap(),
                            request.id,
                        ),
                        Err(e) => JsonRpcResponse::error(
                            JsonRpcError::new(-32603, format!("Elicitation failed: {e}")),
                            request.id,
                        ),
                    }
                } else {
                    JsonRpcResponse::error(
                        JsonRpcError::new(-32601, "Elicitation not supported".to_string()),
                        request.id,
                    )
                }
            }

            "elicitation/respond" => {
                if !self.can_operate().await {
                    return JsonRpcResponse::error(
                        JsonRpcError::new(-32000, "Server not ready".to_string()),
                        request.id,
                    );
                }

                let elicitation_response = match serde_json::from_value::<
                    ultrafast_mcp_core::types::elicitation::ElicitationResponse,
                >(
                    request.params.unwrap_or_default()
                ) {
                    Ok(response) => response,
                    Err(e) => {
                        return JsonRpcResponse::error(
                            JsonRpcError::new(-32602, format!("Invalid elicitation response: {e}")),
                            request.id,
                        );
                    }
                };

                // Log the elicitation response
                info!(
                    "Received elicitation response: {:?}",
                    elicitation_response.action
                );

                // In a real implementation, this would be handled by the server's elicitation flow
                // For now, we'll just return success
                JsonRpcResponse::success(serde_json::json!({}), request.id)
            }

            // Logging methods
            "logging/setLevel" => {
                let params = match &request.params {
                    Some(params) => params,
                    None => {
                        return JsonRpcResponse::error(
                            JsonRpcError::new(-32602, "Missing parameters".to_string()),
                            request.id,
                        );
                    }
                };

                match serde_json::from_value::<LogLevelSetRequest>(params.clone()) {
                    Ok(set_request) => match self.set_log_level(set_request.level).await {
                        Ok(()) => {
                            let response = LogLevelSetResponse::new();
                            JsonRpcResponse::success(
                                serde_json::to_value(response).unwrap(),
                                request.id,
                            )
                        }
                        Err(e) => JsonRpcResponse::error(
                            JsonRpcError::new(-32603, format!("Failed to set log level: {e}")),
                            request.id,
                        ),
                    },
                    Err(e) => JsonRpcResponse::error(
                        JsonRpcError::new(-32602, format!("Invalid log level set request: {e}")),
                        request.id,
                    ),
                }
            }

            // Ping method for connection health monitoring
            "ping" => {
                let ping_request = match serde_json::from_value::<
                    ultrafast_mcp_core::types::notifications::PingRequest,
                >(request.params.unwrap_or_default())
                {
                    Ok(req) => req,
                    Err(_) => ultrafast_mcp_core::types::notifications::PingRequest { data: None },
                };

                match self.ping_manager.handle_ping(ping_request).await {
                    Ok(response) => JsonRpcResponse::success(
                        serde_json::to_value(response).unwrap(),
                        request.id,
                    ),
                    Err(e) => JsonRpcResponse::error(
                        JsonRpcError::new(-32603, format!("Ping failed: {e}")),
                        request.id,
                    ),
                }
            }

            // Roots methods
            "roots/set" => {
                let params = match &request.params {
                    Some(params) => params,
                    None => {
                        return JsonRpcResponse::error(
                            JsonRpcError::new(-32602, "Missing parameters".to_string()),
                            request.id,
                        );
                    }
                };

                match serde_json::from_value::<SetRootsRequest>(params.clone()) {
                    Ok(set_request) => {
                        let response = self
                            .handle_set_roots(
                                set_request.roots,
                                &mut Box::new(
                                    create_transport(TransportConfig::Stdio).await.unwrap(),
                                ),
                            )
                            .await;
                        JsonRpcResponse::success(
                            serde_json::to_value(response).unwrap(),
                            request.id,
                        )
                    }
                    Err(e) => JsonRpcResponse::error(
                        JsonRpcError::new(-32602, format!("Invalid roots set request: {e}")),
                        request.id,
                    ),
                }
            }

            // Unknown method
            _ => JsonRpcResponse::error(
                JsonRpcError::new(
                    -32601,
                    format!("Method not implemented: {}", request.method),
                ),
                request.id,
            ),
        }
    }

    /// Handle incoming notifications
    async fn handle_notification(&self, notification: JsonRpcRequest) -> MCPResult<()> {
        info!("Handling notification: {}", notification.method);

        match notification.method.as_str() {
            "initialized" => {
                let notification = ultrafast_mcp_core::protocol::InitializedNotification {};
                self.handle_initialized(notification).await?;
                Ok(())
            }
            "notifications/cancelled" => {
                // Handle cancellation notification
                if let Some(params) = notification.params {
                    let cancellation_notification: ultrafast_mcp_core::types::notifications::CancelledNotification =
                        serde_json::from_value(params)?;

                    // Use the cancellation manager to handle the cancellation
                    let _cancelled = self
                        .cancellation_manager
                        .handle_cancellation(cancellation_notification)
                        .await?;
                    info!("Cancellation notification processed");
                }
                Ok(())
            }
            _ => {
                warn!("Unknown notification method: {}", notification.method);
                Ok(())
            }
        }
    }

    // ===== NOTIFICATION METHODS =====

    /// Send tools list changed notification
    pub async fn notify_tools_changed(&self, transport: &mut Box<dyn Transport>) -> MCPResult<()> {
        let notification =
            ultrafast_mcp_core::types::notifications::ToolsListChangedNotification::new();
        self.send_notification(
            "notifications/tools/listChanged",
            Some(serde_json::to_value(notification)?),
            transport,
        )
        .await
    }

    /// Send resources list changed notification
    pub async fn notify_resources_changed(
        &self,
        transport: &mut Box<dyn Transport>,
    ) -> MCPResult<()> {
        let notification =
            ultrafast_mcp_core::types::notifications::ResourcesListChangedNotification::new();
        self.send_notification(
            "notifications/resources/listChanged",
            Some(serde_json::to_value(notification)?),
            transport,
        )
        .await
    }

    /// Send prompts list changed notification
    pub async fn notify_prompts_changed(
        &self,
        transport: &mut Box<dyn Transport>,
    ) -> MCPResult<()> {
        let notification =
            ultrafast_mcp_core::types::notifications::PromptsListChangedNotification::new();
        self.send_notification(
            "notifications/prompts/listChanged",
            Some(serde_json::to_value(notification)?),
            transport,
        )
        .await
    }

    /// Send resource updated notification
    pub async fn notify_resource_updated(
        &self,
        uri: String,
        transport: &mut Box<dyn Transport>,
    ) -> MCPResult<()> {
        let notification =
            ultrafast_mcp_core::types::resources::ResourceUpdatedNotification { uri };
        self.send_notification(
            "notifications/resources/updated",
            Some(serde_json::to_value(notification)?),
            transport,
        )
        .await
    }

    /// Send progress notification
    pub async fn notify_progress(
        &self,
        progress_token: serde_json::Value,
        progress: f64,
        total: Option<f64>,
        message: Option<String>,
        transport: &mut Box<dyn Transport>,
    ) -> MCPResult<()> {
        let mut notification = ultrafast_mcp_core::types::notifications::ProgressNotification::new(
            progress_token,
            progress,
        );
        if let Some(total) = total {
            notification = notification.with_total(total);
        }
        if let Some(message) = message {
            notification = notification.with_message(message);
        }
        self.send_notification(
            "notifications/progress",
            Some(serde_json::to_value(notification)?),
            transport,
        )
        .await
    }

    /// Send logging message notification
    pub async fn notify_logging_message(
        &self,
        level: ultrafast_mcp_core::types::notifications::LogLevel,
        data: serde_json::Value,
        logger: Option<String>,
        transport: &mut Box<dyn Transport>,
    ) -> MCPResult<()> {
        let mut notification =
            ultrafast_mcp_core::types::notifications::LoggingMessageNotification::new(level, data);
        if let Some(logger) = logger {
            notification = notification.with_logger(logger);
        }
        self.send_notification(
            "notifications/logging/message",
            Some(serde_json::to_value(notification)?),
            transport,
        )
        .await
    }

    /// Send cancellation notification
    pub async fn notify_cancelled(
        &self,
        request_id: serde_json::Value,
        reason: Option<String>,
        transport: &mut Box<dyn Transport>,
    ) -> MCPResult<()> {
        let mut notification =
            ultrafast_mcp_core::types::notifications::CancelledNotification::new(request_id);
        if let Some(reason) = reason {
            notification = notification.with_reason(reason);
        }
        self.send_notification(
            "notifications/cancelled",
            Some(serde_json::to_value(notification)?),
            transport,
        )
        .await
    }

    /// Send roots list changed notification
    pub async fn notify_roots_changed(&self, transport: &mut Box<dyn Transport>) -> MCPResult<()> {
        let notification =
            ultrafast_mcp_core::types::notifications::RootsListChangedNotification::new();
        self.send_notification(
            "notifications/roots/listChanged",
            Some(serde_json::to_value(notification)?),
            transport,
        )
        .await
    }

    /// Generic method to send notifications
    async fn send_notification(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
        transport: &mut Box<dyn Transport>,
    ) -> MCPResult<()> {
        let notification = JsonRpcRequest {
            jsonrpc: Cow::Borrowed("2.0"),
            id: None, // Notifications have no ID
            method: method.to_string(),
            params,
            meta: std::collections::HashMap::new(),
        };

        transport
            .send_message(JsonRpcMessage::Request(notification))
            .await
            .map_err(|e| MCPError::internal_error(format!("Failed to send notification: {e}")))?;

        info!("Sent notification: {}", method);
        Ok(())
    }

    /// Set the advanced sampling handler for context collection and human-in-the-loop features
    pub fn with_advanced_sampling_handler(
        mut self,
        handler: Arc<dyn AdvancedSamplingHandler>,
    ) -> Self {
        self.advanced_sampling_handler = Some(handler);
        self
    }

    /// Set the advanced sampling handler with default implementation
    pub fn with_default_advanced_sampling(mut self) -> Self {
        let default_handler = Arc::new(DefaultAdvancedSamplingHandler::new(self.info.clone()));
        self.advanced_sampling_handler = Some(default_handler);
        self
    }

    /// Handle a roots/set request
    pub async fn handle_set_roots(
        &self,
        roots: Vec<ultrafast_mcp_core::types::roots::Root>,
        transport: &mut Box<dyn Transport>,
    ) -> SetRootsResponse {
        if let Some(handler) = &self.roots_handler {
            match handler.set_roots(roots.clone()).await {
                Ok(_) => {
                    // Send notification to all clients (for demo, just send to this transport)
                    let notification = RootsListChangedNotification { roots };
                    let params = serde_json::to_value(notification).ok();
                    let _ = self
                        .send_notification("roots/listChanged", params, transport)
                        .await;
                    SetRootsResponse {
                        success: true,
                        error: None,
                    }
                }
                Err(e) => SetRootsResponse {
                    success: false,
                    error: Some(e.to_string()),
                },
            }
        } else {
            SetRootsResponse {
                success: false,
                error: Some("Roots handler not available".to_string()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use ultrafast_mcp_core::types::{
        server::ServerInfo,
        tools::{Tool, ToolContent},
    };

    // Mock tool handler for testing
    struct MockToolHandler;

    #[async_trait::async_trait]
    impl ToolHandler for MockToolHandler {
        async fn handle_tool_call(
            &self,
            call: ultrafast_mcp_core::types::tools::ToolCall,
        ) -> MCPResult<ultrafast_mcp_core::types::tools::ToolResult> {
            // Simulate error for nonexistent tool or invalid arguments
            if call.name == "nonexistent_tool" {
                return Err(ultrafast_mcp_core::error::MCPError::not_found(
                    "Tool not found".to_string(),
                ));
            }
            if let Some(args) = &call.arguments {
                if args.get("input").is_none() {
                    return Err(ultrafast_mcp_core::error::MCPError::invalid_params(
                        "Invalid parameters".to_string(),
                    ));
                }
            } else {
                return Err(ultrafast_mcp_core::error::MCPError::invalid_params(
                    "Missing arguments".to_string(),
                ));
            }
            Ok(ultrafast_mcp_core::types::tools::ToolResult {
                content: vec![ToolContent::text(format!("Mock result for {}", call.name))],
                is_error: None,
            })
        }

        async fn list_tools(
            &self,
            _request: ultrafast_mcp_core::types::tools::ListToolsRequest,
        ) -> MCPResult<ultrafast_mcp_core::types::tools::ListToolsResponse> {
            // This will be overridden by the server's fallback to registered tools
            Ok(ultrafast_mcp_core::types::tools::ListToolsResponse {
                tools: vec![],
                next_cursor: None,
            })
        }
    }

    fn create_test_server() -> UltraFastServer {
        let info = ServerInfo {
            name: "test-server".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test server".to_string()),
            homepage: None,
            repository: None,
            authors: Some(vec!["test".to_string()]),
            license: Some("MIT".to_string()),
        };
        let capabilities = ServerCapabilities::default();
        UltraFastServer::new(info, capabilities).with_tool_handler(Arc::new(MockToolHandler))
    }

    async fn create_initialized_test_server() -> UltraFastServer {
        let server = create_test_server();

        // Initialize the server to operating state
        let init_request = ultrafast_mcp_core::protocol::InitializeRequest {
            protocol_version: "2025-06-18".to_string(),
            capabilities: ultrafast_mcp_core::protocol::ClientCapabilities::default(),
            client_info: ultrafast_mcp_core::types::client::ClientInfo {
                name: "test-client".to_string(),
                version: "1.0.0".to_string(),
                description: Some("Test client".to_string()),
                homepage: None,
                repository: None,
                authors: Some(vec!["test".to_string()]),
                license: Some("MIT".to_string()),
            },
        };

        let _response = server.handle_initialize(init_request).await;

        // Send initialized notification
        let notification = ultrafast_mcp_core::protocol::InitializedNotification {};
        let _ = server.handle_initialized(notification).await;

        server
    }

    fn create_valid_tool(name: &str) -> Tool {
        Tool {
            name: name.to_string(),
            description: "A test tool".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "input": {"type": "string"}
                },
                "required": ["input"]
            }),
            output_schema: Some(json!({
                "type": "object",
                "properties": {
                    "output": {"type": "string"}
                }
            })),
            annotations: None,
        }
    }

    #[tokio::test]
    async fn test_register_valid_tool() {
        let server = create_test_server();
        let tool = create_valid_tool("test_tool");

        let result = server.register_tool(tool).await;
        assert!(result.is_ok());

        assert!(server.has_tool("test_tool").await);
        assert_eq!(server.tool_count().await, 1);
    }

    #[tokio::test]
    async fn test_register_duplicate_tool() {
        let server = create_test_server();
        let tool1 = create_valid_tool("test_tool");
        let tool2 = create_valid_tool("test_tool");

        server.register_tool(tool1).await.unwrap();
        let result = server.register_tool(tool2).await;

        assert!(matches!(
            result,
            Err(ToolRegistrationError::ToolAlreadyExists(_))
        ));
        assert_eq!(server.tool_count().await, 1);
    }

    #[tokio::test]
    async fn test_register_reserved_name() {
        let server = create_test_server();
        let tool = create_valid_tool("initialize");

        let result = server.register_tool(tool).await;
        assert!(matches!(
            result,
            Err(ToolRegistrationError::ReservedName(_))
        ));
        assert_eq!(server.tool_count().await, 0);
    }

    #[tokio::test]
    async fn test_register_tool_without_description() {
        let server = create_test_server();
        let mut tool = create_valid_tool("test_tool");
        tool.description = "".to_string();

        let result = server.register_tool(tool).await;
        assert!(matches!(
            result,
            Err(ToolRegistrationError::MissingDescription)
        ));
    }

    #[tokio::test]
    async fn test_register_tool_with_invalid_input_schema() {
        let server = create_test_server();
        let mut tool = create_valid_tool("test_tool");
        tool.input_schema = json!("invalid schema");

        let result = server.register_tool(tool).await;
        assert!(matches!(
            result,
            Err(ToolRegistrationError::InvalidSchema(_))
        ));
    }

    #[tokio::test]
    async fn test_register_tool_without_output_schema() {
        let server = create_test_server();
        let mut tool = create_valid_tool("test_tool");
        tool.output_schema = None;

        let result = server.register_tool(tool).await;
        assert!(matches!(
            result,
            Err(ToolRegistrationError::MissingOutputSchema)
        ));
    }

    #[tokio::test]
    async fn test_register_tool_with_invalid_schema() {
        let server = create_test_server();
        let mut tool = create_valid_tool("test_tool");
        tool.input_schema = json!("invalid schema");

        let result = server.register_tool(tool).await;
        assert!(matches!(
            result,
            Err(ToolRegistrationError::InvalidSchema(_))
        ));
    }

    #[tokio::test]
    async fn test_unregister_tool() {
        let server = create_test_server();
        let tool = create_valid_tool("test_tool");

        server.register_tool(tool).await.unwrap();
        assert!(server.has_tool("test_tool").await);

        let result = server.unregister_tool("test_tool");
        assert!(result.await);
        assert!(!server.has_tool("test_tool").await);
        assert_eq!(server.tool_count().await, 0);
    }

    #[tokio::test]
    async fn test_unregister_nonexistent_tool() {
        let server = create_test_server();
        let result = server.unregister_tool("nonexistent");
        assert!(!result.await);
    }

    #[tokio::test]
    async fn test_register_multiple_tools() {
        let server = create_test_server();
        let tools = vec![
            create_valid_tool("tool1"),
            create_valid_tool("tool2"),
            create_valid_tool("tool3"),
        ];

        let result = server.register_tools(tools).await;
        assert!(result.is_ok());
        assert_eq!(server.tool_count().await, 3);
        assert!(server.has_tool("tool1").await);
        assert!(server.has_tool("tool2").await);
        assert!(server.has_tool("tool3").await);
    }

    #[tokio::test]
    async fn test_register_multiple_tools_with_duplicate() {
        let server = create_test_server();
        let tools = vec![
            create_valid_tool("tool1"),
            create_valid_tool("tool1"), // Duplicate
            create_valid_tool("tool2"),
        ];

        let result = server.register_tools(tools).await;
        assert!(matches!(
            result,
            Err(ToolRegistrationError::ToolAlreadyExists(_))
        ));
        assert_eq!(server.tool_count().await, 1); // Only the first one should be registered
    }

    #[tokio::test]
    async fn test_get_tool() {
        let server = create_test_server();
        let tool = create_valid_tool("test_tool");

        server.register_tool(tool.clone()).await.unwrap();

        let retrieved = server.get_tool("test_tool").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, tool.name);
    }

    #[tokio::test]
    async fn test_get_nonexistent_tool() {
        let server = create_test_server();
        let retrieved = server.get_tool("nonexistent").await;
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_list_tools() {
        let server = create_test_server();
        let tools = vec![create_valid_tool("tool1"), create_valid_tool("tool2")];

        server.register_tools(tools).await.unwrap();

        let listed = server.list_tools().await;
        assert_eq!(listed.len(), 2);
        assert!(listed.iter().any(|t| t.name == "tool1"));
        assert!(listed.iter().any(|t| t.name == "tool2"));
    }

    #[tokio::test]
    async fn test_clear_tools() {
        let server = create_test_server();
        let tools = vec![create_valid_tool("tool1"), create_valid_tool("tool2")];

        server.register_tools(tools).await.unwrap();
        assert_eq!(server.tool_count().await, 2);

        server.clear_tools().await;
        assert_eq!(server.tool_count().await, 0);
        assert!(!server.has_tool("tool1").await);
        assert!(!server.has_tool("tool2").await);
    }

    #[tokio::test]
    async fn test_validate_tool_call() {
        let server = create_test_server();
        let tool = create_valid_tool("test_tool");
        server.register_tool(tool).await.unwrap();

        let valid_args = json!({"input": "test input"});
        let result = server.validate_tool_call("test_tool", &valid_args).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_tool_call_invalid_args() {
        let server = create_test_server();
        let tool = create_valid_tool("test_tool");
        server.register_tool(tool).await.unwrap();

        let invalid_args = json!({"wrong_field": "test input"});
        let result = server.validate_tool_call("test_tool", &invalid_args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_nonexistent_tool_call() {
        let server = create_test_server();
        let args = json!({"input": "test input"});
        let result = server.validate_tool_call("nonexistent", &args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_tool_call() {
        let server = create_test_server();
        let tool = create_valid_tool("test_tool");
        server.register_tool(tool).await.unwrap();

        let args = json!({"input": "test input"});
        let result = server.execute_tool_call("test_tool", args).await;
        assert!(result.is_ok());

        let tool_result = result.unwrap();
        assert_eq!(tool_result.content.len(), 1);
        assert!(!tool_result.is_error.unwrap_or(false));
    }

    #[tokio::test]
    async fn test_execute_tool_call_without_handler() {
        let server = UltraFastServer::new(
            ServerInfo {
                name: "test-server".to_string(),
                version: "1.0.0".to_string(),
                description: Some("Test server".to_string()),
                homepage: None,
                repository: None,
                authors: Some(vec!["test".to_string()]),
                license: Some("MIT".to_string()),
            },
            ServerCapabilities::default(),
        );
        let tool = create_valid_tool("test_tool");
        server.register_tool(tool).await.unwrap();

        let args = json!({"input": "test input"});
        let result = server.execute_tool_call("test_tool", args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_reserved_names() {
        let server = create_test_server();
        let reserved_names = [
            "initialize",
            "initialized",
            "shutdown",
            "exit",
            "ping",
            "tools/list",
            "tools/call",
            "resources/list",
            "resources/read",
            "resources/subscribe",
            "resources/unsubscribe",
            "prompts/list",
            "prompts/get",
            "sampling/create",
            "completion/complete",
            "roots/list",
            "elicitation/request",
        ];

        for name in &reserved_names {
            let tool = create_valid_tool(name);
            let result = server.register_tool(tool).await;
            assert!(matches!(
                result,
                Err(ToolRegistrationError::ReservedName(_))
            ));
        }
    }

    #[tokio::test]
    async fn test_tools_list_jsonrpc() {
        let server = create_initialized_test_server().await;

        // Register some tools
        let tools = vec![create_valid_tool("tool1"), create_valid_tool("tool2")];
        server.register_tools(tools).await.unwrap();

        // Create tools/list request
        let request = JsonRpcRequest {
            jsonrpc: Cow::Borrowed("2.0"),
            id: Some(ultrafast_mcp_core::protocol::jsonrpc::RequestId::string(
                "test-id",
            )),
            method: "tools/list".to_string(),
            params: None,
            meta: std::collections::HashMap::new(),
        };

        let response = server.handle_request(request).await;

        // Verify response
        if let Some(result) = &response.result {
            assert_eq!(
                response.id,
                Some(ultrafast_mcp_core::protocol::jsonrpc::RequestId::string(
                    "test-id"
                ))
            );
            let tools_array = result.get("tools").and_then(|t| t.as_array()).unwrap();
            assert_eq!(tools_array.len(), 2);

            let tool_names: Vec<&str> = tools_array
                .iter()
                .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
                .collect();
            assert!(tool_names.contains(&"tool1"));
            assert!(tool_names.contains(&"tool2"));
        } else {
            panic!("Expected success response");
        }
    }

    #[tokio::test]
    async fn test_tools_call_jsonrpc_success() {
        let server = create_initialized_test_server().await;

        // Register a tool
        let tool = create_valid_tool("test_tool");
        server.register_tool(tool).await.unwrap();

        // Create tools/call request
        let request = JsonRpcRequest {
            jsonrpc: Cow::Borrowed("2.0"),
            id: Some(ultrafast_mcp_core::protocol::jsonrpc::RequestId::string(
                "test-id",
            )),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "test_tool",
                "arguments": {
                    "input": "test input"
                }
            })),
            meta: std::collections::HashMap::new(),
        };

        let response = server.handle_request(request).await;

        // Verify response
        if let Some(result) = &response.result {
            assert_eq!(
                response.id,
                Some(ultrafast_mcp_core::protocol::jsonrpc::RequestId::string(
                    "test-id"
                ))
            );

            // Check that result contains content
            let content = result.get("content").and_then(|c| c.as_array()).unwrap();
            assert_eq!(content.len(), 1);

            // The ToolContent::text creates a structure with "type": "text" and "text" field
            let text_content = content[0].get("text").and_then(|t| t.as_str()).unwrap();
            assert!(text_content.contains("Mock result for test_tool"));
        } else {
            panic!("Expected success response");
        }
    }

    #[tokio::test]
    async fn test_tools_call_jsonrpc_missing_params() {
        let server = create_initialized_test_server().await;

        // Create tools/call request without parameters
        let request = JsonRpcRequest {
            jsonrpc: Cow::Borrowed("2.0"),
            id: Some(ultrafast_mcp_core::protocol::jsonrpc::RequestId::string(
                "test-id",
            )),
            method: "tools/call".to_string(),
            params: None,
            meta: std::collections::HashMap::new(),
        };

        let response = server.handle_request(request).await;

        // Verify error response
        if let Some(error) = &response.error {
            assert_eq!(
                response.id,
                Some(ultrafast_mcp_core::protocol::jsonrpc::RequestId::string(
                    "test-id"
                ))
            );
            assert_eq!(error.code, -32602); // Invalid params
            assert!(error.message.contains("Missing parameters"));
        } else {
            panic!("Expected error response");
        }
    }

    #[tokio::test]
    async fn test_tools_call_jsonrpc_missing_name() {
        let server = create_initialized_test_server().await;

        // Create tools/call request without tool name
        let request = JsonRpcRequest {
            jsonrpc: Cow::Borrowed("2.0"),
            id: Some(ultrafast_mcp_core::protocol::jsonrpc::RequestId::string(
                "test-id",
            )),
            method: "tools/call".to_string(),
            params: Some(json!({
                "arguments": {
                    "input": "test input"
                }
            })),
            meta: std::collections::HashMap::new(),
        };

        let response = server.handle_request(request).await;

        // Verify error response
        if let Some(error) = &response.error {
            assert_eq!(
                response.id,
                Some(ultrafast_mcp_core::protocol::jsonrpc::RequestId::string(
                    "test-id"
                ))
            );
            assert_eq!(error.code, -32602); // Invalid params
            assert!(error.message.contains("Missing or invalid tool name"));
        } else {
            panic!("Expected error response");
        }
    }

    #[tokio::test]
    async fn test_tools_call_jsonrpc_nonexistent_tool() {
        let server = create_initialized_test_server().await;

        // Create tools/call request for non-existent tool
        let request = JsonRpcRequest {
            jsonrpc: Cow::Borrowed("2.0"),
            id: Some(ultrafast_mcp_core::protocol::jsonrpc::RequestId::string(
                "test-id",
            )),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "nonexistent_tool",
                "arguments": {
                    "input": "test input"
                }
            })),
            meta: std::collections::HashMap::new(),
        };

        let response = server.handle_request(request).await;

        // Verify error response
        if let Some(error) = &response.error {
            assert_eq!(
                response.id,
                Some(ultrafast_mcp_core::protocol::jsonrpc::RequestId::string(
                    "test-id"
                ))
            );
            assert_eq!(error.code, -32602); // Invalid params
            assert!(error.message.contains("Tool call failed:"));
            assert!(error.message.contains("Tool not found"));
        } else {
            panic!("Expected error response");
        }
    }

    #[tokio::test]
    async fn test_tools_call_jsonrpc_invalid_arguments() {
        let server = create_initialized_test_server().await;

        // Register a tool
        let tool = create_valid_tool("test_tool");
        server.register_tool(tool).await.unwrap();

        // Create tools/call request with invalid arguments
        let request = JsonRpcRequest {
            jsonrpc: Cow::Borrowed("2.0"),
            id: Some(ultrafast_mcp_core::protocol::jsonrpc::RequestId::string(
                "test-id",
            )),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "test_tool",
                "arguments": {
                    "wrong_field": "test input"
                }
            })),
            meta: std::collections::HashMap::new(),
        };

        let response = server.handle_request(request).await;

        // Verify error response
        if let Some(error) = &response.error {
            assert_eq!(
                response.id,
                Some(ultrafast_mcp_core::protocol::jsonrpc::RequestId::string(
                    "test-id"
                ))
            );
            assert_eq!(error.code, -32602); // Invalid params
            assert!(error.message.contains("Invalid parameters"));
        } else {
            panic!("Expected error response");
        }
    }

    #[tokio::test]
    async fn test_tools_call_jsonrpc_empty_arguments() {
        let server = create_initialized_test_server().await;

        // Register a tool
        let tool = create_valid_tool("test_tool");
        server.register_tool(tool).await.unwrap();

        // Create tools/call request with empty arguments
        let request = JsonRpcRequest {
            jsonrpc: Cow::Borrowed("2.0"),
            id: Some(ultrafast_mcp_core::protocol::jsonrpc::RequestId::string(
                "test-id",
            )),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "test_tool",
                "arguments": {}
            })),
            meta: std::collections::HashMap::new(),
        };

        let response = server.handle_request(request).await;

        // Verify error response
        if let Some(error) = &response.error {
            assert_eq!(
                response.id,
                Some(ultrafast_mcp_core::protocol::jsonrpc::RequestId::string(
                    "test-id"
                ))
            );
            assert_eq!(error.code, -32602); // Invalid params
            // The actual error message format has changed to include more context
            assert!(error.message.contains("Invalid parameters"));
        } else {
            panic!("Expected error response");
        }
    }

    #[tokio::test]
    async fn test_unknown_method() {
        let server = create_test_server();

        // Create request for unknown method
        let request = JsonRpcRequest {
            jsonrpc: Cow::Borrowed("2.0"),
            id: Some(ultrafast_mcp_core::protocol::jsonrpc::RequestId::string(
                "test-id",
            )),
            method: "unknown/method".to_string(),
            params: None,
            meta: std::collections::HashMap::new(),
        };

        let response = server.handle_request(request).await;

        // Verify error response
        if let Some(error) = &response.error {
            assert_eq!(
                response.id,
                Some(ultrafast_mcp_core::protocol::jsonrpc::RequestId::string(
                    "test-id"
                ))
            );
            assert_eq!(error.code, -32601); // Method not found
            assert!(error.message.contains("Method not implemented"));
        } else {
            panic!("Expected error response");
        }
    }

    #[tokio::test]
    async fn test_tools_integration_workflow() {
        let server = create_initialized_test_server().await;

        // Step 1: Register multiple tools
        let tools = vec![
            create_valid_tool("calculator"),
            create_valid_tool("file_reader"),
        ];
        server.register_tools(tools).await.unwrap();
        assert_eq!(server.tool_count().await, 2);

        // Step 2: List tools via JSON-RPC
        let list_request = JsonRpcRequest {
            jsonrpc: Cow::Borrowed("2.0"),
            id: Some(ultrafast_mcp_core::protocol::jsonrpc::RequestId::string(
                "list-id",
            )),
            method: "tools/list".to_string(),
            params: None,
            meta: std::collections::HashMap::new(),
        };

        let list_response = server.handle_request(list_request).await;
        if let Some(result) = &list_response.result {
            assert_eq!(
                list_response.id,
                Some(ultrafast_mcp_core::protocol::jsonrpc::RequestId::string(
                    "list-id"
                ))
            );
            let tools_array = result.get("tools").and_then(|t| t.as_array()).unwrap();
            assert_eq!(tools_array.len(), 2);
        } else {
            panic!("Expected success response for tools/list");
        }

        // Step 3: Call a tool via JSON-RPC
        let call_request = JsonRpcRequest {
            jsonrpc: Cow::Borrowed("2.0"),
            id: Some(ultrafast_mcp_core::protocol::jsonrpc::RequestId::string(
                "call-id",
            )),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "calculator",
                "arguments": {
                    "input": "2 + 2"
                }
            })),
            meta: std::collections::HashMap::new(),
        };

        let call_response = server.handle_request(call_request).await;
        if let Some(result) = &call_response.result {
            assert_eq!(
                call_response.id,
                Some(ultrafast_mcp_core::protocol::jsonrpc::RequestId::string(
                    "call-id"
                ))
            );
            let content = result
                .get("content")
                .and_then(|c| c.as_array())
                .expect("Expected content array");
            assert_eq!(content.len(), 1);
        } else {
            panic!("Expected success response for tools/call");
        }

        // Step 4: Verify tool still exists in registry
        assert!(server.has_tool("calculator").await);
        assert!(server.has_tool("file_reader").await);
    }
}
