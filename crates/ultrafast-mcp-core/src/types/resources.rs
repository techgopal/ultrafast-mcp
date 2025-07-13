use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

lazy_static! {
    static ref TEMPLATE_VAR_PATTERN: Regex = Regex::new(r"\{([^}]+)\}").unwrap();
    static ref VAR_NAME_PATTERN: Regex = Regex::new(r"^[a-zA-Z][a-zA-Z0-9_]*$").unwrap();
}

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

/// Template security policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSecurityPolicy {
    /// Maximum number of variables allowed
    pub max_variables: usize,

    /// Maximum template length
    pub max_template_length: usize,

    /// Allowed URI schemes
    pub allowed_schemes: Vec<String>,

    /// Blocked URI patterns (regex)
    pub blocked_patterns: Vec<String>,

    /// Variable name restrictions
    pub variable_name_restrictions: VariableNamePolicy,

    /// Whether to allow nested templates
    pub allow_nested_templates: bool,
}

/// Variable name policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableNamePolicy {
    /// Maximum variable name length
    pub max_length: usize,

    /// Allowed characters pattern (regex)
    pub allowed_pattern: String,

    /// Blocked variable names
    pub blocked_names: Vec<String>,
}

impl Default for TemplateSecurityPolicy {
    fn default() -> Self {
        Self {
            max_variables: 10,
            max_template_length: 2048,
            allowed_schemes: vec![
                "http".to_string(),
                "https".to_string(),
                "file".to_string(),
                "ftp".to_string(),
            ],
            blocked_patterns: vec![
                r".*\.\..*".to_string(),          // Path traversal
                r".*[\s]*<script.*".to_string(),  // Script injection
                r".*javascript:.*".to_string(),   // JavaScript protocol
                r".*vbscript:.*".to_string(),     // VBScript protocol
                r".*data:.*".to_string(),         // Data URLs (can be dangerous)
                r".*file:///etc/.*".to_string(),  // System files
                r".*file:///proc/.*".to_string(), // Process files
                r".*localhost.*".to_string(),     // Local access
                r".*127\.0\.0\.1.*".to_string(),  // Local access
                r".*0\.0\.0\.0.*".to_string(),    // Local access
            ],
            variable_name_restrictions: VariableNamePolicy::default(),
            allow_nested_templates: false,
        }
    }
}

impl Default for VariableNamePolicy {
    fn default() -> Self {
        Self {
            max_length: 50,
            allowed_pattern: r"^[a-zA-Z][a-zA-Z0-9_]*$".to_string(),
            blocked_names: vec![
                "password".to_string(),
                "secret".to_string(),
                "token".to_string(),
                "key".to_string(),
                "auth".to_string(),
                "admin".to_string(),
                "root".to_string(),
                "system".to_string(),
                "etc".to_string(),
                "proc".to_string(),
                "tmp".to_string(),
            ],
        }
    }
}

/// Template expansion options
#[derive(Debug)]
pub struct TemplateExpansionOptions {
    /// Security policy to apply
    pub security_policy: TemplateSecurityPolicy,

    /// Whether to URL encode variable values
    pub url_encode_values: bool,

    /// Whether to validate expanded URI
    pub validate_expanded_uri: bool,

    /// Maximum expanded URI length
    pub max_expanded_length: usize,
}

impl Clone for TemplateExpansionOptions {
    fn clone(&self) -> Self {
        Self {
            security_policy: self.security_policy.clone(),
            url_encode_values: self.url_encode_values,
            validate_expanded_uri: self.validate_expanded_uri,
            max_expanded_length: self.max_expanded_length,
        }
    }
}

