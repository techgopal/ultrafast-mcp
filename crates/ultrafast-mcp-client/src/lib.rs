//! MCP client implementation for ULTRAFAST MCP
//!
//! This crate provides a high-level client implementation for the Model Context Protocol.
//!
//! ## Phase 3 Features:
//! - Advanced sampling for LLM integration
//! - Roots management for filesystem security
//! - Elicitation for user input collection
//! - Resource subscriptions with real-time notifications

use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::{oneshot, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use ultrafast_mcp_core::{
    error::{MCPError, McpError, McpResult},
    protocol::{
        capabilities::{ClientCapabilities, ServerCapabilities},
        jsonrpc::{JsonRpcError, JsonRpcMessage, JsonRpcRequest, JsonRpcResponse, RequestId},
        lifecycle::{InitializeRequest, InitializeResponse},
    },
    types::{
        client::ClientInfo,
        elicitation::{ElicitationRequest, ElicitationResponse},
        notifications::{CancelledNotification, PingRequest, PingResponse},
        prompts::{GetPromptRequest, GetPromptResponse, Prompt},
        resources::{ReadResourceRequest, ReadResourceResponse, Resource},
        roots::{ListRootsRequest, ListRootsResponse},
        sampling::{
            CreateMessageRequest, CreateMessageResponse, SamplingRequest, SamplingResponse,
        },
        tools::{Tool, ToolCall, ToolResult},
    },
    utils::{CancellationManager, PingManager},
};
use ultrafast_mcp_transport::{create_transport, Transport, TransportConfig};

// Phase 3: Advanced client types and configurations

/// Client configuration for advanced features
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub name: String,
    pub version: String,
    pub timeout_ms: u64,
    pub max_retries: u32,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            name: "ultrafast-mcp-client".to_string(),
            version: "0.1.0".to_string(),
            timeout_ms: 30000,
            max_retries: 3,
        }
    }
}

/// Progress update information for Phase 3 clients
#[derive(Debug, Clone)]
pub struct ProgressUpdate {
    pub token: String,
    pub progress: f64,
    pub total: Option<f64>,
    pub message: Option<String>,
}

/// Sampling handler type for Phase 3 LLM completions
pub type SamplingHandlerFn = Arc<
    dyn Fn(
            SamplingRequest,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = McpResult<SamplingResponse>> + Send>,
        > + Send
        + Sync,
>;

/// Elicitation handler type for Phase 3 user input
pub type ElicitationHandlerFn = Arc<
    dyn Fn(
            ElicitationRequest,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = McpResult<ElicitationResponse>> + Send>,
        > + Send
        + Sync,
>;

// Phase 3: Advanced client feature traits

/// Handler for server-initiated sampling requests (LLM integration)
#[async_trait::async_trait]
pub trait SamplingHandler: Send + Sync {
    async fn handle_sampling(&self, request: SamplingRequest) -> McpResult<SamplingResponse>;
}

/// Handler for resource change notifications
#[async_trait::async_trait]
pub trait ResourceChangeHandler: Send + Sync {
    async fn handle_change(&self, uri: String, content: serde_json::Value);
}

/// Handler for server-initiated elicitation requests
#[async_trait::async_trait]
pub trait ElicitationHandler: Send + Sync {
    async fn handle_elicitation(
        &self,
        request: ElicitationRequest,
    ) -> McpResult<ElicitationResponse>;
}

/// Handler for server-initiated roots requests
#[async_trait::async_trait]
pub trait RootsHandler: Send + Sync {
    async fn handle_roots(&self, request: ListRootsRequest) -> McpResult<ListRootsResponse>;
}

/// Pending request information
struct PendingRequest {
    sender: oneshot::Sender<McpResult<serde_json::Value>>,
    // timeout field removed as it was unused - timeout is handled via tokio::time::timeout
}

/// MCP Client state
#[derive(Debug, Clone)]
pub enum ClientState {
    Disconnected,
    Connecting,
    Connected,
    Initialized,
}

/// MCP Client implementation
pub struct UltraFastClient {
    info: ClientInfo,
    capabilities: ClientCapabilities,
    state: Arc<RwLock<ClientState>>,
    server_info: Arc<RwLock<Option<ultrafast_mcp_core::types::server::ServerInfo>>>,
    server_capabilities: Arc<RwLock<Option<ServerCapabilities>>>,
    transport: Arc<RwLock<Option<Box<dyn Transport>>>>,
    pending_requests: Arc<RwLock<HashMap<serde_json::Value, PendingRequest>>>,
    request_timeout: Duration,

    // Phase 3: Advanced client features
    sampling_handler: Arc<RwLock<Option<Arc<dyn SamplingHandler>>>>,
    // roots field removed as it was unused - roots should be managed by the server
    resource_subscriptions: Arc<RwLock<HashMap<String, Arc<dyn ResourceChangeHandler>>>>,
    elicitation_handler: Arc<RwLock<Option<Arc<dyn ElicitationHandler>>>>,
    roots_handler: Arc<RwLock<Option<Arc<dyn RootsHandler>>>>,

    // MCP 2025-06-18 utilities
    cancellation_manager: Arc<CancellationManager>,
    ping_manager: Arc<PingManager>,
}

impl std::fmt::Debug for UltraFastClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UltraFastClient")
            .field("info", &self.info)
            .field("capabilities", &self.capabilities)
            .finish()
    }
}

