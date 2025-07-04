//! Elicitation types for MCP
//!
//! Server-initiated user input collection

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Server-initiated request for user input (multi-step support)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ElicitationRequest {
    /// Unique session ID for multi-step workflows
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Step number in the workflow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<u32>,
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

/// State for multi-step elicitation workflows
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct MultiStepElicitationState {
    /// Session ID
    pub session_id: String,
    /// Current step
    pub current_step: u32,
    /// Collected responses
    pub responses: Vec<ElicitationResponse>,
}

/// Response to elicitation request (multi-step support)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ElicitationResponse {
    /// Unique session ID for multi-step workflows
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Step number in the workflow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<u32>,
    /// User's input/choice
    pub value: serde_json::Value,
    /// Whether the user cancelled
    pub cancelled: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_step_elicitation() {
        let req = ElicitationRequest {
            session_id: None,
            step: None,
            prompt: "Enter your name".to_string(),
            input_type: ElicitationInputType::Text {
                placeholder: Some("Name".to_string()),
                sensitive: None,
            },
            validation: None,
        };
        assert_eq!(req.prompt, "Enter your name");
    }

    #[test]
    fn test_multi_step_elicitation_state() {
        let mut state = MultiStepElicitationState {
            session_id: "sess-123".to_string(),
            current_step: 0,
            responses: vec![],
        };
        let resp1 = ElicitationResponse {
            session_id: Some("sess-123".to_string()),
            step: Some(1),
            value: serde_json::json!("Alice"),
            cancelled: None,
        };
        state.current_step = 1;
        state.responses.push(resp1.clone());
        assert_eq!(state.responses.len(), 1);
        assert_eq!(state.responses[0].value, serde_json::json!("Alice"));
        assert_eq!(state.session_id, "sess-123");
    }

    #[test]
    fn test_multi_step_elicitation_request_response() {
        let req = ElicitationRequest {
            session_id: Some("sess-abc".to_string()),
            step: Some(2),
            prompt: "Enter your age".to_string(),
            input_type: ElicitationInputType::Text {
                placeholder: Some("Age".to_string()),
                sensitive: None,
            },
            validation: None,
        };
        let resp = ElicitationResponse {
            session_id: Some("sess-abc".to_string()),
            step: Some(2),
            value: serde_json::json!(30),
            cancelled: None,
        };
        assert_eq!(req.session_id, Some("sess-abc".to_string()));
        assert_eq!(resp.step, Some(2));
        assert_eq!(resp.value, serde_json::json!(30));
    }
}
