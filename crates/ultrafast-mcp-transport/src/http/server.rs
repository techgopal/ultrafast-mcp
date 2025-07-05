//! HTTP transport server implementation
//!
//! This module provides a MCP-compliant Streamable HTTP server implementation
//! that follows the MCP specification for stateless request/response communication.

use crate::{Result, Transport, TransportError};
use async_trait::async_trait;
use axum::{
    extract::State,
    http::{HeaderValue, StatusCode, HeaderMap},
    response::{IntoResponse, Response, sse::Event, Sse},
    Json, Router,
    body::Bytes,
};
use futures::stream::{self, Stream};
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tracing::{error, info};
use ultrafast_mcp_core::protocol::{JsonRpcMessage, JsonRpcRequest, JsonRpcResponse, JsonRpcError};

/// HTTP transport configuration
#[derive(Debug, Clone)]
pub struct HttpTransportConfig {
    pub host: String,
    pub port: u16,
    pub cors_enabled: bool,
    pub protocol_version: String,
    pub allow_origin: Option<String>,
}

impl Default for HttpTransportConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            cors_enabled: true,
            protocol_version: "2025-06-18".to_string(),
            allow_origin: Some("http://localhost:*".to_string()),
        }
    }
}

/// Shared state for HTTP transport
#[derive(Clone)]
pub struct HttpTransportState {
    pub message_sender: broadcast::Sender<(String, JsonRpcMessage)>,
    pub response_sender: broadcast::Sender<(String, JsonRpcMessage)>,
    pub config: HttpTransportConfig,
}

/// HTTP transport server implementation
pub struct HttpTransportServer {
    state: HttpTransportState,
    message_receiver: broadcast::Receiver<(String, JsonRpcMessage)>,
}

impl HttpTransportServer {
    pub fn new(config: HttpTransportConfig) -> Self {
        let (message_sender, message_receiver) = broadcast::channel(1000);
        let (response_sender, _) = broadcast::channel(1000);

        let state = HttpTransportState {
            message_sender,
            response_sender,
            config,
        };

        Self {
            state,
            message_receiver,
        }
    }

    /// Get a message receiver to subscribe to incoming messages
    pub fn get_message_receiver(&self) -> broadcast::Receiver<(String, JsonRpcMessage)> {
        self.state.message_sender.subscribe()
    }

    /// Get the message sender for response delivery
    pub fn get_message_sender(&self) -> broadcast::Sender<(String, JsonRpcMessage)> {
        self.state.message_sender.clone()
    }

    /// Get the response sender for sending responses back to clients
    pub fn get_response_sender(&self) -> broadcast::Sender<(String, JsonRpcMessage)> {
        self.state.response_sender.clone()
    }

    /// Get the transport state
    pub fn get_state(&self) -> HttpTransportState {
        self.state.clone()
    }

    /// Start the HTTP server
    pub async fn run(self) -> Result<()> {
        info!(
            "Starting HTTP transport server on {}:{}",
            self.state.config.host, self.state.config.port
        );

        let app = self.create_router();
        let addr = (self.state.config.host.as_str(), self.state.config.port);

        let listener = tokio::net::TcpListener::bind(addr).await
            .map_err(|e| TransportError::InitializationError {
                message: format!("Failed to bind to address: {}", e),
            })?;

        axum::serve(listener, app.into_make_service())
            .await
            .map_err(|e| TransportError::InitializationError {
                message: format!("Server failed: {}", e),
            })?;

        Ok(())
    }

    fn create_router(&self) -> Router {
        let state = Arc::new(self.state.clone());
        let mut router = Router::new()
            .route("/mcp", axum::routing::post(handle_mcp_post))
            .route("/mcp", axum::routing::get(handle_mcp_get))
            .route("/mcp", axum::routing::delete(handle_mcp_delete));

        if self.state.config.cors_enabled {
            router = router.layer(CorsLayer::permissive());
        }

        router.with_state(state)
    }
}

#[async_trait]
impl Transport for HttpTransportServer {
    async fn send_message(&mut self, message: JsonRpcMessage) -> Result<()> {
        // Broadcast to all connected sessions
        let _ = self.state.message_sender.send(("*".to_string(), message));
        Ok(())
    }

