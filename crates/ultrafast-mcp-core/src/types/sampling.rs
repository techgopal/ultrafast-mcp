use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Type aliases for consistency
pub type CreateMessageRequest = SamplingRequest;
pub type CreateMessageResponse = SamplingResponse;

/// Sampling request (server asking client to perform LLM sampling)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingRequest {
    /// Messages to send to the LLM
    pub messages: Vec<SamplingMessage>,
    
    /// Model preferences
    #[serde(rename = "modelPreferences", skip_serializing_if = "Option::is_none")]
    pub model_preferences: Option<ModelPreferences>,
    
    /// System prompt
    #[serde(rename = "systemPrompt", skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    
    /// Whether to include context from the conversation
    #[serde(rename = "includeContext", skip_serializing_if = "Option::is_none")]
    pub include_context: Option<bool>,
    
    /// Temperature for sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    
    /// Maximum tokens to generate
    #[serde(rename = "maxTokens", skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    
    /// Stop sequences
    #[serde(rename = "stopSequences", skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    
    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

/// Sampling response from client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingResponse {
    /// Role of the response
    pub role: String,
    
    /// Content of the response
    pub content: SamplingContent,
    
    /// Model that was used for sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    
    /// Reason why sampling stopped
    #[serde(rename = "stopReason", skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
}

/// Sampling message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingMessage {
    /// Message role
    pub role: SamplingRole,
    
    /// Message content
    pub content: SamplingContent,
}

/// Sampling message role
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SamplingRole {
    User,
    Assistant,
    System,
}

/// Sampling content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SamplingContent {
    #[serde(rename = "text")]
    Text { text: String },
    
    #[serde(rename = "image")]
    Image { 
        data: String,  // Base64 encoded
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
}

/// Model preferences for sampling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPreferences {
    /// Preferred model hints (not binding)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hints: Option<Vec<ModelHint>>,
    
    /// Cost priority (lower = prefer cheaper models)
    #[serde(rename = "costPriority", skip_serializing_if = "Option::is_none")]
    pub cost_priority: Option<f64>,
    
    /// Speed priority (lower = prefer faster models)  
    #[serde(rename = "speedPriority", skip_serializing_if = "Option::is_none")]
    pub speed_priority: Option<f64>,
    
    /// Intelligence priority (lower = prefer smarter models)
    #[serde(rename = "intelligencePriority", skip_serializing_if = "Option::is_none")]
    pub intelligence_priority: Option<f64>,
}

/// Model hint for sampling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelHint {
    /// Model name hint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    
    /// Model provider hint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

impl SamplingMessage {
    pub fn user(content: SamplingContent) -> Self {
        Self {
            role: SamplingRole::User,
            content,
        }
    }
    
    pub fn assistant(content: SamplingContent) -> Self {
        Self {
            role: SamplingRole::Assistant,
            content,
        }
    }
    
    pub fn system(content: SamplingContent) -> Self {
        Self {
            role: SamplingRole::System,
            content,
        }
    }
}

impl SamplingContent {
    pub fn text(text: String) -> Self {
        Self::Text { text }
    }
    
    pub fn image(data: String, mime_type: String) -> Self {
        Self::Image { data, mime_type }
    }
}
