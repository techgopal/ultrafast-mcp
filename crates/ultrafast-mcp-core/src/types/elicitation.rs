//! Elicitation types for MCP
//!
//! Server-initiated user input collection

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Server-initiated request for user input
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ElicitationRequest {
    /// Prompt to show to the user
    pub prompt: String,
    /// Type of input expected (text, choice, etc.)
    pub input_type: ElicitationInputType,
    /// Optional validation rules
    pub validation: Option<ElicitationValidation>,
}

/// Type of input expected from user
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum ElicitationInputType {
    /// Free text input
    #[serde(rename = "text")]
    Text {
        /// Placeholder text
        placeholder: Option<String>,
        /// Whether input is sensitive (password-like)
        sensitive: Option<bool>,
    },
    /// Choice from predefined options
    #[serde(rename = "choice")]
    Choice {
        /// Available options
        options: Vec<ElicitationChoice>,
        /// Whether multiple choices are allowed
        multiple: Option<bool>,
    },
    /// Confirmation (yes/no)
    #[serde(rename = "confirmation")]
    Confirmation {
        /// Default value
        default: Option<bool>,
    },
}

/// A choice option for elicitation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ElicitationChoice {
    /// Display label for the choice
    pub label: String,
    /// Value to return if selected
    pub value: String,
    /// Optional description
    pub description: Option<String>,
}

/// Validation rules for elicitation input
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ElicitationValidation {
    /// Minimum length for text input
    pub min_length: Option<u32>,
    /// Maximum length for text input
    pub max_length: Option<u32>,
    /// Regex pattern for validation
    pub pattern: Option<String>,
    /// Whether input is required
    pub required: Option<bool>,
}

/// Response to elicitation request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ElicitationResponse {
    /// User's input/choice
    pub value: serde_json::Value,
    /// Whether the user cancelled
    pub cancelled: Option<bool>,
}
