//! UltraFast MCP Client Library
//!
//! A high-performance client implementation for the Model Context Protocol (MCP).

use serde_json::Value;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{RwLock, oneshot};
use tracing::{debug, info, warn, error};
use ultrafast_mcp_core::{
    error::{MCPError, MCPResult},
    protocol::{
        capabilities::{ClientCapabilities, ServerCapabilities},
        jsonrpc::{JsonRpcMessage, JsonRpcRequest, JsonRpcResponse},
        lifecycle::{
            InitializeRequest, InitializeResponse, InitializedNotification, ShutdownRequest,
        },
        version::PROTOCOL_VERSION,
    },
    types::{
        client::ClientInfo,
        completion::{CompleteRequest, CompleteResponse},
        elicitation::{ElicitationRequest, ElicitationResponse},
        prompts::{GetPromptRequest, GetPromptResponse, ListPromptsRequest, ListPromptsResponse},
        resources::{
            ListResourcesRequest, ListResourcesResponse, ReadResourceRequest, ReadResourceResponse,
        },
        sampling::{CreateMessageRequest, CreateMessageResponse},
        server::ServerInfo,
        tools::{ListToolsRequest, ListToolsResponse, ToolCall, ToolResult},
    },
};
use ultrafast_mcp_transport::Transport;

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
    pub fn can_operate(&self) -> bool {
        matches!(self, ClientState::Operating)
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
struct PendingRequest {
    response_sender: oneshot::Sender<JsonRpcMessage>,
    timeout: tokio::time::Instant,
}

/// UltraFast MCP Client
pub struct UltraFastClient {
    info: ClientInfo,
    capabilities: ClientCapabilities,
    state: Arc<RwLock<ClientState>>,
    server_info: Arc<RwLock<Option<ServerInfo>>>,
    server_capabilities: Arc<RwLock<Option<ServerCapabilities>>>,
    negotiated_version: Arc<RwLock<Option<String>>>,
    transport: Arc<RwLock<Option<Box<dyn Transport>>>>,
    request_id_counter: Arc<RwLock<u64>>,
    // NEW: Pending requests map for proper request/response correlation
    pending_requests: Arc<RwLock<HashMap<u64, PendingRequest>>>,
    // NEW: Message receiver task handle
    message_receiver: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    // NEW: Request timeout configuration
    request_timeout: std::time::Duration,
}

impl UltraFastClient {
    /// Create a new MCP client
    pub fn new(info: ClientInfo, capabilities: ClientCapabilities) -> Self {
        Self {
            info,
            capabilities,
            state: Arc::new(RwLock::new(ClientState::Uninitialized)),
            server_info: Arc::new(RwLock::new(None)),
            server_capabilities: Arc::new(RwLock::new(None)),
            negotiated_version: Arc::new(RwLock::new(None)),
            transport: Arc::new(RwLock::new(None)),
            request_id_counter: Arc::new(RwLock::new(1)),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            message_receiver: Arc::new(RwLock::new(None)),
            request_timeout: std::time::Duration::from_secs(30), // Default 30 second timeout
        }
    }

    /// Create a new MCP client with custom timeout
    pub fn new_with_timeout(
        info: ClientInfo, 
        capabilities: ClientCapabilities, 
        timeout: std::time::Duration
    ) -> Self {
        Self {
            info,
            capabilities,
            state: Arc::new(RwLock::new(ClientState::Uninitialized)),
            server_info: Arc::new(RwLock::new(None)),
            server_capabilities: Arc::new(RwLock::new(None)),
            negotiated_version: Arc::new(RwLock::new(None)),
            transport: Arc::new(RwLock::new(None)),
            request_id_counter: Arc::new(RwLock::new(1)),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            message_receiver: Arc::new(RwLock::new(None)),
            request_timeout: timeout,
        }
    }

    /// Set request timeout
    pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.request_timeout = timeout;
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
        let pending_requests = self.pending_requests.clone();
        let _request_timeout = self.request_timeout;

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
                                    if let Ok(id_num) = serde_json::from_value::<u64>(serde_json::to_value(id).unwrap_or_default()) {
                                        let mut pending = pending_requests.write().await;
                                        if let Some(pending_req) = pending.remove(&id_num) {
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
                            JsonRpcMessage::Notification(notification) => {
                                // Handle notification
                                Self::handle_notification_static(notification.clone()).await;
                            }
                            _ => {
                                warn!("Received unexpected message type");
                            }
                        }
                    }
                    Err(e) => {
                        error!("Transport error in message receiver: {}", e);
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

    /// Handle incoming notifications (static method for use in async task)
    async fn handle_notification_static(notification: JsonRpcRequest) {
        debug!("Received notification: {}", notification.method);
        
        match notification.method.as_str() {
            "notifications/tools/listChanged" => {
                info!("Tools list changed notification received");
                // TODO: Emit event or call callback
            }
            "notifications/resources/listChanged" => {
                info!("Resources list changed notification received");
                // TODO: Emit event or call callback
            }
            "notifications/resources/updated" => {
                info!("Resource updated notification received");
                // TODO: Emit event or call callback
            }
            "notifications/prompts/listChanged" => {
                info!("Prompts list changed notification received");
                // TODO: Emit event or call callback
            }
            "notifications/roots/listChanged" => {
                info!("Roots list changed notification received");
                // TODO: Emit event or call callback
            }
            "notifications/progress" => {
                if let Some(params) = &notification.params {
                    if let Ok(progress) = serde_json::from_value::<ultrafast_mcp_core::types::notifications::ProgressNotification>(params.clone()) {
                        info!("Progress notification: {}%", progress.progress);
                        // TODO: Emit event or call callback
                    }
                }
            }
            "notifications/logging/message" => {
                if let Some(params) = &notification.params {
                    if let Ok(log_msg) = serde_json::from_value::<ultrafast_mcp_core::types::notifications::LoggingMessageNotification>(params.clone()) {
                        info!("Log message notification: {:?} - {:?}", log_msg.level, log_msg.data);
                        // TODO: Emit event or call callback
                    }
                }
            }
            "notifications/cancelled" => {
                if let Some(params) = &notification.params {
                    if let Ok(cancelled) = serde_json::from_value::<ultrafast_mcp_core::types::notifications::CancelledNotification>(params.clone()) {
                        info!("Request cancelled notification: {:?}", cancelled.request_id);
                        // TODO: Emit event or call callback
                    }
                }
            }
            _ => {
                warn!("Unknown notification method: {}", notification.method);
            }
        }
    }

    /// Connect to a server using STDIO transport
    pub async fn connect_stdio(&self) -> MCPResult<()> {
        info!("Connecting to MCP server via STDIO");

        let transport = ultrafast_mcp_transport::stdio::StdioTransport::new()
            .await
            .map_err(|e| {
                MCPError::internal_error(format!("Failed to create STDIO transport: {}", e))
            })?;
        self.connect(Box::new(transport)).await
    }

    /// Initialize the MCP connection
    async fn initialize(&self) -> MCPResult<()> {
        info!("Initializing MCP connection");

        // Update state to initializing
        {
            let mut state = self.state.write().await;
            *state = ClientState::Initializing;
        }

        // Create initialize request
        let init_request = InitializeRequest {
            protocol_version: PROTOCOL_VERSION.to_string(),
            client_info: self.info.clone(),
            capabilities: self.capabilities.clone(),
        };

        // Send initialize request
        let response: InitializeResponse = self
            .send_request("initialize", Some(serde_json::to_value(init_request)?))
            .await?;

        // Validate protocol version - accept any supported negotiated version
        if !ultrafast_mcp_core::protocol::version::is_supported_version(&response.protocol_version)
        {
            return Err(MCPError::invalid_request(format!(
                "Negotiated protocol version {} is not supported by client. Supported versions: {:?}",
                response.protocol_version, ultrafast_mcp_core::protocol::version::SUPPORTED_VERSIONS
            )));
        }

        // Log version negotiation result
        if response.protocol_version != PROTOCOL_VERSION {
            info!(
                "Protocol version negotiated: {} -> {}",
                PROTOCOL_VERSION, response.protocol_version
            );
        } else {
            info!(
                "Using requested protocol version: {}",
                response.protocol_version
            );
        }

        // Store the negotiated version
        {
            let mut negotiated_version = self.negotiated_version.write().await;
            *negotiated_version = Some(response.protocol_version.clone());
        }

        // Store server info and capabilities
        {
            let mut server_info = self.server_info.write().await;
            *server_info = Some(response.server_info);
        }
        {
            let mut server_capabilities = self.server_capabilities.write().await;
            *server_capabilities = Some(response.capabilities);
        }

        // Log any warnings from server instructions
        if let Some(ref instructions) = response.instructions {
            warn!("Server instructions: {}", instructions);
        }

        // Send initialized notification
        let initialized_notification = InitializedNotification {};
        self.send_notification(
            "initialized",
            Some(serde_json::to_value(initialized_notification)?),
        )
        .await?;

        // Update state to operating
        {
            let mut state = self.state.write().await;
            *state = ClientState::Operating;
        }

        info!(
            "MCP connection initialized successfully with protocol version: {}",
            response.protocol_version
        );
        Ok(())
    }

    /// Shutdown the client gracefully
    pub async fn shutdown(&self, reason: Option<String>) -> MCPResult<()> {
        let reason_display = reason.as_deref().unwrap_or("No reason provided");
        info!("Shutting down MCP client: {}", reason_display);

        // Update state to shutting down
        {
            let mut state = self.state.write().await;
            *state = ClientState::ShuttingDown;
        }

        // Send shutdown request if connected
        if self.can_operate().await {
            let shutdown_request = ShutdownRequest {
                reason,
            };

            // Try to send shutdown request, but don't fail if it doesn't work
            let _ = self.send_request::<()>("shutdown", Some(serde_json::to_value(shutdown_request)?)).await;
        }

        // Cancel all pending requests
        {
            let mut pending = self.pending_requests.write().await;
            for (id, pending_req) in pending.drain() {
                            let _ = pending_req.response_sender.send(JsonRpcMessage::Response(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: Some(ultrafast_mcp_core::protocol::jsonrpc::RequestId::Number(id as i64)),
                result: None,
                error: Some(ultrafast_mcp_core::protocol::jsonrpc::JsonRpcError {
                    code: -32000,
                    message: "Client shutdown".to_string(),
                    data: None,
                }),
                meta: HashMap::new(),
            }));
            }
        }

        // Stop message receiver task
        {
            let mut receiver_guard = self.message_receiver.write().await;
            if let Some(handle) = receiver_guard.take() {
                handle.abort();
            }
        }

        // Update state to shutdown
        {
            let mut state = self.state.write().await;
            *state = ClientState::Shutdown;
        }

        info!("MCP client shutdown complete");
        Ok(())
    }

    /// Disconnect from the server
    pub async fn disconnect(&self) -> MCPResult<()> {
        self.shutdown(Some("Client disconnect".to_string())).await
    }

    /// Get current client state
    pub async fn get_state(&self) -> ClientState {
        self.state.read().await.clone()
    }

    /// Check if client can perform operations
    pub async fn can_operate(&self) -> bool {
        self.state.read().await.can_operate()
    }

    /// Get server info
    pub async fn get_server_info(&self) -> Option<ServerInfo> {
        self.server_info.read().await.clone()
    }

    /// Get server capabilities
    pub async fn get_server_capabilities(&self) -> Option<ServerCapabilities> {
        self.server_capabilities.read().await.clone()
    }

    /// Get the negotiated protocol version
    pub async fn get_negotiated_version(&self) -> Option<String> {
        self.negotiated_version.read().await.clone()
    }

    /// Get client info
    pub fn info(&self) -> &ClientInfo {
        &self.info
    }

    /// Check if a server capability is supported
    pub async fn check_server_capability(&self, capability: &str) -> MCPResult<bool> {
        let server_caps = self.server_capabilities.read().await;
        match server_caps.as_ref() {
            Some(caps) => Ok(ultrafast_mcp_core::protocol::capabilities::CapabilityNegotiator::supports_capability(caps, capability)),
            None => Err(MCPError::invalid_request("Server capabilities not available".to_string())),
        }
    }

    /// Check if a specific feature within a capability is supported
    pub async fn check_server_feature(&self, capability: &str, feature: &str) -> MCPResult<bool> {
        let server_caps = self.server_capabilities.read().await;
        match server_caps.as_ref() {
            Some(caps) => Ok(
                ultrafast_mcp_core::protocol::capabilities::CapabilityNegotiator::supports_feature(
                    caps, capability, feature,
                ),
            ),
            None => Err(MCPError::invalid_request(
                "Server capabilities not available".to_string(),
            )),
        }
    }

    /// Ensure a capability is supported before making a request
    async fn ensure_capability_supported(&self, capability: &str) -> MCPResult<()> {
        if !self.check_server_capability(capability).await? {
            return Err(MCPError::Protocol(
                ultrafast_mcp_core::error::ProtocolError::CapabilityNotSupported(
                    format!("Capability '{}' not supported by server", capability)
                )
            ));
        }
        Ok(())
    }

    /// List available tools
    pub async fn list_tools(&self, request: ListToolsRequest) -> MCPResult<ListToolsResponse> {
        self.ensure_operational().await?;
        self.ensure_capability_supported("tools").await?;
        self.send_request("tools/list", Some(serde_json::to_value(request)?))
            .await
    }

    /// List available tools with default request
    pub async fn list_tools_default(&self) -> MCPResult<ListToolsResponse> {
        self.list_tools(ListToolsRequest::default()).await
    }

    /// Call a tool
    pub async fn call_tool(&self, tool_call: ToolCall) -> MCPResult<ToolResult> {
        self.ensure_operational().await?;
        self.ensure_capability_supported("tools").await?;
        self.send_request("tools/call", Some(serde_json::to_value(tool_call)?))
            .await
    }

    /// List available resources
    pub async fn list_resources(
        &self,
        request: ListResourcesRequest,
    ) -> MCPResult<ListResourcesResponse> {
        self.ensure_operational().await?;
        self.ensure_capability_supported("resources").await?;
        self.send_request("resources/list", Some(serde_json::to_value(request)?))
            .await
    }

    /// Read a resource
    pub async fn read_resource(
        &self,
        request: ReadResourceRequest,
    ) -> MCPResult<ReadResourceResponse> {
        self.ensure_operational().await?;
        self.ensure_capability_supported("resources").await?;
        self.send_request("resources/read", Some(serde_json::to_value(request)?))
            .await
    }

    /// Subscribe to resource changes
    pub async fn subscribe_resource(&self, uri: String) -> MCPResult<()> {
        self.ensure_operational().await?;
        self.ensure_capability_supported("resources").await?;

        let subscribe_request = ultrafast_mcp_core::types::resources::SubscribeRequest {
            uri,
        };

        self.send_request::<()>("resources/subscribe", Some(serde_json::to_value(subscribe_request)?))
            .await?;

        Ok(())
    }

    /// List available prompts
    pub async fn list_prompts(
        &self,
        request: ListPromptsRequest,
    ) -> MCPResult<ListPromptsResponse> {
        self.ensure_operational().await?;
        self.ensure_capability_supported("prompts").await?;
        self.send_request("prompts/list", Some(serde_json::to_value(request)?))
            .await
    }

    /// Get a prompt
    pub async fn get_prompt(&self, request: GetPromptRequest) -> MCPResult<GetPromptResponse> {
        self.ensure_operational().await?;
        self.ensure_capability_supported("prompts").await?;
        self.send_request("prompts/get", Some(serde_json::to_value(request)?))
            .await
    }

    /// Create a message (sampling)
    pub async fn create_message(
        &self,
        request: CreateMessageRequest,
    ) -> MCPResult<CreateMessageResponse> {
        self.ensure_operational().await?;

        // Sampling requires tools capability on the server
        self.ensure_capability_supported("tools").await?;

        // Check protocol version support for sampling
        let negotiated_version = self.get_negotiated_version().await.ok_or_else(|| {
            MCPError::invalid_request("Protocol version not negotiated".to_string())
        })?;

        if !ultrafast_mcp_core::protocol::capabilities::CapabilityNegotiator::version_supports_feature(&negotiated_version, "sampling") {
            return Err(MCPError::Protocol(
                ultrafast_mcp_core::error::ProtocolError::CapabilityNotSupported(
                    format!("Sampling not supported in protocol version {}", negotiated_version)
                )
            ));
        }

        self.send_request(
            "sampling/createMessage",
            Some(serde_json::to_value(request)?),
        )
        .await
    }

    /// Complete a request
    pub async fn complete(&self, request: CompleteRequest) -> MCPResult<CompleteResponse> {
        self.ensure_operational().await?;
        self.ensure_capability_supported("completion").await?;
        self.send_request("completion/complete", Some(serde_json::to_value(request)?))
            .await
    }

    /// Handle elicitation request
    pub async fn handle_elicitation(
        &self,
        request: ElicitationRequest,
    ) -> MCPResult<ElicitationResponse> {
        self.ensure_operational().await?;
        self.ensure_capability_supported("elicitation").await?;
        self.send_request("elicitation/request", Some(serde_json::to_value(request)?))
            .await
    }

    /// List available roots
    pub async fn list_roots(&self) -> MCPResult<Vec<ultrafast_mcp_core::types::roots::Root>> {
        self.ensure_operational().await?;
        self.ensure_capability_supported("roots").await?;
        self.send_request("roots/list", None).await
    }

    /// Set logging level on server
    pub async fn set_log_level(&self, level: ultrafast_mcp_core::types::notifications::LogLevel) -> MCPResult<()> {
        self.ensure_operational().await?;
        self.ensure_capability_supported("logging").await?;
        
        let request = ultrafast_mcp_core::types::notifications::LogLevelSetRequest::new(level);
        let _response: ultrafast_mcp_core::types::notifications::LogLevelSetResponse = 
            self.send_request("logging/setLevel", Some(serde_json::to_value(request)?)).await?;
        Ok(())
    }

    /// Send ping to server
    pub async fn ping(&self, data: Option<serde_json::Value>) -> MCPResult<ultrafast_mcp_core::types::notifications::PingResponse> {
        self.ensure_operational().await?;
        
        let ping_request = match data {
            Some(data) => ultrafast_mcp_core::types::notifications::PingRequest::new().with_data(data),
            None => ultrafast_mcp_core::types::notifications::PingRequest::new(),
        };
        
        self.send_request("ping", Some(serde_json::to_value(ping_request)?)).await
    }

    /// Send cancellation notification
    pub async fn notify_cancelled(&self, request_id: serde_json::Value, reason: Option<String>) -> MCPResult<()> {
        let mut notification = ultrafast_mcp_core::types::notifications::CancelledNotification::new(request_id);
        if let Some(reason) = reason {
            notification = notification.with_reason(reason);
        }
        self.send_notification("notifications/cancelled", Some(serde_json::to_value(notification)?)).await
    }

    /// Send progress notification
    pub async fn notify_progress(
        &self,
        progress_token: serde_json::Value,
        progress: f64,
        total: Option<f64>,
        message: Option<String>
    ) -> MCPResult<()> {
        let mut notification = ultrafast_mcp_core::types::notifications::ProgressNotification::new(progress_token, progress);
        if let Some(total) = total {
            notification = notification.with_total(total);
        }
        if let Some(message) = message {
            notification = notification.with_message(message);
        }
        self.send_notification("notifications/progress", Some(serde_json::to_value(notification)?)).await
    }

    /// Ensure client is in operational state
    async fn ensure_operational(&self) -> MCPResult<()> {
        if !self.can_operate().await {
            return Err(MCPError::invalid_request(
                "Client is not in operational state".to_string(),
            ));
        }
        Ok(())
    }

    /// Generate a unique request ID
    async fn generate_request_id(&self) -> u64 {
        let mut counter = self.request_id_counter.write().await;
        let id = *counter;
        *counter += 1;
        id
    }

    /// Send a request and wait for response with proper correlation
    async fn send_request<T>(&self, method: &str, params: Option<Value>) -> MCPResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let request_id = self.generate_request_id().await;
        debug!("Sending request: {} (id: {})", method, request_id);

        // Create request
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(ultrafast_mcp_core::protocol::jsonrpc::RequestId::Number(request_id as i64)),
            method: method.to_string(),
            params,
            meta: HashMap::new(),
        };

        // Create response channel
        let (response_sender, response_receiver) = oneshot::channel();

        // Register pending request
        {
            let mut pending = self.pending_requests.write().await;
            pending.insert(request_id, PendingRequest {
                response_sender,
                timeout: tokio::time::Instant::now() + self.request_timeout,
            });
        }

        // Send request
        {
            let mut transport_guard = self.transport.write().await;
            let transport = transport_guard
                .as_mut()
                .ok_or_else(|| MCPError::internal_error("No transport available".to_string()))?;

            transport.send_message(JsonRpcMessage::Request(request)).await
                .map_err(|e| MCPError::internal_error(format!("Failed to send request: {}", e)))?;
        }

        // Wait for response with timeout
        let response = tokio::time::timeout(self.request_timeout, response_receiver)
            .await
            .map_err(|_| MCPError::internal_error("Request timeout".to_string()))?
            .map_err(|_| MCPError::internal_error("Response channel closed".to_string()))?;

        // Clean up pending request
        {
            let mut pending = self.pending_requests.write().await;
            pending.remove(&request_id);
        }

        // Parse response
        match response {
            JsonRpcMessage::Response(response) => {
                if let Some(error) = response.error {
                    return Err(MCPError::from(error));
                }

                if let Some(result) = response.result {
                    serde_json::from_value(result)
                        .map_err(|e| MCPError::serialization_error(e.to_string()))
                } else {
                    Err(MCPError::internal_error("Response has no result".to_string()))
                }
            }
            _ => Err(MCPError::internal_error("Unexpected message type".to_string())),
        }
    }

    /// Send a notification (one-way message)
    async fn send_notification(&self, method: &str, params: Option<Value>) -> MCPResult<()> {
        debug!("Sending notification: {}", method);

        let notification = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: None,
            method: method.to_string(),
            params,
            meta: HashMap::new(),
        };

        let mut transport_guard = self.transport.write().await;
        let transport = transport_guard
            .as_mut()
            .ok_or_else(|| MCPError::internal_error("No transport available".to_string()))?;

        transport.send_message(JsonRpcMessage::Request(notification)).await
            .map_err(|e| MCPError::internal_error(format!("Failed to send notification: {}", e)))?;

        Ok(())
    }
}