    async fn receive_message(&mut self) -> Result<JsonRpcMessage> {
        match self.message_receiver.recv().await {
            Ok((_, message)) => Ok(message),
            Err(_) => Err(TransportError::ConnectionClosed),
        }
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Extract session ID from headers
fn extract_session_id(headers: &HeaderMap) -> Option<String> {
    headers
        .get("mcp-session-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Validate Origin header for security
fn validate_origin(headers: &HeaderMap, config: &HttpTransportConfig) -> bool {
    if let Some(origin) = headers.get("origin") {
        if let Ok(origin_str) = origin.to_str() {
            // For localhost, allow any localhost origin
            if config.host == "127.0.0.1" || config.host == "localhost" {
                return origin_str.contains("localhost") || origin_str.contains("127.0.0.1");
            }
            // For production, check against allowed origin
            if let Some(allowed_origin) = &config.allow_origin {
                return origin_str == allowed_origin;
            }
        }
        return false;
    }
    // Allow requests without Origin header for local development
    config.host == "127.0.0.1" || config.host == "localhost"
}

/// Generate a new session ID
fn generate_session_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

async fn handle_mcp_post(
    State(state): State<Arc<HttpTransportState>>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    if !validate_origin(&headers, &state.config) {
        return (
            StatusCode::FORBIDDEN,
            Json(JsonRpcResponse::error(
                JsonRpcError::new(-32000, "Origin not allowed".to_string()),
                None,
            )),
        ).into_response();
    }
    let session_id = extract_session_id(&headers)
        .unwrap_or_else(|| generate_session_id());
    // Try to parse the body as a JSON-RPC message
    let message: std::result::Result<JsonRpcMessage, serde_json::Error> = serde_json::from_slice(&body);
    let message = match message {
        Ok(msg) => msg,
        Err(_) => {
            return Json(JsonRpcResponse::error(
                JsonRpcError::new(-32700, "Parse error: Invalid JSON-RPC message".to_string()),
                None,
            )).into_response();
        }
    };
    info!("Processing POST request for session {}: {:?}", session_id, message);
    match message {
        JsonRpcMessage::Request(request) => {
            handle_jsonrpc_request(state, session_id, request).await
        }
        JsonRpcMessage::Notification(_) | JsonRpcMessage::Response(_) => {
            handle_notification_or_response(state, session_id, message).await
        }
    }
}

async fn handle_mcp_get(
    State(state): State<Arc<HttpTransportState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !validate_origin(&headers, &state.config) {
        return (
            StatusCode::FORBIDDEN,
            Json(JsonRpcResponse::error(
                JsonRpcError::new(-32000, "Origin not allowed".to_string()),
                None,
            )),
        ).into_response();
    }
    let session_id = extract_session_id(&headers)
        .unwrap_or_else(|| generate_session_id());
    info!("Processing GET request for session {} (SSE stream)", session_id);
    let stream = create_sse_stream(state, session_id);
    Sse::new(stream).into_response()
}

async fn handle_mcp_delete(
    State(state): State<Arc<HttpTransportState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !validate_origin(&headers, &state.config) {
        return (
            StatusCode::FORBIDDEN,
            Json(JsonRpcResponse::error(
                JsonRpcError::new(-32000, "Origin not allowed".to_string()),
                None,
            )),
        ).into_response();
    }
    let session_id = extract_session_id(&headers)
        .unwrap_or_else(|| generate_session_id());
    info!("Terminating session: {}", session_id);
    StatusCode::OK.into_response()
}

/// Handle JSON-RPC requests
async fn handle_jsonrpc_request(
    state: Arc<HttpTransportState>,
    session_id: String,
    request: JsonRpcRequest,
) -> Response {
    // Create a response receiver for this specific request
    let mut response_receiver = state.response_sender.subscribe();
    
    // Send message to server for processing
    if let Err(e) = state.message_sender.send((session_id.clone(), JsonRpcMessage::Request(request.clone()))) {
        error!("Failed to send message to server: {}", e);
        return Json(JsonRpcResponse::error(
            JsonRpcError::new(-32000, format!("Failed to process message: {}", e)),
            request.id,
        )).into_response();
    }

    // Wait for response from server with timeout
    match tokio::time::timeout(
        std::time::Duration::from_secs(5000), // 5 second timeout
        response_receiver.recv()
    ).await {
        Ok(Ok((response_session_id, response_message))) => {
            if response_session_id == session_id || response_session_id == "*" {
                // Return the actual response from the server
                match response_message {
                    JsonRpcMessage::Response(response) => {
                        Json(response).into_response()
                    }
                    _ => {
                        // Unexpected message type
                        Json(JsonRpcResponse::error(
                            JsonRpcError::new(-32000, "Unexpected response type".to_string()),
                            request.id,
                        )).into_response()
                    }
                }
            } else {
                // Wrong session
                error!("Received response for wrong session: expected {}, got {}", 
                       session_id, response_session_id);
                Json(JsonRpcResponse::error(
                    JsonRpcError::new(-32000, "Session mismatch".to_string()),
                    request.id,
                )).into_response()
            }
        }
        Ok(Err(e)) => {
            error!("Failed to receive response: {}", e);
            Json(JsonRpcResponse::error(
                JsonRpcError::new(-32000, format!("Failed to receive response: {}", e)),
                request.id,
            )).into_response()
        }
        Err(_) => {
            error!("Timeout waiting for response from server");
            Json(JsonRpcResponse::error(
                JsonRpcError::new(-32000, "Request timeout".to_string()),
                request.id,
            )).into_response()
        }
    }
}

/// Handle notifications and responses
async fn handle_notification_or_response(
    state: Arc<HttpTransportState>,
    session_id: String,
    message: JsonRpcMessage,
) -> Response {
    // Send message to server for processing
    if let Err(e) = state.message_sender.send((session_id.clone(), message)) {
        error!("Failed to send message to server: {}", e);
        return (
            StatusCode::BAD_REQUEST,
            Json(JsonRpcResponse::error(
                JsonRpcError::new(-32000, format!("Failed to process message: {}", e)),
                None,
            )),
        ).into_response();
    }

    // Return 202 Accepted for notifications and responses
    (
        StatusCode::ACCEPTED,
        [("mcp-session-id", session_id)],
    ).into_response()
}

/// Create SSE stream for server-to-client communication
fn create_sse_stream(
    state: Arc<HttpTransportState>,
    session_id: String,
) -> impl Stream<Item = std::result::Result<Event, axum::Error>> {
    let response_receiver = state.response_sender.subscribe();
    
    stream::unfold(
        (response_receiver, session_id),
        |(mut receiver, session_id)| async move {
            match receiver.recv().await {
                Ok((msg_session_id, message)) => {
                    if msg_session_id == session_id || msg_session_id == "*" {
                        let event_data = serde_json::to_string(&message).unwrap_or_default();
                        Some((
                            Ok(Event::default().data(event_data)),
                            (receiver, session_id),
                        ))
                    } else {
                        // Continue waiting for messages for this session
                        Some((
                            Ok(Event::default().data("")), // Empty event to keep connection alive
                            (receiver, session_id),
                        ))
                    }
                }
                Err(_) => None, // Connection closed
            }
        },
    )
}
