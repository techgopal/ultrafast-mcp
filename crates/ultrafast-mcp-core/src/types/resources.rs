use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Resource definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// Resource URI
    pub uri: String,

    /// Resource name
    pub name: String,

    /// Human-readable description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// MIME type of the resource
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// Resource template definition (for parameterized resources)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTemplate {
    /// URI template (RFC 6570)
    #[serde(rename = "uriTemplate")]
    pub uri_template: String,

    /// Template name
    pub name: String,

    /// Human-readable description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// MIME type of resources created from this template
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// Read resource request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResourceRequest {
    /// Resource URI to read
    pub uri: String,
}

/// Read resource response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResourceResponse {
    /// Resource contents
    pub contents: Vec<ResourceContent>,
}

/// Resource content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResourceContent {
    #[serde(rename = "text")]
    Text {
        text: String,
        #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
        mime_type: Option<String>,
    },

    #[serde(rename = "blob")]
    Blob {
        blob: String, // Base64 encoded
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
}

/// List resources request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourcesRequest {
    /// Optional cursor for pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// List resources response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourcesResponse {
    /// Available resources
    pub resources: Vec<Resource>,

    /// Next cursor for pagination
    #[serde(rename = "nextCursor", skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// List resource templates request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourceTemplatesRequest {
    /// Optional cursor for pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// List resource templates response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourceTemplatesResponse {
    /// Available resource templates
    #[serde(rename = "resourceTemplates")]
    pub resource_templates: Vec<ResourceTemplate>,

    /// Next cursor for pagination
    #[serde(rename = "nextCursor", skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Subscribe to resource request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeRequest {
    /// Resource URI to subscribe to
    pub uri: String,
}

/// Unsubscribe from resource request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsubscribeRequest {
    /// Resource URI to unsubscribe from
    pub uri: String,
}

/// Resource updated notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUpdatedNotification {
    /// Updated resource URI
    pub uri: String,
}

impl Resource {
    pub fn new(uri: String, name: String) -> Self {
        Self {
            uri,
            name,
            description: None,
            mime_type: None,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_mime_type(mut self, mime_type: String) -> Self {
        self.mime_type = Some(mime_type);
        self
    }
}

impl ResourceTemplate {
    pub fn new(uri_template: String, name: String) -> Self {
        Self {
            uri_template,
            name,
            description: None,
            mime_type: None,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_mime_type(mut self, mime_type: String) -> Self {
        self.mime_type = Some(mime_type);
        self
    }
}

impl ResourceContent {
    pub fn text(text: String) -> Self {
        Self::Text {
            text,
            mime_type: Some("text/plain".to_string()),
        }
    }

    pub fn text_with_mime_type(text: String, mime_type: String) -> Self {
        Self::Text {
            text,
            mime_type: Some(mime_type),
        }
    }

    pub fn json(value: &Value) -> Self {
        Self::Text {
            text: serde_json::to_string_pretty(value).unwrap_or_default(),
            mime_type: Some("application/json".to_string()),
        }
    }

    pub fn blob(blob: String, mime_type: String) -> Self {
        Self::Blob { blob, mime_type }
    }
}
