//! UltraFast MCP Client Library
//!
//! A high-performance client implementation for the Model Context Protocol (MCP).

use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use ultrafast_mcp_core::{
    error::{MCPError, MCPResult},
    protocol::{
        capabilities::{ClientCapabilities, ServerCapabilities},
        jsonrpc::{JsonRpcMessage, JsonRpcRequest},
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
        }
    }

    /// Connect to a server using the provided transport
    pub async fn connect(&self, transport: Box<dyn Transport>) -> MCPResult<()> {
        info!("Connecting to MCP server");

        {
            let mut transport_guard = self.transport.write().await;
            *transport_guard = Some(transport);
        }

        // Initialize the connection
        self.initialize().await?;

        info!("Successfully connected to MCP server");
        Ok(())
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
        if !ultrafast_mcp_core::protocol::version::is_supported_version(&response.protocol_version) {
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
            info!("Using requested protocol version: {}", response.protocol_version);
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

        info!("MCP connection initialized successfully with protocol version: {}", response.protocol_version);
        Ok(())
    }

    /// Shutdown the MCP connection
    pub async fn shutdown(&self, reason: Option<String>) -> MCPResult<()> {
        info!("Shutting down MCP connection: {:?}", reason);

        // Update state to shutting down
        {
            let mut state = self.state.write().await;
            *state = ClientState::ShuttingDown;
        }

        // Send shutdown request
        let shutdown_request = ShutdownRequest { reason };
        self.send_request::<()>("shutdown", Some(serde_json::to_value(shutdown_request)?))
            .await?;

        // Update state to shutdown
        {
            let mut state = self.state.write().await;
            *state = ClientState::Shutdown;
        }

        info!("MCP connection shutdown completed");
        Ok(())
    }

    /// Disconnect from the server (alias for shutdown)
    pub async fn disconnect(&self) -> MCPResult<()> {
        self.shutdown(None).await
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
            Some(caps) => Ok(ultrafast_mcp_core::protocol::capabilities::CapabilityNegotiator::supports_feature(caps, capability, feature)),
            None => Err(MCPError::invalid_request("Server capabilities not available".to_string())),
        }
    }

    /// Ensure a capability is supported before making a request
    async fn ensure_capability_supported(&self, capability: &str) -> MCPResult<()> {
        if !self.check_server_capability(capability).await? {
            return Err(MCPError::Protocol(
                ultrafast_mcp_core::error::ProtocolError::CapabilityNotSupported(
                    format!("Server does not support {} capability", capability)
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

    /// List available tools with default parameters
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

    /// Subscribe to a resource (if supported)
    pub async fn subscribe_resource(&self, uri: String) -> MCPResult<()> {
        self.ensure_operational().await?;
        self.ensure_capability_supported("resources").await?;
        
        // Check if resource subscriptions are supported
        if !self.check_server_feature("resources", "subscribe").await? {
            return Err(MCPError::Protocol(
                ultrafast_mcp_core::error::ProtocolError::CapabilityNotSupported(
                    "Server does not support resource subscriptions".to_string()
                )
            ));
        }
        
        let request = serde_json::json!({ "uri": uri });
        self.send_request::<()>("resources/subscribe", Some(request)).await
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

    /// Get a specific prompt
    pub async fn get_prompt(&self, request: GetPromptRequest) -> MCPResult<GetPromptResponse> {
        self.ensure_operational().await?;
        self.ensure_capability_supported("prompts").await?;
        self.send_request("prompts/get", Some(serde_json::to_value(request)?))
            .await
    }

    /// Create a message using sampling
    pub async fn create_message(
        &self,
        request: CreateMessageRequest,
    ) -> MCPResult<CreateMessageResponse> {
        self.ensure_operational().await?;
        
        // Sampling requires tools capability on the server
        self.ensure_capability_supported("tools").await?;
        
        // Check protocol version support for sampling
        let negotiated_version = self.get_negotiated_version().await
            .ok_or_else(|| MCPError::invalid_request("Protocol version not negotiated".to_string()))?;
        
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
        
        // Check protocol version support for elicitation
        let negotiated_version = self.get_negotiated_version().await
            .ok_or_else(|| MCPError::invalid_request("Protocol version not negotiated".to_string()))?;
        
        if !ultrafast_mcp_core::protocol::capabilities::CapabilityNegotiator::version_supports_feature(&negotiated_version, "elicitation") {
            return Err(MCPError::Protocol(
                ultrafast_mcp_core::error::ProtocolError::CapabilityNotSupported(
                    format!("Elicitation not supported in protocol version {}", negotiated_version)
                )
            ));
        }
        
        self.send_request("elicitation/request", Some(serde_json::to_value(request)?))
            .await
    }

    /// Ensure the client is in operational state
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
        *counter += 1;
        *counter
    }

    /// Send a request and wait for response
    async fn send_request<T>(&self, method: &str, params: Option<Value>) -> MCPResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let request_id = self.generate_request_id().await;
        let request = JsonRpcRequest::new(method.to_string(), params, Some(request_id.into()));

        debug!("Sending request: {} (id: {})", method, request_id);

        // Get mutable transport reference for sending
        let mut transport_guard = self.transport.write().await;
        let transport = transport_guard
            .as_mut()
            .ok_or_else(|| MCPError::internal_error("No transport available".to_string()))?;

        // Send request
        transport
            .send_message(JsonRpcMessage::Request(request))
            .await
            .map_err(|e| MCPError::internal_error(format!("Failed to send request: {}", e)))?;

        // Wait for response
        drop(transport_guard); // Release lock before waiting for response
        let response = self.wait_for_response(request_id).await?;

        match response {
            JsonRpcMessage::Response(jsonrpc_response) => {
                if let Some(result) = jsonrpc_response.result {
                    let value: T = serde_json::from_value(result).map_err(|e| {
                        MCPError::internal_error(format!("Failed to deserialize response: {}", e))
                    })?;
                    Ok(value)
                } else if let Some(error) = jsonrpc_response.error {
                    Err(MCPError::invalid_request(format!(
                        "Server error: {}",
                        error.message
                    )))
                } else {
                    Err(MCPError::internal_error(
                        "Invalid response format".to_string(),
                    ))
                }
            }
            _ => Err(MCPError::internal_error(
                "Unexpected message type".to_string(),
            )),
        }
    }

    /// Send a notification
    async fn send_notification(&self, method: &str, params: Option<Value>) -> MCPResult<()> {
        let notification = JsonRpcRequest::notification(method.to_string(), params);

        debug!("Sending notification: {}", method);

        // Get mutable transport reference for sending
        let mut transport_guard = self.transport.write().await;
        let transport = transport_guard
            .as_mut()
            .ok_or_else(|| MCPError::internal_error("No transport available".to_string()))?;

        transport
            .send_message(JsonRpcMessage::Notification(notification))
            .await
            .map_err(|e| MCPError::internal_error(format!("Failed to send notification: {}", e)))?;

        Ok(())
    }

    /// Wait for a specific response
    async fn wait_for_response(&self, request_id: u64) -> MCPResult<JsonRpcMessage> {
        // This is a simplified implementation
        // In a real implementation, you would need to maintain a map of pending requests
        // and handle the message routing properly

        // Get mutable transport reference for receiving
        let mut transport_guard = self.transport.write().await;
        let transport = transport_guard
            .as_mut()
            .ok_or_else(|| MCPError::internal_error("No transport available".to_string()))?;

        // For now, we'll just wait for any message and assume it's the response
        // This is not ideal but works for basic functionality
        loop {
            match transport.receive_message().await {
                Ok(message) => {
                    if let JsonRpcMessage::Response(response) = &message {
                        if let Some(id) = &response.id {
                            if let Ok(id_num) =
                                serde_json::from_value::<u64>(serde_json::to_value(id)?)
                            {
                                if id_num == request_id {
                                    return Ok(message);
                                }
                            }
                        }
                    }
                    // Continue waiting for the correct response
                }
                Err(e) => {
                    return Err(MCPError::internal_error(format!(
                        "Failed to receive response: {}",
                        e
                    )));
                }
            }
        }
    }
}

impl std::fmt::Debug for UltraFastClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UltraFastClient")
            .field("info", &self.info)
            .field("capabilities", &self.capabilities)
            .field("state", &self.state)
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
