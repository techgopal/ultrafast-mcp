//! Common handler trait patterns
//!
//! This module defines common handler traits that are used across
//! different MCP implementations to ensure consistency.

use crate::MCPResult;
use async_trait::async_trait;
use std::any::Any;

/// Base trait for all handlers in the MCP ecosystem
#[async_trait]
pub trait BaseHandler: Send + Sync {
    /// Get the handler name for debugging/logging
    fn name(&self) -> &'static str;

    /// Check if this handler can handle a specific request type
    fn can_handle(&self, request_type: &str) -> bool;

    /// Get handler metadata as Any for downcasting
    fn as_any(&self) -> &dyn Any;
}

/// Common pattern for request/response handlers
#[async_trait]
pub trait RequestHandler<Request, Response>: BaseHandler
where
    Request: Send + Sync + 'static,
    Response: Send + Sync + 'static,
{
    /// Handle a request and return a response
    async fn handle(&self, request: Request) -> MCPResult<Response>;

    /// Validate a request before handling
    async fn validate_request(&self, request: &Request) -> MCPResult<()> {
        // Default implementation - can be overridden
        let _ = request;
        Ok(())
    }

    /// Post-process a response after handling
    async fn post_process_response(&self, response: Response) -> MCPResult<Response> {
        // Default implementation - can be overridden
        Ok(response)
    }
}

/// Common pattern for notification handlers (no response)
#[async_trait]
pub trait NotificationHandler<Notification>: BaseHandler
where
    Notification: Send + Sync + 'static,
{
    /// Handle a notification
    async fn handle_notification(&self, notification: Notification) -> MCPResult<()>;

    /// Validate a notification before handling
    async fn validate_notification(&self, notification: &Notification) -> MCPResult<()> {
        // Default implementation - can be overridden
        let _ = notification;
        Ok(())
    }
}

/// Common pattern for lifecycle handlers
#[async_trait]
pub trait LifecycleHandler: BaseHandler {
    /// Initialize the handler
    async fn initialize(&self) -> MCPResult<()> {
        Ok(())
    }

    /// Shutdown the handler gracefully
    async fn shutdown(&self) -> MCPResult<()> {
        Ok(())
    }

    /// Get handler health status
    async fn health_check(&self) -> MCPResult<bool> {
        Ok(true)
    }
}

/// Macro to implement common handler patterns
#[macro_export]
macro_rules! impl_base_handler {
    ($handler_type:ty, $name:expr) => {
        impl $crate::traits::BaseHandler for $handler_type {
            fn name(&self) -> &'static str {
                $name
            }

            fn can_handle(&self, _request_type: &str) -> bool {
                true // Default implementation
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }
    };

    ($handler_type:ty, $name:expr, $can_handle:expr) => {
        impl $crate::traits::BaseHandler for $handler_type {
            fn name(&self) -> &'static str {
                $name
            }

            fn can_handle(&self, request_type: &str) -> bool {
                $can_handle(request_type)
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestHandler;

    impl_base_handler!(TestHandler, "test_handler");

    #[async_trait]
    impl LifecycleHandler for TestHandler {}

    #[tokio::test]
    async fn test_base_handler() {
        let handler = TestHandler;
        assert_eq!(handler.name(), "test_handler");
        assert!(handler.can_handle("any_request"));
        assert!(handler.initialize().await.is_ok());
        assert!(handler.shutdown().await.is_ok());
        assert!(handler.health_check().await.unwrap());
    }

    struct TestRequestHandler;

    impl_base_handler!(TestRequestHandler, "test_request_handler");

    #[async_trait]
    impl RequestHandler<String, String> for TestRequestHandler {
        async fn handle(&self, request: String) -> MCPResult<String> {
            Ok(format!("Handled: {request}"))
        }
    }

    #[tokio::test]
    async fn test_request_handler() {
        let handler = TestRequestHandler;
        let response = handler.handle("test request".to_string()).await.unwrap();
        assert_eq!(response, "Handled: test request");
    }

    struct TestNotificationHandler;

    impl_base_handler!(TestNotificationHandler, "test_notification_handler");

    #[async_trait]
    impl NotificationHandler<String> for TestNotificationHandler {
        async fn handle_notification(&self, notification: String) -> MCPResult<()> {
            println!("Received notification: {}", notification);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_notification_handler() {
        let handler = TestNotificationHandler;
        assert!(handler
            .handle_notification("test notification".to_string())
            .await
            .is_ok());
    }
}
