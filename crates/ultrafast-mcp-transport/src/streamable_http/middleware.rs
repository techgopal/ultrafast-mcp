use crate::{Result, Transport, TransportError};
use async_trait::async_trait;
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};
use ultrafast_mcp_core::protocol::{JsonRpcMessage, RequestId};

// Static constants to reduce string allocations
const TIMEOUT_KEY: &str = "_timeout";
const START_TIME_KEY: &str = "_start_time";

/// Middleware trait for HTTP transport
#[async_trait]
pub trait TransportMiddleware: Send + Sync {
    /// Process outgoing message
    async fn process_outgoing(&self, message: &mut JsonRpcMessage) -> Result<()>;

    /// Process incoming message
    async fn process_incoming(&self, message: &mut JsonRpcMessage) -> Result<()>;
}

/// Logging middleware for request/response tracking
pub struct LoggingMiddleware {
    log_requests: bool,
    log_responses: bool,
    log_notifications: bool,
}

impl LoggingMiddleware {
    pub fn new() -> Self {
        Self {
            log_requests: true,
            log_responses: true,
            log_notifications: true,
        }
    }

    pub fn requests_only() -> Self {
        Self {
            log_requests: true,
            log_responses: false,
            log_notifications: false,
        }
    }

    pub fn with_notifications(mut self, log_notifications: bool) -> Self {
        self.log_notifications = log_notifications;
        self
    }
}

impl Default for LoggingMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TransportMiddleware for LoggingMiddleware {
    async fn process_outgoing(&self, message: &mut JsonRpcMessage) -> Result<()> {
        match message {
            JsonRpcMessage::Request(req) if self.log_requests => {
                info!("Outgoing request: {} (id: {:?})", req.method, req.id);
                debug!("Request details: {:?}", req);
            }
            JsonRpcMessage::Response(resp) if self.log_responses => {
                if resp.error.is_some() {
                    warn!(
                        "Outgoing error response (id: {:?}): {:?}",
                        resp.id, resp.error
                    );
                } else {
                    info!("Outgoing response (id: {:?})", resp.id);
                }
                debug!("Response details: {:?}", resp);
            }
            JsonRpcMessage::Notification(notif) if self.log_notifications => {
                info!("Outgoing notification: {}", notif.method);
                debug!("Notification details: {:?}", notif);
            }
            _ => {}
        }
        Ok(())
    }

    async fn process_incoming(&self, message: &mut JsonRpcMessage) -> Result<()> {
        match message {
            JsonRpcMessage::Request(req) if self.log_requests => {
                info!("Incoming request: {} (id: {:?})", req.method, req.id);
                debug!("Request details: {:?}", req);
            }
            JsonRpcMessage::Response(resp) if self.log_responses => {
                if resp.error.is_some() {
                    warn!(
                        "Incoming error response (id: {:?}): {:?}",
                        resp.id, resp.error
                    );
                } else {
                    info!("Incoming response (id: {:?})", resp.id);
                }
                debug!("Response details: {:?}", resp);
            }
            JsonRpcMessage::Notification(notif) if self.log_notifications => {
                info!("Incoming notification: {}", notif.method);
                debug!("Notification details: {:?}", notif);
            }
            _ => {}
        }
        Ok(())
    }
}

/// Rate limiting middleware with improved error handling
pub struct RateLimitMiddleware {
    max_requests_per_minute: u32,
    request_count: std::sync::Arc<std::sync::Mutex<(u64, u32)>>, // (timestamp, count)
}

impl RateLimitMiddleware {
    pub fn new(max_requests_per_minute: u32) -> Self {
        Self {
            max_requests_per_minute,
            request_count: std::sync::Arc::new(std::sync::Mutex::new((0, 0))),
        }
    }

    fn check_rate_limit(&self) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| TransportError::InternalError {
                message: "Failed to get system time".to_string(),
            })?
            .as_secs();

        let minute_timestamp = now / 60;

        let mut count_data =
            self.request_count
                .lock()
                .map_err(|_| TransportError::InternalError {
                    message: "Failed to acquire rate limit lock".to_string(),
                })?;

        let (last_minute, count) = *count_data;

        if last_minute == minute_timestamp {
            if count >= self.max_requests_per_minute {
                return Err(TransportError::NetworkError {
                    message: "Rate limit exceeded".to_string(),
                });
            }
            *count_data = (minute_timestamp, count + 1);
        } else {
            *count_data = (minute_timestamp, 1);
        }

        Ok(())
    }
}

#[async_trait]
impl TransportMiddleware for RateLimitMiddleware {
    async fn process_outgoing(&self, _message: &mut JsonRpcMessage) -> Result<()> {
        // Only apply rate limiting to outgoing requests
        self.check_rate_limit()
    }

