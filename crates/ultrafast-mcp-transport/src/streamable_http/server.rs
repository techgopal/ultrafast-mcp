//! HTTP transport server implementation
//!
//! This module provides a MCP-compliant Streamable HTTP server implementation
//! that follows the MCP specification for stateless request/response communication.

use axum::{
    extract::State,
    http::{header::HeaderMap, StatusCode},
    response::{sse::Event, IntoResponse, Response, Sse},
    routing::Router,
    Json,
};
use bytes::Bytes;
use futures::stream::{self, Stream};
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tracing::{error, info};

use ultrafast_mcp_core::{
    protocol::{
        jsonrpc::{JsonRpcError, JsonRpcMessage, JsonRpcRequest, JsonRpcResponse},
        version::PROTOCOL_VERSION,
    },
    utils::{generate_event_id, generate_session_id},
    validation::{validate_origin, validate_protocol_version, validate_session_id},
};
use ultrafast_mcp_monitoring::metrics::RequestTimer;
use ultrafast_mcp_monitoring::{MetricsCollector, MonitoringSystem};

use crate::{Result, Transport, TransportError};
use async_trait::async_trait;

/// HTTP transport configuration
#[derive(Debug, Clone)]
pub struct HttpTransportConfig {
    pub host: String,
    pub port: u16,
    pub cors_enabled: bool,
    pub protocol_version: String,
    pub allow_origin: Option<String>,
    pub monitoring_enabled: bool,
    pub enable_sse_resumability: bool,
}

impl Default for HttpTransportConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            cors_enabled: true,
            protocol_version: PROTOCOL_VERSION.to_string(),
            allow_origin: Some("http://localhost:*".to_string()),
            monitoring_enabled: true,
            enable_sse_resumability: true,
        }
    }
}

/// Shared state for HTTP transport
#[derive(Clone)]
pub struct HttpTransportState {
    pub message_sender: broadcast::Sender<(String, JsonRpcMessage)>,
    pub response_sender: broadcast::Sender<(String, JsonRpcMessage)>,
    pub config: HttpTransportConfig,
    pub metrics: Option<Arc<MetricsCollector>>,
    pub monitoring: Option<Arc<MonitoringSystem>>,
    pub session_store: Arc<tokio::sync::RwLock<std::collections::HashMap<String, SessionInfo>>>,
}

/// Session information for tracking and resumability
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub created_at: std::time::SystemTime,
    pub last_event_id: Option<String>,
    pub active_streams: std::collections::HashSet<String>,
}

impl SessionInfo {
    pub fn new() -> Self {
        Self {
            created_at: std::time::SystemTime::now(),
            last_event_id: None,
            active_streams: std::collections::HashSet::new(),
        }
    }
}

