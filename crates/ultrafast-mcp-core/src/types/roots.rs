//! Roots types for MCP
//! 
//! Filesystem boundary management for security-conscious path validation

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// A filesystem root that defines boundary for file operations
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Root {
    /// The URI of the root (typically file://)
    pub uri: String,
    /// Optional human-readable name for the root
    pub name: Option<String>,
}

/// Request to list available roots
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListRootsRequest {
    // No parameters needed for listing roots
}

/// Response containing available roots
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListRootsResponse {
    /// List of available roots
    pub roots: Vec<Root>,
}

/// Notification sent when the list of roots changes
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RootListChangedNotification {
    /// Updated list of roots
    pub roots: Vec<Root>,
}
