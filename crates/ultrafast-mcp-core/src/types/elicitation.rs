//! Elicitation types for MCP
//!
//! Server-initiated user input collection according to MCP specification 2025-06-18

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Server-initiated request for user input
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ElicitationRequest {
    /// Clear, human-readable explanation of what information is needed and why
    pub message: String,
    
    /// JSON Schema that defines the expected structure of the response
    /// Limited to flat objects with primitive types to simplify client implementation
    #[serde(rename = "requestedSchema")]
    pub requested_schema: serde_json::Value,
}

impl Default for ElicitationRequest {
    fn default() -> Self {
        Self {
            message: String::new(),
            requested_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }
}

/// User response to elicitation request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ElicitationResponse {
    /// The action taken by the user
    pub action: ElicitationAction,
    
    /// The content provided by the user (only present if action is "accept")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<serde_json::Value>,
}

/// User response actions
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ElicitationAction {
    /// User accepts and provides the requested information
    Accept,
    /// User explicitly refuses to provide information
    Decline,
    /// User dismisses without making a choice (e.g., closes dialog)
    Cancel,
}

impl Default for ElicitationResponse {
    fn default() -> Self {
        Self {
            action: ElicitationAction::Cancel,
            content: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_elicitation_request() {
        let request = ElicitationRequest {
            message: "Please provide your GitHub username".to_string(),
            requested_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "username": {
                        "type": "string",
                        "title": "GitHub Username",
                        "description": "Your GitHub username (e.g., octocat)"
                    }
                },
                "required": ["username"]
            }),
        };
        
        assert_eq!(request.message, "Please provide your GitHub username");
        assert!(request.requested_schema.is_object());
    }

    #[test]
    fn test_elicitation_response_accept() {
        let response = ElicitationResponse {
            action: ElicitationAction::Accept,
            content: Some(serde_json::json!({
                "username": "octocat"
            })),
        };
        
        assert!(matches!(response.action, ElicitationAction::Accept));
        assert!(response.content.is_some());
    }

    #[test]
    fn test_elicitation_response_decline() {
        let response = ElicitationResponse {
            action: ElicitationAction::Decline,
            content: None,
        };
        
        assert!(matches!(response.action, ElicitationAction::Decline));
        assert!(response.content.is_none());
    }

    #[test]
    fn test_elicitation_response_cancel() {
        let response = ElicitationResponse {
            action: ElicitationAction::Cancel,
            content: None,
        };
        
        assert!(matches!(response.action, ElicitationAction::Cancel));
        assert!(response.content.is_none());
    }

    #[test]
    fn test_supported_data_types() {
        // Text input
        let text_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "projectName": {
                    "type": "string",
                    "title": "Project Name",
                    "description": "Name for your new project",
                    "minLength": 3,
                    "maxLength": 50
                }
            },
            "required": ["projectName"]
        });

        // Number input
        let number_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "portNumber": {
                    "type": "number",
                    "title": "Port Number",
                    "description": "Port to run the server on",
                    "minimum": 1024,
                    "maximum": 65535
                }
            },
            "required": ["portNumber"]
        });

        // Boolean input
        let boolean_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "enableAnalytics": {
                    "type": "boolean",
                    "title": "Enable Analytics",
                    "description": "Send anonymous usage statistics",
                    "default": false
                }
            },
            "required": ["enableAnalytics"]
        });

        // Selection list
        let selection_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "environment": {
                    "type": "string",
                    "title": "Environment",
                    "enum": ["development", "staging", "production"],
                    "enumNames": ["Development", "Staging", "Production"]
                }
            },
            "required": ["environment"]
        });

        assert!(text_schema.is_object());
        assert!(number_schema.is_object());
        assert!(boolean_schema.is_object());
        assert!(selection_schema.is_object());
    }
}
