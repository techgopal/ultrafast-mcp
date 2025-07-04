use crate::{Result, Transport, TransportError};
use async_trait::async_trait;
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};
use ultrafast_mcp_core::protocol::{JsonRpcMessage, RequestId};

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

/// Rate limiting middleware
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
            .unwrap()
            .as_secs();

        let minute_timestamp = now / 60;

        let mut count_data = self.request_count.lock().unwrap();
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

/// Progress tracking middleware
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
        if let JsonRpcMessage::Request(ref mut req) = message {
            if req.method.contains("tools/call") || req.method.contains("resources/read") {
                // Add progress tracking metadata
                if let Some(ref mut params) = req.params {
                    if let Some(obj) = params.as_object_mut() {
                        obj.insert(
                            "_timeout".to_string(),
                            serde_json::Value::Number(serde_json::Number::from(
                                self.timeout_seconds,
                            )),
                        );
                        obj.insert(
                            "_start_time".to_string(),
                            serde_json::Value::Number(serde_json::Number::from(
                                SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap()
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
            if let Some(ref result) = resp.result {
                if let Some(start_time) = result.get("_start_time").and_then(|v| v.as_u64()) {
                    let elapsed = match SystemTime::now().duration_since(UNIX_EPOCH) {
                        Ok(duration) => duration.as_secs() - start_time,
                        Err(_) => 0,
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

/// Validation middleware for message schema validation
pub struct ValidationMiddleware {
    strict_mode: bool,
    allowed_methods: Vec<String>,
    max_message_size: usize,
    max_params_depth: usize,
}

impl ValidationMiddleware {
    pub fn new() -> Self {
        Self {
            strict_mode: false,
            allowed_methods: vec![
                // MCP 2025-06-18 core methods
                "initialize".to_string(),
                "initialized".to_string(),
                "shutdown".to_string(),
                "exit".to_string(),
                "ping".to_string(),
                "pong".to_string(),
                // Tools methods
                "tools/list".to_string(),
                "tools/call".to_string(),
                // Resources methods
                "resources/list".to_string(),
                "resources/read".to_string(),
                "resources/subscribe".to_string(),
                "resources/unsubscribe".to_string(),
                // Prompts methods
                "prompts/list".to_string(),
                "prompts/get".to_string(),
                // Logging methods
                "logging/log".to_string(),
                // Client methods
                "sampling/sample".to_string(),
                "roots/list".to_string(),
                "roots/read".to_string(),
                "elicitation/request".to_string(),
                "elicitation/respond".to_string(),
                // Completion methods
                "completion/list".to_string(),
                "completion/get".to_string(),
            ],
            max_message_size: 10 * 1024 * 1024, // 10MB
            max_params_depth: 10,
        }
    }

    pub fn strict() -> Self {
        Self {
            strict_mode: true,
            allowed_methods: vec![
                // MCP 2025-06-18 core methods
                "initialize".to_string(),
                "initialized".to_string(),
                "shutdown".to_string(),
                "exit".to_string(),
                "ping".to_string(),
                "pong".to_string(),
                // Tools methods
                "tools/list".to_string(),
                "tools/call".to_string(),
                // Resources methods
                "resources/list".to_string(),
                "resources/read".to_string(),
                "resources/subscribe".to_string(),
                "resources/unsubscribe".to_string(),
                // Prompts methods
                "prompts/list".to_string(),
                "prompts/get".to_string(),
                // Logging methods
                "logging/log".to_string(),
                // Client methods
                "sampling/sample".to_string(),
                "roots/list".to_string(),
                "roots/read".to_string(),
                "elicitation/request".to_string(),
                "elicitation/respond".to_string(),
                // Completion methods
                "completion/list".to_string(),
                "completion/get".to_string(),
            ],
            max_message_size: 5 * 1024 * 1024, // 5MB in strict mode
            max_params_depth: 5,
        }
    }

    pub fn with_allowed_methods(mut self, methods: Vec<String>) -> Self {
        self.allowed_methods = methods;
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

    fn validate_method(&self, method: &str) -> Result<()> {
        if !self.allowed_methods.contains(&method.to_string()) {
            return Err(TransportError::ProtocolError {
                message: format!("Method '{}' not allowed", method),
            });
        }
        Ok(())
    }

    fn validate_request_id(&self, id: &Option<RequestId>) -> Result<()> {
        match id {
            None => {
                if self.strict_mode {
                    return Err(TransportError::ProtocolError {
                        message: "Request ID required in strict mode".to_string(),
                    });
                }
            }
            Some(RequestId::String(s)) => {
                if s.is_empty() {
                    return Err(TransportError::ProtocolError {
                        message: "Request ID cannot be empty string".to_string(),
                    });
                }
                // Check for potential injection patterns
                if s.contains('\0') || s.contains('\n') || s.contains('\r') {
                    return Err(TransportError::ProtocolError {
                        message: "Request ID contains invalid characters".to_string(),
                    });
                }
                // Check for reasonable length
                if s.len() > 100 {
                    return Err(TransportError::ProtocolError {
                        message: "Request ID string too long".to_string(),
                    });
                }
            }
            Some(RequestId::Number(n)) => {
                // Check for reasonable range
                if *n < -999999999 || *n > 999999999 {
                    return Err(TransportError::ProtocolError {
                        message: "Request ID number out of reasonable range".to_string(),
                    });
                }
            }
        }
        Ok(())
    }

    fn sanitize_value(&self, value: &mut Value, depth: usize) -> Result<()> {
        if depth > self.max_params_depth {
            return Err(TransportError::ProtocolError {
                message: format!(
                    "Parameter depth exceeds maximum of {}",
                    self.max_params_depth
                ),
            });
        }

        match value {
            Value::String(s) => {
                // Sanitize strings
                if s.len() > 1024 * 1024 {
                    // 1MB max string length
                    return Err(TransportError::ProtocolError {
                        message: "String parameter too long".to_string(),
                    });
                }
                // Remove null bytes and control characters
                if s.contains('\0') {
                    *s = s.replace('\0', "");
                }
            }
            Value::Array(arr) => {
                // Sanitize arrays
                if arr.len() > 10000 {
                    // Max 10k array elements
                    return Err(TransportError::ProtocolError {
                        message: "Array parameter too large".to_string(),
                    });
                }
                for item in arr.iter_mut() {
                    self.sanitize_value(item, depth + 1)?;
                }
            }
            Value::Object(obj) => {
                // Sanitize objects
                if obj.len() > 1000 {
                    // Max 1k object keys
                    return Err(TransportError::ProtocolError {
                        message: "Object parameter too large".to_string(),
                    });
                }
                for (key, val) in obj.iter_mut() {
                    // Validate key names
                    if key.starts_with('_') && key != "_meta" {
                        return Err(TransportError::ProtocolError {
                            message: format!("Reserved key name '{}' not allowed", key),
                        });
                    }
                    if key.contains('\0') || key.contains('\n') || key.contains('\r') {
                        return Err(TransportError::ProtocolError {
                            message: "Object key contains invalid characters".to_string(),
                        });
                    }
                    self.sanitize_value(val, depth + 1)?;
                }
            }
            _ => {} // Numbers, booleans, null are fine
        }
        Ok(())
    }

    fn validate_uri(&self, uri: &str) -> Result<()> {
        if uri.len() > 2048 {
            return Err(TransportError::ProtocolError {
                message: "URI too long".to_string(),
            });
        }

        // Basic URI validation
        if !uri.contains("://") && !uri.starts_with("file://") && !uri.starts_with("config://") {
            return Err(TransportError::ProtocolError {
                message: "Invalid URI format".to_string(),
            });
        }

        // Check for path traversal attempts (only reject actual traversal patterns)
        if uri.contains("/../")
            || uri.contains("\\..\\")
            || uri.ends_with("/..")
            || uri.ends_with("\\..")
        {
            return Err(TransportError::ProtocolError {
                message: "URI contains path traversal attempt".to_string(),
            });
        }

        Ok(())
    }

    fn validate_mcp_specific(&self, method: &str, params: &mut Option<Value>) -> Result<()> {
        match method {
            "initialize" => {
                if let Some(ref mut params_obj) = params {
                    if let Some(obj) = params_obj.as_object_mut() {
                        // Validate protocol version
                        if let Some(version) = obj.get("protocolVersion") {
                            if let Some(version_str) = version.as_str() {
                                if version_str != "2025-06-18" && version_str != "2024-11-05" {
                                    return Err(TransportError::ProtocolError {
                                        message: format!(
                                            "Unsupported protocol version: {}",
                                            version_str
                                        ),
                                    });
                                }
                            }
                        }
                    }
                }
            }
            "tools/call" => {
                if let Some(ref mut params_obj) = params {
                    if let Some(obj) = params_obj.as_object_mut() {
                        // Validate tool name
                        if let Some(tool_name) = obj.get("name") {
                            if let Some(name_str) = tool_name.as_str() {
                                if name_str.starts_with('_') {
                                    return Err(TransportError::ProtocolError {
                                        message: "Tool name cannot start with underscore"
                                            .to_string(),
                                    });
                                }
                                if name_str.len() > 100 {
                                    return Err(TransportError::ProtocolError {
                                        message: "Tool name too long".to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
            "resources/read" | "resources/subscribe" | "resources/unsubscribe" => {
                if let Some(ref mut params_obj) = params {
                    if let Some(obj) = params_obj.as_object_mut() {
                        // Validate URI
                        if let Some(uri) = obj.get("uri") {
                            if let Some(uri_str) = uri.as_str() {
                                self.validate_uri(uri_str)?;
                            }
                        }
                    }
                }
            }
            "logging/log" => {
                if let Some(ref mut params_obj) = params {
                    if let Some(obj) = params_obj.as_object_mut() {
                        // Validate log level
                        if let Some(level) = obj.get("level") {
                            if let Some(level_str) = level.as_str() {
                                let valid_levels = [
                                    "emergency",
                                    "alert",
                                    "critical",
                                    "error",
                                    "warning",
                                    "notice",
                                    "info",
                                    "debug",
                                ];
                                if !valid_levels.contains(&level_str) {
                                    return Err(TransportError::ProtocolError {
                                        message: format!("Invalid log level: {}", level_str),
                                    });
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
}

impl Default for ValidationMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TransportMiddleware for ValidationMiddleware {
    async fn process_outgoing(&self, message: &mut JsonRpcMessage) -> Result<()> {
        // Validate outgoing message structure
        match message {
            JsonRpcMessage::Request(req) => {
                // Basic JSON-RPC validation
                if req.jsonrpc != "2.0" {
                    return Err(TransportError::ProtocolError {
                        message: "Invalid JSON-RPC version".to_string(),
                    });
                }

                // Validate method
                self.validate_method(&req.method)?;

                // Validate request ID
                self.validate_request_id(&req.id)?;

                // Sanitize parameters
                if let Some(ref mut params) = req.params {
                    self.sanitize_value(params, 0)?;
                }

                // MCP-specific validation
                self.validate_mcp_specific(&req.method, &mut req.params)?;
            }
            JsonRpcMessage::Response(resp) => {
                if resp.jsonrpc != "2.0" {
                    return Err(TransportError::ProtocolError {
                        message: "Invalid JSON-RPC version".to_string(),
                    });
                }

                // Validate response ID
                self.validate_request_id(&resp.id)?;

                // Sanitize result/error
                if let Some(ref mut result) = resp.result {
                    self.sanitize_value(result, 0)?;
                }
                if let Some(ref mut error) = resp.error {
                    // Sanitize error data if present
                    if let Some(ref mut data) = error.data {
                        self.sanitize_value(data, 0)?;
                    }
                }
            }
            JsonRpcMessage::Notification(notif) => {
                if notif.jsonrpc != "2.0" {
                    return Err(TransportError::ProtocolError {
                        message: "Invalid JSON-RPC version".to_string(),
                    });
                }

                // Validate method
                self.validate_method(&notif.method)?;

                // Sanitize parameters
                if let Some(ref mut params) = notif.params {
                    self.sanitize_value(params, 0)?;
                }

                // MCP-specific validation
                self.validate_mcp_specific(&notif.method, &mut notif.params)?;
            }
        }
        Ok(())
    }

    async fn process_incoming(&self, message: &mut JsonRpcMessage) -> Result<()> {
        // Check message size
        let message_size = serde_json::to_string(message).unwrap_or_default().len();
        if message_size > self.max_message_size {
            return Err(TransportError::ProtocolError {
                message: format!(
                    "Message size {} exceeds maximum of {}",
                    message_size, self.max_message_size
                ),
            });
        }

        // Apply same validation as outgoing
        self.process_outgoing(message).await
    }
}

/// Transport wrapper with middleware support
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
        let middleware = if strict {
            ValidationMiddleware::strict()
        } else {
            ValidationMiddleware::new()
        };
        self.with_middleware(Box::new(middleware))
    }
}

#[async_trait]
impl<T: Transport> Transport for MiddlewareTransport<T> {
    async fn send_message(&mut self, mut message: JsonRpcMessage) -> Result<()> {
        // Process through outgoing middleware
        for middleware in &self.middlewares {
            middleware.process_outgoing(&mut message).await?;
        }

        // Send through inner transport
        self.inner.send_message(message).await
    }

    async fn receive_message(&mut self) -> Result<JsonRpcMessage> {
        // Receive from inner transport
        let mut message = self.inner.receive_message().await?;

        // Process through incoming middleware
        for middleware in &self.middlewares {
            middleware.process_incoming(&mut message).await?;
        }

        Ok(message)
    }

    async fn close(&mut self) -> Result<()> {
        self.inner.close().await
    }
}
