//! Monitoring middleware for HTTP and transport layers
//!
//! This module provides middleware components for integrating monitoring
//! capabilities into HTTP servers and transport layers.

use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, info};

use crate::metrics::MetricsCollector;

/// HTTP middleware for request monitoring
pub struct HttpMonitoringMiddleware {
    metrics_collector: Arc<MetricsCollector>,
    enabled: bool,
}

impl HttpMonitoringMiddleware {
    /// Create a new HTTP monitoring middleware
    pub fn new(metrics_collector: Arc<MetricsCollector>) -> Self {
        Self {
            metrics_collector,
            enabled: true,
        }
    }

    /// Create a new HTTP monitoring middleware with custom settings
    pub fn with_config(metrics_collector: Arc<MetricsCollector>, enabled: bool) -> Self {
        Self {
            metrics_collector,
            enabled,
        }
    }

    /// Check if the middleware is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Set whether the middleware is enabled
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Process an incoming HTTP request
    pub async fn process_request(&self, method: &str, path: &str) -> RequestContext {
        if !self.enabled {
            return RequestContext::new(method, path);
        }

        let start = Instant::now();
        let request_id = uuid::Uuid::new_v4().to_string();

        debug!(
            "Processing HTTP request: {} {} (id: {})",
            method, path, request_id
        );

        RequestContext {
            method: method.to_string(),
            path: path.to_string(),
            request_id,
            start_time: Some(start),
            metrics_collector: Some(self.metrics_collector.clone()),
        }
    }

    /// Process a completed HTTP request
    pub async fn process_response(
        &self,
        context: &RequestContext,
        status_code: u16,
        success: bool,
    ) {
        if !self.enabled {
            return;
        }

        if let Some(start_time) = context.start_time {
            let duration = start_time.elapsed();

            // Record metrics
            if let Some(metrics) = &context.metrics_collector {
                metrics
                    .record_request(&context.method, duration, success)
                    .await;
            }

            if success {
                info!(
                    "HTTP request completed: {} {} -> {} ({}ms)",
                    context.method,
                    context.path,
                    status_code,
                    duration.as_millis()
                );
            } else {
                error!(
                    "HTTP request failed: {} {} -> {} ({}ms)",
                    context.method,
                    context.path,
                    status_code,
                    duration.as_millis()
                );
            }
        }
    }
}

/// Context for tracking HTTP requests
pub struct RequestContext {
    pub method: String,
    pub path: String,
    pub request_id: String,
    pub start_time: Option<Instant>,
    pub metrics_collector: Option<Arc<MetricsCollector>>,
}

impl RequestContext {
    /// Create a new request context
    pub fn new(method: &str, path: &str) -> Self {
        Self {
            method: method.to_string(),
            path: path.to_string(),
            request_id: uuid::Uuid::new_v4().to_string(),
            start_time: None,
            metrics_collector: None,
        }
    }

    /// Get the request duration if available
    pub fn duration(&self) -> Option<std::time::Duration> {
        self.start_time.map(|start| start.elapsed())
    }

    /// Check if the request is being monitored
    pub fn is_monitored(&self) -> bool {
        self.start_time.is_some() && self.metrics_collector.is_some()
    }
}

/// Transport middleware for connection monitoring
pub struct TransportMonitoringMiddleware {
    metrics_collector: Arc<MetricsCollector>,
    enabled: bool,
}

impl TransportMonitoringMiddleware {
    /// Create a new transport monitoring middleware
    pub fn new(metrics_collector: Arc<MetricsCollector>) -> Self {
        Self {
            metrics_collector,
            enabled: true,
        }
    }

    /// Create a new transport monitoring middleware with custom settings
    pub fn with_config(metrics_collector: Arc<MetricsCollector>, enabled: bool) -> Self {
        Self {
            metrics_collector,
            enabled,
        }
    }

    /// Check if the middleware is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Set whether the middleware is enabled
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Process a transport connection event
    pub async fn process_connection(&self, connection_id: &str, event_type: ConnectionEvent) {
        if !self.enabled {
            return;
        }

        match event_type {
            ConnectionEvent::Connected => {
                info!("Transport connection established: {}", connection_id);
                self.metrics_collector.update_connection_count(1).await;
            }
            ConnectionEvent::Disconnected => {
                info!("Transport connection closed: {}", connection_id);
                self.metrics_collector.update_connection_count(0).await;
            }
            ConnectionEvent::Error(error_msg) => {
                error!(
                    "Transport connection error: {} - {}",
                    connection_id, error_msg
                );
                self.metrics_collector
                    .record_transport_error(&error_msg)
                    .await;
            }
        }
    }

