use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Type aliases for consistency
pub type CreateMessageRequest = SamplingRequest;
pub type CreateMessageResponse = SamplingResponse;

/// Context inclusion options for sampling requests
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IncludeContext {
    None,
    ThisServer,
    AllServers,
}

/// Stop reason for sampling responses
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StopReason {
    EndTurn,
    StopSequence,
    MaxTokens,
    Other,
}

/// Human-in-the-loop approval status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Modified,
}

/// Context information for sampling requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingContext {
    /// Server information if context is included
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_info: Option<ServerContextInfo>,

    /// Available tools if context is included
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_tools: Option<Vec<ToolContextInfo>>,

    /// Available resources if context is included
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_resources: Option<Vec<ResourceContextInfo>>,

    /// Conversation history if context is included
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_history: Option<Vec<SamplingMessage>>,

    /// User preferences and settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_preferences: Option<UserPreferences>,
}

/// Server context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerContextInfo {
    /// Server name
    pub name: String,

    /// Server version
    pub version: String,

    /// Server description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Server capabilities
    pub capabilities: Vec<String>,
}

/// Tool context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolContextInfo {
    /// Tool name
    pub name: String,

    /// Tool description
    pub description: String,

    /// Tool input schema
    pub input_schema: Value,

    /// Tool annotations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Value>,
}

/// Resource context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContextInfo {
    /// Resource URI
    pub uri: String,

    /// Resource name
    pub name: String,

    /// Resource description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Resource MIME type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// User preferences for sampling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Preferred model family
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_model_family: Option<String>,

    /// Cost sensitivity (0-1, higher = more cost sensitive)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_sensitivity: Option<f64>,

    /// Speed preference (0-1, higher = prefer faster)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed_preference: Option<f64>,

    /// Quality preference (0-1, higher = prefer higher quality)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality_preference: Option<f64>,

    /// Whether to require human approval
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_approval: Option<bool>,

    /// Maximum cost per request (in cents)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_cost_per_request: Option<f64>,
}

/// Enhanced sampling request with context and human-in-the-loop support
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
    pub include_context: Option<IncludeContext>,

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

    /// Human-in-the-loop settings
    #[serde(rename = "humanInTheLoop", skip_serializing_if = "Option::is_none")]
    pub human_in_the_loop: Option<HumanInTheLoopSettings>,

    /// Request ID for tracking
    #[serde(rename = "requestId", skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,

    /// Priority level (0-1, higher = higher priority)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<f64>,

    /// Timeout in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<u32>,
}

/// Human-in-the-loop settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanInTheLoopSettings {
    /// Whether to require prompt approval
    #[serde(
        rename = "requirePromptApproval",
        skip_serializing_if = "Option::is_none"
    )]
    pub require_prompt_approval: Option<bool>,

    /// Whether to require completion approval
    #[serde(
        rename = "requireCompletionApproval",
        skip_serializing_if = "Option::is_none"
    )]
    pub require_completion_approval: Option<bool>,

    /// Whether to allow prompt modification
    #[serde(
        rename = "allowPromptModification",
        skip_serializing_if = "Option::is_none"
    )]
    pub allow_prompt_modification: Option<bool>,

    /// Whether to allow completion modification
    #[serde(
        rename = "allowCompletionModification",
        skip_serializing_if = "Option::is_none"
    )]
    pub allow_completion_modification: Option<bool>,

    /// Approval timeout in seconds
    #[serde(
        rename = "approvalTimeoutSeconds",
        skip_serializing_if = "Option::is_none"
    )]
    pub approval_timeout_seconds: Option<u32>,

    /// Notification settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notifications: Option<NotificationSettings>,
}

/// Notification settings for human-in-the-loop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    /// Whether to show desktop notifications
    #[serde(
        rename = "desktopNotifications",
        skip_serializing_if = "Option::is_none"
    )]
    pub desktop_notifications: Option<bool>,

    /// Whether to show in-app notifications
    #[serde(rename = "inAppNotifications", skip_serializing_if = "Option::is_none")]
    pub in_app_notifications: Option<bool>,

    /// Whether to send email notifications
    #[serde(rename = "emailNotifications", skip_serializing_if = "Option::is_none")]
    pub email_notifications: Option<bool>,

    /// Custom notification message
    #[serde(rename = "customMessage", skip_serializing_if = "Option::is_none")]
    pub custom_message: Option<String>,
}

