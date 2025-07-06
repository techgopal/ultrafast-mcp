use regex::Regex;
use serde::{Deserialize, Serialize};

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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GetPromptRequest {
    /// Prompt name to retrieve
    pub name: String,
    /// Optional arguments for prompt template
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<serde_json::Value>,
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
        data: String, // Base64 encoded
        #[serde(rename = "mimeType")]
        mime_type: String,
    },

    #[serde(rename = "resource")]
    Resource { resource: EmbeddedResourceReference },

    #[serde(rename = "resource_link")]
    ResourceLink {
        name: String,
        uri: String,
    },
}

/// Enhanced embedded resource reference for prompts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedResourceReference {
    /// Resource URI
    pub uri: String,

    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Content fallback if resource is unavailable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback: Option<String>,

    /// Resource inclusion options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ResourceInclusionOptions>,
}

/// Resource inclusion options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInclusionOptions {
    /// Whether to validate resource before inclusion
    #[serde(default = "default_true")]
    pub validate: bool,

    /// Whether to cache the resource content
    #[serde(default = "default_true")]
    pub cache: bool,

    /// Maximum resource size to include (in bytes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_size: Option<usize>,

    /// Allowed MIME types for inclusion
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_mime_types: Option<Vec<String>>,

    /// Security policy for resource inclusion
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_policy: Option<ResourceSecurityPolicy>,

    /// Whether to resolve relative URIs
    #[serde(default = "default_true")]
    pub resolve_relative: bool,

    /// Timeout for resource fetching (in seconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<u64>,
}

/// Security policy for embedded resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSecurityPolicy {
    /// Allowed URI schemes
    pub allowed_schemes: Vec<String>,

    /// Blocked URI patterns (regex)
    pub blocked_patterns: Vec<String>,

    /// Whether to allow external resources
    #[serde(default = "default_false")]
    pub allow_external: bool,

    /// Whether to allow local file access
    #[serde(default = "default_false")]
    pub allow_local_files: bool,

    /// Maximum redirection depth
    #[serde(default = "default_max_redirects")]
    pub max_redirects: u32,

    /// Whether to validate SSL certificates
    #[serde(default = "default_true")]
    pub validate_ssl: bool,
}

impl Default for ResourceSecurityPolicy {
    fn default() -> Self {
        Self {
            allowed_schemes: vec!["https".to_string(), "http".to_string(), "file".to_string()],
            blocked_patterns: vec![
                r".*\.\..*".to_string(),                          // Path traversal
                r".*localhost.*".to_string(),                     // Local access
                r".*127\.0\.0\.1.*".to_string(),                  // Local access
                r".*0\.0\.0\.0.*".to_string(),                    // Local access
                r".*192\.168\..*".to_string(),                    // Private networks
                r".*10\..*".to_string(),                          // Private networks
                r".*172\.(1[6-9]|2[0-9]|3[0-1])\..*".to_string(), // Private networks
            ],
            allow_external: false,
            allow_local_files: false,
            max_redirects: 3,
            validate_ssl: true,
        }
    }
}

impl Default for ResourceInclusionOptions {
    fn default() -> Self {
        Self {
            validate: true,
            cache: true,
            max_size: Some(10 * 1024 * 1024), // 10MB default
            allowed_mime_types: Some(vec![
                "text/plain".to_string(),
                "text/markdown".to_string(),
                "application/json".to_string(),
                "text/html".to_string(),
                "text/css".to_string(),
                "text/javascript".to_string(),
                "image/jpeg".to_string(),
                "image/png".to_string(),
                "image/gif".to_string(),
                "image/svg+xml".to_string(),
            ]),
            security_policy: Some(ResourceSecurityPolicy::default()),
            resolve_relative: true,
            timeout_seconds: Some(30),
        }
    }
}

/// Resource resolution result
#[derive(Debug, Clone)]
pub struct ResourceResolution {
    /// Resolved content
    pub content: String,

    /// MIME type of resolved content
    pub mime_type: String,

    /// Final URI after resolution (may differ from original due to redirects)
    pub final_uri: String,

    /// Size of resolved content in bytes
    pub size: usize,

    /// Whether content was retrieved from cache
    pub from_cache: bool,

    /// Resolution metadata
    pub metadata: ResolutionMetadata,
}

/// Resource resolution metadata
#[derive(Debug, Clone)]
pub struct ResolutionMetadata {
    /// Time taken to resolve resource (in milliseconds)
    pub resolution_time_ms: u64,

