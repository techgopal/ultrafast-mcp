//! Elicitation types for MCP
//!
//! Server-initiated user input collection

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    /// Optional metadata for the elicitation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl Default for ElicitationRequest {
    fn default() -> Self {
        Self {
            session_id: None,
            step: None,
            prompt: String::new(),
            input_type: ElicitationInputType::Text {
                placeholder: None,
                sensitive: None,
                multiline: None,
            },
            validation: None,
            metadata: None,
        }
    }
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
        /// Whether to allow multiline input
        multiline: Option<bool>,
    },
    /// Choice from predefined options
    #[serde(rename = "choice")]
    Choice {
        /// Available options
        options: Vec<ElicitationChoice>,
        /// Whether multiple choices are allowed
        multiple: Option<bool>,
        /// Whether to allow custom input
        allow_custom: Option<bool>,
    },
    /// Confirmation (yes/no)
    #[serde(rename = "confirmation")]
    Confirmation {
        /// Default value
        default: Option<bool>,
    },
    /// File upload
    #[serde(rename = "file")]
    File {
        /// Accepted file types (MIME types or extensions)
        accept: Vec<String>,
        /// Whether multiple files are allowed
        multiple: Option<bool>,
        /// Maximum file size in bytes
        max_size: Option<u64>,
        /// Whether to show file picker or drag-and-drop
        drag_drop: Option<bool>,
    },
    /// Number input
    #[serde(rename = "number")]
    Number {
        /// Minimum value
        min: Option<f64>,
        /// Maximum value
        max: Option<f64>,
        /// Step increment
        step: Option<f64>,
        /// Number of decimal places
        decimals: Option<u32>,
        /// Whether to show slider
        slider: Option<bool>,
    },
    /// Date input
    #[serde(rename = "date")]
    Date {
        /// Minimum date (ISO 8601 format)
        min: Option<String>,
        /// Maximum date (ISO 8601 format)
        max: Option<String>,
        /// Date format to display
        format: Option<String>,
        /// Whether to include time
        include_time: Option<bool>,
    },
    /// Time input
    #[serde(rename = "time")]
    Time {
        /// Time format (12h/24h)
        format: Option<String>,
        /// Minimum time
        min: Option<String>,
        /// Maximum time
        max: Option<String>,
        /// Time step in minutes
        step: Option<u32>,
    },
    /// Color picker
    #[serde(rename = "color")]
    Color {
        /// Color format (hex, rgb, hsl)
        format: Option<String>,
        /// Default color
        default: Option<String>,
        /// Whether to show alpha channel
        alpha: Option<bool>,
    },
    /// Range slider
    #[serde(rename = "range")]
    Range {
        /// Minimum value
        min: f64,
        /// Maximum value
        max: f64,
        /// Step increment
        step: Option<f64>,
        /// Whether to show dual handles (min/max)
        dual: Option<bool>,
        /// Whether to show value labels
        show_labels: Option<bool>,
    },
    /// Email input
    #[serde(rename = "email")]
    Email {
        /// Placeholder text
        placeholder: Option<String>,
        /// Whether to validate email format
        validate: Option<bool>,
        /// Whether to allow multiple emails
        multiple: Option<bool>,
    },
    /// URL input
    #[serde(rename = "url")]
    Url {
        /// Placeholder text
        placeholder: Option<String>,
        /// Whether to validate URL format
        validate: Option<bool>,
        /// Allowed URL schemes
        schemes: Option<Vec<String>>,
    },
    /// Phone number input
    #[serde(rename = "phone")]
    Phone {
        /// Placeholder text
        placeholder: Option<String>,
        /// Country code
        country_code: Option<String>,
        /// Whether to validate phone format
        validate: Option<bool>,
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
    /// Whether this option is disabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    /// Optional icon or image
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

/// Enhanced validation rules for elicitation input
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct ElicitationValidation {
    /// Minimum length for text input
    pub min_length: Option<u32>,
    /// Maximum length for text input
    pub max_length: Option<u32>,
    /// Regex pattern for validation
    pub pattern: Option<String>,
    /// Whether input is required
    pub required: Option<bool>,
    /// Custom validation function name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_validator: Option<String>,
    /// Error message for validation failures
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// Warning message for validation warnings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning_message: Option<String>,
}

/// Enhanced state for multi-step elicitation workflows
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct MultiStepElicitationState {
    /// Session ID
    pub session_id: String,
    /// Current step
    pub current_step: u32,
    /// Total steps in workflow
    pub total_steps: Option<u32>,
    /// Collected responses
    pub responses: Vec<ElicitationResponse>,
    /// Workflow metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    /// Whether workflow is completed
    pub completed: bool,
    /// Timestamp when workflow started
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    /// Timestamp when workflow was last updated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// Enhanced response to elicitation request
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
    /// Validation errors if any
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_errors: Option<Vec<String>>,
    /// Timestamp when response was received
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

/// Elicitation workflow configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ElicitationWorkflow {
    /// Workflow ID
    pub id: String,
    /// Workflow name
    pub name: String,
    /// Workflow description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Steps in the workflow
    pub steps: Vec<ElicitationStep>,
    /// Workflow metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Individual step in an elicitation workflow
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ElicitationStep {
    /// Step number
    pub step: u32,
    /// Step title
    pub title: String,
    /// Step description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Elicitation request for this step
    pub request: ElicitationRequest,
    /// Whether this step is optional
    pub optional: Option<bool>,
    /// Conditions for showing this step
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditions: Option<Vec<ElicitationCondition>>,
}

/// Condition for showing a step
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ElicitationCondition {
    /// Field to check
    pub field: String,
    /// Operator (equals, not_equals, contains, etc.)
    pub operator: String,
    /// Value to compare against
    pub value: serde_json::Value,
}

impl ElicitationInputType {
    /// Get the display name for this input type
    pub fn display_name(&self) -> &'static str {
        match self {
            ElicitationInputType::Text { .. } => "Text Input",
            ElicitationInputType::Choice { .. } => "Choice Selection",
            ElicitationInputType::Confirmation { .. } => "Confirmation",
            ElicitationInputType::File { .. } => "File Upload",
            ElicitationInputType::Number { .. } => "Number Input",
            ElicitationInputType::Date { .. } => "Date Picker",
            ElicitationInputType::Time { .. } => "Time Picker",
            ElicitationInputType::Color { .. } => "Color Picker",
            ElicitationInputType::Range { .. } => "Range Slider",
            ElicitationInputType::Email { .. } => "Email Input",
            ElicitationInputType::Url { .. } => "URL Input",
            ElicitationInputType::Phone { .. } => "Phone Input",
        }
    }

    /// Check if this input type supports validation
    pub fn supports_validation(&self) -> bool {
        matches!(
            self,
            ElicitationInputType::Text { .. }
                | ElicitationInputType::Email { .. }
                | ElicitationInputType::Url { .. }
                | ElicitationInputType::Phone { .. }
                | ElicitationInputType::Number { .. }
        )
    }
}

impl ElicitationValidation {
    /// Create a new validation with required field
    pub fn required() -> Self {
        Self {
            required: Some(true),
            ..Default::default()
        }
    }

    /// Create a new validation with length constraints
    pub fn with_length(min: u32, max: u32) -> Self {
        Self {
            min_length: Some(min),
            max_length: Some(max),
            ..Default::default()
        }
    }

    /// Create a new validation with pattern
    pub fn with_pattern(pattern: String) -> Self {
        Self {
            pattern: Some(pattern),
            ..Default::default()
        }
    }

    /// Validate a value against this validation
    pub fn validate(&self, value: &serde_json::Value) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check if required
        if self.required.unwrap_or(false) && (value.is_null() || (value.is_string() && value.as_str().unwrap_or("").is_empty())) {
            errors.push(
                self.error_message
                    .clone()
                    .unwrap_or_else(|| "Field is required".to_string()),
            );
        }

        // Check length constraints for strings
        if let Some(str_value) = value.as_str() {
            if let Some(min) = self.min_length {
                if str_value.len() < min as usize {
                    errors.push(format!("Minimum length is {} characters", min));
                }
            }
            if let Some(max) = self.max_length {
                if str_value.len() > max as usize {
                    errors.push(format!("Maximum length is {} characters", max));
                }
            }
        }

        // Check pattern
        if let (Some(pattern), Some(str_value)) = (&self.pattern, value.as_str()) {
            if let Ok(regex) = regex::Regex::new(pattern) {
                if !regex.is_match(str_value) {
                    errors.push(
                        self.error_message
                            .clone()
                            .unwrap_or_else(|| "Invalid format".to_string()),
                    );
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl MultiStepElicitationState {
    /// Create a new workflow state
    pub fn new(session_id: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            session_id,
            current_step: 0,
            total_steps: None,
            responses: Vec::new(),
            metadata: None,
            completed: false,
            started_at: Some(now.clone()),
            updated_at: Some(now),
        }
    }

    /// Add a response to the workflow
    pub fn add_response(&mut self, response: ElicitationResponse) {
        self.responses.push(response);
        self.current_step += 1;
        self.updated_at = Some(chrono::Utc::now().to_rfc3339());
    }

    /// Check if workflow is complete
    pub fn is_complete(&self) -> bool {
        if let Some(total) = self.total_steps {
            self.current_step >= total
        } else {
            self.completed
        }
    }

    /// Get the current response
    pub fn get_current_response(&self) -> Option<&ElicitationResponse> {
        self.responses.last()
    }
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
                multiline: None,
            },
            validation: None,
            metadata: None,
        };
        assert_eq!(req.prompt, "Enter your name");
    }

    #[test]
    fn test_advanced_input_types() {
        // Test file input
        let file_input = ElicitationInputType::File {
            accept: vec!["image/*".to_string(), ".pdf".to_string()],
            multiple: Some(true),
            max_size: Some(10 * 1024 * 1024), // 10MB
            drag_drop: Some(true),
        };
        assert_eq!(file_input.display_name(), "File Upload");

        // Test number input
        let number_input = ElicitationInputType::Number {
            min: Some(0.0),
            max: Some(100.0),
            step: Some(1.0),
            decimals: Some(2),
            slider: Some(true),
        };
        assert_eq!(number_input.display_name(), "Number Input");
        assert!(number_input.supports_validation());

        // Test date input
        let date_input = ElicitationInputType::Date {
            min: Some("2024-01-01".to_string()),
            max: Some("2024-12-31".to_string()),
            format: Some("YYYY-MM-DD".to_string()),
            include_time: Some(false),
        };
        assert_eq!(date_input.display_name(), "Date Picker");
    }

    #[test]
    fn test_validation() {
        let validation = ElicitationValidation::with_length(3, 10);
        let short_value = serde_json::json!("ab");
        let long_value = serde_json::json!("abcdefghijklmnop");
        let valid_value = serde_json::json!("hello");

        assert!(validation.validate(&short_value).is_err());
        assert!(validation.validate(&long_value).is_err());
        assert!(validation.validate(&valid_value).is_ok());
    }

    #[test]
    fn test_multi_step_workflow() {
        let mut state = MultiStepElicitationState::new("sess-123".to_string());
        state.total_steps = Some(3);

        assert!(!state.is_complete());
        assert_eq!(state.current_step, 0);

        let response = ElicitationResponse {
            session_id: Some("sess-123".to_string()),
            step: Some(1),
            value: serde_json::json!("Alice"),
            cancelled: None,
            validation_errors: None,
            timestamp: None,
        };

        state.add_response(response);
        assert_eq!(state.current_step, 1);
        assert!(!state.is_complete());

        // Add more responses to complete
        state.add_response(ElicitationResponse {
            session_id: Some("sess-123".to_string()),
            step: Some(2),
            value: serde_json::json!(30),
            cancelled: None,
            validation_errors: None,
            timestamp: None,
        });
        state.add_response(ElicitationResponse {
            session_id: Some("sess-123".to_string()),
            step: Some(3),
            value: serde_json::json!("alice@example.com"),
            cancelled: None,
            validation_errors: None,
            timestamp: None,
        });

        assert!(state.is_complete());
    }

    #[test]
    fn test_workflow_creation() {
        let workflow = ElicitationWorkflow {
            id: "user-registration".to_string(),
            name: "User Registration".to_string(),
            description: Some("Collect user registration information".to_string()),
            steps: vec![
                ElicitationStep {
                    step: 1,
                    title: "Name".to_string(),
                    description: Some("Enter your full name".to_string()),
                    request: ElicitationRequest {
                        prompt: "What is your name?".to_string(),
                        input_type: ElicitationInputType::Text {
                            placeholder: Some("Full name".to_string()),
                            sensitive: None,
                            multiline: None,
                        },
                        validation: Some(ElicitationValidation::required()),
                        ..Default::default()
                    },
                    optional: None,
                    conditions: None,
                },
                ElicitationStep {
                    step: 2,
                    title: "Email".to_string(),
                    description: Some("Enter your email address".to_string()),
                    request: ElicitationRequest {
                        prompt: "What is your email?".to_string(),
                        input_type: ElicitationInputType::Email {
                            placeholder: Some("email@example.com".to_string()),
                            validate: Some(true),
                            multiple: None,
                        },
                        validation: Some(ElicitationValidation::required()),
                        ..Default::default()
                    },
                    optional: None,
                    conditions: None,
                },
            ],
            metadata: None,
        };

        assert_eq!(workflow.id, "user-registration");
        assert_eq!(workflow.steps.len(), 2);
        assert_eq!(workflow.steps[0].title, "Name");
        assert_eq!(workflow.steps[1].title, "Email");
    }
}
