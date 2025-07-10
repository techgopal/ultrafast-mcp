use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio::time::{interval, timeout};

use crate::error::{MCPError, MCPResult};
use crate::types::notifications::{CancelledNotification, PingRequest, PingResponse};

/// Request cancellation manager
#[derive(Debug)]
pub struct CancellationManager {
    /// Active requests that can be cancelled
    active_requests: Arc<RwLock<HashMap<serde_json::Value, CancellableRequest>>>,
}

/// A cancellable request
#[derive(Debug, Clone)]
pub struct CancellableRequest {
    /// Request ID
    pub id: serde_json::Value,

    /// Request method
    pub method: String,

    /// Timestamp when request was created
    pub created_at: u64,

    /// Whether the request has been cancelled
    pub cancelled: bool,

    /// Cancellation reason
    pub cancel_reason: Option<String>,
}

/// Ping manager for connection health monitoring
#[derive(Clone)]
pub struct PingManager {
    /// Ping interval
    ping_interval: Duration,

    /// Ping timeout
    ping_timeout: Duration,

    /// Whether ping monitoring is enabled
    enabled: bool,

    /// Callback for sending ping requests
    ping_sender: Option<Arc<dyn PingSender + Send + Sync>>,
}

/// Trait for sending ping requests
#[async_trait::async_trait]
pub trait PingSender {
    async fn send_ping(&self, request: PingRequest) -> MCPResult<PingResponse>;
}

