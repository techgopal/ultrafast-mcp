//! Context module for UltraFastServer
//!
//! This module provides the Context type that allows tools and handlers to interact
//! with the server for progress tracking, logging, and other operations.

use std::collections::HashMap;
use tracing::{error, info, warn};

/// Context for tool and handler execution
///
/// Provides access to server functionality like progress tracking, logging,
/// and request metadata.
#[derive(Debug, Clone)]
pub struct Context {
    session_id: Option<String>,
    request_id: Option<String>,
    metadata: HashMap<String, serde_json::Value>,
}

impl Context {
    /// Create a new empty context
    pub fn new() -> Self {
        Self {
            session_id: None,
            request_id: None,
            metadata: HashMap::new(),
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

        // TODO: Send actual progress notification to client
        // This would require access to the server's notification system

        Ok(())
    }

    /// Log an info message
    pub async fn log_info(
        &self,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "[{}] {}",
            self.request_id.as_deref().unwrap_or("unknown"),
            message
        );

        // TODO: Send log notification to client

        Ok(())
    }

    /// Log a warning message
    pub async fn log_warn(
        &self,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        warn!(
            "[{}] {}",
            self.request_id.as_deref().unwrap_or("unknown"),
            message
        );

        // TODO: Send log notification to client

        Ok(())
    }

    /// Log an error message
    pub async fn log_error(
        &self,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        error!(
            "[{}] {}",
            self.request_id.as_deref().unwrap_or("unknown"),
            message
        );

        // TODO: Send log notification to client

        Ok(())
    }

    /// Check if the current request has been cancelled
    pub async fn is_cancelled(&self) -> bool {
        // TODO: Check with cancellation manager
        false
    }
}

impl Default for Context {
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