    async fn process_incoming(&self, _message: &mut JsonRpcMessage) -> Result<()> {
        // No rate limiting on incoming messages
        Ok(())
    }
}

/// Progress tracking middleware with improved error handling
pub struct ProgressMiddleware {
    timeout_seconds: u64,
}

impl ProgressMiddleware {
    pub fn new(timeout_seconds: u64) -> Self {
        Self { timeout_seconds }
    }
}

#[async_trait]
impl TransportMiddleware for ProgressMiddleware {
    async fn process_outgoing(&self, message: &mut JsonRpcMessage) -> Result<()> {
        // Add timeout metadata to outgoing requests
        if let JsonRpcMessage::Request(req) = message {
            if req.method.contains("tools/call") || req.method.contains("resources/read") {
                // Add progress tracking metadata
                if let Some(params) = req.params.as_mut() {
                    if let Some(obj) = params.as_object_mut() {
                        obj.insert(
                            TIMEOUT_KEY.to_string(),
                            serde_json::Value::Number(serde_json::Number::from(
                                self.timeout_seconds,
                            )),
                        );
                        obj.insert(
                            START_TIME_KEY.to_string(),
                            serde_json::Value::Number(serde_json::Number::from(
                                SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .map_err(|_| TransportError::InternalError {
                                        message: "Failed to get system time".to_string(),
                                    })?
                                    .as_secs(),
                            )),
                        );
                    }
                }
            }
        }
        Ok(())
    }

    async fn process_incoming(&self, message: &mut JsonRpcMessage) -> Result<()> {
        // Check for timeout on incoming responses
        if let JsonRpcMessage::Response(resp) = message {
            if let Some(result) = resp.result.as_mut() {
                if let Some(start_time) = result.get(START_TIME_KEY).and_then(|v| v.as_u64()) {
                    let elapsed = match SystemTime::now().duration_since(UNIX_EPOCH) {
                        Ok(duration) => duration.as_secs() - start_time,
                        Err(_) => {
                            warn!("Failed to calculate elapsed time for request {:?}", resp.id);
                            0
                        }
                    };

                    if elapsed > self.timeout_seconds {
                        warn!(
                            "Request timed out after {} seconds (id: {:?})",
                            elapsed, resp.id
                        );
                    } else {
                        debug!(
                            "Request completed in {} seconds (id: {:?})",
                            elapsed, resp.id
                        );
                    }
                }
            }
        }
        Ok(())
    }
}

/// Validation middleware for message schema validation with improved error handling and performance
pub struct ValidationMiddleware {
    strict_mode: bool,
    allowed_methods: std::collections::HashSet<&'static str>,
    max_message_size: usize,
    max_params_depth: usize,
}

impl ValidationMiddleware {
    pub fn new() -> Self {
        Self {
            strict_mode: false,
            allowed_methods: std::collections::HashSet::new(),
            max_message_size: 1024 * 1024, // 1MB default
            max_params_depth: 10,
        }
    }

    pub fn strict() -> Self {
        Self {
            strict_mode: true,
            allowed_methods: std::collections::HashSet::new(),
            max_message_size: 1024 * 1024,
            max_params_depth: 10,
        }
    }