impl std::fmt::Debug for UltraFastClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UltraFastClient")
            .field("info", &self.info)
            .field("capabilities", &self.capabilities)
            .field("state", &self.state)
            .field("request_timeout", &self.request_timeout)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let info = ClientInfo::new("test-client".to_string(), "1.0.0".to_string());
        let capabilities = ClientCapabilities::default();

        let client = UltraFastClient::new(info, capabilities);
        assert_eq!(client.get_state().await, ClientState::Uninitialized);
    }

    #[tokio::test]
    async fn test_client_creation_with_timeout() {
        let info = ClientInfo::new("test-client".to_string(), "1.0.0".to_string());
        let capabilities = ClientCapabilities::default();
        let timeout = std::time::Duration::from_secs(60);

        let client = UltraFastClient::new_with_timeout(info, capabilities, timeout);
        assert_eq!(client.get_state().await, ClientState::Uninitialized);
    }

    #[tokio::test]
    async fn test_client_state_transitions() {
        let info = ClientInfo::new("test-client".to_string(), "1.0.0".to_string());
        let capabilities = ClientCapabilities::default();

        let client = UltraFastClient::new(info, capabilities);

        // Initial state
        assert_eq!(client.get_state().await, ClientState::Uninitialized);
        assert!(!client.can_operate().await);

        // After initialization (this would require a mock transport)
        // assert_eq!(client.get_state().await, ClientState::Operating);
        // assert!(client.can_operate().await);
    }
}
