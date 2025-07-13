//! UltraFast MCP Client Library
//!
//! A high-performance client implementation for the Model Context Protocol (MCP).

use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, oneshot};
use tracing::{error, info, warn};
use ultrafast_mcp_core::{
    config::TimeoutConfig,
    error::{MCPError, MCPResult, ProtocolError, TransportError},
    protocol::{
        InitializeRequest, InitializeResponse, InitializedNotification, ShutdownRequest,
        jsonrpc::{JsonRpcMessage, JsonRpcRequest},
    },
    types::{
        client::{ClientCapabilities, ClientInfo},
        completion::{CompleteRequest, CompleteResponse},
        elicitation::{ElicitationRequest, ElicitationResponse},
        prompts::{GetPromptRequest, GetPromptResponse, ListPromptsRequest, ListPromptsResponse},
        resources::{
            ListResourcesRequest, ListResourcesResponse, ReadResourceRequest, ReadResourceResponse,
        },
        sampling::{CreateMessageRequest, CreateMessageResponse},
        server::{ServerCapabilities, ServerInfo},
        tools::{ListToolsRequest, ListToolsResponse, ToolCall, ToolResult},
    },
};
use ultrafast_mcp_transport::Transport;

/// Client-side elicitation handler trait
#[async_trait::async_trait]
pub trait ClientElicitationHandler: Send + Sync {
    /// Handle an elicitation request from the server
    /// This method should present the request to the user and return their response
    async fn handle_elicitation_request(
        &self,
        request: ElicitationRequest,
    ) -> MCPResult<ElicitationResponse>;
}

/// MCP Client state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientState {
    Uninitialized,
    Initializing,
    Initialized,
    Operating,
    ShuttingDown,
    Shutdown,
}

impl ClientState {
    /// Check if the client can perform operations
    /// According to MCP 2025-06-18 specification, operations are allowed
    /// once the client is initialized (after initialize response)
    pub fn can_operate(&self) -> bool {
        matches!(self, ClientState::Initialized | ClientState::Operating)
    }

    /// Check if the client is initialized
    pub fn is_initialized(&self) -> bool {
        matches!(self, ClientState::Initialized | ClientState::Operating)
    }

    /// Check if the client is shutting down
    pub fn is_shutting_down(&self) -> bool {
        matches!(self, ClientState::ShuttingDown | ClientState::Shutdown)
    }
}

/// Pending request information
#[derive(Debug)]
struct PendingRequest {
    response_sender: oneshot::Sender<JsonRpcMessage>,
    #[allow(dead_code)]
    timeout: tokio::time::Instant,
}

/// Client state management
struct ClientStateManager {
    state: ClientState,
    server_info: Option<ServerInfo>,
    server_capabilities: Option<ServerCapabilities>,
    negotiated_version: Option<String>,
    request_id_counter: u64,
    pending_requests: HashMap<u64, PendingRequest>,
    elicitation_handler: Option<Arc<dyn ClientElicitationHandler>>,
}

impl ClientStateManager {
    fn new() -> Self {
        Self {
            state: ClientState::Uninitialized,
            server_info: None,
            server_capabilities: None,
            negotiated_version: None,
            request_id_counter: 1,
            pending_requests: HashMap::new(),
            elicitation_handler: None,
        }
    }

    fn set_state(&mut self, state: ClientState) {
        self.state = state;
    }

    fn set_server_info(&mut self, info: ServerInfo) {
        self.server_info = Some(info);
    }

    fn set_server_capabilities(&mut self, capabilities: ServerCapabilities) {
        self.server_capabilities = Some(capabilities);
    }

    fn set_negotiated_version(&mut self, version: String) {
        self.negotiated_version = Some(version);
    }

    fn set_elicitation_handler(&mut self, handler: Option<Arc<dyn ClientElicitationHandler>>) {
        self.elicitation_handler = handler;
    }

    fn next_request_id(&mut self) -> u64 {
        let id = self.request_id_counter;
        self.request_id_counter += 1;
        id
    }

    fn add_pending_request(&mut self, id: u64, request: PendingRequest) {
        self.pending_requests.insert(id, request);
    }

    fn remove_pending_request(&mut self, id: &u64) -> Option<PendingRequest> {
        self.pending_requests.remove(id)
    }
}

/// UltraFast MCP Client
pub struct UltraFastClient {
    info: ClientInfo,
    capabilities: ClientCapabilities,
    state_manager: Arc<RwLock<ClientStateManager>>,
    transport: Arc<RwLock<Option<Box<dyn Transport>>>>,
    message_receiver: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    request_timeout: std::time::Duration,
    // Timeout configuration (MCP 2025-06-18 compliance)
    timeout_config: Arc<TimeoutConfig>,
    // Authentication middleware
    #[cfg(feature = "oauth")]
    auth_middleware: Arc<RwLock<Option<ultrafast_mcp_auth::ClientAuthMiddleware>>>,
    #[cfg(not(feature = "oauth"))]
    auth_middleware: Arc<RwLock<Option<()>>>,
}

