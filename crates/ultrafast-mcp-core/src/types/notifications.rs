//! Notification types for MCP 2025-06-18 protocol

use serde::{Deserialize, Serialize};

/// Tools list changed notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsListChangedNotification {
    // Empty object for now
}

/// Resources list changed notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesListChangedNotification {
    // Empty object for now
}

/// Prompts list changed notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsListChangedNotification {
    // Empty object for now
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
    /// Echoed data from the request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl ToolsListChangedNotification {
    pub fn new() -> Self {
        Self {}
    }
}

impl ResourcesListChangedNotification {
    pub fn new() -> Self {
        Self {}
    }
}

impl PromptsListChangedNotification {
    pub fn new() -> Self {
        Self {}
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
