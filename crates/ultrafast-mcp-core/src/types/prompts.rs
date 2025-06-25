use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Prompt definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    /// Prompt name (unique identifier)
    pub name: String,
    
    /// Human-readable description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// Prompt arguments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<PromptArgument>>,
}

/// Prompt argument definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    /// Argument name
    pub name: String,
    
    /// Human-readable description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// Whether the argument is required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

/// Get prompt request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPromptRequest {
    /// Prompt name
    pub name: String,
    
    /// Prompt arguments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Value>,
}

/// Get prompt response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPromptResponse {
    /// Prompt description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// Prompt messages
    pub messages: Vec<PromptMessage>,
}

/// Prompt message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMessage {
    /// Message role
    pub role: PromptRole,
    
    /// Message content
    pub content: PromptContent,
}

/// Prompt message role
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PromptRole {
    User,
    Assistant,
    System,
}

/// Prompt content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PromptContent {
    #[serde(rename = "text")]
    Text { text: String },
    
    #[serde(rename = "image")]
    Image { 
        data: String,  // Base64 encoded
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    
    #[serde(rename = "resource")]
    Resource {
        resource: super::tools::ResourceReference,
    },
}

/// List prompts request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPromptsRequest {
    /// Optional cursor for pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// List prompts response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPromptsResponse {
    /// Available prompts
    pub prompts: Vec<Prompt>,
    
    /// Next cursor for pagination
    #[serde(rename = "nextCursor", skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

impl Prompt {
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            arguments: None,
        }
    }
    
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
    
    pub fn with_arguments(mut self, arguments: Vec<PromptArgument>) -> Self {
        self.arguments = Some(arguments);
        self
    }
}

impl PromptArgument {
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            required: None,
        }
    }
    
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
    
    pub fn required(mut self, required: bool) -> Self {
        self.required = Some(required);
        self
    }
}

impl PromptMessage {
    pub fn user(content: PromptContent) -> Self {
        Self {
            role: PromptRole::User,
            content,
        }
    }
    
    pub fn assistant(content: PromptContent) -> Self {
        Self {
            role: PromptRole::Assistant,
            content,
        }
    }
    
    pub fn system(content: PromptContent) -> Self {
        Self {
            role: PromptRole::System,
            content,
        }
    }
}

impl PromptContent {
    pub fn text(text: String) -> Self {
        Self::Text { text }
    }
    
    pub fn image(data: String, mime_type: String) -> Self {
        Self::Image { data, mime_type }
    }
    
    pub fn resource(uri: String) -> Self {
        Self::Resource {
            resource: super::tools::ResourceReference {
                uri,
                description: None,
            }
        }
    }
}

/// Helper for building prompt messages
pub struct PromptMessages {
    messages: Vec<PromptMessage>,
}

impl PromptMessages {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }
    
    pub fn user(mut self, text: &str) -> Self {
        self.messages.push(PromptMessage::user(PromptContent::text(text.to_string())));
        self
    }
    
    pub fn assistant(mut self, text: &str) -> Self {
        self.messages.push(PromptMessage::assistant(PromptContent::text(text.to_string())));
        self
    }
    
    pub fn system(mut self, text: &str) -> Self {
        self.messages.push(PromptMessage::system(PromptContent::text(text.to_string())));
        self
    }
    
    pub fn with_context(mut self, context: &str) -> Self {
        self.messages.push(PromptMessage::system(PromptContent::text(context.to_string())));
        self
    }
    
    pub fn build(self) -> Vec<PromptMessage> {
        self.messages
    }
}

impl Default for PromptMessages {
    fn default() -> Self {
        Self::new()
    }
}