impl SamplingRequest {
    /// Validate the sampling request
    pub fn validate(&self) -> Result<(), String> {
        // Validate messages
        if self.messages.is_empty() {
            return Err("At least one message is required".to_string());
        }

        // Validate temperature range
        if let Some(temp) = self.temperature {
            if !(0.0..=1.0).contains(&temp) {
                return Err("Temperature must be between 0.0 and 1.0".to_string());
            }
        }

        // Validate max tokens
        if let Some(max_tokens) = self.max_tokens {
            if max_tokens == 0 {
                return Err("Max tokens must be greater than 0".to_string());
            }
            if max_tokens > 100000 {
                return Err("Max tokens cannot exceed 100,000".to_string());
            }
        }

        // Validate model preferences if present
        if let Some(ref prefs) = self.model_preferences {
            prefs.validate()?;
        }

        // Validate stop sequences
        if let Some(ref sequences) = self.stop_sequences {
            if sequences.is_empty() {
                return Err("Stop sequences cannot be empty if provided".to_string());
            }
            for seq in sequences {
                if seq.is_empty() {
                    return Err("Stop sequences cannot contain empty strings".to_string());
                }
                if seq.len() > 1000 {
                    return Err("Stop sequences cannot exceed 1000 characters".to_string());
                }
            }
        }

        // Validate priority
        if let Some(priority) = self.priority {
            if !(0.0..=1.0).contains(&priority) {
                return Err("Priority must be between 0.0 and 1.0".to_string());
            }
        }

        // Validate timeout
        if let Some(timeout) = self.timeout_seconds {
            if timeout == 0 {
                return Err("Timeout must be greater than 0".to_string());
            }
            if timeout > 3600 {
                return Err("Timeout cannot exceed 1 hour (3600 seconds)".to_string());
            }
        }

        // Validate human-in-the-loop settings
        if let Some(ref hitl) = self.human_in_the_loop {
            if let Some(timeout) = hitl.approval_timeout_seconds {
                if timeout == 0 {
                    return Err("Approval timeout must be greater than 0".to_string());
                }
                if timeout > 3600 {
                    return Err("Approval timeout cannot exceed 1 hour (3600 seconds)".to_string());
                }
            }
        }

        Ok(())
    }

    /// Check if human approval is required
    pub fn requires_human_approval(&self) -> bool {
        self.human_in_the_loop
            .as_ref()
            .map(|hitl| {
                hitl.require_prompt_approval.unwrap_or(false)
                    || hitl.require_completion_approval.unwrap_or(false)
            })
            .unwrap_or(false)
    }

    /// Get the effective timeout for this request
    pub fn get_effective_timeout(&self) -> u32 {
        self.timeout_seconds
            .or_else(|| {
                self.human_in_the_loop
                    .as_ref()
                    .and_then(|hitl| hitl.approval_timeout_seconds)
            })
            .unwrap_or(300) // Default 5 minutes
    }

    /// Estimate the cost of this request (simplified version)
    pub fn estimate_cost(&self) -> Result<f64, String> {
        let input_tokens = self.estimate_input_tokens()?;
        let output_tokens = self.max_tokens.unwrap_or(1000) as f64;

        // Simple cost estimation: $0.002 per 1K input tokens, $0.012 per 1K output tokens
        let input_cost = (input_tokens as f64 / 1000.0) * 0.002;
        let output_cost = (output_tokens / 1000.0) * 0.012;

        Ok(input_cost + output_cost)
    }

    /// Estimate input tokens for this request
    pub fn estimate_input_tokens(&self) -> Result<u32, String> {
        let mut total_tokens = 0;

        // Count tokens in messages
        for message in &self.messages {
            total_tokens += match &message.content {
                SamplingContent::Text { text } => {
                    // Rough estimation: 1 token ≈ 4 characters
                    (text.len() as f64 / 4.0).ceil() as u32
                }
                SamplingContent::Image { .. } => {
                    // Images typically count as ~85 tokens
                    85
                }
            };
        }

        // Add system prompt tokens
        if let Some(ref system_prompt) = self.system_prompt {
            total_tokens += (system_prompt.len() as f64 / 4.0).ceil() as u32;
        }

        // Add context tokens if included
        if let Some(include_context) = &self.include_context {
            match include_context {
                IncludeContext::None => {}
                IncludeContext::ThisServer => {
                    // Estimate 500 tokens for server context
                    total_tokens += 500;
                }
                IncludeContext::AllServers => {
                    // Estimate 1000 tokens for all servers context
                    total_tokens += 1000;
                }
            }
        }

        Ok(total_tokens)
    }

    /// Check if this request requires image modality
    pub fn requires_image_modality(&self) -> bool {
        self.messages
            .iter()
            .any(|message| matches!(message.content, SamplingContent::Image { .. }))
    }
}

/// Enhanced sampling response with human-in-the-loop support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingResponse {
    /// Role of the response
    pub role: SamplingRole,

    /// Content of the response
    pub content: SamplingContent,

    /// Model that was used for sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Reason why sampling stopped
    #[serde(rename = "stopReason", skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<StopReason>,

    /// Human approval status
    #[serde(rename = "approvalStatus", skip_serializing_if = "Option::is_none")]
    pub approval_status: Option<ApprovalStatus>,

    /// Request ID for tracking
    #[serde(rename = "requestId", skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,

    /// Processing time in milliseconds
    #[serde(rename = "processingTimeMs", skip_serializing_if = "Option::is_none")]
    pub processing_time_ms: Option<u64>,

    /// Cost information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_info: Option<CostInfo>,

    /// Context that was included in the request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub included_context: Option<SamplingContext>,

    /// Human feedback or modifications
    #[serde(rename = "humanFeedback", skip_serializing_if = "Option::is_none")]
    pub human_feedback: Option<HumanFeedback>,

    /// Warnings or considerations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<String>>,
}

