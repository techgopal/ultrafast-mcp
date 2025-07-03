//! Request handlers for MCP server

use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, warn};

use ultrafast_mcp_core::{
    error::{MCPError, McpError, McpResult, ProtocolError},
    protocol::lifecycle::{InitializeRequest, InitializeResponse},
    types::{
        completion::CompleteRequest,
        elicitation::{ElicitationRequest, ElicitationResponse},
        prompts::{GetPromptRequest, ListPromptsRequest, ListPromptsResponse},
        resources::{
            ListResourceTemplatesRequest, ListResourceTemplatesResponse, ListResourcesRequest,
            ListResourcesResponse, ReadResourceRequest,
        },
        roots::ListRootsResponse,
        sampling::CreateMessageRequest,
        tools::{ListToolsRequest, ListToolsResponse, ToolCall},
    },
};

use crate::ResourceSubscriptionHandler;
use crate::{ServerState, UltraFastServer};

impl UltraFastServer {
    /// Handle initialize request
    #[allow(dead_code)]
    pub(crate) async fn handle_initialize(&self, params: Option<Value>) -> McpResult<Value> {
        debug!("Handling initialize request");

        let mut state = self.state.write().await;
        if !matches!(*state, ServerState::Uninitialized) {
            return Err(MCPError::Protocol(ProtocolError::InitializationFailed(
                "Already initialized".to_string(),
            )));
        }

        *state = ServerState::Initializing;
        drop(state);

        let request: InitializeRequest = match params {
            Some(params) => serde_json::from_value(params)
                .map_err(|e| McpError::invalid_params(e.to_string()))?,
            None => return Err(McpError::invalid_params("Missing parameters".to_string())),
        };

        debug!("Client capabilities: {:?}", request.capabilities);

        let response = InitializeResponse {
            protocol_version: "2025-06-18".to_string(),
            capabilities: self.capabilities.clone(),
            server_info: self.info.clone(),
            instructions: None,
        };

        *self.state.write().await = ServerState::Initialized;

        serde_json::to_value(response).map_err(|e| McpError::serialization_error(e.to_string()))
    }

    /// Handle initialized notification
    pub(crate) async fn handle_initialized(&self) -> McpResult<()> {
        debug!("Received initialized notification");

        let state = self.state.read().await;
        if !matches!(*state, ServerState::Initialized) {
            warn!("Received initialized notification but server is not in initialized state");
        }

        Ok(())
    }

    /// Handle list tools request with pagination support
    #[allow(dead_code)]
    pub(crate) async fn handle_list_tools(&self, params: Option<Value>) -> McpResult<Value> {
        debug!("Handling list tools request");

        let request: ListToolsRequest = match params {
            Some(params) => serde_json::from_value(params)
                .map_err(|e| McpError::invalid_params(e.to_string()))?,
            None => ListToolsRequest { cursor: None },
        };

        let response = if let Some(handler) = &self.tool_handler {
            handler.list_tools(request).await?
        } else {
            // Default implementation for backward compatibility
            let tools: Vec<_> = self.tools.read().await.values().cloned().collect();
            ListToolsResponse {
                tools,
                next_cursor: None,
            }
        };

        serde_json::to_value(response).map_err(|e| McpError::serialization_error(e.to_string()))
    }

    /// Handle tool call request
    #[allow(dead_code)]
    pub(crate) async fn handle_tool_call(&self, params: Option<Value>) -> McpResult<Value> {
        debug!("Handling tool call request");

        let call: ToolCall = match params {
            Some(params) => serde_json::from_value(params)
                .map_err(|e| McpError::invalid_params(e.to_string()))?,
            None => return Err(McpError::invalid_params("Missing parameters".to_string())),
        };

        let result = if let Some(handler) = &self.tool_handler {
            handler.handle_tool_call(call).await?
        } else {
            return Err(McpError::method_not_found(
                "No tool handler registered".to_string(),
            ));
        };

        serde_json::to_value(result).map_err(|e| McpError::serialization_error(e.to_string()))
    }

    /// Handle list resources request with pagination support
    pub(crate) async fn handle_list_resources(&self, params: Option<Value>) -> McpResult<Value> {
        debug!("Handling list resources request");

        let request: ListResourcesRequest = match params {
            Some(params) => serde_json::from_value(params)
                .map_err(|e| McpError::invalid_params(e.to_string()))?,
            None => ListResourcesRequest { cursor: None },
        };

        let response = if let Some(handler) = &self.resource_handler {
            handler.list_resources(request).await?
        } else {
            // Default implementation for backward compatibility
            let resources: Vec<_> = self.resources.read().await.values().cloned().collect();
            ListResourcesResponse {
                resources,
                next_cursor: None,
            }
        };

        serde_json::to_value(response).map_err(|e| McpError::serialization_error(e.to_string()))
    }