impl Default for SessionInfo {
    fn default() -> Self {
        Self::new()
    }
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
            metrics: None,
            monitoring: None,
            session_store: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        };

        Self {
            state,
            message_receiver,
        }
    }

    pub fn with_metrics(mut self, metrics: Arc<MetricsCollector>) -> Self {
        self.state.metrics = Some(metrics);
        self
    }

    pub fn with_monitoring(mut self, monitoring: Arc<MonitoringSystem>) -> Self {
        self.state.monitoring = Some(monitoring);
        self
    }

    pub fn get_message_receiver(&self) -> broadcast::Receiver<(String, JsonRpcMessage)> {
        self.state.message_sender.subscribe()
    }

    pub fn get_message_sender(&self) -> broadcast::Sender<(String, JsonRpcMessage)> {
        self.state.message_sender.clone()
    }

    pub fn get_response_sender(&self) -> broadcast::Sender<(String, JsonRpcMessage)> {
        self.state.response_sender.clone()
    }

    pub fn get_state(&self) -> HttpTransportState {
        self.state.clone()
    }

    pub fn get_metrics(&self) -> Option<Arc<MetricsCollector>> {
        self.state.metrics.clone()
    }

    pub fn get_monitoring(&self) -> Option<Arc<MonitoringSystem>> {
        self.state.monitoring.clone()
    }

    pub async fn run(self) -> Result<()> {
        info!(
            "Starting HTTP transport server on {}:{}",
            self.state.config.host, self.state.config.port
        );

        let app = self.create_router();
        let addr = (self.state.config.host.as_str(), self.state.config.port);

        let listener = tokio::net::TcpListener::bind(addr).await.map_err(|e| {
            TransportError::InitializationError {
                message: format!("Failed to bind to address: {}", e),
            }
        })?;

        // Start monitoring HTTP server if enabled
        if let Some(monitoring) = &self.state.monitoring {
            let monitoring_addr =
                format!("{}:{}", self.state.config.host, self.state.config.port + 1)
                    .parse()
                    .map_err(|e| TransportError::InitializationError {
                        message: format!("Failed to parse monitoring address: {}", e),
                    })?;

            let monitoring_clone = monitoring.clone();
            tokio::spawn(async move {
                if let Err(e) = monitoring_clone.start_http_server(monitoring_addr).await {
                    error!("Failed to start monitoring server: {}", e);
                }
            });
        }

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

/// Extract protocol version from headers
fn extract_protocol_version(headers: &HeaderMap) -> Option<String> {
    headers
        .get("mcp-protocol-version")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Extract Last-Event-ID from headers for SSE resumability
fn extract_last_event_id(headers: &HeaderMap) -> Option<String> {
    headers
        .get("last-event-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

// Validation functions moved to ultrafast_mcp_core::validation

/// Validate Origin header for security using core validation
fn validate_origin_header(headers: &HeaderMap, config: &HttpTransportConfig) -> bool {
    let origin = headers.get("origin").and_then(|v| v.to_str().ok());

    validate_origin(origin, config.allow_origin.as_deref(), &config.host)
}

/// Validate protocol version header using core validation
fn validate_protocol_version_header(version: &str) -> bool {
    validate_protocol_version(version).is_ok()
}

/// Validate session ID format using core validation  
fn validate_session_id_header(session_id: &str) -> bool {
    validate_session_id(session_id).is_ok()
}

// Session ID and event ID generation functions moved to ultrafast_mcp_core::utils

async fn handle_mcp_post(
    State(state): State<Arc<HttpTransportState>>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    // Start request timer for monitoring
    let timer = state
        .metrics
        .as_ref()
        .map(|metrics| RequestTimer::start("mcp_post", metrics.clone()));

    let result = handle_mcp_post_internal(state, headers, body).await;

    // Record metrics
    if let Some(timer) = timer {
        let success = result.status() == StatusCode::OK;
        timer.finish(success).await;
    }

    result
}

async fn handle_mcp_post_internal(
    state: Arc<HttpTransportState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    // Validate Origin header
    if !validate_origin_header(&headers, &state.config) {
        return (
            StatusCode::FORBIDDEN,
            Json(JsonRpcResponse::error(
                JsonRpcError::new(-32000, "Origin not allowed".to_string()),
                None,
            )),
        )
            .into_response();
    }

    // Validate protocol version header if present
    if let Some(protocol_version) = extract_protocol_version(&headers) {
        if !validate_protocol_version_header(&protocol_version) {
            return (
                StatusCode::BAD_REQUEST,
                Json(JsonRpcResponse::error(
                    JsonRpcError::new(
                        -32000,
                        format!("Unsupported protocol version: {}", protocol_version),
                    ),
                    None,
                )),
            )
                .into_response();
        }
    }

    // Check if this is an initial connection (initialize request or empty body)
    let is_initial_connection = body.is_empty() || {
        if let Ok(message) = serde_json::from_slice::<JsonRpcMessage>(&body) {
            matches!(message, JsonRpcMessage::Request(req) if req.method == "initialize")
        } else {
            false
        }
    };

    let session_id = if is_initial_connection {
        extract_session_id(&headers).unwrap_or_else(generate_session_id)
    } else {
        match extract_session_id(&headers) {
            Some(id) => {
                if !validate_session_id_header(&id) {
                    return Json(JsonRpcResponse::error(
                        JsonRpcError::new(-32000, "Invalid session ID format".to_string()),
                        None,
                    ))
                    .into_response();
                }
                id
            }
            None => {
                return Json(JsonRpcResponse::error(
                    JsonRpcError::new(-32000, "Missing session ID".to_string()),
                    None,
                ))
                .into_response();
            }
        }
    };

    // Store session info
    {
        let mut sessions = state.session_store.write().await;
        sessions
            .entry(session_id.clone())
            .or_insert_with(SessionInfo::new);
    }

    // Try to parse the body as a JSON-RPC message
    let message: std::result::Result<JsonRpcMessage, serde_json::Error> =
        serde_json::from_slice(&body);
    let message = match message {
        Ok(msg) => msg,
        Err(_) => {
            return Json(JsonRpcResponse::error(
                JsonRpcError::new(-32700, "Parse error: Invalid JSON-RPC message".to_string()),
                None,
            ))
            .into_response();
        }
    };

    info!(
        "Processing POST request for session {}: {:?}",
        session_id, message
    );
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
    if !validate_origin_header(&headers, &state.config) {
        return (
            StatusCode::FORBIDDEN,
            Json(JsonRpcResponse::error(
                JsonRpcError::new(-32000, "Origin not allowed".to_string()),
                None,
            )),
        )
            .into_response();
    }

    // Validate protocol version header if present
    if let Some(protocol_version) = extract_protocol_version(&headers) {
        if !validate_protocol_version_header(&protocol_version) {
            return (
                StatusCode::BAD_REQUEST,
                Json(JsonRpcResponse::error(
                    JsonRpcError::new(
                        -32000,
                        format!("Unsupported protocol version: {}", protocol_version),
                    ),
                    None,
                )),
            )
                .into_response();
        }
    }

    let session_id = extract_session_id(&headers).unwrap_or_else(generate_session_id);
    let last_event_id = extract_last_event_id(&headers);

    info!(
        "Processing GET request for session {} (SSE stream){}",
        session_id,
        last_event_id
            .as_ref()
            .map(|id| format!(", resuming from event {}", id))
            .unwrap_or_default()
    );

    // Store session info
    {
        let mut sessions = state.session_store.write().await;
        let session_info = sessions
            .entry(session_id.clone())
            .or_insert_with(SessionInfo::new);
        if let Some(event_id) = &last_event_id {
            session_info.last_event_id = Some(event_id.clone());
        }
    }

    let stream = create_sse_stream(state, session_id, last_event_id);
    Sse::new(stream).into_response()
}

async fn handle_mcp_delete(
    State(state): State<Arc<HttpTransportState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !validate_origin_header(&headers, &state.config) {
        return (
            StatusCode::FORBIDDEN,
            Json(JsonRpcResponse::error(
                JsonRpcError::new(-32000, "Origin not allowed".to_string()),
                None,
            )),
        )
            .into_response();
    }

    // Validate protocol version header if present
    if let Some(protocol_version) = extract_protocol_version(&headers) {
        if !validate_protocol_version_header(&protocol_version) {
            return (
                StatusCode::BAD_REQUEST,
                Json(JsonRpcResponse::error(
                    JsonRpcError::new(
                        -32000,
                        format!("Unsupported protocol version: {}", protocol_version),
                    ),
                    None,
                )),
            )
                .into_response();
        }
    }

    let session_id = extract_session_id(&headers).unwrap_or_else(generate_session_id);

    // Remove session from store
    {
        let mut sessions = state.session_store.write().await;
        sessions.remove(&session_id);
    }

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
    if let Err(e) = state
        .message_sender
        .send((session_id.clone(), JsonRpcMessage::Request(request.clone())))
    {
        error!("Failed to send message to server: {}", e);
        return Json(JsonRpcResponse::error(
            JsonRpcError::new(-32000, format!("Failed to process message: {}", e)),
            request.id,
        ))
        .into_response();
    }

    // Wait for response from server with timeout
    match tokio::time::timeout(
        std::time::Duration::from_secs(5000), // 5 second timeout
        response_receiver.recv(),
    )
    .await
    {
        Ok(Ok((response_session_id, response_message))) => {
            if response_session_id == session_id || response_session_id == "*" {
                // Return the actual response from the server
                match response_message {
                    JsonRpcMessage::Response(response) => (
                        StatusCode::OK,
                        [
                            ("mcp-session-id", response_session_id),
                            (
                                "mcp-protocol-version",
                                state.config.protocol_version.clone(),
                            ),
                        ],
                        Json(response),
                    )
                        .into_response(),
                    _ => {
                        // Unexpected message type
                        Json(JsonRpcResponse::error(
                            JsonRpcError::new(-32000, "Unexpected response type".to_string()),
                            request.id,
                        ))
                        .into_response()
                    }
                }
            } else {
                // Wrong session
                error!(
                    "Received response for wrong session: expected {}, got {}",
                    session_id, response_session_id
                );
                Json(JsonRpcResponse::error(
                    JsonRpcError::new(-32000, "Session mismatch".to_string()),
                    request.id,
                ))
                .into_response()
            }
        }
        Ok(Err(e)) => {
            error!("Failed to receive response: {}", e);
            Json(JsonRpcResponse::error(
                JsonRpcError::new(-32000, format!("Failed to receive response: {}", e)),
                request.id,
            ))
            .into_response()
        }
        Err(_) => {
            error!("Timeout waiting for response from server");
            Json(JsonRpcResponse::error(
                JsonRpcError::new(-32000, "Request timeout".to_string()),
                request.id,
            ))
            .into_response()
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
        )
            .into_response();
    }

    // Return 202 Accepted for notifications and responses
    (StatusCode::ACCEPTED, [("mcp-session-id", session_id)]).into_response()
}

/// Create SSE stream for server-to-client communication
fn create_sse_stream(
    state: Arc<HttpTransportState>,
    session_id: String,
    last_event_id: Option<String>,
) -> impl Stream<Item = std::result::Result<Event, axum::Error>> {
    let response_receiver = state.response_sender.subscribe();
    let enable_resumability = state.config.enable_sse_resumability;

    stream::unfold(
        (
            response_receiver,
            session_id,
            last_event_id,
            enable_resumability,
        ),
        |(mut receiver, session_id, last_event_id, enable_resumability)| async move {
            match receiver.recv().await {
                Ok((msg_session_id, message)) => {
                    if msg_session_id == session_id || msg_session_id == "*" {
                        let event_data = serde_json::to_string(&message).unwrap_or_default();
                        let mut event = Event::default().data(event_data);

                        // Add event ID for resumability if enabled
                        if enable_resumability {
                            let event_id = generate_event_id();
                            event = event.id(event_id);
                        }

                        // Add keep-alive comment
                        event = event.comment("keep-alive");

                        Some((
                            Ok(event),
                            (receiver, session_id, last_event_id, enable_resumability),
                        ))
                    } else {
                        // Skip messages for other sessions, send keep-alive comment
                        Some((
                            Ok(Event::default().comment("keep-alive")),
                            (receiver, session_id, last_event_id, enable_resumability),
                        ))
                    }
                }
                Err(_) => None, // Connection closed
            }
        },
    )
}