impl UltraFastClient {
    /// Create a new client with the given info and capabilities
    pub fn new(info: ClientInfo, capabilities: ClientCapabilities) -> Self {
        Self {
            info,
            capabilities,
            state: Arc::new(RwLock::new(ClientState::Disconnected)),
            server_info: Arc::new(RwLock::new(None)),
            server_capabilities: Arc::new(RwLock::new(None)),
            transport: Arc::new(RwLock::new(None)),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            request_timeout: Duration::from_secs(10), // Reduced from 30 to 10 seconds

            // Phase 3: Initialize advanced client features
            sampling_handler: Arc::new(RwLock::new(None)),
            // roots initialization removed as field was unused
            resource_subscriptions: Arc::new(RwLock::new(HashMap::new())),
            elicitation_handler: Arc::new(RwLock::new(None)),
            roots_handler: Arc::new(RwLock::new(None)),

            // MCP 2025-06-18 utilities
            cancellation_manager: Arc::new(CancellationManager::new()),
            ping_manager: Arc::new(PingManager::default()),
        }
    }

    /// Set request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// Connect to server using STDIO transport
    pub async fn connect_stdio(&self) -> McpResult<()> {
        let transport = create_transport(TransportConfig::Stdio)
            .await
            .map_err(|e| MCPError::internal_error(format!("Transport creation failed: {}", e)))?;
        self.connect_with_transport(transport).await
    }

    /// Connect to server using Streamable HTTP transport (recommended)
    ///
    /// This is the preferred method for high-performance HTTP communication.
    /// Streamable HTTP provides 10x better performance than HTTP+SSE under load.
    pub async fn connect_streamable_http(&self, url: &str) -> McpResult<()> {
        let transport_config = TransportConfig::Streamable {
            base_url: url.to_string(),
            auth_token: None,
            session_id: None,
        };

        let transport = create_transport(transport_config)
            .await
            .map_err(|e| MCPError::internal_error(format!("Transport creation failed: {}", e)))?;
        self.connect_with_transport(transport).await
    }