    /// Number of redirects followed
    pub redirects_followed: u32,

    /// Whether SSL validation was performed
    pub ssl_validated: bool,

    /// Security warnings encountered
    pub security_warnings: Vec<String>,

    /// Validation errors encountered
    pub validation_errors: Vec<String>,
}

/// Resource resolution errors
#[derive(Debug, thiserror::Error)]
pub enum ResourceResolutionError {
    #[error("Invalid URI: {0}")]
    InvalidUri(String),

    #[error("Disallowed URI scheme: {0}")]
    DisallowedScheme(String),

    #[error("Blocked URI pattern: {0}")]
    BlockedPattern(String),

    #[error("Resource too large: {0} bytes (max: {1})")]
    ResourceTooLarge(usize, usize),

    #[error("Disallowed MIME type: {0}")]
    DisallowedMimeType(String),

    #[error("External resources not allowed")]
    ExternalResourcesNotAllowed,

    #[error("Local files not allowed")]
    LocalFilesNotAllowed,

    #[error("Too many redirects: {0} (max: {1})")]
    TooManyRedirects(u32, u32),

    #[error("SSL validation failed: {0}")]
    SslValidationFailed(String),

    #[error("Resource fetch timeout")]
    Timeout,

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Content validation failed: {0}")]
    ContentValidationFailed(String),

    #[error("Security violation: {0}")]
    SecurityViolation(String),
}

/// Prompt embedded resource validator
pub struct EmbeddedResourceValidator {
    security_policy: ResourceSecurityPolicy,
}

impl EmbeddedResourceValidator {
    pub fn new(security_policy: ResourceSecurityPolicy) -> Self {
        Self { security_policy }
    }

    /// Validate an embedded resource reference
    pub fn validate_reference(
        &self,
        resource: &EmbeddedResourceReference,
    ) -> Result<(), ResourceResolutionError> {
        // Validate URI format
        self.validate_uri_format(&resource.uri)?;

        // Validate URI scheme
        self.validate_uri_scheme(&resource.uri)?;

        // Check blocked patterns
        self.check_blocked_patterns(&resource.uri)?;

        // Validate security restrictions
        self.validate_security_restrictions(&resource.uri)?;

        // Validate inclusion options if present
        if let Some(ref options) = resource.options {
            self.validate_inclusion_options(options)?;
        }

        Ok(())
    }

    /// Validate URI format
    fn validate_uri_format(&self, uri: &str) -> Result<(), ResourceResolutionError> {
        if uri.is_empty() {
            return Err(ResourceResolutionError::InvalidUri(
                "URI cannot be empty".to_string(),
            ));
        }

        // Basic URI format validation
        if !uri.contains(':') {
            return Err(ResourceResolutionError::InvalidUri(
                "URI must contain a scheme".to_string(),
            ));
        }

        // Check for obviously malformed URIs
        if uri.contains("..") {
            return Err(ResourceResolutionError::SecurityViolation(
                "Path traversal detected".to_string(),
            ));
        }

        if uri.to_lowercase().contains("<script") {
            return Err(ResourceResolutionError::SecurityViolation(
                "Script injection detected".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate URI scheme
    fn validate_uri_scheme(&self, uri: &str) -> Result<(), ResourceResolutionError> {
        if let Some(colon_pos) = uri.find(':') {
            let scheme = &uri[..colon_pos].to_lowercase();
            if !self
                .security_policy
                .allowed_schemes
                .iter()
                .any(|s| s.to_lowercase() == *scheme)
            {
                return Err(ResourceResolutionError::DisallowedScheme(
                    scheme.to_string(),
                ));
            }
        }
        Ok(())
    }

    /// Check blocked patterns
    fn check_blocked_patterns(&self, uri: &str) -> Result<(), ResourceResolutionError> {
        for pattern in &self.security_policy.blocked_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if regex.is_match(uri) {
                    return Err(ResourceResolutionError::BlockedPattern(pattern.clone()));
                }
            }
        }
        Ok(())
    }

    /// Validate security restrictions
    fn validate_security_restrictions(&self, uri: &str) -> Result<(), ResourceResolutionError> {
        let uri_lower = uri.to_lowercase();

        // Check for local file access first (since file:// is not external)
        if !self.security_policy.allow_local_files && uri_lower.starts_with("file://") {
            return Err(ResourceResolutionError::LocalFilesNotAllowed);
        }

        // Check for external resources
        if !self.security_policy.allow_external
            && (uri_lower.starts_with("http://") || uri_lower.starts_with("https://"))
        {
            return Err(ResourceResolutionError::ExternalResourcesNotAllowed);
        }

        Ok(())
    }

    /// Validate inclusion options
    fn validate_inclusion_options(
        &self,
        options: &ResourceInclusionOptions,
    ) -> Result<(), ResourceResolutionError> {
        // Validate max size
        if let Some(max_size) = options.max_size {
            if max_size > 100 * 1024 * 1024 {
                // 100MB absolute limit
                return Err(ResourceResolutionError::SecurityViolation(
                    "Max size too large".to_string(),
                ));
            }
        }

        // Validate timeout
        if let Some(timeout) = options.timeout_seconds {
            if timeout > 300 {
                // 5 minutes absolute limit
                return Err(ResourceResolutionError::SecurityViolation(
                    "Timeout too large".to_string(),
                ));
            }
        }

        Ok(())
    }
}

// Helper functions for default values
fn default_true() -> bool {
    true
}
fn default_false() -> bool {
    false
}
fn default_max_redirects() -> u32 {
    3
}

/// List prompts request
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
            resource: EmbeddedResourceReference {
                uri,
                description: None,
                fallback: None,
                options: None,
            },
        }
    }

    pub fn resource_with_options(
        uri: String,
        description: Option<String>,
        options: ResourceInclusionOptions,
    ) -> Self {
        Self::Resource {
            resource: EmbeddedResourceReference {
                uri,
                description,
                fallback: None,
                options: Some(options),
            },
        }
    }

    pub fn resource_with_fallback(uri: String, fallback: String) -> Self {
        Self::Resource {
            resource: EmbeddedResourceReference {
                uri,
                description: None,
                fallback: Some(fallback),
                options: None,
            },
        }
    }

    pub fn resource_link(name: String, uri: String) -> Self {
        Self::ResourceLink { name, uri }
    }
}

