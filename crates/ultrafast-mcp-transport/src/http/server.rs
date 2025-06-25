use crate::{http::session::{SessionStore, MessageQueue}, Transport, TransportError, Result};
use crate::http::{pool::{ConnectionPool, PoolConfig}, rate_limit::{RateLimiter, RateLimitConfig}};
use async_trait::async_trait;
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post, any},
    Json, Router,
};
use axum::response::sse::{Event, Sse, KeepAlive};
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tower_http::cors::CorsLayer;
use ultrafast_mcp_core::protocol::JsonRpcMessage;
use tracing::info;
use tokio::sync::RwLock;

#[cfg(feature = "http")]
use ultrafast_mcp_auth::{TokenValidator, extract_bearer_token};

/// HTTP transport configuration
#[derive(Debug, Clone)]
pub struct HttpTransportConfig {
    pub host: String,
    pub port: u16,
    pub session_timeout_secs: u64,
    pub max_message_retries: u32,
    pub cors_enabled: bool,
    pub auth_required: bool,
    pub protocol_version: String,
    pub enable_streamable_http: bool,
    pub enable_legacy_endpoints: bool,
    // Production features
    pub rate_limit_config: RateLimitConfig,
    pub connection_pool_config: PoolConfig,
    pub request_timeout: Duration,
    pub max_request_size: usize,
    pub enable_compression: bool,
}

impl Default for HttpTransportConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            session_timeout_secs: 300, // 5 minutes
            max_message_retries: 3,
            cors_enabled: true,
            auth_required: false,
            protocol_version: "2025-06-18".to_string(),
            enable_streamable_http: true,
            enable_legacy_endpoints: false,
            // Production defaults
            rate_limit_config: RateLimitConfig::default(),
            connection_pool_config: PoolConfig::default(),
            request_timeout: Duration::from_secs(30),
            max_request_size: 1024 * 1024, // 1MB
            enable_compression: true,
        }
    }
}

/// Shared state for HTTP transport
#[derive(Clone)]
pub struct HttpTransportState {
    pub session_store: SessionStore,
    pub message_queue: MessageQueue,
    pub message_sender: broadcast::Sender<(String, JsonRpcMessage)>,
    pub config: HttpTransportConfig,
    pub rate_limiter: Arc<RateLimiter>,
    pub connection_pool: Arc<ConnectionPool>,
    #[cfg(feature = "http")]
    pub token_validator: Option<TokenValidator>,
}

/// HTTP transport server implementation
pub struct HttpTransportServer {
    state: HttpTransportState,
    message_receiver: broadcast::Receiver<(String, JsonRpcMessage)>,
}

impl HttpTransportServer {
    pub fn new(config: HttpTransportConfig) -> Self {
        let (message_sender, message_receiver) = broadcast::channel(1000);
        
        let rate_limiter = Arc::new(RateLimiter::new(config.rate_limit_config.clone()));
        let connection_pool = Arc::new(ConnectionPool::new(config.connection_pool_config.clone()));
        
        let state = HttpTransportState {
            session_store: SessionStore::new(config.session_timeout_secs),
            message_queue: MessageQueue::new(config.max_message_retries),
            message_sender,
            rate_limiter: rate_limiter.clone(),
            connection_pool: connection_pool.clone(),
            config,
            #[cfg(feature = "http")]
            token_validator: None,
        };
        
        // Start background cleanup tasks
        crate::http::rate_limit::start_rate_limit_cleanup(rate_limiter);
        crate::http::pool::start_pool_cleanup(connection_pool);
        
        Self {
            state,
            message_receiver,
        }
    }
    
    /// Get a message receiver to subscribe to incoming messages
    /// This enables the MCP server to process HTTP transport messages
    pub fn get_message_receiver(&self) -> broadcast::Receiver<(String, JsonRpcMessage)> {
        self.state.message_sender.subscribe()
    }
    
    /// Get the message sender for response delivery
    pub fn get_message_sender(&self) -> broadcast::Sender<(String, JsonRpcMessage)> {
        self.state.message_sender.clone()
    }
    