impl Default for TemplateExpansionOptions {
    fn default() -> Self {
        Self {
            security_policy: TemplateSecurityPolicy::default(),
            url_encode_values: true,
            validate_expanded_uri: true,
            max_expanded_length: 4096,
        }
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

    /// Parse the URI template and extract variable names
    pub fn parse_variables(&self) -> Vec<String> {
        let mut variables = Vec::new();
        for cap in TEMPLATE_VAR_PATTERN.captures_iter(&self.uri_template) {
            if let Some(var_name) = cap.get(1) {
                let var_name = var_name.as_str().trim();
                if !var_name.is_empty() {
                    variables.push(var_name.to_string());
                }
            }
        }
        variables
    }

    /// Expand the URI template with provided variables
    pub fn expand(&self, variables: &HashMap<String, String>) -> Result<String, TemplateError> {
        self.expand_with_options(variables, &TemplateExpansionOptions::default())
    }

    /// Expand the URI template with options
    pub fn expand_with_options(
        &self,
        variables: &HashMap<String, String>,
        options: &TemplateExpansionOptions,
    ) -> Result<String, TemplateError> {
        // Validate template first
        self.validate_with_policy(&options.security_policy)?;

        let mut result = self.uri_template.clone();

        for cap in TEMPLATE_VAR_PATTERN.captures_iter(&self.uri_template) {
            if let Some(var_match) = cap.get(0) {
                let full_match = var_match.as_str();
                if let Some(var_name) = cap.get(1) {
                    let var_name = var_name.as_str().trim();

                    if let Some(mut value) = variables.get(var_name).cloned() {
                        // URL encode if requested
                        if options.url_encode_values {
                            value = urlencoding::encode(&value).to_string();
                        }

                        result = result.replace(full_match, &value);
                    } else {
                        return Err(TemplateError::MissingVariable(var_name.to_string()));
                    }
                }
            }
        }

        // Check if there are any remaining unexpanded variables
        if TEMPLATE_VAR_PATTERN.is_match(&result) {
            return Err(TemplateError::IncompleteExpansion);
        }

        // Validate expanded URI length
        if result.len() > options.max_expanded_length {
            return Err(TemplateError::ExpandedUriTooLong(
                result.len(),
                options.max_expanded_length,
            ));
        }

        // Validate expanded URI if requested
        if options.validate_expanded_uri {
            self.validate_expanded_uri(&result, &options.security_policy)?;
        }

        Ok(result)
    }

    /// Validate the URI template format
    pub fn validate(&self) -> Result<(), TemplateError> {
        self.validate_with_policy(&TemplateSecurityPolicy::default())
    }

    /// Validate the URI template with security policy
    pub fn validate_with_policy(
        &self,
        policy: &TemplateSecurityPolicy,
    ) -> Result<(), TemplateError> {
        // Check template length
        if self.uri_template.len() > policy.max_template_length {
            return Err(TemplateError::TemplateTooLong(
                self.uri_template.len(),
                policy.max_template_length,
            ));
        }

        // Check for nested templates if not allowed (double braces) - do this first
        if !policy.allow_nested_templates
            && (self.uri_template.contains("{{") || self.uri_template.contains("}}"))
        {
            return Err(TemplateError::NestedTemplatesNotAllowed);
        }

        let variables = self.parse_variables();

        // Check variable count
        if variables.len() > policy.max_variables {
            return Err(TemplateError::TooManyVariables(
                variables.len(),
                policy.max_variables,
            ));
        }

        // Check for duplicate variables
        let mut seen = std::collections::HashSet::new();
        for var in &variables {
            if !seen.insert(var) {
                return Err(TemplateError::DuplicateVariable(var.clone()));
            }
        }

        // Validate variable names
        let var_pattern = Regex::new(&policy.variable_name_restrictions.allowed_pattern)
            .map_err(|e| TemplateError::InvalidVariablePattern(e.to_string()))?;

        for var in &variables {
            // Check variable name length
            if var.len() > policy.variable_name_restrictions.max_length {
                return Err(TemplateError::VariableNameTooLong(
                    var.clone(),
                    var.len(),
                    policy.variable_name_restrictions.max_length,
                ));
            }

            // Check variable name pattern
            if !var_pattern.is_match(var) {
                return Err(TemplateError::InvalidVariableName(var.clone()));
            }

            // Check blocked variable names
            if policy
                .variable_name_restrictions
                .blocked_names
                .contains(var)
            {
                return Err(TemplateError::BlockedVariableName(var.clone()));
            }
        }

        // Validate URI scheme
        self.validate_uri_scheme(&policy.allowed_schemes)?;

        // Check blocked patterns
        for pattern in &policy.blocked_patterns {
            let regex = Regex::new(pattern).map_err(|e| {
                TemplateError::InvalidBlockedPattern(pattern.clone(), e.to_string())
            })?;
            if regex.is_match(&self.uri_template) {
                return Err(TemplateError::BlockedPattern(pattern.clone()));
            }
        }

        Ok(())
    }

    /// Validate URI scheme
    fn validate_uri_scheme(&self, allowed_schemes: &[String]) -> Result<(), TemplateError> {
        // Extract scheme from template
        if let Some(colon_pos) = self.uri_template.find(':') {
            let scheme = &self.uri_template[..colon_pos].to_lowercase();
            if !allowed_schemes.iter().any(|s| s.to_lowercase() == *scheme) {
                return Err(TemplateError::DisallowedScheme(scheme.to_string()));
            }
        }
        Ok(())
    }

    /// Validate expanded URI
    fn validate_expanded_uri(
        &self,
        uri: &str,
        policy: &TemplateSecurityPolicy,
    ) -> Result<(), TemplateError> {
        // Check blocked patterns on expanded URI
        for pattern in &policy.blocked_patterns {
            let regex = Regex::new(pattern).map_err(|e| {
                TemplateError::InvalidBlockedPattern(pattern.clone(), e.to_string())
            })?;
            if regex.is_match(uri) {
                return Err(TemplateError::BlockedExpandedUri(pattern.clone()));
            }
        }

        // Additional security checks on expanded URI
        // Check for path traversal (both encoded and unencoded)
        if uri.contains("..") || uri.contains("%2E%2E") || uri.contains("%2e%2e") {
            return Err(TemplateError::PathTraversal);
        }

        // Check for dangerous protocols (both encoded and unencoded)
        let uri_lower = uri.to_lowercase();
        if uri_lower.contains("javascript:")
            || uri_lower.contains("vbscript:")
            || uri_lower.contains("javascript%3a")
            || uri_lower.contains("vbscript%3a")
        {
            return Err(TemplateError::DangerousProtocol);
        }

        // Check for script injection (both encoded and unencoded)
        if uri_lower.contains("<script") || uri_lower.contains("%3cscript") {
            return Err(TemplateError::ScriptInjection);
        }

        Ok(())
    }

    /// Get template metadata
    pub fn get_metadata(&self) -> TemplateMetadata {
        let variables = self.parse_variables();

        TemplateMetadata {
            variable_count: variables.len(),
            variables,
            template_length: self.uri_template.len(),
            has_query_parameters: self.uri_template.contains('?'),
            has_fragments: self.uri_template.contains('#'),
            estimated_complexity: self.calculate_complexity(),
        }
    }

    /// Calculate template complexity score
    fn calculate_complexity(&self) -> u32 {
        let mut complexity = 0;

        // Base complexity
        complexity += self.uri_template.len() as u32 / 10;

        // Variable complexity
        complexity += self.parse_variables().len() as u32 * 2;

        // Query parameter complexity
        if self.uri_template.contains('?') {
            complexity += 5;
        }

        // Fragment complexity
        if self.uri_template.contains('#') {
            complexity += 3;
        }

        // Nested structure complexity
        let nesting_depth = self.uri_template.matches('{').count();
        complexity += nesting_depth as u32;

        complexity
    }
}

/// Template metadata
#[derive(Debug, Clone)]
pub struct TemplateMetadata {
    pub variable_count: usize,
    pub variables: Vec<String>,
    pub template_length: usize,
    pub has_query_parameters: bool,
    pub has_fragments: bool,
    pub estimated_complexity: u32,
}

/// Template expansion errors
#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("Missing required variable: {0}")]
    MissingVariable(String),