    /// Handle list resource templates request with pagination support
    pub(crate) async fn handle_list_resource_templates(
        &self,
        params: Option<Value>,
    ) -> McpResult<Value> {
        debug!("Handling list resource templates request");

        let request: ListResourceTemplatesRequest = match params {
            Some(params) => serde_json::from_value(params)
                .map_err(|e| McpError::invalid_params(e.to_string()))?,
            None => ListResourceTemplatesRequest { cursor: None },
        };

        let response = if let Some(handler) = &self.resource_handler {
            handler.list_resource_templates(request).await?
        } else {
            // Default implementation for backward compatibility
            let resource_templates: Vec<_> = self
                .resource_templates
                .read()
                .await
                .values()
                .cloned()
                .collect();
            ListResourceTemplatesResponse {
                resource_templates,
                next_cursor: None,
            }
        };

        serde_json::to_value(response).map_err(|e| McpError::serialization_error(e.to_string()))
    }

    /// Handle read resource request
    pub(crate) async fn handle_read_resource(&self, params: Option<Value>) -> McpResult<Value> {
        debug!("Handling read resource request");

        let request: ReadResourceRequest = match params {
            Some(params) => serde_json::from_value(params)
                .map_err(|e| McpError::invalid_params(e.to_string()))?,
            None => return Err(McpError::invalid_params("Missing parameters".to_string())),
        };

        let response = if let Some(handler) = &self.resource_handler {
            handler.read_resource(request).await?
        } else {
            return Err(McpError::method_not_found(
                "No resource handler registered".to_string(),
            ));
        };

        serde_json::to_value(response).map_err(|e| McpError::serialization_error(e.to_string()))
    }

    /// Handle list prompts request with pagination support
    pub(crate) async fn handle_list_prompts(&self, params: Option<Value>) -> McpResult<Value> {
        debug!("Handling list prompts request");

        let request: ListPromptsRequest = match params {
            Some(params) => serde_json::from_value(params)
                .map_err(|e| McpError::invalid_params(e.to_string()))?,
            None => ListPromptsRequest { cursor: None },
        };

        let response = if let Some(handler) = &self.prompt_handler {
            handler.list_prompts(request).await?
        } else {
            // Default implementation for backward compatibility
            let prompts: Vec<_> = self.prompts.read().await.values().cloned().collect();
            ListPromptsResponse {
                prompts,
                next_cursor: None,
            }
        };

        serde_json::to_value(response).map_err(|e| McpError::serialization_error(e.to_string()))
    }

    /// Handle get prompt request
    pub(crate) async fn handle_get_prompt(&self, params: Option<Value>) -> McpResult<Value> {
        debug!("Handling get prompt request");

        let request: GetPromptRequest = match params {
            Some(params) => serde_json::from_value(params)
                .map_err(|e| McpError::invalid_params(e.to_string()))?,
            None => return Err(McpError::invalid_params("Missing parameters".to_string())),
        };

        let response = if let Some(handler) = &self.prompt_handler {
            handler.get_prompt(request).await?
        } else {
            return Err(McpError::method_not_found(
                "No prompt handler registered".to_string(),
            ));
        };

        serde_json::to_value(response).map_err(|e| McpError::serialization_error(e.to_string()))
    }

    /// Handle completion request
    pub(crate) async fn handle_completion(&self, params: Option<Value>) -> McpResult<Value> {
        debug!("Handling completion request");

        let request: CompleteRequest = match params {
            Some(params) => serde_json::from_value(params)
                .map_err(|e| McpError::invalid_params(e.to_string()))?,
            None => return Err(McpError::invalid_params("Missing parameters".to_string())),
        };

        let response = if let Some(handler) = &self.completion_handler {
            handler.complete(request).await?
        } else {
            return Err(McpError::method_not_found(
                "No completion handler registered".to_string(),
            ));
        };

        serde_json::to_value(response).map_err(|e| McpError::serialization_error(e.to_string()))
    }

    /// Handle create message request
    pub(crate) async fn handle_create_message(&self, params: Option<Value>) -> McpResult<Value> {
        debug!("Handling create message request");

        let request: CreateMessageRequest = match params {
            Some(params) => serde_json::from_value(params)
                .map_err(|e| McpError::invalid_params(e.to_string()))?,
            None => return Err(McpError::invalid_params("Missing parameters".to_string())),
        };

        let response = if let Some(handler) = &self.sampling_handler {
            handler.create_message(request).await?
        } else {
            return Err(McpError::method_not_found(
                "No sampling handler registered".to_string(),
            ));
        };

        serde_json::to_value(response).map_err(|e| McpError::serialization_error(e.to_string()))
    }

    // Phase 3: Advanced handler methods

    /// Handle list roots request
    pub(crate) async fn handle_list_roots(&self, _params: Option<Value>) -> McpResult<Value> {
        debug!("Handling list roots request");

        let roots = if let Some(handler) = &self.roots_handler {
            handler.list_roots().await?
        } else {
            // Default empty roots if no handler
            Vec::new()
        };

        let response = ListRootsResponse { roots };

        serde_json::to_value(response).map_err(|e| McpError::serialization_error(e.to_string()))
    }