impl CancellationManager {
    /// Create a new cancellation manager
    pub fn new() -> Self {
        Self {
            active_requests: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new request for cancellation tracking
    pub async fn register_request(&self, id: serde_json::Value, method: String) -> MCPResult<()> {
        let request = CancellableRequest {
            id: id.clone(),
            method,
            created_at: current_timestamp(),
            cancelled: false,
            cancel_reason: None,
        };

        let mut active = self.active_requests.write().await;
        active.insert(id, request);
        Ok(())
    }

    /// Cancel a request
    pub async fn cancel_request(
        &self,
        id: &serde_json::Value,
        reason: Option<String>,
    ) -> MCPResult<bool> {
        let mut active = self.active_requests.write().await;

        if let Some(request) = active.get_mut(id) {
            if !request.cancelled {
                request.cancelled = true;
                request.cancel_reason = reason;
                return Ok(true);
            }
        }

        // Request not found or already cancelled
        Ok(false)
    }

    /// Check if a request has been cancelled
    pub async fn is_cancelled(&self, id: &serde_json::Value) -> bool {
        let active = self.active_requests.read().await;
        active.get(id).map(|r| r.cancelled).unwrap_or(false)
    }

    /// Remove a completed request
    pub async fn complete_request(&self, id: &serde_json::Value) -> MCPResult<()> {
        let mut active = self.active_requests.write().await;
        active.remove(id);
        Ok(())
    }

    /// Get all active requests
    pub async fn active_requests(&self) -> Vec<CancellableRequest> {
        let active = self.active_requests.read().await;
        active.values().cloned().collect()
    }

    /// Clean up old requests (older than max_age)
    pub async fn cleanup_old_requests(&self, max_age: Duration) -> MCPResult<usize> {
        let cutoff = current_timestamp() - max_age.as_secs();
        let mut active = self.active_requests.write().await;

        let original_len = active.len();
        active.retain(|_, request| request.created_at > cutoff);
        let removed = original_len - active.len();

        Ok(removed)
    }

    /// Handle a cancellation notification
    pub async fn handle_cancellation(
        &self,
        notification: CancelledNotification,
    ) -> MCPResult<bool> {
        self.cancel_request(&notification.request_id, notification.reason)
            .await
    }
}

impl PingManager {
    /// Create a new ping manager
    pub fn new(ping_interval: Duration, ping_timeout: Duration) -> Self {
        Self {
            ping_interval,
            ping_timeout,
            enabled: false,
            ping_sender: None,
        }
    }

    /// Set the ping sender
    pub fn with_sender(mut self, sender: Arc<dyn PingSender + Send + Sync>) -> Self {
        self.ping_sender = Some(sender);
        self
    }

    /// Enable ping monitoring
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable ping monitoring
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Start periodic ping monitoring
    pub async fn start_monitoring(&self) -> MCPResult<()> {
        if !self.enabled || self.ping_sender.is_none() {
            return Err(MCPError::internal_error(
                "Ping monitoring not properly configured".to_string(),
            ));
        }

        let sender = self.ping_sender.as_ref().unwrap().clone();
        let ping_interval = self.ping_interval;
        let ping_timeout = self.ping_timeout;

        tokio::spawn(async move {
            let mut interval = interval(ping_interval);

            loop {
                interval.tick().await;

                let ping_request = PingRequest::new().with_data(serde_json::json!({
                    "timestamp": current_timestamp(),
                    "keepalive": true
                }));

                match timeout(ping_timeout, sender.send_ping(ping_request)).await {
                    Ok(Ok(_response)) => {
                        // Ping successful
                        tracing::debug!("Ping successful");
                    }
                    Ok(Err(e)) => {
                        // Ping failed
                        tracing::warn!("Ping failed: {}", e);
                        // Could implement reconnection logic here
                        break;
                    }
                    Err(_) => {
                        // Ping timed out
                        tracing::warn!("Ping timed out after {:?}", ping_timeout);
                        // Could implement reconnection logic here
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Handle a ping request and return a pong response
    pub async fn handle_ping(&self, request: PingRequest) -> MCPResult<PingResponse> {
        // Echo back the data as per MCP 2025-06-18 specification
        Ok(PingResponse { data: request.data })
    }
}

impl Default for CancellationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for PingManager {
    fn default() -> Self {
        Self::new(Duration::from_secs(30), Duration::from_secs(5))
    }
}

impl std::fmt::Debug for PingManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PingManager")
            .field("ping_interval", &self.ping_interval)
            .field("ping_timeout", &self.ping_timeout)
            .field("enabled", &self.enabled)
            .field("ping_sender", &"<callback>")
            .finish()
    }
}

/// Get current Unix timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_cancellation_manager() {
        let manager = CancellationManager::new();

        let request_id = serde_json::json!("test-request-1");

        // Register request
        manager
            .register_request(request_id.clone(), "test_method".to_string())
            .await
            .unwrap();

        // Check not cancelled initially
        assert!(!manager.is_cancelled(&request_id).await);

        // Cancel request
        let cancelled = manager
            .cancel_request(&request_id, Some("User requested".to_string()))
            .await
            .unwrap();
        assert!(cancelled);

        // Check now cancelled
        assert!(manager.is_cancelled(&request_id).await);

        // Complete request
        manager.complete_request(&request_id).await.unwrap();

        // Check removed from active requests
        assert!(!manager.is_cancelled(&request_id).await);
    }

    #[tokio::test]
    async fn test_cancellation_cleanup() {
        let manager = CancellationManager::new();

        // Register multiple requests
        for i in 0..5 {
            let request_id = serde_json::json!(format!("test-request-{}", i));
            manager
                .register_request(request_id, "test_method".to_string())
                .await
                .unwrap();
        }

        // Wait a bit
        sleep(Duration::from_millis(100)).await;

        // Cleanup old requests (very short max age)
        let removed = manager
            .cleanup_old_requests(Duration::from_millis(50))
            .await
            .unwrap();
        assert_eq!(removed, 5);
    }

    #[tokio::test]
    async fn test_ping_manager() {
        let manager = PingManager::new(Duration::from_secs(1), Duration::from_secs(1));

        let request = PingRequest::new().with_data(serde_json::json!({"test": "data"}));
        let response = manager.handle_ping(request).await.unwrap();

        // PingResponse should echo back the data as per MCP 2025-06-18 specification
        assert_eq!(format!("{:?}", response), "PingResponse { data: Some(Object {\"test\": String(\"data\")}) }");
    }

    #[test]
    fn test_ping_response() {
        let response = PingResponse::new();
        // PingResponse is empty as per MCP 2025-06-18 specification
        assert_eq!(format!("{:?}", response), "PingResponse { data: None }");
    }
}