    /// Process a transport message event
    pub async fn process_message(&self, connection_id: &str, event_type: MessageEvent) {
        if !self.enabled {
            return;
        }

        match event_type {
            MessageEvent::Sent(bytes) => {
                debug!(
                    "Transport message sent: {} bytes on {}",
                    bytes, connection_id
                );
                self.metrics_collector.record_transport_send(bytes).await;
            }
            MessageEvent::Received(bytes) => {
                debug!(
                    "Transport message received: {} bytes on {}",
                    bytes, connection_id
                );
                self.metrics_collector.record_transport_receive(bytes).await;
            }
            MessageEvent::Error(error_msg) => {
                error!("Transport message error: {} - {}", connection_id, error_msg);
                self.metrics_collector
                    .record_transport_error(&error_msg)
                    .await;
            }
        }
    }
}

/// Transport connection events
#[derive(Debug, Clone)]
pub enum ConnectionEvent {
    Connected,
    Disconnected,
    Error(String),
}

/// Transport message events
#[derive(Debug, Clone)]
pub enum MessageEvent {
    Sent(u64),
    Received(u64),
    Error(String),
}

/// Middleware manager for coordinating multiple middleware components
pub struct MiddlewareManager {
    http_middleware: Option<HttpMonitoringMiddleware>,
    transport_middleware: Option<TransportMonitoringMiddleware>,
    enabled: bool,
}

impl MiddlewareManager {
    /// Create a new middleware manager
    pub fn new() -> Self {
        Self {
            http_middleware: None,
            transport_middleware: None,
            enabled: true,
        }
    }

    /// Create a new middleware manager with metrics collector
    pub fn with_metrics(metrics_collector: Arc<MetricsCollector>) -> Self {
        Self {
            http_middleware: Some(HttpMonitoringMiddleware::new(metrics_collector.clone())),
            transport_middleware: Some(TransportMonitoringMiddleware::new(metrics_collector)),
            enabled: true,
        }
    }

    /// Set HTTP middleware
    pub fn with_http_middleware(mut self, middleware: HttpMonitoringMiddleware) -> Self {
        self.http_middleware = Some(middleware);
        self
    }

    /// Set transport middleware
    pub fn with_transport_middleware(mut self, middleware: TransportMonitoringMiddleware) -> Self {
        self.transport_middleware = Some(middleware);
        self
    }

    /// Check if the manager is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Set whether the manager is enabled
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Get HTTP middleware reference
    pub fn http_middleware(&self) -> Option<&HttpMonitoringMiddleware> {
        self.http_middleware.as_ref()
    }

    /// Get transport middleware reference
    pub fn transport_middleware(&self) -> Option<&TransportMonitoringMiddleware> {
        self.transport_middleware.as_ref()
    }

    /// Get HTTP middleware mutable reference
    pub fn http_middleware_mut(&mut self) -> Option<&mut HttpMonitoringMiddleware> {
        self.http_middleware.as_mut()
    }

    /// Get transport middleware mutable reference
    pub fn transport_middleware_mut(&mut self) -> Option<&mut TransportMonitoringMiddleware> {
        self.transport_middleware.as_mut()
    }

    /// Process an HTTP request
    pub async fn process_http_request(&self, method: &str, path: &str) -> RequestContext {
        if let Some(middleware) = &self.http_middleware {
            middleware.process_request(method, path).await
        } else {
            RequestContext::new(method, path)
        }
    }

    /// Process an HTTP response
    pub async fn process_http_response(
        &self,
        context: &RequestContext,
        status_code: u16,
        success: bool,
    ) {
        if let Some(middleware) = &self.http_middleware {
            middleware
                .process_response(context, status_code, success)
                .await;
        }
    }

    /// Process a transport connection event
    pub async fn process_transport_connection(&self, connection_id: &str, event: ConnectionEvent) {
        if let Some(middleware) = &self.transport_middleware {
            middleware.process_connection(connection_id, event).await;
        }
    }

    /// Process a transport message event
    pub async fn process_transport_message(&self, connection_id: &str, event: MessageEvent) {
        if let Some(middleware) = &self.transport_middleware {
            middleware.process_message(connection_id, event).await;
        }
    }
}

