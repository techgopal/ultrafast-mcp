//! Notification types for MCP 2025-06-18 protocol

use serde::{Deserialize, Serialize};

/// Tools list changed notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsListChangedNotification {
    /// Optional metadata about the change
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Resources list changed notification  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesListChangedNotification {
    /// Optional metadata about the change
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Prompts list changed notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsListChangedNotification {
    /// Optional metadata about the change
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Roots list changed notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootsListChangedNotification {
    /// Optional metadata about the change
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Logging message notification - MCP 2025-06-18 compliant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingMessageNotification {
    /// Log level (RFC 5424 compliant)
    pub level: LogLevel,

    /// Log data (arbitrary JSON-serializable data)
    pub data: serde_json::Value,

    /// Optional logger name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logger: Option<String>,
}

/// Log level set request - MCP 2025-06-18 compliant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogLevelSetRequest {
    /// Minimum log level to receive
    pub level: LogLevel,
}

/// Log level set response - MCP 2025-06-18 compliant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogLevelSetResponse {
    // Empty response as per MCP 2025-06-18 specification
}

/// Log level - RFC 5424 compliant levels
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Notice,
    Warning,
    Error,
    Critical,
    Alert,
    Emergency,
}

/// Cancellation notification for MCP 2025-06-18 protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelledNotification {
    /// The ID of the request to cancel
    #[serde(rename = "requestId")]
    pub request_id: serde_json::Value,

    /// Optional reason for cancellation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Progress notification for MCP 2025-06-18 protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressNotification {
    /// Progress token from the original request
    #[serde(rename = "progressToken")]
    pub progress_token: serde_json::Value,

    /// Current progress value
    pub progress: f64,

    /// Optional total value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<f64>,

    /// Optional human-readable message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Ping request for connection health monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingRequest {
    /// Optional data to echo back
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Ping response for connection health monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingResponse {
    /// Echo back any data that was sent in the request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Client capability notification - sent when client capabilities change
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClientCapabilityNotification {
    /// Updated client capabilities
    pub capabilities: crate::protocol::capabilities::ClientCapabilities,

    /// Optional reason for the capability change
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Server capability notification - sent when server capabilities change
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerCapabilityNotification {
    /// Updated server capabilities
    pub capabilities: crate::protocol::capabilities::ServerCapabilities,

    /// Optional reason for the capability change
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Connection status notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatusNotification {
    /// Current connection status
    pub status: ConnectionStatus,

    /// Optional message about the status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Optional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Connection status enum
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Reconnecting,
    Error,
}

/// Request timeout notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestTimeoutNotification {
    /// Request ID that timed out
    #[serde(rename = "requestId")]
    pub request_id: serde_json::Value,

    /// Timeout duration in milliseconds
    #[serde(rename = "timeoutMs")]
    pub timeout_ms: u64,

    /// Optional reason for the timeout
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Rate limit notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitNotification {
    /// Rate limit type
    #[serde(rename = "limitType")]
    pub limit_type: RateLimitType,

    /// Current request count
    #[serde(rename = "currentCount")]
    pub current_count: u64,

    /// Maximum allowed requests
    #[serde(rename = "maxCount")]
    pub max_count: u64,

    /// Time window in seconds
    #[serde(rename = "windowSeconds")]
    pub window_seconds: u64,

    /// Time until reset in seconds
    #[serde(rename = "resetInSeconds")]
    pub reset_in_seconds: u64,
}

/// Rate limit type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RateLimitType {
    RequestsPerMinute,
    RequestsPerHour,
    RequestsPerDay,
    BytesPerMinute,
    BytesPerHour,
    BytesPerDay,
}

impl ToolsListChangedNotification {
    pub fn new() -> Self {
        Self { metadata: None }
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

impl ResourcesListChangedNotification {
    pub fn new() -> Self {
        Self { metadata: None }
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

impl PromptsListChangedNotification {
    pub fn new() -> Self {
        Self { metadata: None }
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

impl RootsListChangedNotification {
    pub fn new() -> Self {
        Self { metadata: None }
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

impl LoggingMessageNotification {
    pub fn new(level: LogLevel, data: serde_json::Value) -> Self {
        Self {
            level,
            data,
            logger: None,
        }
    }

    pub fn with_logger(mut self, logger: String) -> Self {
        self.logger = Some(logger);
        self
    }
}

impl LogLevelSetRequest {
    pub fn new(level: LogLevel) -> Self {
        Self { level }
    }
}

impl LogLevelSetResponse {
    pub fn new() -> Self {
        Self {}
    }
}

impl CancelledNotification {
    pub fn new(request_id: serde_json::Value) -> Self {
        Self {
            request_id,
            reason: None,
        }
    }

    pub fn with_reason(mut self, reason: String) -> Self {
        self.reason = Some(reason);
        self
    }
}

impl ProgressNotification {
    pub fn new(progress_token: serde_json::Value, progress: f64) -> Self {
        Self {
            progress_token,
            progress,
            total: None,
            message: None,
        }
    }

    pub fn with_total(mut self, total: f64) -> Self {
        self.total = Some(total);
        self
    }

    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }
}

impl PingRequest {
    pub fn new() -> Self {
        Self { data: None }
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }
}

impl PingResponse {
    pub fn new() -> Self {
        Self { data: None }
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }
}

impl Default for ToolsListChangedNotification {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ResourcesListChangedNotification {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for PromptsListChangedNotification {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CancelledNotification {
    fn default() -> Self {
        Self::new(serde_json::Value::Null)
    }
}

impl Default for ProgressNotification {
    fn default() -> Self {
        Self::new(serde_json::Value::Null, 0.0)
    }
}

impl Default for PingRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for PingResponse {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for LogLevelSetResponse {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for RootsListChangedNotification {
    fn default() -> Self {
        Self::new()
    }
}





impl Default for ConnectionStatusNotification {
    fn default() -> Self {
        Self {
            status: ConnectionStatus::Connected,
            message: None,
            metadata: None,
        }
    }
}

impl Default for RequestTimeoutNotification {
    fn default() -> Self {
        Self {
            request_id: serde_json::Value::Null,
            timeout_ms: 30000,
            reason: None,
        }
    }
}

impl Default for RateLimitNotification {
    fn default() -> Self {
        Self {
            limit_type: RateLimitType::RequestsPerMinute,
            current_count: 0,
            max_count: 60,
            window_seconds: 60,
            reset_in_seconds: 60,
        }
    }
}