/// Cost information for the sampling request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostInfo {
    /// Total cost in cents
    #[serde(rename = "totalCostCents")]
    pub total_cost_cents: f64,

    /// Input token cost in cents
    #[serde(rename = "inputCostCents")]
    pub input_cost_cents: f64,

    /// Output token cost in cents
    #[serde(rename = "outputCostCents")]
    pub output_cost_cents: f64,

    /// Input tokens used
    #[serde(rename = "inputTokens")]
    pub input_tokens: u32,

    /// Output tokens generated
    #[serde(rename = "outputTokens")]
    pub output_tokens: u32,

    /// Model used for cost calculation
    pub model: String,
}

/// Human feedback for sampling requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanFeedback {
    /// Whether the prompt was modified
    #[serde(rename = "promptModified")]
    pub prompt_modified: bool,

    /// Whether the completion was modified
    #[serde(rename = "completionModified")]
    pub completion_modified: bool,

    /// Reason for approval/rejection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// User comments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,

    /// Approval timestamp
    #[serde(rename = "approvalTimestamp", skip_serializing_if = "Option::is_none")]
    pub approval_timestamp: Option<chrono::DateTime<chrono::Utc>>,
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
        data: String, // Base64 encoded
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
    #[serde(
        rename = "intelligencePriority",
        skip_serializing_if = "Option::is_none"
    )]
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

/// Model capability information for intelligent selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapability {
    /// Model identifier
    pub model_id: String,

    /// Model provider (e.g., "openai", "anthropic", "google")
    pub provider: String,

    /// Model display name
    pub display_name: String,

    /// Model version or variant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Cost per 1K input tokens (in cents)
    pub cost_per_1k_input_tokens: f64,

    /// Cost per 1K output tokens (in cents)
    pub cost_per_1k_output_tokens: f64,

    /// Relative speed score (1-10, higher is faster)
    pub speed_score: f64,

    /// Relative intelligence score (1-10, higher is more capable)
    pub intelligence_score: f64,

    /// Maximum context length
    pub max_context_length: u32,

    /// Supported modalities
    pub modalities: Vec<Modality>,

    /// Whether the model supports function calling
    pub supports_function_calling: bool,

    /// Whether the model supports streaming
    pub supports_streaming: bool,

    /// Additional metadata about the model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

/// Supported input/output modalities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Modality {
    Text,
    Image,
    Audio,
    Video,
    Code,
}

/// Model selection context for intelligent choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSelectionContext {
    /// Available models with their capabilities
    pub available_models: Vec<ModelCapability>,

    /// Current request context
    pub request_context: RequestContext,

    /// User preferences for this selection
    pub preferences: ModelPreferences,

    /// Previous model performance data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance_history: Option<Vec<ModelPerformanceRecord>>,
}

/// Context information about the current request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    /// Estimated input token count
    pub estimated_input_tokens: u32,

    /// Estimated output token count
    pub estimated_output_tokens: u32,

    /// Required modalities for this request
    pub required_modalities: Vec<Modality>,

    /// Whether function calling is required
    pub requires_function_calling: bool,

    /// Whether streaming is preferred
    pub prefers_streaming: bool,

    /// Task complexity level (0-1, higher = more complex)
    pub complexity_level: f64,

    /// Response time sensitivity (0-1, higher = more time sensitive)
    pub time_sensitivity: f64,

    /// Quality requirements (0-1, higher = higher quality needed)
    pub quality_requirements: f64,
}

/// Historical performance data for a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformanceRecord {
    /// Model identifier
    pub model_id: String,

    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,

    /// Success rate (0.0-1.0)
    pub success_rate: f64,

    /// User satisfaction score (1-10)
    pub satisfaction_score: f64,

    /// Number of samples this record is based on
    pub sample_count: u32,

    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Result of model selection with reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSelectionResult {
    /// Selected model
    pub selected_model: ModelCapability,

    /// Selection confidence score (0.0-1.0)
    pub confidence_score: f64,

    /// Reasoning for this selection
    pub selection_reasoning: SelectionReasoning,

    /// Alternative models considered
    pub alternatives: Vec<AlternativeModel>,

    /// Estimated cost for this request
    pub estimated_cost_cents: f64,

    /// Estimated response time
    pub estimated_response_time_ms: f64,
}

/// Detailed reasoning for model selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionReasoning {
    /// Primary factors that influenced the decision
    pub primary_factors: Vec<SelectionFactor>,

    /// Weighted scores for each priority
    pub priority_scores: PriorityScores,

    /// Trade-offs made in the selection
    pub trade_offs: Vec<String>,

    /// Warnings or considerations
    pub warnings: Vec<String>,
}

/// Factors that influenced model selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionFactor {
    /// Factor name
    pub factor: String,

    /// Importance weight (0.0-1.0)
    pub weight: f64,

    /// How this factor influenced the decision
    pub reasoning: String,
}

/// Calculated priority scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityScores {
    /// Cost optimization score (0.0-1.0)
    pub cost_score: f64,

    /// Speed optimization score (0.0-1.0)
    pub speed_score: f64,

    /// Intelligence optimization score (0.0-1.0)
    pub intelligence_score: f64,

    /// Overall composite score (0.0-1.0)
    pub composite_score: f64,
}

