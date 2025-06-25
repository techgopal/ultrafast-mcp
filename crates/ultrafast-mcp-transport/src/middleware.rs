use crate::{Transport, TransportError, Result};
use async_trait::async_trait;
use ultrafast_mcp_core::protocol::JsonRpcMessage;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn, debug};

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
            },
            JsonRpcMessage::Response(resp) if self.log_responses => {
                if resp.error.is_some() {
                    warn!("Outgoing error response (id: {:?}): {:?}", resp.id, resp.error);
                } else {
                    info!("Outgoing response (id: {:?})", resp.id);
                }
                debug!("Response details: {:?}", resp);
            },
            JsonRpcMessage::Notification(notif) if self.log_notifications => {
                info!("Outgoing notification: {}", notif.method);
                debug!("Notification details: {:?}", notif);
            },
            _ => {}
        }
        Ok(())
    }
    
    async fn process_incoming(&self, message: &mut JsonRpcMessage) -> Result<()> {
        match message {
            JsonRpcMessage::Request(req) if self.log_requests => {
                info!("Incoming request: {} (id: {:?})", req.method, req.id);
                debug!("Request details: {:?}", req);
            },
            JsonRpcMessage::Response(resp) if self.log_responses => {
                if resp.error.is_some() {
                    warn!("Incoming error response (id: {:?}): {:?}", resp.id, resp.error);
                } else {
                    info!("Incoming response (id: {:?})", resp.id);
                }
                debug!("Response details: {:?}", resp);
            },
            JsonRpcMessage::Notification(notif) if self.log_notifications => {
                info!("Incoming notification: {}", notif.method);
                debug!("Notification details: {:?}", notif);
            },
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
                }.into());
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
                        obj.insert("_timeout".to_string(), serde_json::Value::Number(
                            serde_json::Number::from(self.timeout_seconds)
                        ));
                        obj.insert("_start_time".to_string(), serde_json::Value::Number(
                            serde_json::Number::from(
                                SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs()
                            )
                        ));
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
                if let Some(start_time) = result.get("_start_time")
                    .and_then(|v| v.as_u64()) 
                {
                    let elapsed = match SystemTime::now().duration_since(UNIX_EPOCH) {
                        Ok(duration) => duration.as_secs() - start_time,
                        Err(_) => 0,
                    };
                    
                    if elapsed > self.timeout_seconds {
                        warn!("Request timed out after {} seconds (id: {:?})", elapsed, resp.id);
                    } else {
                        debug!("Request completed in {} seconds (id: {:?})", elapsed, resp.id);
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
}

impl ValidationMiddleware {
    pub fn new() -> Self {
        Self {
            strict_mode: false,
        }
    }
    
    pub fn strict() -> Self {
        Self {
            strict_mode: true,
        }
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
                if req.jsonrpc != "2.0" {
                    return Err(TransportError::ProtocolError {
                        message: "Invalid JSON-RPC version".to_string(),
                    }.into());
                }
                if req.id.is_none() && self.strict_mode {
                    return Err(TransportError::ProtocolError {
                        message: "Request ID required in strict mode".to_string(),
                    }.into());
                }
            },
            JsonRpcMessage::Response(resp) => {
                if resp.jsonrpc != "2.0" {
                    return Err(TransportError::ProtocolError {
                        message: "Invalid JSON-RPC version".to_string(),
                    }.into());
                }
            },
            JsonRpcMessage::Notification(notif) => {
                if notif.jsonrpc != "2.0" {
                    return Err(TransportError::ProtocolError {
                        message: "Invalid JSON-RPC version".to_string(),
                    }.into());
                }
            },
        }
        Ok(())
    }
    
    async fn process_incoming(&self, message: &mut JsonRpcMessage) -> Result<()> {
        // Validate incoming message structure (same logic)
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
