use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Type aliases for consistency
///
/// NOTE: These aliases are provided for backward compatibility and ergonomics.
/// ToolCall is an alias for ToolCallRequest - use ToolCallRequest for clarity in new code.
/// ToolResult is an alias for ToolCallResponse - use ToolCallResponse for clarity in new code.
///
/// # Examples
/// ```
/// use ultrafast_mcp_core::types::{ToolCall, ToolResult};
/// use ultrafast_mcp_core::types::tools::{ToolCallRequest, ToolCallResponse};
///
/// // Preferred: Use the full type names
/// let request = ToolCallRequest { name: "my_tool".to_string(), arguments: None };
/// let response = ToolCallResponse { content: vec![], is_error: None };
///
/// // Legacy: Using aliases (still works but less clear)
/// let request: ToolCall = ToolCallRequest { name: "my_tool".to_string(), arguments: None };
/// let response: ToolResult = ToolCallResponse { content: vec![], is_error: None };
/// ```
pub type ToolCall = ToolCallRequest;
pub type ToolResult = ToolCallResponse;

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Tool name (unique identifier)
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// JSON Schema for input parameters
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,

    /// Optional JSON Schema for output
    #[serde(rename = "outputSchema", skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<Value>,
}

/// Tool call request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRequest {
    /// Tool name to call
    pub name: String,

    /// Arguments to pass to the tool
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Value>,
}

/// Tool call response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResponse {
    /// Tool execution result
    pub content: Vec<ToolContent>,

    /// Whether the tool execution was cancelled
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

/// Tool content (result of tool execution)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolContent {
    #[serde(rename = "text")]
    Text { text: String },

    #[serde(rename = "image")]
    Image {
        data: String, // Base64 encoded
        #[serde(rename = "mimeType")]
        mime_type: String,
    },

    #[serde(rename = "resource")]
    Resource { resource: ResourceReference },
}

/// Reference to a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceReference {
    /// Resource URI
    pub uri: String,

    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// List tools request
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListToolsRequest {
    /// Optional cursor for pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// List tools response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsResponse {
    /// Available tools
    pub tools: Vec<Tool>,

    /// Next cursor for pagination
    #[serde(rename = "nextCursor", skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

impl Tool {
    pub fn new(name: String, description: String, input_schema: Value) -> Self {
        Self {
            name,
            description,
            input_schema,
            output_schema: None,
        }
    }

    pub fn with_output_schema(mut self, schema: Value) -> Self {
        self.output_schema = Some(schema);
        self
    }

    /// Validate the tool definition
    pub fn validate(&self) -> Result<(), crate::error::ToolError> {
        // Validate name
        if self.name.is_empty() {
            return Err(crate::error::ToolError::InvalidInput(
                "Tool name cannot be empty".to_string(),
            ));
        }

        // Check for reserved names
        if self.name.starts_with("rpc.") {
            return Err(crate::error::ToolError::InvalidInput(format!(
                "Tool name '{}' is reserved",
                self.name
            )));
        }

        // Validate name format (alphanumeric, hyphens, underscores only)
        if !self
            .name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(crate::error::ToolError::InvalidInput(
                "Tool name can only contain alphanumeric characters, hyphens, and underscores"
                    .to_string(),
            ));
        }

        // Validate description
        if self.description.is_empty() {
            return Err(crate::error::ToolError::InvalidInput(
                "Tool description cannot be empty".to_string(),
            ));
        }

        // Validate input schema
        if !self.input_schema.is_object() {
            return Err(crate::error::ToolError::InvalidInput(
                "Input schema must be a JSON object".to_string(),
            ));
        }

        // Validate output schema if present
        if let Some(ref output_schema) = self.output_schema {
            if !output_schema.is_object() {
                return Err(crate::error::ToolError::InvalidInput(
                    "Output schema must be a JSON object".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Validate tool call arguments against input schema
    pub fn validate_arguments(&self, arguments: &Value) -> Result<(), crate::error::ToolError> {
        use crate::schema::validation::validate_against_schema;

        validate_against_schema(arguments, &self.input_schema).map_err(|e| {
            crate::error::ToolError::SchemaValidation(format!(
                "Tool argument validation failed: {e}"
            ))
        })
    }
}

impl ToolContent {
    pub fn text(text: String) -> Self {
        Self::Text { text }
    }

    pub fn image(data: String, mime_type: String) -> Self {
        Self::Image { data, mime_type }
    }

    pub fn resource(uri: String) -> Self {
        Self::Resource {
            resource: ResourceReference {
                uri,
                description: None,
            },
        }
    }

    pub fn resource_with_description(uri: String, description: String) -> Self {
        Self::Resource {
            resource: ResourceReference {
                uri,
                description: Some(description),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_validation() {
        // Valid tool
        let valid_tool = Tool::new(
            "valid_tool".to_string(),
            "A valid tool".to_string(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string"}
                }
            }),
        );
        assert!(valid_tool.validate().is_ok());

        // Invalid tool - empty name
        let invalid_tool = Tool::new(
            "".to_string(),
            "A tool with empty name".to_string(),
            serde_json::json!({"type": "object"}),
        );
        assert!(invalid_tool.validate().is_err());

        // Invalid tool - reserved name
        let reserved_tool = Tool::new(
            "rpc.test".to_string(),
            "A tool with reserved name".to_string(),
            serde_json::json!({"type": "object"}),
        );
        assert!(reserved_tool.validate().is_err());

        // Invalid tool - invalid name characters
        let invalid_chars_tool = Tool::new(
            "tool@test".to_string(),
            "A tool with invalid characters".to_string(),
            serde_json::json!({"type": "object"}),
        );
        assert!(invalid_chars_tool.validate().is_err());

        // Invalid tool - empty description
        let empty_desc_tool = Tool::new(
            "valid_tool".to_string(),
            "".to_string(),
            serde_json::json!({"type": "object"}),
        );
        assert!(empty_desc_tool.validate().is_err());

        // Invalid tool - non-object input schema
        let invalid_schema_tool = Tool::new(
            "valid_tool".to_string(),
            "A valid tool".to_string(),
            serde_json::json!("not an object"),
        );
        assert!(invalid_schema_tool.validate().is_err());
    }

    #[test]
    fn test_tool_argument_validation() {
        let tool = Tool::new(
            "test_tool".to_string(),
            "A test tool".to_string(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string"},
                    "age": {"type": "integer"}
                },
                "required": ["name"]
            }),
        );

        // Valid arguments
        let valid_args = serde_json::json!({
            "name": "Alice",
            "age": 30
        });
        assert!(tool.validate_arguments(&valid_args).is_ok());

        // Invalid arguments - missing required field
        let invalid_args = serde_json::json!({
            "age": 30
        });
        assert!(tool.validate_arguments(&invalid_args).is_err());

        // Invalid arguments - wrong type
        let wrong_type_args = serde_json::json!({
            "name": "Alice",
            "age": "not a number"
        });
        assert!(tool.validate_arguments(&wrong_type_args).is_err());
    }
}