    /// Get the message queue for response delivery
    pub fn get_message_queue(&self) -> MessageQueue {
        self.state.message_queue.clone()
    }
    
    /// Get the transport state for access to session management
    pub fn get_state(&self) -> HttpTransportState {
        self.state.clone()
    }
    
    #[cfg(feature = "http")]
    pub fn with_auth(mut self, token_validator: TokenValidator) -> Self {
        self.state.token_validator = Some(token_validator);
        self.state.config.auth_required = true;
        self
    }
    
    /// Start the HTTP server
    pub async fn run(self) -> Result<()> {
        let app = self.create_router();
        
        let addr = format!("{}:{}", self.state.config.host, self.state.config.port);
        let listener = tokio::net::TcpListener::bind(&addr).await
            .map_err(|e| TransportError::ConnectionError { 
                message: format!("Failed to bind to {}: {}", addr, e) 
            })?;
        
        tracing::info!("HTTP MCP server listening on {}", addr);
        
        // Start session cleanup task
        let cleanup_state = self.state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                cleanup_state.session_store.cleanup_expired_sessions().await;
            }
        });
        
        // Start message queueing task - listens to broadcast channel and queues messages
        let queue_state = self.state.clone();
        let mut message_receiver = self.message_receiver;
        tokio::spawn(async move {
            info!("Message queueing task started");
            while let Ok((session_id, message)) = message_receiver.recv().await {
                info!("Received message for session {}: {:?}", session_id, message);
                match &message {
                    JsonRpcMessage::Request(_) | JsonRpcMessage::Notification(_) => {
                        // Forward to server for processing but DO NOT queue for client polling
                        // The server will handle the request and queue the response
                        info!("Forwarding request/notification for session {} to server", session_id);
                    }
                    JsonRpcMessage::Response(_) => {
                        // Only queue responses for client polling
                        queue_state.message_queue.enqueue_message(session_id.clone(), message.clone()).await;
                        info!("Queued response for session {}", session_id);
                    }
                }
            }
            info!("Message queueing task ended");
        });
        
        // Use into_make_service() for Axum 0.8 compatibility
        axum::serve(listener, app.into_make_service()).await
            .map_err(|e| TransportError::ConnectionError { 
                message: format!("Server error: {}", e) 
            })?;
        
        Ok(())
    }
    
    fn create_router(&self) -> Router {
        let state = self.state.clone();
        let mut router = Router::new();
        
        if state.config.enable_streamable_http {
            // Primary Streamable HTTP endpoint - unified POST/GET with optional SSE upgrade
            let state_clone = state.clone();
            router = router.route("/mcp", any(move |headers, query, payload| {
                handle_streamable_mcp_stateless(headers, query, payload, state_clone)
            }));
        }
        
        if state.config.enable_legacy_endpoints {
            // Legacy endpoints for backward compatibility
            let state_clone1 = state.clone();
            let state_clone2 = state.clone();
            let state_clone3 = state.clone();
            let state_clone4 = state.clone();
            
            router = router
                .route("/mcp/legacy", post(move |headers, payload| {
                    handle_mcp_request_stateless(headers, payload, state_clone1)
                }))
                .route("/mcp/connect", get(move |headers, query| {
                    handle_mcp_connect_stateless(headers, query, state_clone2)
                }))
                .route("/mcp/messages", get(move |headers, query| {
                    handle_get_messages_stateless(headers, query, state_clone3)
                }))
                .route("/mcp/ack", post(move |headers, payload| {
                    handle_acknowledge_message_stateless(headers, payload, state_clone4)
                }));
        }
        
        if state.config.cors_enabled {
            router = router.layer(CorsLayer::permissive());
        }
        
        router
    }
}

#[async_trait]
impl Transport for HttpTransportServer {
    async fn send_message(&mut self, message: JsonRpcMessage) -> Result<()> {
        // Broadcast to all connected sessions
        // In a real implementation, you'd want to target specific sessions
        let _ = self.state.message_sender.send(("*".to_string(), message));
        Ok(())
    }
    