/// Alternative model that was considered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeModel {
    /// Alternative model
    pub model: ModelCapability,

    /// Composite score for this alternative
    pub score: f64,

    /// Reason it wasn't selected
    pub rejection_reason: String,
}

impl ModelPreferences {
    /// Create preferences optimized for cost
    pub fn cost_optimized() -> Self {
        Self {
            hints: None,
            cost_priority: Some(0.1),  // Highest priority (lowest number)
            speed_priority: Some(0.7), // Much lower priority
            intelligence_priority: Some(0.9), // Lowest priority
        }
    }

    /// Create preferences optimized for speed
    pub fn speed_optimized() -> Self {
        Self {
            hints: None,
            cost_priority: Some(0.7),         // Much lower priority
            speed_priority: Some(0.1),        // Highest priority (lowest number)
            intelligence_priority: Some(0.9), // Lowest priority
        }
    }

    /// Create preferences optimized for intelligence/quality
    pub fn intelligence_optimized() -> Self {
        Self {
            hints: None,
            cost_priority: Some(0.9),         // Lowest priority
            speed_priority: Some(0.7),        // Much lower priority
            intelligence_priority: Some(0.1), // Highest priority (lowest number)
        }
    }

    /// Create balanced preferences
    pub fn balanced() -> Self {
        Self {
            hints: None,
            cost_priority: Some(0.33),
            speed_priority: Some(0.33),
            intelligence_priority: Some(0.34),
        }
    }

    /// Validate that priorities are within valid ranges
    pub fn validate(&self) -> Result<(), String> {
        if let Some(cost) = self.cost_priority {
            if !(0.0..=1.0).contains(&cost) {
                return Err("Cost priority must be between 0.0 and 1.0".to_string());
            }
        }

        if let Some(speed) = self.speed_priority {
            if !(0.0..=1.0).contains(&speed) {
                return Err("Speed priority must be between 0.0 and 1.0".to_string());
            }
        }

        if let Some(intelligence) = self.intelligence_priority {
            if !(0.0..=1.0).contains(&intelligence) {
                return Err("Intelligence priority must be between 0.0 and 1.0".to_string());
            }
        }

        Ok(())
    }

    /// Get normalized priority weights that sum to 1.0
    pub fn get_normalized_weights(&self) -> (f64, f64, f64) {
        let cost = self.cost_priority.unwrap_or(0.5);
        let speed = self.speed_priority.unwrap_or(0.5);
        let intelligence = self.intelligence_priority.unwrap_or(0.5);

        // Convert priorities to weights (lower priority = higher weight)
        // Since priorities are now 0-1, we invert them: weight = 1 - priority
        let cost_weight = 1.0 - cost;
        let speed_weight = 1.0 - speed;
        let intelligence_weight = 1.0 - intelligence;

        let total = cost_weight + speed_weight + intelligence_weight;

        if total == 0.0 {
            (1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0)
        } else {
            (
                cost_weight / total,
                speed_weight / total,
                intelligence_weight / total,
            )
        }
    }
}

/// Intelligent model selector that considers preferences and context
#[derive(Debug, Clone)]
pub struct ModelSelector {
    /// Model scoring strategy
    scoring_strategy: ScoringStrategy,

    /// Minimum confidence threshold for selection
    min_confidence_threshold: f64,

    /// Maximum alternatives to consider
    max_alternatives: usize,
}

/// Strategy for scoring and ranking models
#[derive(Debug, Clone)]
pub enum ScoringStrategy {
    /// Weighted sum of normalized scores
    WeightedSum,

    /// Pareto-optimal selection with trade-off analysis
    ParetoOptimal,

    /// Machine learning-based selection (placeholder for future ML models)
    MLBased,
}

impl Default for ModelSelector {
    fn default() -> Self {
        Self {
            scoring_strategy: ScoringStrategy::WeightedSum,
            min_confidence_threshold: 0.3, // Lower threshold for better usability
            max_alternatives: 3,
        }
    }
}

impl ModelSelector {
    /// Create a new model selector with custom configuration
    pub fn new(strategy: ScoringStrategy, min_confidence: f64, max_alternatives: usize) -> Self {
        Self {
            scoring_strategy: strategy,
            min_confidence_threshold: min_confidence,
            max_alternatives,
        }
    }

    /// Select the best model for the given context
    pub fn select_model(
        &self,
        context: &ModelSelectionContext,
    ) -> Result<ModelSelectionResult, String> {
        // Validate preferences
        context.preferences.validate()?;

        // Filter models based on requirements
        let candidate_models =
            self.filter_candidate_models(&context.available_models, &context.request_context)?;

        if candidate_models.is_empty() {
            return Err("No models meet the requirements".to_string());
        }

        // Score each candidate model
        let scored_models = self.score_models(&candidate_models, context)?;

        // Select the best model
        let best_model = scored_models
            .first()
            .ok_or("No valid models found after scoring")?;

        // Check confidence threshold
        if best_model.1 < self.min_confidence_threshold {
            return Err(format!(
                "Best model confidence {} below threshold {}",
                best_model.1, self.min_confidence_threshold
            ));
        }

        // Build selection result
        self.build_selection_result(&scored_models, context)
    }

