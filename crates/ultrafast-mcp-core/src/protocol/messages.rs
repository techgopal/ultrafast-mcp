use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP protocol messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message type
    pub message_type: String,
    /// Message content
    pub content: serde_json::Value,
    /// Optional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Notification message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Method name
    pub method: String,
    /// Parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// Progress notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressNotification {
    /// Progress token
    pub token: String,
    /// Progress value
    pub value: serde_json::Value,
}

/// Log message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogMessage {
    /// Log level
    pub level: LogLevel,
    /// Message content
    pub message: String,
    /// Optional logger name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logger: Option<String>,
    /// Optional data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Log levels
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
