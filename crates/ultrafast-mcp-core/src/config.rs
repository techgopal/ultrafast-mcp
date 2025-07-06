//! Configuration module for UltraFast MCP Core
//!
//! This module contains all configuration constants, settings, and configuration
//! structures used throughout the UltraFast MCP implementation.

/// Protocol configuration constants
pub mod protocol {
    /// Current MCP protocol version
    pub const CURRENT_VERSION: &str = "2025-06-18";
    
    /// All supported protocol versions (latest first)
    pub const SUPPORTED_VERSIONS: &[&str] = &["2025-06-18", "2024-11-05"];
    
    /// JSON-RPC version string
    pub const JSONRPC_VERSION: &str = "2.0";
}

/// Request ID configuration constants
pub mod request_id {
    /// Maximum length for request ID strings
    pub const MAX_STRING_LENGTH: usize = 1000;
    /// Minimum value for numeric request IDs
    pub const MIN_NUMBER_VALUE: i64 = -999_999_999;
    /// Maximum value for numeric request IDs
    pub const MAX_NUMBER_VALUE: i64 = 999_999_999;
}

/// Validation configuration constants
pub mod validation {
    /// Maximum length for tool names
    pub const MAX_TOOL_NAME_LENGTH: usize = 100;
    /// Maximum length for method names
    pub const MAX_METHOD_NAME_LENGTH: usize = 100;
    /// Maximum length for URIs
    pub const MAX_URI_LENGTH: usize = 2048;
    /// Maximum length for error messages
    pub const MAX_ERROR_MESSAGE_LENGTH: usize = 1000;
}

/// Performance configuration constants
pub mod performance {
    /// Default timeout for requests (in seconds)
    pub const DEFAULT_REQUEST_TIMEOUT: u64 = 30;
    /// Default timeout for connections (in seconds)
    pub const DEFAULT_CONNECTION_TIMEOUT: u64 = 10;
    /// Maximum concurrent requests per connection
    pub const MAX_CONCURRENT_REQUESTS: usize = 100;
    /// Default buffer size for reading/writing
    pub const DEFAULT_BUFFER_SIZE: usize = 8192;
}

/// Security configuration constants
pub mod security {
    /// Maximum number of authentication attempts
    pub const MAX_AUTH_ATTEMPTS: u32 = 3;
    /// Token expiration time (in seconds)
    pub const TOKEN_EXPIRATION_TIME: u64 = 3600; // 1 hour
    /// Maximum token length
    pub const MAX_TOKEN_LENGTH: usize = 4096;
}

/// Logging configuration constants
pub mod logging {
    /// Default log level
    pub const DEFAULT_LOG_LEVEL: &str = "info";
    /// Maximum log message length
    pub const MAX_LOG_MESSAGE_LENGTH: usize = 10000;
    /// Default log format
    pub const DEFAULT_LOG_FORMAT: &str = "json";
}

/// Main configuration structure
#[derive(Debug, Clone)]
pub struct Config {
    /// Protocol configuration
    pub protocol: ProtocolConfig,
    /// Request ID configuration
    pub request_id: RequestIdConfig,
    /// Validation configuration
    pub validation: ValidationConfig,
    /// Performance configuration
    pub performance: PerformanceConfig,
    /// Security configuration
    pub security: SecurityConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
}

/// Protocol configuration
#[derive(Debug, Clone)]
pub struct ProtocolConfig {
    /// Current protocol version
    pub version: String,
    /// Supported protocol versions
    pub supported_versions: Vec<String>,
    /// JSON-RPC version
    pub jsonrpc_version: String,
}

/// Request ID configuration
#[derive(Debug, Clone)]
pub struct RequestIdConfig {
    /// Maximum string length
    pub max_string_length: usize,
    /// Minimum number value
    pub min_number_value: i64,
    /// Maximum number value
    pub max_number_value: i64,
}

/// Validation configuration
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Maximum tool name length
    pub max_tool_name_length: usize,
    /// Maximum method name length
    pub max_method_name_length: usize,
    /// Maximum URI length
    pub max_uri_length: usize,
    /// Maximum error message length
    pub max_error_message_length: usize,
}

/// Performance configuration
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Request timeout in seconds
    pub request_timeout: u64,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// Maximum concurrent requests
    pub max_concurrent_requests: usize,
    /// Buffer size
    pub buffer_size: usize,
}

/// Security configuration
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Maximum authentication attempts
    pub max_auth_attempts: u32,
    /// Token expiration time in seconds
    pub token_expiration_time: u64,
    /// Maximum token length
    pub max_token_length: usize,
}

/// Logging configuration
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Log level
    pub level: String,
    /// Maximum log message length
    pub max_message_length: usize,
    /// Log format
    pub format: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            protocol: ProtocolConfig::default(),
            request_id: RequestIdConfig::default(),
            validation: ValidationConfig::default(),
            performance: PerformanceConfig::default(),
            security: SecurityConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self {
            version: protocol::CURRENT_VERSION.to_string(),
            supported_versions: protocol::SUPPORTED_VERSIONS.iter().map(|s| s.to_string()).collect(),
            jsonrpc_version: protocol::JSONRPC_VERSION.to_string(),
        }
    }
}

impl Default for RequestIdConfig {
    fn default() -> Self {
        Self {
            max_string_length: request_id::MAX_STRING_LENGTH,
            min_number_value: request_id::MIN_NUMBER_VALUE,
            max_number_value: request_id::MAX_NUMBER_VALUE,
        }
    }
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_tool_name_length: validation::MAX_TOOL_NAME_LENGTH,
            max_method_name_length: validation::MAX_METHOD_NAME_LENGTH,
            max_uri_length: validation::MAX_URI_LENGTH,
            max_error_message_length: validation::MAX_ERROR_MESSAGE_LENGTH,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            request_timeout: performance::DEFAULT_REQUEST_TIMEOUT,
            connection_timeout: performance::DEFAULT_CONNECTION_TIMEOUT,
            max_concurrent_requests: performance::MAX_CONCURRENT_REQUESTS,
            buffer_size: performance::DEFAULT_BUFFER_SIZE,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            max_auth_attempts: security::MAX_AUTH_ATTEMPTS,
            token_expiration_time: security::TOKEN_EXPIRATION_TIME,
            max_token_length: security::MAX_TOKEN_LENGTH,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: logging::DEFAULT_LOG_LEVEL.to_string(),
            max_message_length: logging::MAX_LOG_MESSAGE_LENGTH,
            format: logging::DEFAULT_LOG_FORMAT.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.protocol.version, protocol::CURRENT_VERSION);
        assert_eq!(config.request_id.max_string_length, request_id::MAX_STRING_LENGTH);
        assert_eq!(config.validation.max_tool_name_length, validation::MAX_TOOL_NAME_LENGTH);
        assert_eq!(config.performance.request_timeout, performance::DEFAULT_REQUEST_TIMEOUT);
        assert_eq!(config.security.max_auth_attempts, security::MAX_AUTH_ATTEMPTS);
        assert_eq!(config.logging.level, logging::DEFAULT_LOG_LEVEL);
    }

    #[test]
    fn test_protocol_config_default() {
        let config = ProtocolConfig::default();
        assert_eq!(config.version, protocol::CURRENT_VERSION);
        assert_eq!(config.jsonrpc_version, protocol::JSONRPC_VERSION);
        assert!(!config.supported_versions.is_empty());
    }
} 