    /// Filter models based on hard requirements
    fn filter_candidate_models(
        &self,
        models: &[ModelCapability],
        context: &RequestContext,
    ) -> Result<Vec<ModelCapability>, String> {
        let mut candidates = Vec::new();

        for model in models {
            // Check context length requirement
            let required_context = context.estimated_input_tokens + context.estimated_output_tokens;
            if model.max_context_length < required_context {
                continue;
            }

            // Check modality requirements
            let has_required_modalities = context
                .required_modalities
                .iter()
                .all(|modality| model.modalities.contains(modality));
            if !has_required_modalities {
                continue;
            }

            // Check function calling requirement
            if context.requires_function_calling && !model.supports_function_calling {
                continue;
            }

            // Check streaming preference (not a hard requirement)
            // We include all models but will score streaming capability

            candidates.push(model.clone());
        }

        Ok(candidates)
    }

    /// Score models based on preferences and context
    fn score_models(
        &self,
        models: &[ModelCapability],
        context: &ModelSelectionContext,
    ) -> Result<Vec<(ModelCapability, f64)>, String> {
        let mut scored_models = Vec::new();
        let (cost_weight, speed_weight, intelligence_weight) =
            context.preferences.get_normalized_weights();

        for model in models {
            let score = match self.scoring_strategy {
                ScoringStrategy::WeightedSum => self.calculate_weighted_sum_score(
                    model,
                    context,
                    cost_weight,
                    speed_weight,
                    intelligence_weight,
                )?,
                ScoringStrategy::ParetoOptimal => self.calculate_pareto_score(model, context)?,
                ScoringStrategy::MLBased => {
                    // Placeholder for future ML-based scoring
                    self.calculate_weighted_sum_score(
                        model,
                        context,
                        cost_weight,
                        speed_weight,
                        intelligence_weight,
                    )?
                }
            };

            scored_models.push((model.clone(), score));
        }

        // Sort by score (descending)
        scored_models.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        Ok(scored_models)
    }

    /// Calculate weighted sum score for a model
    fn calculate_weighted_sum_score(
        &self,
        model: &ModelCapability,
        context: &ModelSelectionContext,
        cost_weight: f64,
        speed_weight: f64,
        intelligence_weight: f64,
    ) -> Result<f64, String> {
        // Normalize scores to 0-1 range
        let cost_score = self.calculate_cost_score(model, &context.request_context);
        let speed_score = self.calculate_speed_score(model, &context.request_context);
        let intelligence_score = self.calculate_intelligence_score(model, &context.request_context);

        // Apply performance history if available
        let performance_multiplier = if let Some(ref history) = context.performance_history {
            self.get_performance_multiplier(model, history)
        } else {
            1.0
        };

        // Calculate weighted sum
        let base_score = cost_score * cost_weight
            + speed_score * speed_weight
            + intelligence_score * intelligence_weight;

        // Apply performance multiplier
        let final_score = base_score * performance_multiplier;

        // Don't clamp final score to allow high-performing models to score higher
        Ok(final_score.max(0.0))
    }

    /// Calculate cost efficiency score (higher is better/cheaper)
    fn calculate_cost_score(&self, model: &ModelCapability, context: &RequestContext) -> f64 {
        let input_cost =
            (context.estimated_input_tokens as f64 / 1000.0) * model.cost_per_1k_input_tokens;
        let output_cost =
            (context.estimated_output_tokens as f64 / 1000.0) * model.cost_per_1k_output_tokens;
        let total_cost = input_cost + output_cost;

        // Use exponential scoring to create dramatic differences between cheap and expensive models
        let normalized_cost = (total_cost / 10.0).min(1.0); // Normalize to 0-1 range

        // Exponential decay: cheap models get much higher scores than expensive ones
        let score = (-normalized_cost * 4.0).exp(); // e^(-4x) creates steep curve

        score.clamp(0.0, 1.0)
    }

    /// Calculate speed score based on model capability and request context
    fn calculate_speed_score(&self, model: &ModelCapability, context: &RequestContext) -> f64 {
        let base_score = model.speed_score / 10.0; // Normalize to 0-1

        // Apply exponential scaling to dramatically favor faster models for time-sensitive tasks
        let mut score = base_score.powf(0.5); // Square root scaling

        // Bonus for streaming if preferred
        if context.prefers_streaming && model.supports_streaming {
            score = (score * 1.2).min(2.0);
        }

        // Major speed bonus for time-sensitive tasks - exponential scaling
        if context.time_sensitivity > 0.7 {
            let time_factor = (context.time_sensitivity - 0.7) / 0.3; // 0.0 to 1.0
            let speed_bonus = base_score.powf(2.0) * time_factor; // Exponential bonus
            score += speed_bonus;
        }

        // Clamp to allow fast models to score much higher than slow ones
        score.clamp(0.0, 2.0)
    }