impl Default for MiddlewareManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Middleware configuration
#[derive(Debug, Clone)]
pub struct MiddlewareConfig {
    pub enabled: bool,
    pub http_enabled: bool,
    pub transport_enabled: bool,
    pub request_timeout: std::time::Duration,
    pub max_connections: u32,
}

impl Default for MiddlewareConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            http_enabled: true,
            transport_enabled: true,
            request_timeout: std::time::Duration::from_secs(30),
            max_connections: 1000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_http_middleware() {
        let metrics_collector = Arc::new(MetricsCollector::new());
        let middleware = HttpMonitoringMiddleware::new(metrics_collector);

        assert!(middleware.is_enabled());

        // Test request processing
        let context = middleware.process_request("GET", "/test").await;
        assert_eq!(context.method, "GET");
        assert_eq!(context.path, "/test");
        assert!(context.is_monitored());

        // Test response processing
        middleware.process_response(&context, 200, true).await;

        // Test disabled middleware
        let disabled_middleware =
            HttpMonitoringMiddleware::with_config(Arc::new(MetricsCollector::new()), false);
        assert!(!disabled_middleware.is_enabled());

        let context = disabled_middleware
            .process_request("POST", "/disabled")
            .await;
        assert!(!context.is_monitored());
    }

    #[tokio::test]
    async fn test_transport_middleware() {
        let metrics_collector = Arc::new(MetricsCollector::new());
        let middleware = TransportMonitoringMiddleware::new(metrics_collector);

        assert!(middleware.is_enabled());

        // Test connection events
        middleware
            .process_connection("conn1", ConnectionEvent::Connected)
            .await;
        middleware
            .process_connection("conn1", ConnectionEvent::Disconnected)
            .await;
        middleware
            .process_connection("conn1", ConnectionEvent::Error("test error".to_string()))
            .await;

        // Test message events
        middleware
            .process_message("conn1", MessageEvent::Sent(1024))
            .await;
        middleware
            .process_message("conn1", MessageEvent::Received(2048))
            .await;
        middleware
            .process_message("conn1", MessageEvent::Error("message error".to_string()))
            .await;
    }

    #[tokio::test]
    async fn test_middleware_manager() {
        let metrics_collector = Arc::new(MetricsCollector::new());
        let manager = MiddlewareManager::with_metrics(metrics_collector);

        assert!(manager.is_enabled());
        assert!(manager.http_middleware().is_some());
        assert!(manager.transport_middleware().is_some());

        // Test HTTP processing
        let context = manager.process_http_request("GET", "/test").await;
        manager.process_http_response(&context, 200, true).await;

        // Test transport processing
        manager
            .process_transport_connection("conn1", ConnectionEvent::Connected)
            .await;
        manager
            .process_transport_message("conn1", MessageEvent::Sent(1024))
            .await;
    }

    #[tokio::test]
    async fn test_request_context() {
        let context = RequestContext::new("POST", "/api/test");

        assert_eq!(context.method, "POST");
        assert_eq!(context.path, "/api/test");
        assert!(!context.is_monitored());
        assert!(context.duration().is_none());

        // Test with monitoring
        let metrics_collector = Arc::new(MetricsCollector::new());
        let mut monitored_context = RequestContext::new("GET", "/monitored");
        monitored_context.start_time = Some(Instant::now());
        monitored_context.metrics_collector = Some(metrics_collector);

        assert!(monitored_context.is_monitored());
        assert!(monitored_context.duration().is_some());
    }

    #[tokio::test]
    async fn test_middleware_config() {
        let config = MiddlewareConfig::default();

        assert!(config.enabled);
        assert!(config.http_enabled);
        assert!(config.transport_enabled);
        assert_eq!(config.request_timeout, std::time::Duration::from_secs(30));
        assert_eq!(config.max_connections, 1000);

        let custom_config = MiddlewareConfig {
            enabled: false,
            http_enabled: false,
            transport_enabled: true,
            request_timeout: std::time::Duration::from_secs(60),
            max_connections: 500,
        };

        assert!(!custom_config.enabled);
        assert!(!custom_config.http_enabled);
        assert!(custom_config.transport_enabled);
        assert_eq!(
            custom_config.request_timeout,
            std::time::Duration::from_secs(60)
        );
        assert_eq!(custom_config.max_connections, 500);
    }
}