    pub fn with_allowed_methods(mut self, methods: Vec<String>) -> Self {
        let leaked: std::collections::HashSet<&'static str> = methods.into_iter().map(|m| Box::leak(m.into_boxed_str()) as &'static str).collect();
        self.allowed_methods = leaked;
        self
    }

    pub fn with_max_message_size(mut self, size: usize) -> Self {
        self.max_message_size = size;
        self
    }

    pub fn with_max_params_depth(mut self, depth: usize) -> Self {
        self.max_params_depth = depth;
        self
    }

    fn get_static_method(method: &str) -> Option<&'static str> {
        // Simple string matching for allowed methods
        match method {
            "initialize" => Some("initialize"),
            "notifications/initialized" => Some("notifications/initialized"),
            "tools/list" => Some("tools/list"),
            "tools/call" => Some("tools/call"),
            "resources/list" => Some("resources/list"),
            "resources/read" => Some("resources/read"),
            "prompts/list" => Some("prompts/list"),
            "prompts/get" => Some("prompts/get"),
            "notifications/progress" => Some("notifications/progress"),
            "notifications/message" => Some("notifications/message"),
            "notifications/cancelled" => Some("notifications/cancelled"),
            "logging/log" => Some("logging/log"),
            "ping" => Some("ping"),
            "pong" => Some("pong"),
            _ => None,
        }
    }

    fn validate_method(&self, method: &str) -> Result<()> {
        // If custom allowed_methods is set, only allow methods in that set
        if !self.allowed_methods.is_empty() {
            if !self.allowed_methods.contains(method) {
                return Err(TransportError::ProtocolError {
                    message: format!("Method '{method}' not allowed"),
                });
            }
            return Ok(());
        }
        
        // If no custom allowed_methods, check against static methods
        if Self::get_static_method(method).is_none() {
            return Err(TransportError::ProtocolError {
                message: format!("Method '{method}' not allowed"),
            });
        }
        Ok(())
    }

    fn validate_request_id(&self, id: &Option<RequestId>) -> Result<()> {
        if self.strict_mode && id.is_none() {
            return Err(TransportError::ProtocolError {
                message: "Request ID required in strict mode".to_string(),
            });
        }
        if let Some(id) = id {
            // Validate ID format and length
            let id_str = id.to_string();
            if id_str.is_empty() || id_str.len() > 100 {
                return Err(TransportError::ProtocolError {
                    message: "Invalid request ID format or length".to_string(),
                });
            }
        }
        Ok(())
    }

    fn sanitize_value(&self, value: &mut Value, depth: usize) -> Result<()> {
        if depth > self.max_params_depth {
            return Err(TransportError::ProtocolError {
                message: format!("Parameter depth exceeds maximum of {}", self.max_params_depth),
            });
        }
        match value {
            Value::Object(obj) => {
                if obj.len() > 1000 {
                    return Err(TransportError::ProtocolError {
                        message: "Object parameter too large".to_string(),
                    });
                }
                for (k, v) in obj.iter_mut() {
                    if k.starts_with("_") && k != "_meta" {
                        return Err(TransportError::ProtocolError {
                            message: format!("Reserved key name '{k}' not allowed"),
                        });
                    }
                    self.sanitize_value(v, depth + 1)?;
                }
            }
            Value::Array(arr) => {
                if arr.len() > 10000 {
                    return Err(TransportError::ProtocolError {
                        message: "Array parameter too large".to_string(),
                    });
                }
                for item in arr.iter_mut() {
                    self.sanitize_value(item, depth + 1)?;
                }
            }
            Value::String(s) => {
                // Remove null bytes
                if s.contains('\0') {
                    *s = s.replace('\0', "");
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn validate_uri(&self, uri: &str) -> Result<()> {
        if uri.is_empty() {
            return Err(TransportError::ProtocolError {
                message: "URI cannot be empty".to_string(),
            });
        }
        if uri.len() > 2048 {
            return Err(TransportError::ProtocolError {
                message: "URI too long".to_string(),
            });
        }
        // Basic URI validation
        if !uri.starts_with("file://") && !uri.starts_with("http://") && !uri.starts_with("https://") {
            return Err(TransportError::ProtocolError {
                message: format!("Unsupported URI scheme: {uri}"),
            });
        }
        // Path traversal check
        if uri.contains("../") || uri.contains("..\\") {
            return Err(TransportError::ProtocolError {
                message: "URI contains path traversal attempt".to_string(),
            });
        }
        Ok(())
    }

    fn validate_mcp_specific(&self, method: &str, params: &mut Option<Value>) -> Result<()> {
        match method {
            "resources/read" => {
                if let Some(params) = params {
                    if let Some(obj) = params.as_object() {
                        if let Some(uri) = obj.get("uri") {
                            if let Some(uri_str) = uri.as_str() {
                                self.validate_uri(uri_str)?;
                            }
                        }
                    }
                }
            }
            "tools/call" => {
                if let Some(params) = params {
                    if let Some(obj) = params.as_object() {
                        // Validate tool call parameters
                        if let Some(name) = obj.get("name") {
                            if let Some(name_str) = name.as_str() {
                                if name_str.is_empty() {
                                    return Err(TransportError::ProtocolError {
                                        message: "Tool name cannot be empty".to_string(),
                                    });
                                }
                                if name_str.starts_with('_') {
                                    return Err(TransportError::ProtocolError {
                                        message: "Tool name cannot start with underscore".to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
            "logging/log" => {
                if let Some(params) = params {
                    if let Some(obj) = params.as_object() {
                        if let Some(level) = obj.get("level") {
                            if let Some(level_str) = level.as_str() {
                                match level_str {
                                    "trace" | "debug" | "info" | "warn" | "error" => {},
                                    _ => {
                                        return Err(TransportError::ProtocolError {
                                            message: "Invalid log level".to_string(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
            "initialize" => {
                if let Some(params) = params {
                    if let Some(obj) = params.as_object() {
                        if let Some(version) = obj.get("protocolVersion") {
                            if let Some(version_str) = version.as_str() {
                                match version_str {
                                    "2025-06-18" | "2025-03-26" | "2024-11-05" => {},
                                    _ => {
                                        return Err(TransportError::ProtocolError {
                                            message: "Unsupported protocol version".to_string(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn check_message_size(&self, message: &JsonRpcMessage) -> Result<()> {
        let vec = serde_json::to_vec(message).unwrap_or_default();
        let size = vec.len();
        let buffer = 1024;
        if size > self.max_message_size + buffer {
            eprintln!("ValidationMiddleware: message size {} exceeds limit {} (buffered limit: {})", size, self.max_message_size, self.max_message_size + buffer);
            eprintln!("Serialized message: {}", String::from_utf8_lossy(&vec));
            return Err(TransportError::ProtocolError {
                message: format!("Message size {} exceeds limit {}", size, self.max_message_size),
            });
        }
        Ok(())
    }

    fn check_jsonrpc_version(&self, version: &str) -> Result<()> {
        if version != "2.0" {
            return Err(TransportError::ProtocolError {
                message: "Invalid JSON-RPC version".to_string(),
            });
        }
        Ok(())
    }
}

impl Default for ValidationMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TransportMiddleware for ValidationMiddleware {
    async fn process_outgoing(&self, message: &mut JsonRpcMessage) -> Result<()> {
        self.check_message_size(message)?;
        match message {
            JsonRpcMessage::Request(req) => {
                self.check_jsonrpc_version(&req.jsonrpc)?;
                self.validate_method(&req.method)?;
                self.validate_request_id(&req.id)?;
                if let Some(params) = req.params.as_mut() {
                    self.sanitize_value(params, 0)?;
                }
                self.validate_mcp_specific(&req.method, &mut req.params)?;
            }
            JsonRpcMessage::Response(resp) => {
                self.check_jsonrpc_version(&resp.jsonrpc)?;
                self.validate_request_id(&resp.id)?;
                if let Some(result) = resp.result.as_mut() {
                    self.sanitize_value(result, 0)?;
                }
                if let Some(error) = resp.error.as_mut() {
                    if let Some(data) = error.data.as_mut() {
                        self.sanitize_value(data, 0)?;
                    }
                }
            }
            JsonRpcMessage::Notification(notif) => {
                self.check_jsonrpc_version(&notif.jsonrpc)?;
                self.validate_method(&notif.method)?;
                if let Some(params) = notif.params.as_mut() {
                    self.sanitize_value(params, 0)?;
                }
            }
        }
        Ok(())
    }

    async fn process_incoming(&self, message: &mut JsonRpcMessage) -> Result<()> {
        self.process_outgoing(message).await
    }
}

/// Transport wrapper that applies middleware
pub struct MiddlewareTransport<T: Transport> {
    inner: T,
    middlewares: Vec<Box<dyn TransportMiddleware>>,
}

impl<T: Transport> MiddlewareTransport<T> {
    pub fn new(transport: T) -> Self {
        Self {
            inner: transport,
            middlewares: Vec::new(),
        }
    }

    pub fn with_middleware(mut self, middleware: Box<dyn TransportMiddleware>) -> Self {
        self.middlewares.push(middleware);
        self
    }

    pub fn add_logging(self) -> Self {
        self.with_middleware(Box::new(LoggingMiddleware::new()))
    }

    pub fn add_rate_limiting(self, max_requests_per_minute: u32) -> Self {
        self.with_middleware(Box::new(RateLimitMiddleware::new(max_requests_per_minute)))
    }

    pub fn add_progress_tracking(self, timeout_seconds: u64) -> Self {
        self.with_middleware(Box::new(ProgressMiddleware::new(timeout_seconds)))
    }

    pub fn add_validation(self, strict: bool) -> Self {
        let validation = if strict {
            ValidationMiddleware::strict()
        } else {
            ValidationMiddleware::new()
        };
        self.with_middleware(Box::new(validation))
    }
}

#[async_trait]
impl<T: Transport> Transport for MiddlewareTransport<T> {
    async fn send_message(&mut self, mut message: JsonRpcMessage) -> Result<()> {
        // Apply outgoing middleware
        for middleware in &self.middlewares {
            middleware.process_outgoing(&mut message).await?;
        }

        self.inner.send_message(message).await
    }

    async fn receive_message(&mut self) -> Result<JsonRpcMessage> {
        let mut message = self.inner.receive_message().await?;

        // Apply incoming middleware
        for middleware in &self.middlewares {
            middleware.process_incoming(&mut message).await?;
        }

        Ok(message)
    }

    async fn close(&mut self) -> Result<()> {
        self.inner.close().await
    }
} 