use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Implementation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationMetadata {
    /// Implementation name
    pub name: String,
    /// Implementation version
    pub version: String,
    /// Optional additional metadata
    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}

/// Protocol metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMetadata {
    /// Protocol version
    pub version: String,
    /// Supported features
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<Vec<String>>,
    /// Optional additional metadata
    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}

/// Request metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetadata {
    /// Request ID for correlation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    /// User agent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    /// Optional additional metadata
    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}

/// Response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    /// Processing time in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_time_ms: Option<u64>,
    /// Server information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<ImplementationMetadata>,
    /// Optional additional metadata
    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}