    /// Calculate intelligence/capability score
    fn calculate_intelligence_score(
        &self,
        model: &ModelCapability,
        context: &RequestContext,
    ) -> f64 {
        let base_score = model.intelligence_score / 10.0; // Normalize to 0-1

        // Apply exponential scaling to dramatically favor higher intelligence models
        let mut score = base_score.powf(0.5); // Square root scaling

        // Adjust based on task complexity - exponential bonus for high intelligence on complex tasks
        if context.complexity_level > 0.7 {
            let complexity_factor = (context.complexity_level - 0.7) / 0.3; // 0.0 to 1.0
            let intelligence_bonus = base_score.powf(2.0) * complexity_factor; // Exponential bonus
            score += intelligence_bonus;
        }

        // Adjust based on quality requirements - similar exponential bonus
        if context.quality_requirements > 0.8 {
            let quality_factor = (context.quality_requirements - 0.8) / 0.2; // 0.0 to 1.0
            let intelligence_bonus = base_score.powf(2.0) * quality_factor; // Exponential bonus
            score += intelligence_bonus;
        }

        // Clamp to prevent extreme values but allow high scores for truly smart models
        score.clamp(0.0, 2.0)
    }

    /// Calculate Pareto-optimal score (simplified version)
    fn calculate_pareto_score(
        &self,
        model: &ModelCapability,
        context: &ModelSelectionContext,
    ) -> Result<f64, String> {
        // For now, fall back to weighted sum
        // In a full implementation, this would use Pareto frontier analysis
        let (cost_weight, speed_weight, intelligence_weight) =
            context.preferences.get_normalized_weights();
        self.calculate_weighted_sum_score(
            model,
            context,
            cost_weight,
            speed_weight,
            intelligence_weight,
        )
    }

    /// Get performance multiplier based on historical data
    fn get_performance_multiplier(
        &self,
        model: &ModelCapability,
        history: &[ModelPerformanceRecord],
    ) -> f64 {
        if let Some(record) = history.iter().find(|r| r.model_id == model.model_id) {
            // Use success rate and satisfaction score to create multiplier
            let success_component = record.success_rate.powf(2.0); // Exponential penalty for low success rates
            let satisfaction_component = (record.satisfaction_score / 10.0).powf(2.0); // Exponential reward for high satisfaction
            let performance_score = (success_component + satisfaction_component) / 2.0;

            // More aggressive multiplier range from 0.5 to 1.5 to strongly influence selection
            0.5 + performance_score
        } else {
            1.0 // No history, use neutral multiplier
        }
    }

    /// Build comprehensive selection result
    fn build_selection_result(
        &self,
        scored_models: &[(ModelCapability, f64)],
        context: &ModelSelectionContext,
    ) -> Result<ModelSelectionResult, String> {
        let (selected_model, confidence_score) = scored_models.first().unwrap();

        // Calculate estimated cost
        let input_cost = (context.request_context.estimated_input_tokens as f64 / 1000.0)
            * selected_model.cost_per_1k_input_tokens;
        let output_cost = (context.request_context.estimated_output_tokens as f64 / 1000.0)
            * selected_model.cost_per_1k_output_tokens;
        let estimated_cost = input_cost + output_cost;

        // Estimate response time (simplified)
        let base_response_time = 2000.0 / selected_model.speed_score; // Base formula
        let token_factor = context.request_context.estimated_output_tokens as f64 * 10.0; // ~10ms per token
        let estimated_response_time = base_response_time + token_factor;

        // Build selection reasoning
        let selection_reasoning = self.build_selection_reasoning(scored_models, context);

        // Build alternatives list
        let alternatives = scored_models
            .iter()
            .skip(1)
            .take(self.max_alternatives)
            .map(|(model, score)| AlternativeModel {
                model: model.clone(),
                score: *score,
                rejection_reason: format!("Lower composite score: {score:.3}"),
            })
            .collect();

        Ok(ModelSelectionResult {
            selected_model: selected_model.clone(),
            confidence_score: *confidence_score,
            selection_reasoning,
            alternatives,
            estimated_cost_cents: estimated_cost,
            estimated_response_time_ms: estimated_response_time,
        })
    }

    /// Build detailed selection reasoning
    fn build_selection_reasoning(
        &self,
        scored_models: &[(ModelCapability, f64)],
        context: &ModelSelectionContext,
    ) -> SelectionReasoning {
        let selected_model = &scored_models[0].0;
        let (cost_weight, speed_weight, intelligence_weight) =
            context.preferences.get_normalized_weights();

        // Primary factors
        let mut primary_factors = Vec::new();

        if cost_weight > 0.4 {
            primary_factors.push(SelectionFactor {
                factor: "Cost Efficiency".to_string(),
                weight: cost_weight,
                reasoning: format!(
                    "Cost per request: ${:.4}",
                    self.calculate_request_cost(selected_model, &context.request_context)
                ),
            });
        }

        if speed_weight > 0.4 {
            primary_factors.push(SelectionFactor {
                factor: "Response Speed".to_string(),
                weight: speed_weight,
                reasoning: format!("Speed score: {}/10", selected_model.speed_score),
            });
        }

        if intelligence_weight > 0.4 {
            primary_factors.push(SelectionFactor {
                factor: "Model Intelligence".to_string(),
                weight: intelligence_weight,
                reasoning: format!(
                    "Intelligence score: {}/10",
                    selected_model.intelligence_score
                ),
            });
        }

        // Calculate priority scores
        let priority_scores = PriorityScores {
            cost_score: self.calculate_cost_score(selected_model, &context.request_context),
            speed_score: self.calculate_speed_score(selected_model, &context.request_context),
            intelligence_score: self
                .calculate_intelligence_score(selected_model, &context.request_context),
            composite_score: scored_models[0].1,
        };

        // Trade-offs and warnings
        let trade_offs = self.identify_trade_offs(selected_model, context);
        let warnings = self.identify_warnings(selected_model, context);

        SelectionReasoning {
            primary_factors,
            priority_scores,
            trade_offs,
            warnings,
        }
    }

