//! Completion types for MCP 2025-06-18 protocol

use serde::{Deserialize, Serialize};

/// Completion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteRequest {
    /// Reference type ("prompts", "resource_templates")
    #[serde(rename = "ref")]
    pub ref_type: String,
    
    /// Reference name (prompt name, resource template URI)
    #[serde(rename = "name")]
    pub ref_name: String,
    
    /// Argument to complete (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub argument: Option<String>,
}

/// Completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteResponse {
    /// Completion results
    pub completion: Completion,
}

/// Completion result set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Completion {
    /// Completion values
    pub values: Vec<CompletionValue>,
    
    /// Total number of possible completions (for pagination)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u32>,
    
    /// Whether there are more completions available
    #[serde(rename = "hasMore", skip_serializing_if = "Option::is_none")]
    pub has_more: Option<bool>,
}

/// Individual completion value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionValue {
    /// Completion value
    pub value: String,
    
    /// Optional label (display name)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    
    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl CompletionValue {
    /// Create a simple completion value
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: None,
            description: None,
        }
    }
    
    /// Create a completion value with label
    pub fn with_label(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: Some(label.into()),
            description: None,
        }
    }
    
    /// Create a completion value with description
    pub fn with_description(value: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: None,
            description: Some(description.into()),
        }
    }
}

impl Completion {
    /// Create a new completion set
    pub fn new(values: Vec<CompletionValue>) -> Self {
        Self {
            values,
            total: None,
            has_more: None,
        }
    }
    
    /// Create with metadata
    pub fn with_metadata(values: Vec<CompletionValue>, total: u32, has_more: bool) -> Self {
        Self {
            values,
            total: Some(total),
            has_more: Some(has_more),
        }
    }
}
