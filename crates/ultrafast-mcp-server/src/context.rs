//! Context module for UltraFastServer
//!
//! This module provides the Context type that allows tools and handlers to interact
//! with the server for progress tracking, logging, and other operations.

use serde_json::Value;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use ultrafast_mcp_core::{
    error::MCPResult,
    protocol::jsonrpc::{JsonRpcMessage, JsonRpcRequest},
    types::notifications::{LogLevel, LoggingMessageNotification, ProgressNotification},
};

/// Simple cancellation manager for tracking cancelled requests
#[derive(Debug, Clone)]
pub struct CancellationManager {
    cancelled_requests: Arc<tokio::sync::RwLock<std::collections::HashSet<String>>>,
}

impl CancellationManager {
    pub fn new() -> Self {
        Self {
            cancelled_requests: Arc::new(
                tokio::sync::RwLock::new(std::collections::HashSet::new()),
            ),
        }
    }

    pub async fn cancel_request(&self, request_id: &str) {
        let mut requests = self.cancelled_requests.write().await;
        requests.insert(request_id.to_string());
    }

    pub async fn is_cancelled(&self, request_id: &str) -> bool {
        let requests = self.cancelled_requests.read().await;
        requests.contains(request_id)
    }

    pub async fn clear_cancelled(&self, request_id: &str) {
        let mut requests = self.cancelled_requests.write().await;
        requests.remove(request_id);
    }
}

impl Default for CancellationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Logger configuration for the context
#[derive(Debug, Clone)]
pub struct LoggerConfig {
    /// Minimum log level to process
    pub min_level: LogLevel,
    /// Whether to send notifications to client
    pub send_notifications: bool,
    /// Whether to include structured output
    pub structured_output: bool,
    /// Maximum log message length
    pub max_message_length: usize,
    /// Whether to include timestamps
    pub include_timestamps: bool,
    /// Whether to include logger name
    pub include_logger_name: bool,
    /// Custom logger name
    pub logger_name: Option<String>,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            min_level: LogLevel::Info,
            send_notifications: true,
            structured_output: true,
            max_message_length: 4096,
            include_timestamps: true,
            include_logger_name: true,
            logger_name: None,
        }
    }
}

/// Notification sender for sending messages to the client
type NotificationSender = Arc<
    dyn Fn(
            JsonRpcMessage,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = MCPResult<()>> + Send>>
        + Send
        + Sync,
>;

/// Context for tool and handler execution
///
/// Provides access to server functionality like progress tracking, logging,
/// and request metadata.
#[derive(Clone)]
pub struct Context {
    session_id: Option<String>,
    request_id: Option<String>,
    metadata: HashMap<String, serde_json::Value>,
    logger_config: LoggerConfig,
    notification_sender: Option<NotificationSender>,
    cancellation_manager: Option<Arc<CancellationManager>>,
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("session_id", &self.session_id)
            .field("request_id", &self.request_id)
            .field("metadata", &self.metadata)
            .field("logger_config", &self.logger_config)
            .field("notification_sender", &self.notification_sender.is_some())
            .finish()
    }
}

impl Context {
    /// Create a new empty context
    pub fn new() -> Self {
        Self {
            session_id: None,
            request_id: None,
            metadata: HashMap::new(),
            logger_config: LoggerConfig::default(),
            notification_sender: None,
            cancellation_manager: None,
        }
    }