    #[error("Template expansion incomplete - unexpanded variables remain")]
    IncompleteExpansion,

    #[error("Duplicate variable in template: {0}")]
    DuplicateVariable(String),

    #[error("Invalid variable name: {0}")]
    InvalidVariableName(String),

    #[error("Template too long: {0} characters (max: {1})")]
    TemplateTooLong(usize, usize),

    #[error("Too many variables: {0} (max: {1})")]
    TooManyVariables(usize, usize),

    #[error("Variable name too long: '{0}' has {1} characters (max: {2})")]
    VariableNameTooLong(String, usize, usize),

    #[error("Blocked variable name: {0}")]
    BlockedVariableName(String),

    #[error("Nested templates not allowed")]
    NestedTemplatesNotAllowed,

    #[error("Disallowed URI scheme: {0}")]
    DisallowedScheme(String),

    #[error("Blocked pattern matched: {0}")]
    BlockedPattern(String),

    #[error("Blocked pattern in expanded URI: {0}")]
    BlockedExpandedUri(String),

    #[error("Invalid blocked pattern '{0}': {1}")]
    InvalidBlockedPattern(String, String),

    #[error("Invalid variable pattern: {0}")]
    InvalidVariablePattern(String),

    #[error("Expanded URI too long: {0} characters (max: {1})")]
    ExpandedUriTooLong(usize, usize),