    async fn receive_message(&mut self) -> Result<JsonRpcMessage> {
        match self.message_receiver.recv().await {
            Ok((_, message)) => Ok(message),
            Err(_) => Err(TransportError::ConnectionClosed.into()),
        }
    }
    
    async fn close(&mut self) -> Result<()> {
        // HTTP transport doesn't need explicit closing
        Ok(())
    }
}

/// Request/response types for HTTP API
#[derive(Debug, Serialize, Deserialize)]
pub struct McpConnectQuery {
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpConnectResponse {
    pub session_id: String,
    pub protocol_version: String,
    pub pending_messages: Vec<JsonRpcMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpRequest {
    pub session_id: String,
    pub message: JsonRpcMessage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpResponse {
    pub success: bool,
    pub message_id: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetMessagesQuery {
    pub session_id: String,
    pub since: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetMessagesResponse {
    pub messages: Vec<JsonRpcMessage>,
    pub has_more: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AckMessageRequest {
    pub session_id: String,
    pub message_id: String,
}

/// Streamable HTTP request/response types
#[derive(Debug, Serialize, Deserialize)]
pub struct StreamableMcpRequest {
    pub session_id: Option<String>,
    pub message: Option<JsonRpcMessage>,
    pub upgrade_to_stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamableMcpResponse {
    pub session_id: String,
    pub protocol_version: String,
    pub message_id: Option<String>,
    pub success: bool,
    pub error: Option<String>,
    pub pending_messages: Option<Vec<JsonRpcMessage>>,
}

/// Handle unified Streamable HTTP endpoint (GET/POST /mcp)
/// This implements the PRD specification for single unified endpoint with optional SSE upgrade
/// Handle SSE upgrade for streaming communication
async fn handle_sse_upgrade(
    query: HashMap<String, String>,
    _headers: HeaderMap,
    state: HttpTransportState,
) -> Response {
    let session_id = query.get("session_id")
        .cloned()
        .unwrap_or_else(|| {
            #[cfg(feature = "http")]
            return ultrafast_mcp_auth::generate_session_id();
            #[cfg(not(feature = "http"))]
            return uuid::Uuid::new_v4().to_string();
        });
    
    // Create or retrieve session
    let _session = state.session_store.create_session(session_id.clone()).await;
    
    // Create SSE stream for this session
    let message_stream = BroadcastStream::new(state.message_sender.subscribe())
        .filter_map(move |result| {
            let session_id = session_id.clone();
            async move {
                match result {
                    Ok((target_session, message)) => {
                        // Only send messages for this session or broadcast messages
                        if target_session == session_id || target_session == "*" {
                            let json_data = serde_json::to_string(&message).ok()?;
                            Some(Ok::<Event, std::convert::Infallible>(Event::default()
                                .event("mcp-message")
                                .data(json_data)))
                        } else {
                            None
                        }
                    }
                    Err(_) => Some(Ok::<Event, std::convert::Infallible>(Event::default()
                        .event("error")
                        .data("Stream error"))),
                }
            }
        });
    
    let sse = Sse::new(message_stream)
        .keep_alive(KeepAlive::default());
    
    sse.into_response()
}

/// Handle regular streamable connection (GET without streaming)
async fn handle_streamable_connect(
    query: HashMap<String, String>,
    state: HttpTransportState,
) -> Response {
    let session_id = query.get("session_id")
        .cloned()
        .unwrap_or_else(|| {
            #[cfg(feature = "http")]
            return ultrafast_mcp_auth::generate_session_id();
            #[cfg(not(feature = "http"))]
            return uuid::Uuid::new_v4().to_string();
        });
    
    // Create or retrieve session
    let _session = state.session_store.create_session(session_id.clone()).await;
    
    // Get pending messages
    let pending_messages = state.message_queue.get_pending_messages(&session_id).await
        .into_iter()
        .map(|qm| qm.message)
        .collect();
    
    let response = StreamableMcpResponse {
        session_id,
        protocol_version: state.config.protocol_version.clone(),
        message_id: None,
        success: true,
        error: None,
        pending_messages: Some(pending_messages),
    };
    
    Json(response).into_response()
}

/// Handle streamable message sending (POST without streaming)
async fn handle_streamable_message(
    request: StreamableMcpRequest,
    state: HttpTransportState,
) -> Response {
    // Only generate a new session ID if session_id is missing
    let session_id = if let Some(sid) = request.session_id.clone() {
        tracing::debug!("Using provided session_id: {}", sid);
        sid
    } else {
        let new_id = {
            #[cfg(feature = "http")]
            { ultrafast_mcp_auth::generate_session_id() }
            #[cfg(not(feature = "http"))]
            { uuid::Uuid::new_v4().to_string() }
        };
        tracing::debug!("Generated new session_id: {}", new_id);
        new_id
    };

    // Only create session if it doesn't exist
    if state.session_store.get_session(&session_id).await.is_none() {
        tracing::debug!("Creating new session: {}", session_id);
        let _session = state.session_store.create_session(session_id.clone()).await;
    }

    // Validate session
    if state.session_store.get_session(&session_id).await.is_none() {
        return (StatusCode::BAD_REQUEST, "Invalid session").into_response();
    }

    if let Some(message) = request.message {
        // Extract message ID for response
        let message_id = match &message {
            JsonRpcMessage::Request(req) => req.id.as_ref().map(|id| format!("{:?}", id)),
            JsonRpcMessage::Response(resp) => resp.id.as_ref().map(|id| format!("{:?}", id)),
            JsonRpcMessage::Notification(_) => None,
        };

        // Broadcast message to transport handlers for processing
        let _ = state.message_sender.send((session_id.clone(), message));

        // Wait with exponential backoff for the server to process the message and queue the response
        // Check for response multiple times with increasing delays
        let mut check_delay = 10u64; // Start with 10ms
        let max_delay = 500u64; // Cap at 500ms per check
        let max_total_time = 2000u64; // Maximum 2 seconds total
        let start_time = std::time::Instant::now();
        
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(check_delay)).await;
            
            // Check if we have a response yet
            let current_pending = state.message_queue.get_pending_messages(&session_id).await;
            let response_count = current_pending.len();
            
            if response_count > 0 {
                tracing::debug!("Found {} pending messages after {}ms", response_count, start_time.elapsed().as_millis());
                break;
            }
            
            // Exponential backoff
            check_delay = (check_delay * 2).min(max_delay);
            
            // Timeout check
            if start_time.elapsed().as_millis() > max_total_time as u128 {
                tracing::debug!("Timed out waiting for response after {}ms", start_time.elapsed().as_millis());
                break;
            }
        }

        // Get any pending messages (excluding the original request message)
        let pending_messages = state.message_queue.get_pending_messages(&session_id).await
            .into_iter()
            .filter(|qm| {
                // Filter out the original request message to avoid loops
                if let JsonRpcMessage::Request(req) = &qm.message {
                    if let Some(req_id) = &req.id {
                        if let Some(msg_id) = &message_id {
                            // Compare IDs using Debug format
                            format!("{:?}", req_id) != msg_id.to_string()
                        } else {
                            true
                        }
                    } else {
                        true
                    }
                } else {
                    true
                }
            })
            .map(|qm| qm.message)
            .collect();

        let response = StreamableMcpResponse {
            session_id,
            protocol_version: state.config.protocol_version.clone(),
            message_id,
            success: true,
            error: None,
            pending_messages: Some(pending_messages),
        };

        Json(response).into_response()
    } else {
        // If no message, treat as connection request and return pending messages
        let pending_messages = state.message_queue.get_pending_messages(&session_id).await
            .into_iter()
            .map(|qm| qm.message)
            .collect();
        let response = StreamableMcpResponse {
            session_id,
            protocol_version: state.config.protocol_version.clone(),
            message_id: None,
            success: true,
            error: None,
            pending_messages: Some(pending_messages),
        };
        Json(response).into_response()
    }
}

/// Validate authentication and rate limiting for requests
async fn validate_authentication_and_limits(
    headers: &HeaderMap,
    state: &HttpTransportState,
    client_identifier: &str,
) -> std::result::Result<(), Response> {
    // Rate limiting check first
    if let Err(e) = state.rate_limiter.check_rate_limit(client_identifier).await {
        return Err((StatusCode::TOO_MANY_REQUESTS, format!("Rate limit exceeded: {}", e)).into_response());
    }
    
    // Then authentication
    #[cfg(feature = "http")]
    if state.config.auth_required {
        if let Some(validator) = &state.token_validator {
            if let Some(auth_header) = headers.get("authorization") {
                if let Ok(auth_str) = auth_header.to_str() {
                    match extract_bearer_token(auth_str) {
                        Ok(token) => {
                            if let Err(e) = validator.validate_token(token).await {
                                return Err((StatusCode::UNAUTHORIZED, format!("Authentication failed: {}", e)).into_response());
                            }
                        }
                        Err(e) => {
                            return Err((StatusCode::UNAUTHORIZED, format!("Invalid authorization header: {}", e)).into_response());
                        }
                    }
                } else {
                    return Err((StatusCode::UNAUTHORIZED, "Invalid authorization header").into_response());
                }
            } else {
                return Err((StatusCode::UNAUTHORIZED, "Authorization required").into_response());
            }
        }
    }
    Ok(())
}

/// Extract client identifier for rate limiting (IP address or session ID)
fn extract_client_identifier(headers: &HeaderMap, query: &HashMap<String, String>) -> String {
    // Try to get session ID first
    if let Some(session_id) = query.get("session_id") {
        return session_id.clone();
    }
    
    // Fall back to IP address
    headers.get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            headers.get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "unknown".to_string())
}

/// Validate authentication for requests
async fn validate_authentication(
    headers: &HeaderMap,
    state: &HttpTransportState,
) -> std::result::Result<(), Response> {
    #[cfg(feature = "http")]
    if state.config.auth_required {
        if let Some(validator) = &state.token_validator {
            if let Some(auth_header) = headers.get("authorization") {
                if let Ok(auth_str) = auth_header.to_str() {
                    match extract_bearer_token(auth_str) {
                        Ok(token) => {
                            if let Err(e) = validator.validate_token(token).await {
                                return Err((StatusCode::UNAUTHORIZED, format!("Authentication failed: {}", e)).into_response());
                            }
                        }
                        Err(e) => {
                            return Err((StatusCode::UNAUTHORIZED, format!("Invalid authorization header: {}", e)).into_response());
                        }
                    }
                } else {
                    return Err((StatusCode::UNAUTHORIZED, "Invalid authorization header").into_response());
                }
            } else {
                return Err((StatusCode::UNAUTHORIZED, "Authorization required").into_response());
            }
        }
    }
    Ok(())
}

/// Handle MCP connection establishment (GET /mcp)
/// Stateless handler functions for Axum 0.8 compatibility

/// Stateless version of handle_streamable_mcp
async fn handle_streamable_mcp_stateless(
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    body: Option<Json<StreamableMcpRequest>>,
    state: HttpTransportState,
) -> Response {
    // Check HTTP method from headers or body
    let wants_streaming = headers.get("upgrade")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_lowercase().contains("sse"))
        .unwrap_or(false) ||
        headers.get("accept")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("text/event-stream"))
        .unwrap_or(false);

    // For stateless handling, we'll treat it as a POST if body is present, otherwise GET
    if let Some(Json(request)) = body {
        if wants_streaming || request.upgrade_to_stream.unwrap_or(false) {
            // Client requested streaming upgrade in POST
            handle_sse_upgrade(query, headers, state).await
        } else if request.message.is_none() {
            // Initial connection - no message provided, treat as connection request
            handle_streamable_connect(query, state).await
        } else {
            // Regular POST - send message and return response
            handle_streamable_message(request, state).await
        }
    } else {
        if wants_streaming {
            // Upgrade to SSE streaming
            handle_sse_upgrade(query, headers, state).await
        } else {
            // Regular GET - establish session and return pending messages
            handle_streamable_connect(query, state).await
        }
    }
}

/// Stateless version of handle_mcp_request
async fn handle_mcp_request_stateless(
    headers: HeaderMap,
    Json(request): Json<McpRequest>,
    state: HttpTransportState,
) -> Response {
    let client_id = extract_client_identifier(&headers, &HashMap::new());
    
    // Validate authentication and rate limits
    if let Err(auth_response) = validate_authentication_and_limits(&headers, &state, &client_id).await {
        return auth_response;
    }

    // Validate session
    if state.session_store.get_session(&request.session_id).await.is_none() {
        return (StatusCode::BAD_REQUEST, "Invalid session").into_response();
    }

    // Process the message (broadcast to message handlers)
    let message_id = match &request.message {
        JsonRpcMessage::Request(req) => req.id.as_ref().map(|id| format!("{:?}", id)),
        JsonRpcMessage::Response(resp) => resp.id.as_ref().map(|id| format!("{:?}", id)),
        JsonRpcMessage::Notification(_) => None,
    };

    // Broadcast message to transport handlers for processing
    // Don't queue incoming messages - only responses should be queued
    let _ = state.message_sender.send((request.session_id.clone(), request.message));

    let response = McpResponse {
        success: true,
        message_id,
        error: None,
    };

    Json(response).into_response()
}

/// Stateless version of handle_mcp_connect
async fn handle_mcp_connect_stateless(
    headers: HeaderMap,
    Query(params): Query<McpConnectQuery>,
    state: HttpTransportState,
) -> Response {
    // Validate authentication if required
    if let Err(auth_response) = validate_authentication(&headers, &state).await {
        return auth_response;
    }

    let session_id = params.session_id.unwrap_or_else(|| {
        format!("session_{}", uuid::Uuid::new_v4())
    });

    // Create or refresh session
    let _session = state.session_store.create_session(session_id.clone()).await;

    // Get any pending messages for this session
    let pending_messages = state.message_queue.get_pending_messages(&session_id).await
        .into_iter()
        .map(|qm| qm.message)
        .collect();

    let response = McpConnectResponse {
        session_id,
        protocol_version: state.config.protocol_version.clone(),
        pending_messages,
    };

    Json(response).into_response()
}

/// Stateless version of handle_get_messages
async fn handle_get_messages_stateless(
    headers: HeaderMap,
    Query(params): Query<GetMessagesQuery>,
    state: HttpTransportState,
) -> Response {
    // Validate authentication if required
    if let Err(auth_response) = validate_authentication(&headers, &state).await {
        return auth_response;
    }

    // Validate session
    if state.session_store.get_session(&params.session_id).await.is_none() {
        return (StatusCode::BAD_REQUEST, "Invalid session").into_response();
    }

    let messages = state.message_queue.get_pending_messages(&params.session_id).await
        .into_iter()
        .filter(|msg| {
            if let Some(since) = params.since {
                msg.timestamp > since
            } else {
                true
            }
        })
        .map(|qm| qm.message)
        .collect::<Vec<_>>();

    let response = GetMessagesResponse {
        messages,
        has_more: false, // Simplified for now
    };

    Json(response).into_response()
}

/// Stateless version of handle_acknowledge_message
async fn handle_acknowledge_message_stateless(
    headers: HeaderMap,
    Json(request): Json<AckMessageRequest>,
    state: HttpTransportState,
) -> Response {
    // Validate authentication if required
    if let Err(auth_response) = validate_authentication(&headers, &state).await {
        return auth_response;
    }

    // Validate session
    if state.session_store.get_session(&request.session_id).await.is_none() {
        return (StatusCode::BAD_REQUEST, "Invalid session").into_response();
    }

    // Acknowledge the message
    state.message_queue.acknowledge_message(&request.session_id, &request.message_id).await;

    let response = McpResponse {
        success: true,
        message_id: Some(request.message_id),
        error: None,
    };

    Json(response).into_response()
}