    /// Handle resource subscription request
    pub(crate) async fn handle_subscribe_resource(
        &self,
        params: Option<Value>,
    ) -> McpResult<Value> {
        debug!("Handling resource subscription request");

        let uri: String = match params {
            Some(params) => {
                let parsed = serde_json::from_value::<serde_json::Map<String, Value>>(params)
                    .map_err(|e| McpError::invalid_params(e.to_string()))?;
                parsed
                    .get("uri")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        McpError::invalid_params("Missing or invalid uri parameter".to_string())
                    })?
                    .to_string()
            }
            None => return Err(McpError::invalid_params("Missing parameters".to_string())),
        };

        if let Some(handler) = &self.subscription_handler {
            handler.subscribe(uri.clone()).await?;
        }

        // Track subscription
        let mut subscriptions = self.resource_subscriptions.write().await;
        subscriptions
            .entry(uri)
            .or_insert_with(Vec::new)
            .push("default_client".to_string());

        Ok(serde_json::json!({}))
    }

    /// Handle resource unsubscription request
    pub(crate) async fn handle_unsubscribe_resource(
        &self,
        params: Option<Value>,
    ) -> McpResult<Value> {
        debug!("Handling resource unsubscription request");

        let uri: String = match params {
            Some(params) => {
                let parsed = serde_json::from_value::<serde_json::Map<String, Value>>(params)
                    .map_err(|e| McpError::invalid_params(e.to_string()))?;
                parsed
                    .get("uri")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        McpError::invalid_params("Missing or invalid uri parameter".to_string())
                    })?
                    .to_string()
            }
            None => return Err(McpError::invalid_params("Missing parameters".to_string())),
        };

        if let Some(handler) = &self.subscription_handler {
            handler.unsubscribe(uri.clone()).await?;
        }

        // Remove subscription
        let mut subscriptions = self.resource_subscriptions.write().await;
        if let Some(clients) = subscriptions.get_mut(&uri) {
            clients.retain(|client| client != "default_client");
            if clients.is_empty() {
                subscriptions.remove(&uri);
            }
        }

        Ok(serde_json::json!({}))
    }

    /// Send server-initiated elicitation request to client
    pub async fn send_elicitation_request(
        &self,
        request: ElicitationRequest,
    ) -> McpResult<ElicitationResponse> {
        debug!("Sending elicitation request to client");

        if let Some(handler) = &self.elicitation_handler {
            handler.handle_elicitation(request).await
        } else {
            Err(McpError::internal_error(
                "No elicitation handler configured".to_string(),
            ))
        }
    }

    /// Notify clients of resource changes
    pub async fn notify_resource_change(
        &self,
        uri: String,
        content: serde_json::Value,
    ) -> McpResult<()> {
        debug!("Notifying resource change for: {}", uri);

        if let Some(handler) = &self.subscription_handler {
            handler.notify_change(uri, content).await?;
        }

        Ok(())
    }
}

/// Simple in-memory resource subscription manager
pub struct MemoryResourceSubscriptionHandler {
    subscriptions: Arc<RwLock<HashMap<String, Vec<String>>>>, // URI -> client IDs
    resource_change_tx: broadcast::Sender<(String, serde_json::Value)>,
}

impl MemoryResourceSubscriptionHandler {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1000);

        Self {
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            resource_change_tx: tx,
        }
    }

    /// Get a receiver for resource change notifications
    pub fn subscribe_to_changes(&self) -> broadcast::Receiver<(String, serde_json::Value)> {
        self.resource_change_tx.subscribe()
    }

    /// Simulate a resource change (for testing)
    pub async fn simulate_resource_change(
        &self,
        uri: String,
        content: serde_json::Value,
    ) -> McpResult<()> {
        let _ = self.resource_change_tx.send((uri, content));
        Ok(())
    }
}

#[async_trait::async_trait]
impl ResourceSubscriptionHandler for MemoryResourceSubscriptionHandler {
    async fn subscribe(&self, uri: String) -> McpResult<()> {
        let mut subscriptions = self.subscriptions.write().await;
        let clients = subscriptions.entry(uri.clone()).or_insert_with(Vec::new);

        // For demo purposes, we use a fixed client ID
        let client_id = "demo-client".to_string();
        if !clients.contains(&client_id) {
            clients.push(client_id);
        }

        tracing::info!("Client subscribed to resource: {}", uri);
        Ok(())
    }

    async fn unsubscribe(&self, uri: String) -> McpResult<()> {
        let mut subscriptions = self.subscriptions.write().await;
        if let Some(clients) = subscriptions.get_mut(&uri) {
            clients.retain(|id| id != "demo-client");
            if clients.is_empty() {
                subscriptions.remove(&uri);
            }
        }

        tracing::info!("Client unsubscribed from resource: {}", uri);
        Ok(())
    }

    async fn notify_change(&self, uri: String, content: serde_json::Value) -> McpResult<()> {
        let subscriptions = self.subscriptions.read().await;
        if let Some(clients) = subscriptions.get(&uri) {
            if !clients.is_empty() {
                let _ = self.resource_change_tx.send((uri.clone(), content));
                tracing::info!(
                    "Notified {} clients of resource change: {}",
                    clients.len(),
                    uri
                );
            }
        }
        Ok(())
    }
}

impl Default for MemoryResourceSubscriptionHandler {
    fn default() -> Self {
        Self::new()
    }
}
