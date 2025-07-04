use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Type aliases for consistency
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