    /// Calculate request cost for a model
    fn calculate_request_cost(&self, model: &ModelCapability, context: &RequestContext) -> f64 {
        let input_cost = (context.estimated_input_tokens as f64 / 1000.0)
            * model.cost_per_1k_input_tokens
            / 100.0;
        let output_cost = (context.estimated_output_tokens as f64 / 1000.0)
            * model.cost_per_1k_output_tokens
            / 100.0;
        input_cost + output_cost
    }

    /// Identify trade-offs made in selection
    fn identify_trade_offs(
        &self,
        selected_model: &ModelCapability,
        context: &ModelSelectionContext,
    ) -> Vec<String> {
        let mut trade_offs = Vec::new();

        // Check if we're trading cost for performance
        if selected_model.cost_per_1k_input_tokens > 50.0 {
            trade_offs.push("Higher cost for better performance".to_string());
        }

        // Check if we're trading speed for intelligence
        if selected_model.speed_score < 5.0 && selected_model.intelligence_score > 8.0 {
            trade_offs.push("Slower response for higher quality output".to_string());
        }

        // Check context length limitations
        let required_context = context.request_context.estimated_input_tokens
            + context.request_context.estimated_output_tokens;
        if selected_model.max_context_length < required_context * 2 {
            trade_offs.push(
                "Limited context window may affect performance on very long inputs".to_string(),
            );
        }

        trade_offs
    }

