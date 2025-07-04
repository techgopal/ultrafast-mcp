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
    schema::validation::validate_tool_schema,
    types::{
        notifications::{LogLevel, LogLevelSetRequest, LogLevelSetResponse},
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

use crate::context::{Context, LoggerConfig};
use crate::handlers::*;

/// MCP Server state
#[derive(Debug, Clone)]
pub enum ServerState {
    Uninitialized,
    Initializing,
    Initialized,
    Shutdown,
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
    #[allow(dead_code)]
    resources: Arc<RwLock<HashMap<String, Resource>>>,
    #[allow(dead_code)]
    resource_templates: Arc<RwLock<HashMap<String, ResourceTemplate>>>,
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
            logging_config: Arc::new(RwLock::new(ServerLoggingConfig::default())),

            #[cfg(feature = "monitoring")]
            monitoring_system: None,
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
                "Input schema: {}",
                e
            )));
        }

        if let Some(output_schema) = &tool.output_schema {
            if let Err(e) = validate_tool_schema(output_schema) {
                return Err(ToolRegistrationError::InvalidSchema(format!(
                    "Output schema: {}",
                    e
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
        let tool = tool
            .ok_or_else(|| MCPError::invalid_request(format!("Tool '{}' not found", tool_name)))?;

        ultrafast_mcp_core::schema::validation::validate_tool_input(arguments, &tool.input_schema)
            .map_err(|e| {
                MCPError::invalid_request(format!(
                    "Tool '{}' input validation failed: {}",
                    tool_name, e
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
            .map_err(|e| MCPError::internal_error(format!("Tool execution failed: {}", e)))
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
        match request.method.as_str() {
            "tools/list" => {
                // List all registered tools
                let tools = self.list_tools().await;
                let response = serde_json::json!({
                    "tools": tools
                });
                JsonRpcResponse::success(response, request.id)
            }
            "tools/call" => {
                // Parse tool call arguments
                let params = match &request.params {
                    Some(params) => params,
                    None => {
                        return JsonRpcResponse::error(
                            JsonRpcError::new(-32602, "Missing parameters".to_string()),
                            request.id,
                        );
                    }
                };
                // Expect { name: String, arguments: Object }
                let tool_name = params.get("name").and_then(|v| v.as_str());
                let arguments = params
                    .get("arguments")
                    .cloned()
                    .unwrap_or(serde_json::json!({}));
                if let Some(tool_name) = tool_name {
                    match self.execute_tool_call(tool_name, arguments).await {
                        Ok(result) => JsonRpcResponse::success(
                            serde_json::to_value(result).unwrap(),
                            request.id,
                        ),
                        Err(e) => JsonRpcResponse::error(
                            JsonRpcError::new(-32603, format!("Tool call failed: {}", e)),
                            request.id,
                        ),
                    }
                } else {
                    JsonRpcResponse::error(
                        JsonRpcError::new(-32602, "Missing or invalid tool name".to_string()),
                        request.id,
                    )
                }
            }
            "logging/setLevel" => {
                // Handle log level set request
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
                            JsonRpcError::new(-32603, format!("Failed to set log level: {}", e)),
                            request.id,
                        ),
                    },
                    Err(e) => JsonRpcResponse::error(
                        JsonRpcError::new(-32602, format!("Invalid log level set request: {}", e)),
                        request.id,
                    ),
                }
            }
            _ => JsonRpcResponse::error(
                JsonRpcError::new(-32601, "Method not implemented".to_string()),
                request.id,
            ),
        }
    }

    /// Handle incoming notifications
    async fn handle_notification(&self, _notification: JsonRpcRequest) -> MCPResult<()> {
        // Implementation will be moved to handlers module
        Ok(())
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
            Ok(ultrafast_mcp_core::types::tools::ToolResult {
                content: vec![ToolContent::text(format!("Mock result for {}", call.name))],
                is_error: None,
            })
        }

        async fn list_tools(
            &self,
            _request: ultrafast_mcp_core::types::tools::ListToolsRequest,
        ) -> MCPResult<ultrafast_mcp_core::types::tools::ListToolsResponse> {
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
        let server = create_test_server();

        // Register some tools
        let tools = vec![create_valid_tool("tool1"), create_valid_tool("tool2")];
        server.register_tools(tools).await.unwrap();

        // Create tools/list request
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
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
        let server = create_test_server();

        // Register a tool
        let tool = create_valid_tool("test_tool");
        server.register_tool(tool).await.unwrap();

        // Create tools/call request
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
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
        let server = create_test_server();

        // Create tools/call request without parameters
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
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
        let server = create_test_server();

        // Create tools/call request without tool name
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
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
        let server = create_test_server();

        // Create tools/call request for non-existent tool
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
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
            assert_eq!(error.code, -32603); // Internal error
            assert!(error.message.contains("Tool call failed"));
        } else {
            panic!("Expected error response");
        }
    }

    #[tokio::test]
    async fn test_tools_call_jsonrpc_invalid_arguments() {
        let server = create_test_server();

        // Register a tool
        let tool = create_valid_tool("test_tool");
        server.register_tool(tool).await.unwrap();

        // Create tools/call request with invalid arguments
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
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
            assert_eq!(error.code, -32603); // Internal error
            assert!(error.message.contains("Tool call failed"));
        } else {
            panic!("Expected error response");
        }
    }

    #[tokio::test]
    async fn test_tools_call_jsonrpc_empty_arguments() {
        let server = create_test_server();

        // Register a tool
        let tool = create_valid_tool("test_tool");
        server.register_tool(tool).await.unwrap();

        // Create tools/call request with empty arguments
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(ultrafast_mcp_core::protocol::jsonrpc::RequestId::string(
                "test-id",
            )),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "test_tool"
                // No arguments field
            })),
            meta: std::collections::HashMap::new(),
        };

        let response = server.handle_request(request).await;

        // Verify error response (should fail validation due to missing required input)
        if let Some(error) = &response.error {
            assert_eq!(
                response.id,
                Some(ultrafast_mcp_core::protocol::jsonrpc::RequestId::string(
                    "test-id"
                ))
            );
            assert_eq!(error.code, -32603); // Internal error
            assert!(error.message.contains("Tool call failed"));
        } else {
            panic!("Expected error response");
        }
    }

    #[tokio::test]
    async fn test_unknown_method() {
        let server = create_test_server();

        // Create request for unknown method
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
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
        let server = create_test_server();

        // Step 1: Register multiple tools
        let tools = vec![
            create_valid_tool("calculator"),
            create_valid_tool("file_reader"),
        ];
        server.register_tools(tools).await.unwrap();
        assert_eq!(server.tool_count().await, 2);

        // Step 2: List tools via JSON-RPC
        let list_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
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
            jsonrpc: "2.0".to_string(),
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
            let content = result.get("content").and_then(|c| c.as_array()).unwrap();
            assert_eq!(content.len(), 1);
        } else {
            panic!("Expected success response for tools/call");
        }

        // Step 4: Verify tool still exists in registry
        assert!(server.has_tool("calculator").await);
        assert!(server.has_tool("file_reader").await);
    }
}