    /// Connect to server using HTTP+SSE transport (legacy compatibility)
    ///
    /// ⚠️ **DEPRECATED**: SSE transport is deprecated per MCP 2025-03-26 specification.
    /// Use `connect_streamable_http()` instead for better proxy compatibility and performance.
    ///
    /// This method provides backward compatibility with HTTP+SSE from MCP 2024-11-05.
    #[deprecated(
        since = "0.1.0",
        note = "Use connect_streamable_http() instead. SSE transport is deprecated per MCP 2025-03-26 specification."
    )]
    pub async fn connect_http_sse(&self, url: &str) -> McpResult<()> {
        let transport_config = TransportConfig::HttpSse {
            base_url: url.to_string(),
            auth_token: None,
            session_id: None,
        };

        let transport = create_transport(transport_config)
            .await
            .map_err(|e| MCPError::internal_error(format!("Transport creation failed: {}", e)))?;
        self.connect_with_transport(transport).await
    }

    /// Connect to server using the configured transport
    ///
    /// This is a generic connect method that uses whatever transport has been configured.
    /// For most use cases, prefer the specific connect methods like `connect_streamable_http`.
    pub async fn connect(&self) -> McpResult<()> {
        // Check if transport is already configured
        let transport_guard = self.transport.read().await;
        if let Some(ref _transport) = *transport_guard {
            // Transport already configured, just initialize
            drop(transport_guard);
            self.initialize().await?;
            *self.state.write().await = ClientState::Initialized;
            info!("Successfully connected and initialized");
            Ok(())
        } else {
            // No transport configured, use stdio as default
            drop(transport_guard);
            self.connect_stdio().await
        }
    }

    /// Configure transport for this client
    ///
    /// Use this method to set up a custom transport before calling `connect()`.
    pub async fn with_transport(&mut self, transport: Box<dyn Transport>) -> &mut Self {
        *self.transport.write().await = Some(transport);
        self
    }

    /// Connect to server using custom transport
    pub async fn connect_with_transport(&self, transport: Box<dyn Transport>) -> McpResult<()> {
        info!("Connecting to MCP server");

        *self.state.write().await = ClientState::Connecting;
        *self.transport.write().await = Some(transport);

        // Start message handling task BEFORE initialization for all transports
        let _client = Arc::new(RwLock::new(self));
        self.start_message_handler().await?;

        // Give the message handler a moment to start up
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Initialize connection using the unified request/response system
        self.initialize().await?;

        *self.state.write().await = ClientState::Initialized;
        info!("Successfully connected and initialized");

        Ok(())
    }

    /// Initialize the connection
    async fn initialize(&self) -> McpResult<()> {
        debug!("Initializing connection");

        let request = InitializeRequest {
            protocol_version: "2025-06-18".to_string(),
            capabilities: self.capabilities.clone(),
            client_info: self.info.clone(),
        };

        // Use the unified request/response system for all transports
        let response: InitializeResponse = self
            .send_request("initialize", Some(serde_json::to_value(request)?))
            .await?;

        *self.server_info.write().await = Some(response.server_info);
        *self.server_capabilities.write().await = Some(response.capabilities);

        // Send initialized notification
        self.send_notification("initialized", None).await?;

        debug!("Initialization complete");
        Ok(())
    }

    /// Start the message handler task
    async fn start_message_handler(&self) -> McpResult<()> {
        let transport = self.transport.clone();
        let pending_requests = self.pending_requests.clone();
        let sampling_handler = self.sampling_handler.clone();
        let elicitation_handler = self.elicitation_handler.clone();
        let roots_handler = self.roots_handler.clone();
        let resource_subscriptions = self.resource_subscriptions.clone();
        let _cancellation_manager = self.cancellation_manager.clone();
        let _ping_manager = self.ping_manager.clone();

        tokio::spawn(async move {
            info!("Message handler task started");
            loop {
                let message = {
                    let mut transport_guard = transport.write().await;
                    if let Some(ref mut transport) = transport_guard.as_mut() {
                        debug!("Message handler calling transport.receive_message()");
                        match transport.receive_message().await {
                            Ok(msg) => {
                                debug!("Message handler received message: {:?}", msg);
                                msg
                            }
                            Err(ultrafast_mcp_transport::TransportError::ConnectionClosed) => {
                                debug!("Transport closed");
                                break;
                            }
                            Err(ultrafast_mcp_transport::TransportError::ProtocolError {
                                message,
                            }) if message.contains("No messages available") => {
                                // This is expected behavior for polling transports - continue polling
                                debug!("No messages available, continuing to poll");
                                continue;
                            }
                            Err(e) => {
                                error!("Error receiving message: {}", e);
                                break;
                            }
                        }
                    } else {
                        debug!("No transport available, breaking message handler loop");
                        break;
                    }
                };

                let message_value = match serde_json::to_value(&message) {
                    Ok(value) => value,
                    Err(e) => {
                        error!("Failed to serialize incoming message to JSON value: {}", e);
                        continue;
                    }
                };

                let params = MessageHandlerParams {
                    pending_requests: &pending_requests,
                    sampling_handler: &sampling_handler,
                    roots_handler: &roots_handler,
                    elicitation_handler: &elicitation_handler,
                    resource_subscriptions: &resource_subscriptions,
                    transport: &transport,
                };

                if let Err(e) = Self::handle_incoming_message(message_value, params).await {
                    error!("Error handling incoming message: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Handle incoming messages from the server
    async fn handle_incoming_message(
        message: serde_json::Value,
        handler_params: MessageHandlerParams<'_>,
    ) -> McpResult<()> {
        let json_message: JsonRpcMessage = serde_json::from_value(message)
            .map_err(|e| McpError::serialization_error(e.to_string()))?;

        match json_message {
            JsonRpcMessage::Response(response) => {
                let mut pending = handler_params.pending_requests.write().await;
                if let Some(ref id) = response.id {
                    let id_value = serde_json::to_value(id)?;
                    if let Some(pending_request) = pending.remove(&id_value) {
                        let result = if let Some(error) = response.error {
                            Err(McpError::from(error))
                        } else {
                            Ok(response.result.unwrap_or(serde_json::Value::Null))
                        };

                        if pending_request.sender.send(result).is_err() {
                            warn!("Failed to send response to pending request");
                        }
                    }
                }
            }
            JsonRpcMessage::Request(request) => {
                debug!("Received server request: {}", request.method);

                // Handle server-initiated requests
                let response_result = match request.method.as_str() {
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

                        // Create ping response
                        let ping_response = PingResponse::new()
                            .with_data(ping_request.data.unwrap_or(serde_json::json!({})));

                        Some(Ok(serde_json::to_value(ping_response)?))
                    }
                    "sampling/createMessage" => {
                        if let Some(params) = request.params {
                            match serde_json::from_value::<SamplingRequest>(params) {
                                Ok(sampling_request) => {
                                    let handler_guard =
                                        handler_params.sampling_handler.read().await;
                                    if let Some(ref handler) = *handler_guard {
                                        match handler.handle_sampling(sampling_request).await {
                                            Ok(sampling_response) => {
                                                info!("Successfully handled sampling request");
                                                Some(Ok(serde_json::to_value(sampling_response)?))
                                            }
                                            Err(e) => {
                                                error!("Error handling sampling request: {}", e);
                                                Some(Err(e))
                                            }
                                        }
                                    } else {
                                        warn!("No sampling handler configured");
                                        Some(Err(McpError::internal_error(
                                            "No sampling handler configured".to_string(),
                                        )))
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to deserialize sampling request: {}", e);
                                    Some(Err(McpError::serialization_error(e.to_string())))
                                }
                            }
                        } else {
                            error!("Sampling request missing parameters");
                            Some(Err(McpError::invalid_request(
                                "Missing parameters".to_string(),
                            )))
                        }
                    }
                    "elicitation/create" => {
                        if let Some(params) = request.params {
                            match serde_json::from_value::<ElicitationRequest>(params) {
                                Ok(elicitation_request) => {
                                    let handler_guard =
                                        handler_params.elicitation_handler.read().await;
                                    if let Some(ref handler) = *handler_guard {
                                        match handler.handle_elicitation(elicitation_request).await
                                        {
                                            Ok(elicitation_response) => {
                                                info!("Successfully handled elicitation request");
                                                Some(Ok(serde_json::to_value(
                                                    elicitation_response,
                                                )?))
                                            }
                                            Err(e) => {
                                                error!("Error handling elicitation request: {}", e);
                                                Some(Err(e))
                                            }
                                        }
                                    } else {
                                        warn!("No elicitation handler configured");
                                        Some(Err(McpError::internal_error(
                                            "No elicitation handler configured".to_string(),
                                        )))
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to deserialize elicitation request: {}", e);
                                    Some(Err(McpError::serialization_error(e.to_string())))
                                }
                            }
                        } else {
                            error!("Elicitation request missing parameters");
                            Some(Err(McpError::invalid_request(
                                "Missing parameters".to_string(),
                            )))
                        }
                    }
                    "roots/list" => {
                        if let Some(params) = request.params {
                            match serde_json::from_value::<ListRootsRequest>(params) {
                                Ok(roots_request) => {
                                    let handler_guard = handler_params.roots_handler.read().await;
                                    if let Some(ref handler) = *handler_guard {
                                        match handler.handle_roots(roots_request).await {
                                            Ok(roots_response) => {
                                                info!("Successfully handled roots request");
                                                Some(Ok(serde_json::to_value(roots_response)?))
                                            }
                                            Err(e) => {
                                                error!("Error handling roots request: {}", e);
                                                Some(Err(e))
                                            }
                                        }
                                    } else {
                                        warn!("No roots handler configured");
                                        Some(Err(McpError::internal_error(
                                            "No roots handler configured".to_string(),
                                        )))
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to deserialize roots request: {}", e);
                                    Some(Err(McpError::serialization_error(e.to_string())))
                                }
                            }
                        } else {
                            error!("Roots request missing parameters");
                            Some(Err(McpError::invalid_request(
                                "Missing parameters".to_string(),
                            )))
                        }
                    }
                    _ => {
                        debug!("Unknown server request method: {}", request.method);
                        None
                    }
                };

                // Send response back to server if this was a request
                if let Some(id) = request.id {
                    let response = match response_result {
                        Some(Ok(result)) => JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: Some(result),
                            error: None,
                            id: Some(id),
                            meta: std::collections::HashMap::new(),
                        },
                        Some(Err(error)) => JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32603, // Internal error
                                message: error.to_string(),
                                data: None,
                            }),
                            id: Some(id),
                            meta: std::collections::HashMap::new(),
                        },
                        None => JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32601, // Method not found
                                message: "Method not found".to_string(),
                                data: None,
                            }),
                            id: Some(id),
                            meta: std::collections::HashMap::new(),
                        },
                    };

                    // Send response back
                    let mut transport_guard = handler_params.transport.write().await;
                    if let Some(ref mut transport) = transport_guard.as_mut() {
                        let message = JsonRpcMessage::Response(response);
                        if let Err(e) = transport
                            .send_message(serde_json::from_value(serde_json::to_value(message)?)?)
                            .await
                        {
                            error!("Failed to send response: {}", e);
                        }
                    }
                }
            }
            JsonRpcMessage::Notification(notification) => {
                debug!("Received server notification: {}", notification.method);

                // Handle server notifications
                match notification.method.as_str() {
                    "notifications/resources/updated" => {
                        if let Some(params) = notification.params {
                            if let (Some(uri), Some(content)) =
                                (params.get("uri"), params.get("content"))
                            {
                                if let Ok(uri_str) = serde_json::from_value::<String>(uri.clone()) {
                                    let subscriptions =
                                        handler_params.resource_subscriptions.read().await;
                                    if let Some(handler) = subscriptions.get(&uri_str) {
                                        handler.handle_change(uri_str, content.clone()).await;
                                        info!("Successfully handled resource update notification");
                                    } else {
                                        debug!("No subscription handler for resource: {}", uri_str);
                                    }
                                } else {
                                    error!("Invalid URI in resource update notification");
                                }
                            } else {
                                error!("Resource update notification missing uri or content");
                            }
                        } else {
                            error!("Resource update notification missing parameters");
                        }
                    }
                    "notifications/cancelled" => {
                        debug!("Cancellation notification received");
                        if let Some(params) = notification.params {
                            match serde_json::from_value::<CancelledNotification>(params) {
                                Ok(cancel_notification) => {
                                    debug!(
                                        "Request {} was cancelled: {:?}",
                                        cancel_notification.request_id, cancel_notification.reason
                                    );

                                    // Cancel any pending requests with this ID
                                    let mut pending = handler_params.pending_requests.write().await;
                                    if let Some(pending_request) =
                                        pending.remove(&cancel_notification.request_id)
                                    {
                                        let cancel_error = McpError::internal_error(
                                            cancel_notification
                                                .reason
                                                .unwrap_or_else(|| "Request cancelled".to_string()),
                                        );

                                        if pending_request.sender.send(Err(cancel_error)).is_err() {
                                            warn!("Failed to send cancellation to pending request");
                                        }
                                    }
                                }
                                Err(e) => {
                                    warn!("Invalid cancellation notification: {}", e);
                                }
                            }
                        }
                    }
                    "notifications/progress" => {
                        debug!("Progress notification received");
                    }
                    _ => {
                        debug!("Unknown notification method: {}", notification.method);
                    }
                }
            }
        }

        Ok(())
    }

    /// Send a JSON-RPC request and wait for response
    async fn send_request<T>(&self, method: &str, params: Option<serde_json::Value>) -> McpResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let request_id = RequestId::String(Uuid::new_v4().to_string());
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: Some(request_id.clone()),
            meta: std::collections::HashMap::new(),
        };

        let (sender, receiver) = oneshot::channel();
        // Register pending request
        {
            let mut pending = self.pending_requests.write().await;
            pending.insert(
                serde_json::to_value(&request_id)?,
                PendingRequest { sender },
            );
        }

        // Send request
        {
            let mut transport_guard = self.transport.write().await;
            if let Some(ref mut transport) = transport_guard.as_mut() {
                let message = JsonRpcMessage::Request(request);
                transport
                    .send_message(serde_json::from_value(serde_json::to_value(message)?)?)
                    .await
                    .map_err(|e| MCPError::internal_error(format!("Send failed: {}", e)))?;
            } else {
                return Err(McpError::transport_error("Not connected".to_string()));
            }
        }

        // Wait for response
        let result = tokio::time::timeout(self.request_timeout, receiver)
            .await
            .map_err(|_| McpError::request_timeout())?
            .map_err(|_| McpError::internal_error("Request cancelled".to_string()))??;

        serde_json::from_value(result).map_err(|e| McpError::serialization_error(e.to_string()))
    }

    /// Send a JSON-RPC notification
    async fn send_notification(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> McpResult<()> {
        let notification = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: None, // Notifications don't have IDs
            meta: std::collections::HashMap::new(),
        };

        let mut transport_guard = self.transport.write().await;
        if let Some(ref mut transport) = transport_guard.as_mut() {
            let message = JsonRpcMessage::Notification(notification);
            transport
                .send_message(serde_json::from_value(serde_json::to_value(message)?)?)
                .await
                .map_err(|e| MCPError::internal_error(format!("Send failed: {}", e)))?;
        } else {
            return Err(McpError::transport_error("Not connected".to_string()));
        }

        Ok(())
    }

    /// List available tools
    pub async fn list_tools(&self) -> McpResult<Vec<Tool>> {
        let response: serde_json::Value = self.send_request("tools/list", None).await?;
        let tools = response["tools"]
            .as_array()
            .ok_or_else(|| McpError::invalid_response("Missing tools array".to_string()))?;

        tools
            .iter()
            .map(|t| serde_json::from_value(t.clone()))
            .collect::<Result<Vec<Tool>, _>>()
            .map_err(|e| McpError::serialization_error(e.to_string()))
    }

    /// Call a tool
    pub async fn call_tool(&self, call: ToolCall) -> McpResult<ToolResult> {
        let params = serde_json::to_value(call)?;
        self.send_request("tools/call", Some(params)).await
    }

    /// List available resources
    pub async fn list_resources(&self) -> McpResult<Vec<Resource>> {
        let response: serde_json::Value = self.send_request("resources/list", None).await?;
        let resources = response["resources"]
            .as_array()
            .ok_or_else(|| McpError::invalid_response("Missing resources array".to_string()))?;

        resources
            .iter()
            .map(|r| serde_json::from_value(r.clone()))
            .collect::<Result<Vec<Resource>, _>>()
            .map_err(|e| McpError::serialization_error(e.to_string()))
    }

    /// Read a resource
    pub async fn read_resource(
        &self,
        request: ReadResourceRequest,
    ) -> McpResult<ReadResourceResponse> {
        let params = serde_json::to_value(request)?;
        self.send_request("resources/read", Some(params)).await
    }

    /// List available prompts
    pub async fn list_prompts(&self) -> McpResult<Vec<Prompt>> {
        let response: serde_json::Value = self.send_request("prompts/list", None).await?;
        let prompts = response["prompts"]
            .as_array()
            .ok_or_else(|| McpError::invalid_response("Missing prompts array".to_string()))?;

        prompts
            .iter()
            .map(|p| serde_json::from_value(p.clone()))
            .collect::<Result<Vec<Prompt>, _>>()
            .map_err(|e| McpError::serialization_error(e.to_string()))
    }

    /// Get a prompt
    pub async fn get_prompt(&self, request: GetPromptRequest) -> McpResult<GetPromptResponse> {
        let params = serde_json::to_value(request)?;
        self.send_request("prompts/get", Some(params)).await
    }

    /// Create a message (sampling)
    pub async fn create_message(
        &self,
        request: CreateMessageRequest,
    ) -> McpResult<CreateMessageResponse> {
        let params = serde_json::to_value(request)?;
        self.send_request("sampling/createMessage", Some(params))
            .await
    }

    /// Disconnect from server
    pub async fn disconnect(&self) -> McpResult<()> {
        info!("Disconnecting from server");

        let mut transport_guard = self.transport.write().await;
        if let Some(ref mut transport) = transport_guard.as_mut() {
            transport
                .close()
                .await
                .map_err(|e| MCPError::internal_error(format!("Close failed: {}", e)))?;
        }
        *transport_guard = None;

        *self.state.write().await = ClientState::Disconnected;

        Ok(())
    }

    // Phase 3: Advanced Client Features

    /// Set sampling handler for server-initiated LLM completions
    pub async fn set_sampling_handler(&self, handler: Arc<dyn SamplingHandler>) -> McpResult<()> {
        *self.sampling_handler.write().await = Some(handler);
        Ok(())
    }

    /// Set roots handler for server-initiated roots requests
    pub async fn set_roots_handler(&self, handler: Arc<dyn RootsHandler>) -> McpResult<()> {
        *self.roots_handler.write().await = Some(handler);
        Ok(())
    }

    /// List available filesystem roots
    pub async fn list_roots(&self) -> McpResult<ListRootsResponse> {
        let request = ListRootsRequest {};
        let params = serde_json::to_value(request)?;
        self.send_request("roots/list", Some(params)).await
    }

    /// Subscribe to resource changes  
    pub async fn subscribe_resource<F>(&self, uri: String, handler: F) -> McpResult<()>
    where
        F: Fn(
                String,
                serde_json::Value,
            ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
            + Send
            + Sync
            + 'static,
    {
        // Store the handler
        let boxed_handler = Arc::new(ResourceSubscriptionHandler {
            callback: Box::new(handler),
        });
        self.resource_subscriptions
            .write()
            .await
            .insert(uri.clone(), boxed_handler);

        // Send subscription request
        let params = serde_json::json!({ "uri": uri });
        let _response: serde_json::Value = self
            .send_request("resources/subscribe", Some(params))
            .await?;

        Ok(())
    }

    /// Unsubscribe from resource changes
    pub async fn unsubscribe_resource(&self, uri: String) -> McpResult<()> {
        self.resource_subscriptions.write().await.remove(&uri);

        let params = serde_json::json!({ "uri": uri });
        let _response: serde_json::Value = self
            .send_request("resources/unsubscribe", Some(params))
            .await?;

        Ok(())
    }

    /// Set elicitation handler for server-initiated user input requests
    pub async fn set_elicitation_handler(
        &self,
        handler: Arc<dyn ElicitationHandler>,
    ) -> McpResult<()> {
        *self.elicitation_handler.write().await = Some(handler);
        Ok(())
    }

    /// Get the cancellation manager
    pub fn cancellation_manager(&self) -> Arc<CancellationManager> {
        self.cancellation_manager.clone()
    }

    /// Get the ping manager
    pub fn ping_manager(&self) -> Arc<PingManager> {
        self.ping_manager.clone()
    }

    /// Send a ping request to the server
    pub async fn ping(&self, data: Option<serde_json::Value>) -> McpResult<PingResponse> {
        let mut request = PingRequest::new();
        if let Some(data) = data {
            request = request.with_data(data);
        }

        let params = serde_json::to_value(request)?;
        let response: PingResponse = self.send_request("ping", Some(params)).await?;
        Ok(response)
    }

    /// Cancel a request
    pub async fn cancel_request(
        &self,
        request_id: serde_json::Value,
        reason: Option<String>,
    ) -> McpResult<()> {
        let mut notification = CancelledNotification::new(request_id.clone());
        if let Some(reason) = reason.clone() {
            notification = notification.with_reason(reason);
        }

        // Send cancellation notification to server
        let json_rpc_notification = JsonRpcRequest::notification(
            "notifications/cancelled".to_string(),
            Some(serde_json::to_value(notification)?),
        );

        let mut transport = self.transport.write().await;
        if let Some(ref mut transport) = *transport {
            transport
                .send_message(JsonRpcMessage::Request(json_rpc_notification))
                .await
                .map_err(|e| {
                    MCPError::internal_error(format!("Failed to send cancellation: {}", e))
                })?;
        }

        // Also cancel locally
        self.cancellation_manager
            .cancel_request(&request_id, reason)
            .await?;

        Ok(())
    }

    /// Check if a request has been cancelled
    pub async fn is_request_cancelled(&self, request_id: &serde_json::Value) -> bool {
        self.cancellation_manager.is_cancelled(request_id).await
    }

    /// Register a request for cancellation tracking
    pub async fn register_request(
        &self,
        request_id: serde_json::Value,
        method: String,
    ) -> McpResult<()> {
        self.cancellation_manager
            .register_request(request_id, method)
            .await
    }
}

/// Type alias for complex callback function type
type ResourceChangeCallback = Box<
    dyn Fn(
            String,
            serde_json::Value,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
        + Send
        + Sync,
>;

/// Wrapper for resource subscription callbacks
pub struct ResourceSubscriptionHandler {
    callback: ResourceChangeCallback,
}

#[async_trait::async_trait]
impl ResourceChangeHandler for ResourceSubscriptionHandler {
    async fn handle_change(&self, uri: String, content: serde_json::Value) {
        (self.callback)(uri, content).await;
    }
}

/// Parameters for handling incoming messages
struct MessageHandlerParams<'a> {
    pending_requests: &'a Arc<RwLock<HashMap<serde_json::Value, PendingRequest>>>,
    sampling_handler: &'a Arc<RwLock<Option<Arc<dyn SamplingHandler>>>>,
    roots_handler: &'a Arc<RwLock<Option<Arc<dyn RootsHandler>>>>,
    elicitation_handler: &'a Arc<RwLock<Option<Arc<dyn ElicitationHandler>>>>,
    resource_subscriptions: &'a Arc<RwLock<HashMap<String, Arc<dyn ResourceChangeHandler>>>>,
    transport: &'a Arc<RwLock<Option<Box<dyn Transport>>>>,
}