    /// Identify warnings or considerations
    fn identify_warnings(
        &self,
        selected_model: &ModelCapability,
        context: &ModelSelectionContext,
    ) -> Vec<String> {
        let mut warnings = Vec::new();

        // Warn about high cost
        let request_cost = self.calculate_request_cost(selected_model, &context.request_context);
        if request_cost > 0.10 {
            warnings.push(format!("High cost per request: ${request_cost:.4}"));
        }

        // Warn about potential slow response
        if selected_model.speed_score < 3.0 && context.request_context.time_sensitivity > 0.7 {
            warnings.push(
                "Selected model may be slower than desired for time-sensitive request".to_string(),
            );
        }

        // Warn about context limits
        let context_utilization = (context.request_context.estimated_input_tokens
            + context.request_context.estimated_output_tokens)
            as f64
            / selected_model.max_context_length as f64;
        if context_utilization > 0.8 {
            warnings.push("High context utilization may lead to truncation".to_string());
        }

        warnings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_model(
        id: &str,
        cost_input: f64,
        cost_output: f64,
        speed: f64,
        intelligence: f64,
    ) -> ModelCapability {
        ModelCapability {
            model_id: id.to_string(),
            provider: "test".to_string(),
            display_name: format!("Test Model {id}"),
            version: None,
            cost_per_1k_input_tokens: cost_input,
            cost_per_1k_output_tokens: cost_output,
            speed_score: speed,
            intelligence_score: intelligence,
            max_context_length: 4096,
            modalities: vec![Modality::Text],
            supports_function_calling: true,
            supports_streaming: true,
            metadata: None,
        }
    }

    #[test]
    fn test_model_preferences_validation() {
        let mut prefs = ModelPreferences::balanced();
        assert!(prefs.validate().is_ok());

        prefs.cost_priority = Some(1.5); // Invalid - should be 0-1
        assert!(prefs.validate().is_err());
    }

    #[test]
    fn test_normalized_weights() {
        let prefs = ModelPreferences {
            hints: None,
            cost_priority: Some(0.2),         // High priority (low number)
            speed_priority: Some(0.8),        // Low priority (high number)
            intelligence_priority: Some(0.5), // Medium priority
        };

        let (cost_weight, speed_weight, intelligence_weight) = prefs.get_normalized_weights();

        // Cost should have highest weight (lowest priority number)
        assert!(cost_weight > speed_weight);
        assert!(cost_weight > intelligence_weight);

        // Weights should sum to approximately 1.0
        let sum = cost_weight + speed_weight + intelligence_weight;
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_model_selection() {
        let models = vec![
            create_test_model("cheap", 1.0, 2.0, 6.0, 5.0), // Cheap but average
            create_test_model("fast", 10.0, 20.0, 9.0, 7.0), // Fast but expensive
            create_test_model("smart", 15.0, 30.0, 4.0, 9.5), // Smart but slow/expensive
        ];

        let context = ModelSelectionContext {
            available_models: models,
            request_context: RequestContext {
                estimated_input_tokens: 100,
                estimated_output_tokens: 200,
                required_modalities: vec![Modality::Text],
                requires_function_calling: false,
                prefers_streaming: false,
                complexity_level: 0.5,
                time_sensitivity: 0.5,
                quality_requirements: 0.5,
            },
            preferences: ModelPreferences::cost_optimized(),
            performance_history: None,
        };

        let selector = ModelSelector::default();
        let result = selector.select_model(&context);

        assert!(result.is_ok());
        let result = result.unwrap();

        // Should select the cheap model for cost-optimized preferences
        assert_eq!(result.selected_model.model_id, "cheap");
        assert!(result.confidence_score > 0.0);
        assert!(!result.alternatives.is_empty());
    }

    #[test]
    fn test_speed_optimized_selection() {
        let models = vec![
            create_test_model("cheap", 1.0, 2.0, 6.0, 5.0),
            create_test_model("fast", 10.0, 20.0, 9.0, 7.0),
            create_test_model("smart", 15.0, 30.0, 4.0, 9.5),
        ];

        let context = ModelSelectionContext {
            available_models: models,
            request_context: RequestContext {
                estimated_input_tokens: 100,
                estimated_output_tokens: 200,
                required_modalities: vec![Modality::Text],
                requires_function_calling: false,
                prefers_streaming: false,
                complexity_level: 0.5,
                time_sensitivity: 0.9, // High time sensitivity
                quality_requirements: 0.5,
            },
            preferences: ModelPreferences::speed_optimized(),
            performance_history: None,
        };

        let selector = ModelSelector::default();
        let result = selector.select_model(&context).unwrap();

        // Should select the fast model for speed-optimized preferences
        assert_eq!(result.selected_model.model_id, "fast");
    }

    #[test]
    fn test_intelligence_optimized_selection() {
        let models = vec![
            create_test_model("cheap", 1.0, 2.0, 6.0, 5.0),
            create_test_model("fast", 10.0, 20.0, 9.0, 7.0),
            create_test_model("smart", 15.0, 30.0, 4.0, 9.5),
        ];

        let context = ModelSelectionContext {
            available_models: models,
            request_context: RequestContext {
                estimated_input_tokens: 100,
                estimated_output_tokens: 200,
                required_modalities: vec![Modality::Text],
                requires_function_calling: false,
                prefers_streaming: false,
                complexity_level: 0.9, // High complexity
                time_sensitivity: 0.3,
                quality_requirements: 0.95, // High quality requirements
            },
            preferences: ModelPreferences::intelligence_optimized(),
            performance_history: None,
        };

        let selector = ModelSelector::default();
        let result = selector.select_model(&context).unwrap();

        // Should select the smart model for intelligence-optimized preferences
        assert_eq!(result.selected_model.model_id, "smart");
    }

    #[test]
    fn test_model_filtering() {
        let models = vec![
            ModelCapability {
                model_id: "limited".to_string(),
                provider: "test".to_string(),
                display_name: "Limited Model".to_string(),
                version: None,
                cost_per_1k_input_tokens: 1.0,
                cost_per_1k_output_tokens: 2.0,
                speed_score: 8.0,
                intelligence_score: 6.0,
                max_context_length: 100, // Too small
                modalities: vec![Modality::Text],
                supports_function_calling: false, // Missing required feature
                supports_streaming: true,
                metadata: None,
            },
            create_test_model("suitable", 5.0, 10.0, 7.0, 8.0),
        ];

        let context = ModelSelectionContext {
            available_models: models,
            request_context: RequestContext {
                estimated_input_tokens: 1000, // Exceeds limited model's context
                estimated_output_tokens: 200,
                required_modalities: vec![Modality::Text],
                requires_function_calling: true, // Limited model doesn't support this
                prefers_streaming: false,
                complexity_level: 0.5,
                time_sensitivity: 0.5,
                quality_requirements: 0.5,
            },
            preferences: ModelPreferences::balanced(),
            performance_history: None,
        };

        let selector = ModelSelector::default();
        let result = selector.select_model(&context).unwrap();

        // Should select the suitable model, filtering out the limited one
        assert_eq!(result.selected_model.model_id, "suitable");
    }

    #[test]
    fn test_performance_history_influence() {
        let models = vec![
            create_test_model("reliable", 5.0, 10.0, 7.0, 8.0),
            create_test_model("unreliable", 3.0, 6.0, 8.0, 8.5),
        ];

        let performance_history = vec![
            ModelPerformanceRecord {
                model_id: "reliable".to_string(),
                avg_response_time_ms: 1000.0,
                success_rate: 0.98,
                satisfaction_score: 9.0,
                sample_count: 100,
                last_updated: Utc::now(),
            },
            ModelPerformanceRecord {
                model_id: "unreliable".to_string(),
                avg_response_time_ms: 800.0,
                success_rate: 0.75,      // Poor success rate
                satisfaction_score: 6.0, // Low satisfaction
                sample_count: 50,
                last_updated: Utc::now(),
            },
        ];

        let context = ModelSelectionContext {
            available_models: models,
            request_context: RequestContext {
                estimated_input_tokens: 100,
                estimated_output_tokens: 200,
                required_modalities: vec![Modality::Text],
                requires_function_calling: false,
                prefers_streaming: false,
                complexity_level: 0.5,
                time_sensitivity: 0.5,
                quality_requirements: 0.5,
            },
            preferences: ModelPreferences::balanced(),
            performance_history: Some(performance_history),
        };

        let selector = ModelSelector::default();
        let result = selector.select_model(&context).unwrap();

        // Should prefer the reliable model despite potentially higher cost
        assert_eq!(result.selected_model.model_id, "reliable");
    }
}