impl EmbeddedResourceReference {
    pub fn new(uri: String) -> Self {
        Self {
            uri,
            description: None,
            fallback: None,
            options: None,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_fallback(mut self, fallback: String) -> Self {
        self.fallback = Some(fallback);
        self
    }

    pub fn with_options(mut self, options: ResourceInclusionOptions) -> Self {
        self.options = Some(options);
        self
    }

    /// Validate the embedded resource reference
    pub fn validate(&self) -> Result<(), ResourceResolutionError> {
        let validator = EmbeddedResourceValidator::new(ResourceSecurityPolicy::default());
        validator.validate_reference(self)
    }

    /// Validate with custom security policy
    pub fn validate_with_policy(
        &self,
        policy: &ResourceSecurityPolicy,
    ) -> Result<(), ResourceResolutionError> {
        let validator = EmbeddedResourceValidator::new(policy.clone());
        validator.validate_reference(self)
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
        self.messages
            .push(PromptMessage::user(PromptContent::text(text.to_string())));
        self
    }

    pub fn assistant(mut self, text: &str) -> Self {
        self.messages
            .push(PromptMessage::assistant(PromptContent::text(
                text.to_string(),
            )));
        self
    }

    pub fn system(mut self, text: &str) -> Self {
        self.messages
            .push(PromptMessage::system(PromptContent::text(text.to_string())));
        self
    }

    pub fn with_context(mut self, context: &str) -> Self {
        self.messages
            .push(PromptMessage::system(PromptContent::text(
                context.to_string(),
            )));
        self
    }

    pub fn with_resource(mut self, role: PromptRole, uri: &str) -> Self {
        self.messages.push(PromptMessage {
            role,
            content: PromptContent::resource(uri.to_string()),
        });
        self
    }

    pub fn with_embedded_resource(
        mut self,
        role: PromptRole,
        resource: EmbeddedResourceReference,
    ) -> Self {
        self.messages.push(PromptMessage {
            role,
            content: PromptContent::Resource { resource },
        });
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_resource_reference_creation() {
        let resource = EmbeddedResourceReference::new("https://example.com/data.json".to_string());
        assert_eq!(resource.uri, "https://example.com/data.json");
        assert!(resource.description.is_none());
        assert!(resource.fallback.is_none());
        assert!(resource.options.is_none());
    }

    #[test]
    fn test_embedded_resource_reference_with_options() {
        let options = ResourceInclusionOptions::default();
        let resource = EmbeddedResourceReference::new("https://example.com/data.json".to_string())
            .with_description("Test resource".to_string())
            .with_fallback("fallback content".to_string())
            .with_options(options);

        assert_eq!(resource.uri, "https://example.com/data.json");
        assert_eq!(resource.description, Some("Test resource".to_string()));
        assert_eq!(resource.fallback, Some("fallback content".to_string()));
        assert!(resource.options.is_some());
    }

    #[test]
    fn test_resource_security_policy_defaults() {
        let policy = ResourceSecurityPolicy::default();
        assert_eq!(policy.allowed_schemes, vec!["https", "http", "file"]);
        assert!(!policy.allow_external);
        assert!(!policy.allow_local_files);
        assert_eq!(policy.max_redirects, 3);
        assert!(policy.validate_ssl);
    }

    #[test]
    fn test_resource_inclusion_options_defaults() {
        let options = ResourceInclusionOptions::default();
        assert!(options.validate);
        assert!(options.cache);
        assert_eq!(options.max_size, Some(10 * 1024 * 1024));
        assert!(options.allowed_mime_types.is_some());
        assert!(options.security_policy.is_some());
        assert!(options.resolve_relative);
        assert_eq!(options.timeout_seconds, Some(30));
    }

    #[test]
    fn test_embedded_resource_validator_valid_uri() {
        let validator = EmbeddedResourceValidator::new(ResourceSecurityPolicy::default());
        let resource = EmbeddedResourceReference::new("https://example.com/data.json".to_string());

        // Should fail because external resources are not allowed by default
        let result = validator.validate_reference(&resource);
        assert!(result.is_err());
        match result {
            Err(ResourceResolutionError::ExternalResourcesNotAllowed) => {}
            _ => panic!("Expected ExternalResourcesNotAllowed error"),
        }
    }

    #[test]
    fn test_embedded_resource_validator_with_external_allowed() {
        let policy = ResourceSecurityPolicy {
            allow_external: true,
            ..Default::default()
        };
        let validator = EmbeddedResourceValidator::new(policy);
        let resource = EmbeddedResourceReference::new("https://example.com/data.json".to_string());

        let result = validator.validate_reference(&resource);
        assert!(result.is_ok());
    }

    #[test]
    fn test_embedded_resource_validator_invalid_uri() {
        let validator = EmbeddedResourceValidator::new(ResourceSecurityPolicy::default());
        let resource = EmbeddedResourceReference::new("".to_string());

        let result = validator.validate_reference(&resource);
        assert!(result.is_err());
        match result {
            Err(ResourceResolutionError::InvalidUri(_)) => {}
            _ => panic!("Expected InvalidUri error"),
        }
    }

    #[test]
    fn test_embedded_resource_validator_disallowed_scheme() {
        let validator = EmbeddedResourceValidator::new(ResourceSecurityPolicy::default());
        let resource = EmbeddedResourceReference::new("javascript:alert('xss')".to_string());

        let result = validator.validate_reference(&resource);
        assert!(result.is_err());
        match result {
            Err(ResourceResolutionError::DisallowedScheme(scheme)) => {
                assert_eq!(scheme, "javascript")
            }
            _ => panic!("Expected DisallowedScheme error"),
        }
    }

    #[test]
    fn test_embedded_resource_validator_blocked_pattern() {
        let validator = EmbeddedResourceValidator::new(ResourceSecurityPolicy::default());
        let resource = EmbeddedResourceReference::new("https://localhost/data.json".to_string());

        let result = validator.validate_reference(&resource);
        assert!(result.is_err());
        match result {
            Err(ResourceResolutionError::BlockedPattern(_)) => {}
            _ => panic!("Expected BlockedPattern error"),
        }
    }

    #[test]
    fn test_embedded_resource_validator_path_traversal() {
        let validator = EmbeddedResourceValidator::new(ResourceSecurityPolicy::default());
        let resource = EmbeddedResourceReference::new("https://example.com/../secret".to_string());

        let result = validator.validate_reference(&resource);
        assert!(result.is_err());
        match result {
            Err(ResourceResolutionError::SecurityViolation(_)) => {}
            _ => panic!("Expected SecurityViolation error"),
        }
    }

    #[test]
    fn test_embedded_resource_validator_script_injection() {
        let validator = EmbeddedResourceValidator::new(ResourceSecurityPolicy::default());
        let resource = EmbeddedResourceReference::new(
            "https://example.com/<script>alert('xss')</script>".to_string(),
        );

        let result = validator.validate_reference(&resource);
        assert!(result.is_err());
        match result {
            Err(ResourceResolutionError::SecurityViolation(_)) => {}
            _ => panic!("Expected SecurityViolation error"),
        }
    }

    #[test]
    fn test_embedded_resource_validator_local_file_blocked() {
        let validator = EmbeddedResourceValidator::new(ResourceSecurityPolicy::default());
        let resource = EmbeddedResourceReference::new("file:///etc/passwd".to_string());

        let result = validator.validate_reference(&resource);
        assert!(result.is_err());
        match result {
            Err(ResourceResolutionError::LocalFilesNotAllowed) => {}
            _ => panic!("Expected LocalFilesNotAllowed error, got: {:?}", result),
        }
    }

    #[test]
    fn test_embedded_resource_validator_with_local_files_allowed() {
        let policy = ResourceSecurityPolicy {
            allow_local_files: true,
            ..Default::default()
        };
        let validator = EmbeddedResourceValidator::new(policy);
        let resource = EmbeddedResourceReference::new("file:///tmp/data.json".to_string());

        let result = validator.validate_reference(&resource);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inclusion_options_validation_max_size_too_large() {
        let policy = ResourceSecurityPolicy {
            allow_external: true, // Allow external to test max size validation
            ..Default::default()
        };
        let validator = EmbeddedResourceValidator::new(policy);
        let options = ResourceInclusionOptions {
            max_size: Some(200 * 1024 * 1024), // 200MB
            ..Default::default()
        };
        let resource = EmbeddedResourceReference::new("https://example.com/data.json".to_string())
            .with_options(options);

        let result = validator.validate_reference(&resource);
        assert!(result.is_err());
        match result {
            Err(ResourceResolutionError::SecurityViolation(_)) => {}
            _ => panic!("Expected SecurityViolation error"),
        }
    }

    #[test]
    fn test_inclusion_options_validation_timeout_too_large() {
        let policy = ResourceSecurityPolicy {
            allow_external: true, // Allow external to test timeout validation
            ..Default::default()
        };
        let validator = EmbeddedResourceValidator::new(policy);
        let options = ResourceInclusionOptions {
            timeout_seconds: Some(400), // 400 seconds
            ..Default::default()
        };
        let resource = EmbeddedResourceReference::new("https://example.com/data.json".to_string())
            .with_options(options);

        let result = validator.validate_reference(&resource);
        assert!(result.is_err());
        match result {
            Err(ResourceResolutionError::SecurityViolation(_)) => {}
            _ => panic!("Expected SecurityViolation error"),
        }
    }

    #[test]
    fn test_prompt_content_resource_creation() {
        let content = PromptContent::resource("https://example.com/data.json".to_string());
        match content {
            PromptContent::Resource { resource } => {
                assert_eq!(resource.uri, "https://example.com/data.json");
                assert!(resource.description.is_none());
                assert!(resource.fallback.is_none());
                assert!(resource.options.is_none());
            }
            _ => panic!("Expected Resource variant"),
        }
    }

    #[test]
    fn test_prompt_content_resource_with_options() {
        let options = ResourceInclusionOptions::default();
        let content = PromptContent::resource_with_options(
            "https://example.com/data.json".to_string(),
            Some("Test resource".to_string()),
            options,
        );

        match content {
            PromptContent::Resource { resource } => {
                assert_eq!(resource.uri, "https://example.com/data.json");
                assert_eq!(resource.description, Some("Test resource".to_string()));
                assert!(resource.options.is_some());
            }
            _ => panic!("Expected Resource variant"),
        }
    }

    #[test]
    fn test_prompt_content_resource_with_fallback() {
        let content = PromptContent::resource_with_fallback(
            "https://example.com/data.json".to_string(),
            "fallback content".to_string(),
        );

        match content {
            PromptContent::Resource { resource } => {
                assert_eq!(resource.uri, "https://example.com/data.json");
                assert_eq!(resource.fallback, Some("fallback content".to_string()));
            }
            _ => panic!("Expected Resource variant"),
        }
    }

    #[test]
    fn test_prompt_messages_with_resource() {
        let messages = PromptMessages::new()
            .user("Hello")
            .with_resource(PromptRole::System, "https://example.com/context.json")
            .assistant("World")
            .build();

        assert_eq!(messages.len(), 3);

        match &messages[1].content {
            PromptContent::Resource { resource } => {
                assert_eq!(resource.uri, "https://example.com/context.json");
            }
            _ => panic!("Expected Resource content"),
        }
    }

    #[test]
    fn test_prompt_messages_with_embedded_resource() {
        let resource = EmbeddedResourceReference::new("https://example.com/data.json".to_string())
            .with_description("Test data".to_string())
            .with_fallback("fallback data".to_string());

        let messages = PromptMessages::new()
            .user("Hello")
            .with_embedded_resource(PromptRole::System, resource)
            .assistant("World")
            .build();

        assert_eq!(messages.len(), 3);

        match &messages[1].content {
            PromptContent::Resource { resource } => {
                assert_eq!(resource.uri, "https://example.com/data.json");
                assert_eq!(resource.description, Some("Test data".to_string()));
                assert_eq!(resource.fallback, Some("fallback data".to_string()));
            }
            _ => panic!("Expected Resource content"),
        }
    }

    #[test]
    fn test_embedded_resource_validation_with_custom_policy() {
        let policy = ResourceSecurityPolicy {
            allowed_schemes: vec!["https".to_string()],
            allow_external: true,
            ..Default::default()
        };

        let resource = EmbeddedResourceReference::new("https://example.com/data.json".to_string());
        let result = resource.validate_with_policy(&policy);
        assert!(result.is_ok());

        let resource_http =
            EmbeddedResourceReference::new("http://example.com/data.json".to_string());
        let result = resource_http.validate_with_policy(&policy);
        assert!(result.is_err());
        match result {
            Err(ResourceResolutionError::DisallowedScheme(scheme)) => assert_eq!(scheme, "http"),
            _ => panic!("Expected DisallowedScheme error"),
        }
    }

    #[test]
    fn test_embedded_resource_validate_convenience_method() {
        let resource = EmbeddedResourceReference::new("https://example.com/data.json".to_string());
        let result = resource.validate();
        assert!(result.is_err()); // Should fail because external resources not allowed by default
    }

    #[test]
    fn test_resource_security_policy_private_networks() {
        let validator = EmbeddedResourceValidator::new(ResourceSecurityPolicy::default());

        let private_uris = vec![
            "https://192.168.1.1/data.json",
            "https://10.0.0.1/data.json",
            "https://172.16.0.1/data.json",
        ];

        for uri in private_uris {
            let resource = EmbeddedResourceReference::new(uri.to_string());
            let result = validator.validate_reference(&resource);
            assert!(result.is_err());
            match result {
                Err(ResourceResolutionError::BlockedPattern(_)) => {}
                _ => panic!("Expected BlockedPattern error for {}", uri),
            }
        }
    }

    #[test]
    fn test_prompt_creation_and_validation() {
        let prompt = Prompt::new("test_prompt".to_string())
            .with_description("A test prompt".to_string())
            .with_arguments(vec![PromptArgument::new("context".to_string())
                .with_description("Context data".to_string())
                .required(true)]);

        assert_eq!(prompt.name, "test_prompt");
        assert_eq!(prompt.description, Some("A test prompt".to_string()));
        assert!(prompt.arguments.is_some());

        let args = prompt.arguments.unwrap();
        assert_eq!(args.len(), 1);
        assert_eq!(args[0].name, "context");
        assert_eq!(args[0].required, Some(true));
    }

    #[test]
    fn test_complex_prompt_with_multiple_content_types() {
        let resource =
            EmbeddedResourceReference::new("https://example.com/context.json".to_string())
                .with_description("Context data".to_string())
                .with_fallback("default context".to_string());

        let messages = PromptMessages::new()
            .system("You are a helpful assistant.")
            .with_embedded_resource(PromptRole::System, resource)
            .user("Please analyze the data")
            .build();

        assert_eq!(messages.len(), 3);

        // Check system message
        match &messages[0].content {
            PromptContent::Text { text } => {
                assert_eq!(text, "You are a helpful assistant.");
            }
            _ => panic!("Expected Text content"),
        }

        // Check embedded resource
        match &messages[1].content {
            PromptContent::Resource { resource } => {
                assert_eq!(resource.uri, "https://example.com/context.json");
                assert_eq!(resource.description, Some("Context data".to_string()));
                assert_eq!(resource.fallback, Some("default context".to_string()));
            }
            _ => panic!("Expected Resource content"),
        }

        // Check user message
        match &messages[2].content {
            PromptContent::Text { text } => {
                assert_eq!(text, "Please analyze the data");
            }
            _ => panic!("Expected Text content"),
        }
    }
}