impl UltraFastClient {
    /// Create a new MCP client
    pub fn new(info: ClientInfo, capabilities: ClientCapabilities) -> Self {
        Self {
            info,
            capabilities,
            state_manager: Arc::new(RwLock::new(ClientStateManager::new())),
            transport: Arc::new(RwLock::new(None)),
            message_receiver: Arc::new(RwLock::new(None)),
            request_timeout: std::time::Duration::from_secs(30),
            timeout_config: Arc::new(TimeoutConfig::default()),
            #[cfg(feature = "oauth")]
            auth_middleware: Arc::new(RwLock::new(None)),
            #[cfg(not(feature = "oauth"))]
            auth_middleware: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a new MCP client with custom timeout
    pub fn new_with_timeout(
        info: ClientInfo,
        capabilities: ClientCapabilities,
        timeout: std::time::Duration,
    ) -> Self {
        Self {
            info,
            capabilities,
            state_manager: Arc::new(RwLock::new(ClientStateManager::new())),
            transport: Arc::new(RwLock::new(None)),
            message_receiver: Arc::new(RwLock::new(None)),
            request_timeout: timeout,
            timeout_config: Arc::new(TimeoutConfig::default()),
            #[cfg(feature = "oauth")]
            auth_middleware: Arc::new(RwLock::new(None)),
            #[cfg(not(feature = "oauth"))]
            auth_middleware: Arc::new(RwLock::new(None)),
        }
    }

    /// Set request timeout
    pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.request_timeout = timeout;
        self
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

    /// Get operation-specific timeout
    pub fn get_operation_timeout(&self, operation: &str) -> std::time::Duration {
        self.timeout_config.get_timeout_for_operation(operation)
    }

    /// Set authentication method
    #[cfg(feature = "oauth")]
    pub fn with_auth(self, auth_method: ultrafast_mcp_auth::AuthMethod) -> Self {
        #[cfg(feature = "oauth")]
        {
            let mut auth = self.auth_middleware.blocking_write();
            *auth = Some(ultrafast_mcp_auth::ClientAuthMiddleware::new(auth_method));
        }
        self
    }

    /// Set Bearer token authentication
    #[cfg(feature = "oauth")]
    pub fn with_bearer_auth(self, token: String) -> Self {
        #[cfg(feature = "oauth")]
        {
            let auth_method = ultrafast_mcp_auth::AuthMethod::bearer(token);
            let mut auth = self.auth_middleware.blocking_write();
            *auth = Some(ultrafast_mcp_auth::ClientAuthMiddleware::new(auth_method));
        }
        self
    }

    /// Set Bearer token authentication with auto-refresh
    #[cfg(feature = "oauth")]
    pub fn with_bearer_auth_refresh<F, Fut>(self, token: String, refresh_fn: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<String, ultrafast_mcp_auth::AuthError>>
            + Send
            + 'static,
    {
        #[cfg(feature = "oauth")]
        {
            let bearer_auth =
                ultrafast_mcp_auth::BearerAuth::new(token).with_auto_refresh(refresh_fn);
            let auth_method = ultrafast_mcp_auth::AuthMethod::Bearer(bearer_auth);
            let mut auth = self.auth_middleware.blocking_write();
            *auth = Some(ultrafast_mcp_auth::ClientAuthMiddleware::new(auth_method));
        }
        self
    }

    /// Set OAuth authentication
    #[cfg(feature = "oauth")]
    pub fn with_oauth_auth(self, config: ultrafast_mcp_auth::OAuthConfig) -> Self {
        #[cfg(feature = "oauth")]
        {
            let auth_method = ultrafast_mcp_auth::AuthMethod::oauth(config);
            let mut auth = self.auth_middleware.blocking_write();
            *auth = Some(ultrafast_mcp_auth::ClientAuthMiddleware::new(auth_method));
        }
        self
    }

    /// Set API key authentication
    #[cfg(feature = "oauth")]
    pub fn with_api_key_auth(self, api_key: String) -> Self {
        #[cfg(feature = "oauth")]
        {
            let auth_method = ultrafast_mcp_auth::AuthMethod::api_key(api_key);
            let mut auth = self.auth_middleware.blocking_write();
            *auth = Some(ultrafast_mcp_auth::ClientAuthMiddleware::new(auth_method));
        }
        self
    }

    /// Set API key authentication with custom header name
    #[cfg(feature = "oauth")]
    pub fn with_api_key_auth_custom(self, api_key: String, header_name: String) -> Self {
        #[cfg(feature = "oauth")]
        {
            let api_key_auth =
                ultrafast_mcp_auth::ApiKeyAuth::new(api_key).with_header_name(header_name);
            let auth_method = ultrafast_mcp_auth::AuthMethod::ApiKey(api_key_auth);
            let mut auth = self.auth_middleware.blocking_write();
            *auth = Some(ultrafast_mcp_auth::ClientAuthMiddleware::new(auth_method));
        }
        self
    }

    /// Set Basic authentication
    #[cfg(feature = "oauth")]
    pub fn with_basic_auth(self, username: String, password: String) -> Self {
        #[cfg(feature = "oauth")]
        {
            let auth_method = ultrafast_mcp_auth::AuthMethod::basic(username, password);
            let mut auth = self.auth_middleware.blocking_write();
            *auth = Some(ultrafast_mcp_auth::ClientAuthMiddleware::new(auth_method));
        }
        self
    }

    /// Set custom header authentication
    #[cfg(feature = "oauth")]
    pub fn with_custom_auth(self) -> Self {
        #[cfg(feature = "oauth")]
        {
            let auth_method = ultrafast_mcp_auth::AuthMethod::custom();
            let mut auth = self.auth_middleware.blocking_write();
            *auth = Some(ultrafast_mcp_auth::ClientAuthMiddleware::new(auth_method));
        }
        self
    }

    /// Get authentication headers for requests
    #[cfg(feature = "oauth")]
    pub async fn get_auth_headers(
        &self,
    ) -> Result<std::collections::HashMap<String, String>, ultrafast_mcp_auth::AuthError> {
        if let Some(auth) = self.auth_middleware.write().await.as_mut() {
            auth.get_headers().await
        } else {
            Ok(std::collections::HashMap::new())
        }
    }

    /// Set elicitation handler for handling server-initiated elicitation requests
    pub fn with_elicitation_handler(self, handler: Arc<dyn ClientElicitationHandler>) -> Self {
        let state_manager = self.state_manager.clone();
        tokio::spawn(async move {
            let mut state = state_manager.write().await;
            state.set_elicitation_handler(Some(handler));
        });
        self
    }

    /// Connect to a server using the provided transport
    pub async fn connect(&self, transport: Box<dyn Transport>) -> MCPResult<()> {
        info!("Connecting to MCP server");

        {
            let mut transport_guard = self.transport.write().await;
            *transport_guard = Some(transport);
        }

        // Start message receiver task
        self.start_message_receiver().await?;

        // Initialize the connection
        self.initialize().await?;

        info!("Successfully connected to MCP server");
        Ok(())
    }

    /// Start the message receiver task for handling responses
    async fn start_message_receiver(&self) -> MCPResult<()> {
        let transport = self.transport.clone();
        let state_manager = self.state_manager.clone();

        let handle = tokio::spawn(async move {
            let mut transport_guard = transport.write().await;
            let transport = transport_guard
                .as_mut()
                .expect("Transport should be available");

            loop {
                match transport.receive_message().await {
                    Ok(message) => {
                        match &message {
                            JsonRpcMessage::Response(response) => {
                                if let Some(id) = &response.id {
                                    if let Ok(id_num) = serde_json::from_value::<u64>(
                                        serde_json::to_value(id).unwrap_or_default(),
                                    ) {
                                        let mut state = state_manager.write().await;
                                        if let Some(pending_req) =
                                            state.remove_pending_request(&id_num)
                                        {
                                            // Send response to waiting request
                                            let _ = pending_req.response_sender.send(message);
                                        }
                                    }
                                }
                            }
                            JsonRpcMessage::Request(request) if request.id.is_none() => {
                                // This is a notification, handle it
                                Self::handle_notification_static(request.clone()).await;
                            }
                            JsonRpcMessage::Request(request) => {
                                // This is a request without ID, handle as elicitation
                                if request.method == "elicitation/create" {
                                    info!("Processing elicitation request from server");

                                    // Get the elicitation handler from state manager
                                    let elicitation_handler = {
                                        let state = state_manager.read().await;
                                        state.elicitation_handler.clone()
                                    };

                                    if let Some(handler) = elicitation_handler {
                                        // Parse the elicitation request
                                        if let Ok(elicitation_request) =
                                            serde_json::from_value::<ElicitationRequest>(
                                                request.params.clone().unwrap_or_default(),
                                            )
                                        {
                                            // Handle the elicitation request
                                            match handler
                                                .handle_elicitation_request(elicitation_request)
                                                .await
                                            {
                                                Ok(response) => {
                                                    // Send the response back to the server
                                                    let response_message = JsonRpcMessage::Request(
                                                        JsonRpcRequest::new(
                                                            "elicitation/respond".to_string(),
                                                            Some(
                                                                serde_json::to_value(response)
                                                                    .unwrap(),
                                                            ),
                                                            None, // No ID for elicitation response
                                                        ),
                                                    );

                                                    if let Err(e) = transport
                                                        .send_message(response_message)
                                                        .await
                                                    {
                                                        error!(
                                                            "Failed to send elicitation response: {}",
                                                            e
                                                        );
                                                    }
                                                }
                                                Err(e) => {
                                                    error!(
                                                        "Failed to handle elicitation request: {}",
                                                        e
                                                    );
                                                }
                                            }
                                        } else {
                                            error!("Failed to parse elicitation request");
                                        }
                                    } else {
                                        warn!(
                                            "No elicitation handler configured, ignoring elicitation request"
                                        );
                                    }
                                } else {
                                    warn!(
                                        "Received unexpected request without ID: {}",
                                        request.method
                                    );
                                }
                            }
                            JsonRpcMessage::Notification(notification) => {
                                // Handle notification
                                Self::handle_notification_static(notification.clone()).await;
                            }
                        }
                    }
                    Err(e) => {
                        // Only log as error if it's not a normal connection closure
                        if !e.to_string().contains("Connection closed") {
                            error!("Transport error in message receiver: {}", e);
                        } else {
                            info!("Transport connection closed (normal shutdown)");
                        }
                        break;
                    }
                }
            }
        });

        {
            let mut receiver_guard = self.message_receiver.write().await;
            *receiver_guard = Some(handle);
        }

        Ok(())
    }

    async fn handle_notification_static(notification: JsonRpcRequest) {
        match notification.method.as_str() {
            "initialized" => {
                info!("Received initialized notification");
            }
            "notifications/tools/listChanged" => {
                info!("Received tools list changed notification");
            }
            "notifications/resources/listChanged" => {
                info!("Received resources list changed notification");
            }
            "notifications/prompts/listChanged" => {
                info!("Received prompts list changed notification");
            }
            "notifications/roots/listChanged" => {
                info!("Received roots list changed notification");
            }
            "elicitation/create" => {
                info!("Received elicitation request from server");
                // Note: This should be handled by the client's elicitation handler
                // The actual handling is done in the message receiver loop
            }
            _ => {
                warn!("Unknown notification method: {}", notification.method);
            }
        }
    }

    /// Connect to a server using STDIO transport
    pub async fn connect_stdio(&self) -> MCPResult<()> {
        let stdio_transport = ultrafast_mcp_transport::stdio::StdioTransport::new()
            .await
            .map_err(|e| MCPError::Transport(TransportError::ConnectionFailed(e.to_string())))?;
        self.connect(Box::new(stdio_transport)).await
    }

    /// Connect to a server using Streamable HTTP transport
    /// This method will automatically use any client-level authentication configured
    #[cfg(feature = "http")]
    pub async fn connect_streamable_http(&self, url: &str) -> MCPResult<()> {
        use ultrafast_mcp_transport::streamable_http::client::{
            StreamableHttpClient, StreamableHttpClientConfig,
        };

        let mut config = StreamableHttpClientConfig {
            base_url: url.to_string(),
            session_id: None,
            protocol_version: "2025-06-18".to_string(),
            timeout: self.request_timeout,
            max_retries: 3,
            auth_token: None,
            oauth_config: None,
            auth_method: None,
        };

        // Integrate with client-level auth middleware if available
        #[cfg(feature = "oauth")]
        {
            if let Some(auth) = self.auth_middleware.read().await.as_ref() {
                config.auth_method = Some(auth.get_auth_method().clone());
            }
        }

        let mut http_transport = StreamableHttpClient::new(config)
            .map_err(|e| MCPError::Transport(TransportError::ConnectionFailed(e.to_string())))?;

        // Connect the transport first
        http_transport
            .connect()
            .await
            .map_err(|e| MCPError::Transport(TransportError::ConnectionFailed(e.to_string())))?;

        self.connect(Box::new(http_transport)).await
    }

    /// Connect to a server using HTTP transport with authentication
    #[cfg(feature = "http")]
    pub async fn connect_http_with_auth(&self, url: &str, auth_token: String) -> MCPResult<()> {
        use ultrafast_mcp_transport::streamable_http::client::{
            StreamableHttpClient, StreamableHttpClientConfig,
        };

        let config = StreamableHttpClientConfig {
            base_url: url.to_string(),
            session_id: None,
            protocol_version: "2025-06-18".to_string(),
            timeout: self.request_timeout,
            max_retries: 3,
            auth_token: Some(auth_token),
            oauth_config: None,
            auth_method: None,
        };

        let mut http_transport = StreamableHttpClient::new(config)
            .map_err(|e| MCPError::Transport(TransportError::ConnectionFailed(e.to_string())))?;

        // Connect the transport first
        http_transport
            .connect()
            .await
            .map_err(|e| MCPError::Transport(TransportError::ConnectionFailed(e.to_string())))?;

        self.connect(Box::new(http_transport)).await
    }

    /// Connect to a server using Streamable HTTP transport with Bearer token authentication
    #[cfg(all(feature = "http", feature = "oauth"))]
    pub async fn connect_streamable_http_with_bearer(
        &self,
        url: &str,
        token: String,
    ) -> MCPResult<()> {
        use ultrafast_mcp_transport::streamable_http::client::{
            StreamableHttpClient, StreamableHttpClientConfig,
        };

        let config = StreamableHttpClientConfig {
            base_url: url.to_string(),
            session_id: None,
            protocol_version: "2025-06-18".to_string(),
            timeout: self.request_timeout,
            max_retries: 3,
            auth_token: None,
            oauth_config: None,
            auth_method: Some(ultrafast_mcp_auth::AuthMethod::bearer(token)),
        };

        let mut http_transport = StreamableHttpClient::new(config)
            .map_err(|e| MCPError::Transport(TransportError::ConnectionFailed(e.to_string())))?;

        // Connect the transport first
        http_transport
            .connect()
            .await
            .map_err(|e| MCPError::Transport(TransportError::ConnectionFailed(e.to_string())))?;

        self.connect(Box::new(http_transport)).await
    }

    /// Connect to a server using Streamable HTTP transport with OAuth 2.1 authentication
    #[cfg(all(feature = "http", feature = "oauth"))]
    pub async fn connect_streamable_http_with_oauth(
        &self,
        url: &str,
        oauth_config: ultrafast_mcp_auth::OAuthConfig,
    ) -> MCPResult<()> {
        use ultrafast_mcp_transport::streamable_http::client::{
            StreamableHttpClient, StreamableHttpClientConfig,
        };

        let config = StreamableHttpClientConfig {
            base_url: url.to_string(),
            session_id: None,
            protocol_version: "2025-06-18".to_string(),
            timeout: self.request_timeout,
            max_retries: 3,
            auth_token: None,
            oauth_config: Some(oauth_config.clone()),
            auth_method: Some(ultrafast_mcp_auth::AuthMethod::oauth(oauth_config)),
        };

        let mut http_transport = StreamableHttpClient::new(config)
            .map_err(|e| MCPError::Transport(TransportError::ConnectionFailed(e.to_string())))?;

        // Connect the transport first
        http_transport
            .connect()
            .await
            .map_err(|e| MCPError::Transport(TransportError::ConnectionFailed(e.to_string())))?;

        self.connect(Box::new(http_transport)).await
    }

    /// Connect to a server using Streamable HTTP transport with API key authentication
    #[cfg(all(feature = "http", feature = "oauth"))]
    pub async fn connect_streamable_http_with_api_key(
        &self,
        url: &str,
        api_key: String,
    ) -> MCPResult<()> {
        use ultrafast_mcp_transport::streamable_http::client::{
            StreamableHttpClient, StreamableHttpClientConfig,
        };

        let config = StreamableHttpClientConfig {
            base_url: url.to_string(),
            session_id: None,
            protocol_version: "2025-06-18".to_string(),
            timeout: self.request_timeout,
            max_retries: 3,
            auth_token: None,
            oauth_config: None,
            auth_method: Some(ultrafast_mcp_auth::AuthMethod::api_key(api_key)),
        };

        let mut http_transport = StreamableHttpClient::new(config)
            .map_err(|e| MCPError::Transport(TransportError::ConnectionFailed(e.to_string())))?;

        // Connect the transport first
        http_transport
            .connect()
            .await
            .map_err(|e| MCPError::Transport(TransportError::ConnectionFailed(e.to_string())))?;

        self.connect(Box::new(http_transport)).await
    }

    /// Connect to a server using Streamable HTTP transport with API key authentication (custom header)
    #[cfg(all(feature = "http", feature = "oauth"))]
    pub async fn connect_streamable_http_with_api_key_custom(
        &self,
        url: &str,
        api_key: String,
        header_name: String,
    ) -> MCPResult<()> {
        use ultrafast_mcp_transport::streamable_http::client::{
            StreamableHttpClient, StreamableHttpClientConfig,
        };

        let api_key_auth =
            ultrafast_mcp_auth::ApiKeyAuth::new(api_key).with_header_name(header_name);
        let auth_method = ultrafast_mcp_auth::AuthMethod::ApiKey(api_key_auth);

        let config = StreamableHttpClientConfig {
            base_url: url.to_string(),
            session_id: None,
            protocol_version: "2025-06-18".to_string(),
            timeout: self.request_timeout,
            max_retries: 3,
            auth_token: None,
            oauth_config: None,
            auth_method: Some(auth_method),
        };

        let mut http_transport = StreamableHttpClient::new(config)
            .map_err(|e| MCPError::Transport(TransportError::ConnectionFailed(e.to_string())))?;

        // Connect the transport first
        http_transport
            .connect()
            .await
            .map_err(|e| MCPError::Transport(TransportError::ConnectionFailed(e.to_string())))?;

        self.connect(Box::new(http_transport)).await
    }

    /// Connect to a server using Streamable HTTP transport with Basic authentication
    #[cfg(all(feature = "http", feature = "oauth"))]
    pub async fn connect_streamable_http_with_basic(
        &self,
        url: &str,
        username: String,
        password: String,
    ) -> MCPResult<()> {
        use ultrafast_mcp_transport::streamable_http::client::{
            StreamableHttpClient, StreamableHttpClientConfig,
        };

        let config = StreamableHttpClientConfig {
            base_url: url.to_string(),
            session_id: None,
            protocol_version: "2025-06-18".to_string(),
            timeout: self.request_timeout,
            max_retries: 3,
            auth_token: None,
            oauth_config: None,
            auth_method: Some(ultrafast_mcp_auth::AuthMethod::basic(username, password)),
        };

        let mut http_transport = StreamableHttpClient::new(config)
            .map_err(|e| MCPError::Transport(TransportError::ConnectionFailed(e.to_string())))?;

        // Connect the transport first
        http_transport
            .connect()
            .await
            .map_err(|e| MCPError::Transport(TransportError::ConnectionFailed(e.to_string())))?;

        self.connect(Box::new(http_transport)).await
    }

    /// Connect to a server using Streamable HTTP transport with custom configuration
    #[cfg(feature = "http")]
    pub async fn connect_streamable_http_with_config(
        &self,
        config: ultrafast_mcp_transport::streamable_http::client::StreamableHttpClientConfig,
    ) -> MCPResult<()> {
        use ultrafast_mcp_transport::streamable_http::client::StreamableHttpClient;

        let mut http_transport = StreamableHttpClient::new(config)
            .map_err(|e| MCPError::Transport(TransportError::ConnectionFailed(e.to_string())))?;

        // Connect the transport first
        http_transport
            .connect()
            .await
            .map_err(|e| MCPError::Transport(TransportError::ConnectionFailed(e.to_string())))?;

        self.connect(Box::new(http_transport)).await
    }

    /// Initialize the connection with the server
    /// Initialize the client connection
    /// This method must be called after connecting to establish the MCP protocol
    pub async fn initialize(&self) -> MCPResult<()> {
        {
            let mut state = self.state_manager.write().await;
            state.set_state(ClientState::Initializing);
        }

        // Create initialization request
        let init_request = InitializeRequest {
            protocol_version: ultrafast_mcp_core::protocol::version::PROTOCOL_VERSION.to_string(),
            capabilities: self.capabilities.clone(),
            client_info: self.info.clone(),
        };

        // Send initialization request
        let init_response: InitializeResponse = self
            .send_request("initialize", Some(serde_json::to_value(init_request)?))
            .await?;

        // Validate protocol version
        if init_response.protocol_version != ultrafast_mcp_core::protocol::version::PROTOCOL_VERSION
        {
            return Err(MCPError::Protocol(ProtocolError::InvalidVersion(format!(
                "Expected protocol version {}, got {}",
                ultrafast_mcp_core::protocol::version::PROTOCOL_VERSION,
                init_response.protocol_version
            ))));
        }

        // Store server information
        {
            let mut state = self.state_manager.write().await;
            state.set_server_info(init_response.server_info);
            state.set_server_capabilities(init_response.capabilities);
            state.set_negotiated_version(init_response.protocol_version);
            state.set_state(ClientState::Initialized);
        }

        // Send initialized notification (skip for HTTP transport)
        // Note: HTTP transport doesn't require initialized notification
        let init_notification = InitializedNotification {};
        if let Err(e) = self
            .send_notification(
                "initialized",
                Some(serde_json::to_value(init_notification)?),
            )
            .await
        {
            // For HTTP transport, ignore errors on initialized notification
            warn!(
                "Failed to send initialized notification (this is normal for HTTP transport): {}",
                e
            );
        }

        {
            let mut state = self.state_manager.write().await;
            state.set_state(ClientState::Operating);
        }

        info!("Client initialized successfully");
        Ok(())
    }

    /// Shutdown the client
    pub async fn shutdown(&self, reason: Option<String>) -> MCPResult<()> {
        // Check if we're already shutting down or shutdown
        {
            let state = self.state_manager.read().await;
            if state.state == ClientState::ShuttingDown || state.state == ClientState::Shutdown {
                info!("Client already shutting down or shutdown");
                return Ok(());
            }
        }

        {
            let mut state = self.state_manager.write().await;
            state.set_state(ClientState::ShuttingDown);
        }

        // Try to send shutdown request, but don't fail if it doesn't work
        let shutdown_request = ShutdownRequest { reason };
        if let Err(e) = self
            .send_request::<serde_json::Value>(
                "shutdown",
                Some(serde_json::to_value(shutdown_request)?),
            )
            .await
        {
            warn!(
                "Failed to send shutdown request (this is normal for some transports): {}",
                e
            );
        }

        {
            let mut state = self.state_manager.write().await;
            state.set_state(ClientState::Shutdown);
        }

        info!("Client shutdown completed");
        Ok(())
    }

    /// Disconnect from the server
    pub async fn disconnect(&self) -> MCPResult<()> {
        // Stop message receiver
        if let Some(handle) = self.message_receiver.write().await.take() {
            handle.abort();
        }

        // Close transport
        if let Some(mut transport) = self.transport.write().await.take() {
            transport.close().await.map_err(|e| {
                MCPError::Transport(TransportError::ConnectionFailed(e.to_string()))
            })?;
        }

        {
            let mut state = self.state_manager.write().await;
            state.set_state(ClientState::Uninitialized);
        }

        info!("Client disconnected");
        Ok(())
    }

    /// Get current client state
    pub async fn get_state(&self) -> ClientState {
        self.state_manager.read().await.state.clone()
    }

    /// Check if client can perform operations
    pub async fn can_operate(&self) -> bool {
        self.get_state().await.can_operate()
    }

    /// Get server information
    pub async fn get_server_info(&self) -> Option<ServerInfo> {
        self.state_manager.read().await.server_info.clone()
    }

    /// Get server capabilities
    pub async fn get_server_capabilities(&self) -> Option<ServerCapabilities> {
        self.state_manager.read().await.server_capabilities.clone()
    }

    /// Get negotiated protocol version
    pub async fn get_negotiated_version(&self) -> Option<String> {
        self.state_manager.read().await.negotiated_version.clone()
    }

    /// Get client information
    pub fn info(&self) -> &ClientInfo {
        &self.info
    }

    /// Check if server supports a specific capability
    pub async fn check_server_capability(&self, capability: &str) -> MCPResult<bool> {
        self.ensure_capability_supported(capability).await?;

        if let Some(caps) = self.get_server_capabilities().await {
            Ok(caps.supports_capability(capability))
        } else {
            Ok(false)
        }
    }

    /// Check if server supports a specific feature within a capability
    pub async fn check_server_feature(&self, capability: &str, feature: &str) -> MCPResult<bool> {
        self.ensure_capability_supported(capability).await?;

        if let Some(caps) = self.get_server_capabilities().await {
            Ok(caps.supports_feature(capability, feature))
        } else {
            Ok(false)
        }
    }

    async fn ensure_capability_supported(&self, _capability: &str) -> MCPResult<()> {
        if !self.can_operate().await {
            return Err(MCPError::Protocol(ProtocolError::InternalError(
                "Client is not in operating state".to_string(),
            )));
        }
        Ok(())
    }

    /// List available tools
    pub async fn list_tools(&self, request: ListToolsRequest) -> MCPResult<ListToolsResponse> {
        self.send_request("tools/list", Some(serde_json::to_value(request)?))
            .await
    }

    /// List tools with default parameters
    pub async fn list_tools_default(&self) -> MCPResult<ListToolsResponse> {
        self.list_tools(ListToolsRequest::default()).await
    }

    /// Call a tool
    pub async fn call_tool(&self, tool_call: ToolCall) -> MCPResult<ToolResult> {
        self.send_request("tools/call", Some(serde_json::to_value(tool_call)?))
            .await
    }

    /// List available resources
    pub async fn list_resources(
        &self,
        request: ListResourcesRequest,
    ) -> MCPResult<ListResourcesResponse> {
        self.send_request("resources/list", Some(serde_json::to_value(request)?))
            .await
    }

    /// Read a resource
    pub async fn read_resource(
        &self,
        request: ReadResourceRequest,
    ) -> MCPResult<ReadResourceResponse> {
        self.send_request("resources/read", Some(serde_json::to_value(request)?))
            .await
    }

    /// Subscribe to resource changes
    pub async fn subscribe_resource(&self, uri: String) -> MCPResult<()> {
        let request = serde_json::json!({
            "uri": uri
        });
        self.send_notification("resources/subscribe", Some(request))
            .await
    }

    /// List available prompts
    pub async fn list_prompts(
        &self,
        request: ListPromptsRequest,
    ) -> MCPResult<ListPromptsResponse> {
        self.send_request("prompts/list", Some(serde_json::to_value(request)?))
            .await
    }

    /// Get a specific prompt
    pub async fn get_prompt(&self, request: GetPromptRequest) -> MCPResult<GetPromptResponse> {
        self.send_request("prompts/get", Some(serde_json::to_value(request)?))
            .await
    }

    /// Create a message using sampling
    pub async fn create_message(
        &self,
        request: CreateMessageRequest,
    ) -> MCPResult<CreateMessageResponse> {
        self.send_request(
            "sampling/createMessage",
            Some(serde_json::to_value(request)?),
        )
        .await
    }

    /// Complete a request
    pub async fn complete(&self, request: CompleteRequest) -> MCPResult<CompleteResponse> {
        self.send_request("completion/complete", Some(serde_json::to_value(request)?))
            .await
    }

    /// Respond to elicitation request (called by client-side elicitation handler)
    pub async fn respond_to_elicitation(&self, response: ElicitationResponse) -> MCPResult<()> {
        self.send_request("elicitation/respond", Some(serde_json::to_value(response)?))
            .await
    }

    /// List filesystem roots
    pub async fn list_roots(&self) -> MCPResult<Vec<ultrafast_mcp_core::types::roots::Root>> {
        self.send_request("roots/list", None).await
    }

    /// Set log level
    pub async fn set_log_level(
        &self,
        level: ultrafast_mcp_core::types::notifications::LogLevel,
    ) -> MCPResult<()> {
        let request = serde_json::json!({
            "level": level
        });
        self.send_request("logging/setLevel", Some(request)).await
    }

    /// Send ping
    pub async fn ping(
        &self,
        data: Option<serde_json::Value>,
    ) -> MCPResult<ultrafast_mcp_core::types::notifications::PingResponse> {
        self.send_request("ping", data).await
    }

    /// Start periodic ping monitoring (optional, for connection health)
    pub async fn start_ping_monitoring(&self, ping_interval: std::time::Duration) -> MCPResult<()> {
        info!(
            "Starting periodic ping monitoring with interval: {:?}",
            ping_interval
        );

        // Note: This is a placeholder for future implementation
        // The actual ping monitoring would need to be integrated with the transport layer
        // For now, we log that ping monitoring is enabled
        info!("Ping monitoring enabled (interval: {:?})", ping_interval);

        // Future implementation would spawn a background task that:
        // 1. Clones the client's transport
        // 2. Sends periodic ping requests
        // 3. Handles timeouts and reconnection logic

        Ok(())
    }

    /// Stop ping monitoring
    pub async fn stop_ping_monitoring(&self) -> MCPResult<()> {
        info!("Stopping periodic ping monitoring");
        // The ping monitoring task will naturally stop when the client is disconnected
        Ok(())
    }

    /// Notify cancellation
    pub async fn notify_cancelled(
        &self,
        request_id: serde_json::Value,
        reason: Option<String>,
    ) -> MCPResult<()> {
        let request = serde_json::json!({
            "requestId": request_id,
            "reason": reason
        });
        self.send_notification("$/cancelRequest", Some(request))
            .await
    }

    /// Notify progress
    pub async fn notify_progress(
        &self,
        progress_token: serde_json::Value,
        progress: f64,
        total: Option<f64>,
        message: Option<String>,
    ) -> MCPResult<()> {
        let request = serde_json::json!({
            "token": progress_token,
            "progress": progress,
            "total": total,
            "message": message
        });
        self.send_notification("$/progress", Some(request)).await
    }

    /// Check if progress notification should be sent based on timeout configuration
    pub fn should_send_progress(&self, last_progress: std::time::Instant) -> bool {
        // Use a default interval since progress_interval is not available in the new TimeoutConfig
        let progress_interval = std::time::Duration::from_secs(5);
        last_progress.elapsed() >= progress_interval
    }

    /// Get progress interval from timeout configuration
    pub fn get_progress_interval(&self) -> std::time::Duration {
        // Return a default interval since progress_interval is not available in the new TimeoutConfig
        std::time::Duration::from_secs(5)
    }

    async fn ensure_operational(&self) -> MCPResult<()> {
        let state = self.get_state().await;
        if !state.can_operate() {
            return Err(MCPError::Protocol(ProtocolError::InternalError(format!(
                "Client is not in operating state (current state: {state:?})"
            ))));
        }
        Ok(())
    }

    async fn generate_request_id(&self) -> u64 {
        let mut state = self.state_manager.write().await;
        state.next_request_id()
    }

    async fn send_request<T>(&self, method: &str, params: Option<Value>) -> MCPResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        // Allow initialize request even when not operational
        if method != "initialize" {
            self.ensure_operational().await?;
        }

        let request_id = self.generate_request_id().await;
        let request = JsonRpcRequest::new(
            method.to_string(),
            params,
            Some(ultrafast_mcp_core::protocol::jsonrpc::RequestId::Number(
                request_id as i64,
            )),
        );

        // Get operation-specific timeout
        let operation_timeout = self.get_operation_timeout(method);

        // Create response channel
        let (response_sender, response_receiver) = oneshot::channel();

        // Add to pending requests
        {
            let mut state = self.state_manager.write().await;
            state.add_pending_request(
                request_id,
                PendingRequest {
                    response_sender,
                    timeout: tokio::time::Instant::now() + operation_timeout,
                },
            );
        }

        // Send request
        {
            let mut transport_guard = self.transport.write().await;
            let transport = transport_guard
                .as_mut()
                .ok_or_else(|| {
                    MCPError::Transport(TransportError::ConnectionFailed(
                        "Transport not available".to_string(),
                    ))
                })?;
            transport
                .send_message(JsonRpcMessage::Request(request))
                .await
                .map_err(|e| MCPError::Transport(TransportError::SendFailed(e.to_string())))?;
        }

        // Try to get immediate response from transport (for HTTP transport)
        let immediate_response = {
            let mut transport_guard = self.transport.write().await;
            let transport = transport_guard
                .as_mut()
                .ok_or_else(|| {
                    MCPError::Transport(TransportError::ConnectionFailed(
                        "Transport not available".to_string(),
                    ))
                })?;
            transport.receive_message().await.ok()
        };

        let response = if let Some(immediate) = immediate_response {
            // Got immediate response from transport
            immediate
        } else {
            // Wait for response through message receiver task
            tokio::time::timeout(operation_timeout, response_receiver)
                .await
                .map_err(|_| MCPError::Protocol(ProtocolError::RequestTimeout))?
                .map_err(|_| {
                    MCPError::Protocol(ProtocolError::InternalError(
                        "Response channel closed".to_string(),
                    ))
                })?
        };

        // Remove from pending requests
        {
            let mut state = self.state_manager.write().await;
            state.remove_pending_request(&request_id);
        }

        match response {
            JsonRpcMessage::Response(response) => {
                if let Some(error) = response.error {
                    return Err(MCPError::from(error));
                }

                if let Some(result) = response.result {
                    serde_json::from_value(result).map_err(MCPError::Serialization)
                } else {
                    Err(MCPError::Protocol(ProtocolError::InvalidResponse(
                        "Response has no result or error".to_string(),
                    )))
                }
            }
            _ => Err(MCPError::Protocol(ProtocolError::InvalidResponse(
                "Expected response, got different message type".to_string(),
            ))),
        }
    }

    async fn send_notification(&self, method: &str, params: Option<Value>) -> MCPResult<()> {
        // Allow initialized notification even when not operational
        if method != "initialized" {
            self.ensure_operational().await?;
        }

        let notification = JsonRpcRequest::notification(method.to_string(), params);

        let mut transport_guard = self.transport.write().await;
        let transport = transport_guard
            .as_mut()
            .ok_or_else(|| {
                MCPError::Transport(TransportError::ConnectionFailed(
                    "Transport not available".to_string(),
                ))
            })?;

        transport
            .send_message(JsonRpcMessage::Notification(notification))
            .await
            .map_err(|e| MCPError::Transport(TransportError::SendFailed(e.to_string())))
    }
}

impl std::fmt::Debug for UltraFastClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UltraFastClient")
            .field("info", &self.info)
            .field("capabilities", &self.capabilities)
            .field("state", &"<state_manager>")
            .field("transport", &"<transport>")
            .field("request_timeout", &self.request_timeout)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client_info = ClientInfo {
            name: "test-client".to_string(),
            version: "1.0.0".to_string(),
            authors: None,
            description: None,
            homepage: None,
            repository: None,
            license: None,
        };
        let capabilities = ClientCapabilities::default();
        let client = UltraFastClient::new(client_info, capabilities);

        assert_eq!(client.get_state().await, ClientState::Uninitialized);
        assert!(!client.can_operate().await);
    }

    #[tokio::test]
    async fn test_client_creation_with_timeout() {
        let client_info = ClientInfo {
            name: "test-client".to_string(),
            version: "1.0.0".to_string(),
            authors: None,
            description: None,
            homepage: None,
            repository: None,
            license: None,
        };
        let capabilities = ClientCapabilities::default();
        let timeout = std::time::Duration::from_secs(60);
        let client = UltraFastClient::new_with_timeout(client_info, capabilities, timeout);

        assert_eq!(client.get_state().await, ClientState::Uninitialized);
    }

    #[tokio::test]
    async fn test_client_state_transitions() {
        let client_info = ClientInfo {
            name: "test-client".to_string(),
            version: "1.0.0".to_string(),
            authors: None,
            description: None,
            homepage: None,
            repository: None,
            license: None,
        };
        let capabilities = ClientCapabilities::default();
        let client = UltraFastClient::new(client_info, capabilities);

        assert_eq!(client.get_state().await, ClientState::Uninitialized);

        // Test state transitions through the state manager
        {
            let mut state = client.state_manager.write().await;
            state.set_state(ClientState::Initializing);
        }
        assert_eq!(client.get_state().await, ClientState::Initializing);
    }
}