    #[error("Path traversal detected in URI")]
    PathTraversal,

    #[error("Dangerous protocol detected in URI")]
    DangerousProtocol,

    #[error("Script injection detected in URI")]
    ScriptInjection,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReadResourceRequest {
    pub uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListResourcesRequest {
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListResourceTemplatesRequest {
    pub cursor: Option<String>,
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
        uri: String,
        text: String,
        #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
        mime_type: Option<String>,
    },

    #[serde(rename = "blob")]
    Blob {
        uri: String,
        blob: String, // Base64 encoded
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
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

/// Response to a resource subscription request
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubscribeResponse {
    // Empty response as per MCP 2025-06-18 specification
}

impl SubscribeResponse {
    pub fn new() -> Self {
        Self::default()
    }
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

impl ResourceContent {
    pub fn text(uri: String, text: String) -> Self {
        Self::Text {
            uri,
            text,
            mime_type: Some("text/plain".to_string()),
        }
    }

    pub fn text_with_mime_type(uri: String, text: String, mime_type: String) -> Self {
        Self::Text {
            uri,
            text,
            mime_type: Some(mime_type),
        }
    }

    pub fn json(uri: String, value: &Value) -> Self {
        Self::Text {
            uri,
            text: serde_json::to_string_pretty(value).unwrap_or_default(),
            mime_type: Some("application/json".to_string()),
        }
    }

    pub fn blob(uri: String, blob: String, mime_type: String) -> Self {
        Self::Blob {
            uri,
            blob,
            mime_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_template_creation() {
        let template = ResourceTemplate::new(
            "https://api.example.com/users/{user_id}/posts/{post_id}".to_string(),
            "user_post".to_string(),
        );

        assert_eq!(template.name, "user_post");
        assert_eq!(
            template.uri_template,
            "https://api.example.com/users/{user_id}/posts/{post_id}"
        );
    }

    #[test]
    fn test_parse_variables() {
        let template = ResourceTemplate::new(
            "https://api.example.com/users/{user_id}/posts/{post_id}".to_string(),
            "user_post".to_string(),
        );

        let variables = template.parse_variables();
        assert_eq!(variables.len(), 2);
        assert!(variables.contains(&"user_id".to_string()));
        assert!(variables.contains(&"post_id".to_string()));
    }

    #[test]
    fn test_parse_variables_with_spaces() {
        let template = ResourceTemplate::new(
            "https://api.example.com/users/{ user_id }/posts/{ post_id }".to_string(),
            "user_post".to_string(),
        );

        let variables = template.parse_variables();
        assert_eq!(variables.len(), 2);
        assert!(variables.contains(&"user_id".to_string()));
        assert!(variables.contains(&"post_id".to_string()));
    }

    #[test]
    fn test_expand_template() {
        let template = ResourceTemplate::new(
            "https://api.example.com/users/{user_id}/posts/{post_id}".to_string(),
            "user_post".to_string(),
        );

        let mut variables = HashMap::new();
        variables.insert("user_id".to_string(), "123".to_string());
        variables.insert("post_id".to_string(), "456".to_string());

        let result = template.expand(&variables).unwrap();
        assert_eq!(result, "https://api.example.com/users/123/posts/456");
    }

    #[test]
    fn test_expand_template_missing_variable() {
        let template = ResourceTemplate::new(
            "https://api.example.com/users/{user_id}/posts/{post_id}".to_string(),
            "user_post".to_string(),
        );

        let mut variables = HashMap::new();
        variables.insert("user_id".to_string(), "123".to_string());
        // Missing post_id

        let result = template.expand(&variables);
        assert!(result.is_err());
        match result {
            Err(TemplateError::MissingVariable(var)) => assert_eq!(var, "post_id"),
            _ => panic!("Expected MissingVariable error"),
        }
    }

    #[test]
    fn test_validate_template() {
        let template = ResourceTemplate::new(
            "https://api.example.com/users/{user_id}/posts/{post_id}".to_string(),
            "user_post".to_string(),
        );

        assert!(template.validate().is_ok());
    }

    #[test]
    fn test_validate_template_duplicate_variables() {
        let template = ResourceTemplate::new(
            "https://api.example.com/users/{user_id}/posts/{user_id}".to_string(),
            "user_post".to_string(),
        );

        let result = template.validate();
        assert!(result.is_err());
        match result {
            Err(TemplateError::DuplicateVariable(var)) => assert_eq!(var, "user_id"),
            _ => panic!("Expected DuplicateVariable error"),
        }
    }

    #[test]
    fn test_validate_template_invalid_variable_name() {
        let template = ResourceTemplate::new(
            "https://api.example.com/users/{123user_id}/posts/{post_id}".to_string(),
            "user_post".to_string(),
        );

        let result = template.validate();
        assert!(result.is_err());
        match result {
            Err(TemplateError::InvalidVariableName(var)) => assert_eq!(var, "123user_id"),
            _ => panic!("Expected InvalidVariableName error"),
        }
    }

    #[test]
    fn test_template_security_validation() {
        // Test blocked variable names
        let template = ResourceTemplate::new(
            "https://api.example.com/users/{password}".to_string(),
            "user_info".to_string(),
        );

        let result = template.validate();
        assert!(result.is_err());
        match result {
            Err(TemplateError::BlockedVariableName(var)) => assert_eq!(var, "password"),
            _ => panic!("Expected BlockedVariableName error"),
        }
    }

    #[test]
    fn test_template_too_many_variables() {
        let mut template_str = "https://api.example.com".to_string();
        for i in 0..15 {
            template_str.push_str(&format!("/{{var{i}}}"));
        }

        let template = ResourceTemplate::new(template_str, "too_many_vars".to_string());

        let result = template.validate();
        assert!(result.is_err());
        match result {
            Err(TemplateError::TooManyVariables(count, max)) => {
                assert_eq!(count, 15);
                assert_eq!(max, 10);
            }
            _ => panic!("Expected TooManyVariables error"),
        }
    }

    #[test]
    fn test_template_too_long() {
        let long_template = "https://api.example.com/".to_string() + &"a".repeat(3000);
        let template = ResourceTemplate::new(long_template, "long_template".to_string());

        let result = template.validate();
        assert!(result.is_err());
        match result {
            Err(TemplateError::TemplateTooLong(len, max)) => {
                assert!(len > 2048);
                assert_eq!(max, 2048);
            }
            _ => panic!("Expected TemplateTooLong error"),
        }
    }

    #[test]
    fn test_blocked_patterns() {
        let template = ResourceTemplate::new(
            "https://api.example.com/../secret/{user_id}".to_string(),
            "blocked_template".to_string(),
        );

        let result = template.validate();
        assert!(result.is_err());
        match result {
            Err(TemplateError::BlockedPattern(_)) => {}
            _ => panic!("Expected BlockedPattern error"),
        }
    }

    #[test]
    fn test_disallowed_scheme() {
        let template = ResourceTemplate::new(
            "javascript:alert('xss')/{user_id}".to_string(),
            "malicious_template".to_string(),
        );

        let result = template.validate();
        assert!(result.is_err());
        match result {
            Err(TemplateError::DisallowedScheme(scheme)) => assert_eq!(scheme, "javascript"),
            _ => panic!("Expected DisallowedScheme error"),
        }
    }

    #[test]
    fn test_url_encoding_expansion() {
        let template = ResourceTemplate::new(
            "https://api.example.com/search/{query}".to_string(),
            "search".to_string(),
        );

        let mut variables = HashMap::new();
        variables.insert("query".to_string(), "hello world & more".to_string());

        let options = TemplateExpansionOptions {
            url_encode_values: true,
            ..Default::default()
        };

        let result = template.expand_with_options(&variables, &options).unwrap();
        assert!(result.contains("hello%20world%20%26%20more"));
    }

    #[test]
    fn test_expanded_uri_validation() {
        let template = ResourceTemplate::new(
            "https://api.example.com/{path}".to_string(),
            "path_template".to_string(),
        );

        let mut variables = HashMap::new();
        variables.insert("path".to_string(), "../secret".to_string());

        let options = TemplateExpansionOptions {
            url_encode_values: false, // Don't URL encode to test path traversal detection
            ..Default::default()
        };

        let result = template.expand_with_options(&variables, &options);
        assert!(result.is_err());
        match result {
            Err(TemplateError::PathTraversal) | Err(TemplateError::BlockedExpandedUri(_)) => {}
            _ => panic!(
                "Expected PathTraversal or BlockedExpandedUri error, got: {result:?}"
            ),
        }
    }

    #[test]
    fn test_expanded_uri_too_long() {
        let template = ResourceTemplate::new(
            "https://api.example.com/{data}".to_string(),
            "data_template".to_string(),
        );

        let mut variables = HashMap::new();
        variables.insert("data".to_string(), "a".repeat(5000));

        let options = TemplateExpansionOptions {
            max_expanded_length: 1000,
            ..Default::default()
        };

        let result = template.expand_with_options(&variables, &options);
        assert!(result.is_err());
        match result {
            Err(TemplateError::ExpandedUriTooLong(len, max)) => {
                assert!(len > 1000);
                assert_eq!(max, 1000);
            }
            _ => panic!("Expected ExpandedUriTooLong error"),
        }
    }

    #[test]
    fn test_template_metadata() {
        let template = ResourceTemplate::new(
            "https://api.example.com/users/{user_id}/posts/{post_id}?limit={limit}#section"
                .to_string(),
            "complex_template".to_string(),
        );

        let metadata = template.get_metadata();
        assert_eq!(metadata.variable_count, 3);
        assert!(metadata.variables.contains(&"user_id".to_string()));
        assert!(metadata.variables.contains(&"post_id".to_string()));
        assert!(metadata.variables.contains(&"limit".to_string()));
        assert!(metadata.has_query_parameters);
        assert!(metadata.has_fragments);
        assert!(metadata.estimated_complexity > 0);
    }

    #[test]
    fn test_custom_security_policy() {
        let mut policy = TemplateSecurityPolicy {
            max_variables: 2,
            ..Default::default()
        };
        policy
            .blocked_patterns
            .push(r".*example\.com.*".to_string());

        let template = ResourceTemplate::new(
            "https://api.example.com/users/{user_id}".to_string(),
            "custom_policy_test".to_string(),
        );

        let result = template.validate_with_policy(&policy);
        assert!(result.is_err());
        match result {
            Err(TemplateError::BlockedPattern(_)) => {}
            _ => panic!("Expected BlockedPattern error"),
        }
    }

    #[test]
    fn test_variable_name_length_limit() {
        let template = ResourceTemplate::new(
            format!("https://api.example.com/{{{}}}", "a".repeat(100)),
            "long_var_name".to_string(),
        );

        let result = template.validate();
        assert!(result.is_err());
        match result {
            Err(TemplateError::VariableNameTooLong(name, len, max)) => {
                assert_eq!(name, "a".repeat(100));
                assert_eq!(len, 100);
                assert_eq!(max, 50);
            }
            _ => panic!("Expected VariableNameTooLong error"),
        }
    }

    #[test]
    fn test_nested_templates_blocked() {
        let template = ResourceTemplate::new(
            "https://api.example.com/{{nested}}/users/{user_id}".to_string(),
            "nested_template".to_string(),
        );

        let result = template.validate();
        assert!(result.is_err());
        match result {
            Err(TemplateError::NestedTemplatesNotAllowed) => {}
            _ => panic!(
                "Expected NestedTemplatesNotAllowed error, got: {result:?}"
            ),
        }
    }

    #[test]
    fn test_script_injection_in_expanded_uri() {
        let template = ResourceTemplate::new(
            "https://api.example.com/{script}".to_string(),
            "script_test".to_string(),
        );

        let mut variables = HashMap::new();
        variables.insert(
            "script".to_string(),
            "<script>alert('xss')</script>".to_string(),
        );

        let options = TemplateExpansionOptions {
            url_encode_values: false, // Don't URL encode to test script injection detection
            ..Default::default()
        };

        let result = template.expand_with_options(&variables, &options);
        assert!(result.is_err());
        match result {
            Err(TemplateError::ScriptInjection) | Err(TemplateError::BlockedExpandedUri(_)) => {}
            _ => panic!(
                "Expected ScriptInjection or BlockedExpandedUri error, got: {result:?}"
            ),
        }
    }

    #[test]
    fn test_resource_content_creation() {
        let text_content =
            ResourceContent::text("Hello, World!".to_string(), "Hello, World!".to_string());
        match text_content {
            ResourceContent::Text {
                uri,
                text,
                mime_type,
            } => {
                assert_eq!(uri, "Hello, World!".to_string());
                assert_eq!(text, "Hello, World!");
                assert_eq!(mime_type, Some("text/plain".to_string()));
            }
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_resource_content_json() {
        let json_value = serde_json::json!({"key": "value"});
        let json_content = ResourceContent::json("Hello, World!".to_string(), &json_value);
        match json_content {
            ResourceContent::Text {
                uri,
                text,
                mime_type,
            } => {
                assert_eq!(uri, "Hello, World!".to_string());
                assert!(text.contains("key"));
                assert!(text.contains("value"));
                assert_eq!(mime_type, Some("application/json".to_string()));
            }
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_resource_content_blob() {
        let blob_content = ResourceContent::blob(
            "Hello, World!".to_string(),
            "base64data".to_string(),
            "image/png".to_string(),
        );
        match blob_content {
            ResourceContent::Blob {
                uri,
                blob,
                mime_type,
            } => {
                assert_eq!(uri, "Hello, World!".to_string());
                assert_eq!(blob, "base64data");
                assert_eq!(mime_type, "image/png");
            }
            _ => panic!("Expected Blob variant"),
        }
    }
}