    /// Create a context with session ID
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Create a context with request ID
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }

    /// Add metadata to the context
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Configure the logger for this context
    pub fn with_logger_config(mut self, config: LoggerConfig) -> Self {
        self.logger_config = config;
        self
    }

    /// Set the notification sender
    pub fn with_notification_sender(mut self, sender: NotificationSender) -> Self {
        self.notification_sender = Some(sender);
        self
    }

    /// Set the cancellation manager
    pub fn with_cancellation_manager(mut self, manager: Arc<CancellationManager>) -> Self {
        self.cancellation_manager = Some(manager);
        self
    }

    /// Get the session ID
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Get the request ID
    pub fn request_id(&self) -> Option<&str> {
        self.request_id.as_deref()
    }

    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// Set the minimum log level
    pub fn set_log_level(&mut self, level: LogLevel) {
        self.logger_config.min_level = level;
    }

    /// Get the current minimum log level
    pub fn get_log_level(&self) -> &LogLevel {
        &self.logger_config.min_level
    }

    /// Check if a log level should be processed
    fn should_log(&self, level: &LogLevel) -> bool {
        let level_priority = log_level_priority(level);
        let min_priority = log_level_priority(&self.logger_config.min_level);
        level_priority >= min_priority
    }

    /// Send a progress update
    ///
    /// # Arguments
    /// * `message` - Progress message
    /// * `progress` - Current progress value (0.0 to 1.0)
    /// * `total` - Optional total value
    pub async fn progress(
        &self,
        message: &str,
        progress: f64,
        total: Option<f64>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Log progress for debugging
        if let Some(total) = total {
            info!(
                "Progress: {} - {:.2}/{:.2} ({:.1}%)",
                message,
                progress,
                total,
                (progress / total) * 100.0
            );
        } else {
            info!("Progress: {} - {:.2}", message, progress);
        }

        // Send progress notification if sender is available
        if let Some(sender) = &self.notification_sender {
            let progress_token = self
                .request_id()
                .map(|id| serde_json::Value::String(id.to_string()))
                .unwrap_or(serde_json::Value::Null);

            let mut notification = ProgressNotification::new(progress_token, progress)
                .with_message(message.to_string());

            if let Some(total) = total {
                notification = notification.with_total(total);
            }

            let notification_request = JsonRpcRequest {
                jsonrpc: Cow::Borrowed("2.0"),
                id: None, // Notifications don't have IDs
                method: "notifications/progress".to_string(),
                params: Some(serde_json::to_value(notification)?),
                meta: std::collections::HashMap::new(),
            };

            sender(JsonRpcMessage::Notification(notification_request)).await?;
        }

        Ok(())
    }

    /// Log a debug message
    pub async fn log_debug(
        &self,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.log_with_level(LogLevel::Debug, message, None).await
    }

    /// Log a debug message with structured data
    pub async fn log_debug_structured(
        &self,
        message: &str,
        data: Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.log_with_level(LogLevel::Debug, message, Some(data))
            .await
    }

    /// Log an info message
    pub async fn log_info(
        &self,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.log_with_level(LogLevel::Info, message, None).await
    }

    /// Log an info message with structured data
    pub async fn log_info_structured(
        &self,
        message: &str,
        data: Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.log_with_level(LogLevel::Info, message, Some(data))
            .await
    }

    /// Log a notice message
    pub async fn log_notice(
        &self,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.log_with_level(LogLevel::Notice, message, None).await
    }

    /// Log a notice message with structured data
    pub async fn log_notice_structured(
        &self,
        message: &str,
        data: Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.log_with_level(LogLevel::Notice, message, Some(data))
            .await
    }

    /// Log a warning message
    pub async fn log_warn(
        &self,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.log_with_level(LogLevel::Warning, message, None).await
    }

    /// Log a warning message with structured data
    pub async fn log_warn_structured(
        &self,
        message: &str,
        data: Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.log_with_level(LogLevel::Warning, message, Some(data))
            .await
    }

    /// Log an error message
    pub async fn log_error(
        &self,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.log_with_level(LogLevel::Error, message, None).await
    }

    /// Log an error message with structured data
    pub async fn log_error_structured(
        &self,
        message: &str,
        data: Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.log_with_level(LogLevel::Error, message, Some(data))
            .await
    }

    /// Log a critical message
    pub async fn log_critical(
        &self,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.log_with_level(LogLevel::Critical, message, None).await
    }

    /// Log a critical message with structured data
    pub async fn log_critical_structured(
        &self,
        message: &str,
        data: Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.log_with_level(LogLevel::Critical, message, Some(data))
            .await
    }

    /// Log an alert message
    pub async fn log_alert(
        &self,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.log_with_level(LogLevel::Alert, message, None).await
    }

    /// Log an alert message with structured data
    pub async fn log_alert_structured(
        &self,
        message: &str,
        data: Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.log_with_level(LogLevel::Alert, message, Some(data))
            .await
    }

    /// Log an emergency message
    pub async fn log_emergency(
        &self,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.log_with_level(LogLevel::Emergency, message, None)
            .await
    }

    /// Log an emergency message with structured data
    pub async fn log_emergency_structured(
        &self,
        message: &str,
        data: Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.log_with_level(LogLevel::Emergency, message, Some(data))
            .await
    }

    /// Internal method to log with a specific level
    async fn log_with_level(
        &self,
        level: LogLevel,
        message: &str,
        structured_data: Option<Value>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Check if this level should be logged
        if !self.should_log(&level) {
            return Ok(());
        }

        // Truncate message if too long
        let truncated_message = if message.len() > self.logger_config.max_message_length {
            let mut truncated = message[..self.logger_config.max_message_length - 3].to_string();
            truncated.push_str("...");
            truncated
        } else {
            message.to_string()
        };

        // Create structured log data
        let log_data = if self.logger_config.structured_output {
            let mut data_obj = serde_json::Map::new();

            // Add basic message
            data_obj.insert(
                "message".to_string(),
                Value::String(truncated_message.clone()),
            );

            // Add request context
            if let Some(request_id) = &self.request_id {
                data_obj.insert("request_id".to_string(), Value::String(request_id.clone()));
            }

            if let Some(session_id) = &self.session_id {
                data_obj.insert("session_id".to_string(), Value::String(session_id.clone()));
            }

            // Add timestamp if configured
            if self.logger_config.include_timestamps {
                let timestamp = chrono::Utc::now().to_rfc3339();
                data_obj.insert("timestamp".to_string(), Value::String(timestamp));
            }

            // Add logger name if configured
            if self.logger_config.include_logger_name {
                let logger_name = self
                    .logger_config
                    .logger_name
                    .as_deref()
                    .unwrap_or("ultrafast-mcp-server");
                data_obj.insert("logger".to_string(), Value::String(logger_name.to_string()));
            }

            // Add level
            data_obj.insert(
                "level".to_string(),
                Value::String(format!("{level:?}").to_lowercase()),
            );

            // Add any structured data
            if let Some(data) = structured_data {
                data_obj.insert("data".to_string(), data);
            }

            // Add metadata
            if !self.metadata.is_empty() {
                data_obj.insert(
                    "metadata".to_string(),
                    Value::Object(
                        self.metadata
                            .iter()
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect(),
                    ),
                );
            }

            Value::Object(data_obj)
        } else {
            // Simple string message
            Value::String(truncated_message.clone())
        };

        // Log to tracing system based on level
        let request_context = self.request_id.as_deref().unwrap_or("unknown");
        match level {
            LogLevel::Debug => debug!("[{}] {}", request_context, truncated_message),
            LogLevel::Info => info!("[{}] {}", request_context, truncated_message),
            LogLevel::Notice => info!("[{}] NOTICE: {}", request_context, truncated_message),
            LogLevel::Warning => warn!("[{}] {}", request_context, truncated_message),
            LogLevel::Error => error!("[{}] {}", request_context, truncated_message),
            LogLevel::Critical => error!("[{}] CRITICAL: {}", request_context, truncated_message),
            LogLevel::Alert => error!("[{}] ALERT: {}", request_context, truncated_message),
            LogLevel::Emergency => error!("[{}] EMERGENCY: {}", request_context, truncated_message),
        }

        // Send logging notification to client if configured and sender is available
        if self.logger_config.send_notifications {
            if let Some(sender) = &self.notification_sender {
                let logger_name = self
                    .logger_config
                    .logger_name
                    .as_deref()
                    .unwrap_or("ultrafast-mcp-server");

                let notification = LoggingMessageNotification::new(level, log_data)
                    .with_logger(logger_name.to_string());

                let notification_request = JsonRpcRequest {
                    jsonrpc: Cow::Borrowed("2.0"),
                    id: None, // Notifications don't have IDs
                    method: "notifications/message".to_string(),
                    params: Some(serde_json::to_value(notification)?),
                    meta: std::collections::HashMap::new(),
                };

                // Send notification but don't fail if it doesn't work
                if let Err(e) = sender(JsonRpcMessage::Notification(notification_request)).await {
                    // Log the error but don't propagate it
                    error!("Failed to send logging notification: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Check if the current request has been cancelled
    pub async fn is_cancelled(&self) -> bool {
        if let Some(cancellation_manager) = &self.cancellation_manager {
            if let Some(request_id) = &self.request_id {
                cancellation_manager.is_cancelled(request_id).await
            } else {
                false
            }
        } else {
            false
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

/// Get numeric priority for log level (higher = more urgent)
fn log_level_priority(level: &LogLevel) -> u8 {
    match level {
        LogLevel::Debug => 0,
        LogLevel::Info => 1,
        LogLevel::Notice => 2,
        LogLevel::Warning => 3,
        LogLevel::Error => 4,
        LogLevel::Critical => 5,
        LogLevel::Alert => 6,
        LogLevel::Emergency => 7,
    }
}

/// Structured logger builder for easy configuration
pub struct ContextLogger {
    config: LoggerConfig,
}

impl ContextLogger {
    pub fn new() -> Self {
        Self {
            config: LoggerConfig::default(),
        }
    }

    pub fn with_min_level(mut self, level: LogLevel) -> Self {
        self.config.min_level = level;
        self
    }

    pub fn with_notifications(mut self, send_notifications: bool) -> Self {
        self.config.send_notifications = send_notifications;
        self
    }

    pub fn with_structured_output(mut self, structured: bool) -> Self {
        self.config.structured_output = structured;
        self
    }

    pub fn with_max_message_length(mut self, length: usize) -> Self {
        self.config.max_message_length = length;
        self
    }

    pub fn with_timestamps(mut self, include: bool) -> Self {
        self.config.include_timestamps = include;
        self
    }

    pub fn with_logger_name(mut self, name: String) -> Self {
        self.config.logger_name = Some(name);
        self
    }

    pub fn build(self) -> LoggerConfig {
        self.config
    }
}

impl Default for ContextLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_context_creation() {
        let ctx = Context::new()
            .with_session_id("session-123".to_string())
            .with_request_id("request-456".to_string())
            .with_metadata("key".to_string(), serde_json::json!("value"));

        assert_eq!(ctx.session_id(), Some("session-123"));
        assert_eq!(ctx.request_id(), Some("request-456"));
        assert_eq!(ctx.get_metadata("key"), Some(&serde_json::json!("value")));
    }

    #[tokio::test]
    async fn test_context_logging() {
        let ctx = Context::new().with_request_id("test-request".to_string());

        // These should not panic
        ctx.log_info("Test info message").await.unwrap();
        ctx.log_warn("Test warning message").await.unwrap();
        ctx.log_error("Test error message").await.unwrap();
    }

    #[tokio::test]
    async fn test_context_progress() {
        let ctx = Context::new();

        // Test progress tracking
        ctx.progress("Starting operation", 0.0, Some(1.0))
            .await
            .unwrap();
        ctx.progress("Halfway done", 0.5, Some(1.0)).await.unwrap();
        ctx.progress("Completed", 1.0, Some(1.0)).await.unwrap();

        // Test without total
        ctx.progress("Indeterminate progress", 0.3, None)
            .await
            .unwrap();
    }
}
