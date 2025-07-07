//! Configuration constants for UltraFast MCP Core

/// Protocol configuration constants
pub const CURRENT_VERSION: &str = "2025-06-18";
pub const SUPPORTED_VERSIONS: &[&str] = &["2025-06-18", "2025-03-26", "2024-11-05"];
pub const JSONRPC_VERSION: &str = "2.0";

/// Request ID configuration constants
pub const MAX_REQUEST_ID_LENGTH: usize = 1000;
pub const MIN_REQUEST_ID_NUMBER: i64 = -999_999_999;
pub const MAX_REQUEST_ID_NUMBER: i64 = 999_999_999;

/// Validation configuration constants
pub const MAX_TOOL_NAME_LENGTH: usize = 100;
pub const MAX_METHOD_NAME_LENGTH: usize = 100;
pub const MAX_URI_LENGTH: usize = 2048;
pub const MAX_ERROR_MESSAGE_LENGTH: usize = 1000;

/// Performance configuration constants
pub const DEFAULT_REQUEST_TIMEOUT: u64 = 30;
pub const DEFAULT_CONNECTION_TIMEOUT: u64 = 10;
pub const MAX_CONCURRENT_REQUESTS: usize = 100;
pub const DEFAULT_BUFFER_SIZE: usize = 8192;

/// Security configuration constants
pub const MAX_AUTH_ATTEMPTS: u32 = 3;
pub const TOKEN_EXPIRATION_TIME: u64 = 3600; // 1 hour
pub const MAX_TOKEN_LENGTH: usize = 4096;

/// Logging configuration constants
pub const DEFAULT_LOG_LEVEL: &str = "info";
pub const MAX_LOG_MESSAGE_LENGTH: usize = 10000;
pub const DEFAULT_LOG_FORMAT: &str = "json";

/// Validation error types
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Tool name '{0}' exceeds maximum length of {1}")]
    ToolNameTooLong(String, usize),
    #[error("Method name '{0}' exceeds maximum length of {1}")]
    MethodNameTooLong(String, usize),
    #[error("URI '{0}' exceeds maximum length of {1}")]
    UriTooLong(String, usize),
    #[error("Error message '{0}' exceeds maximum length of {1}")]
    ErrorMessageTooLong(String, usize),
    #[error("Request ID '{0}' exceeds maximum length of {1}")]
    RequestIdTooLong(String, usize),
    #[error("Request ID number {0} is outside valid range [{1}, {2}]")]
    RequestIdNumberOutOfRange(i64, i64, i64),
    #[error("Token length {0} exceeds maximum length of {1}")]
    TokenTooLong(usize, usize),
    #[error("Log message length {0} exceeds maximum length of {1}")]
    LogMessageTooLong(usize, usize),
}

/// Validation functions to enforce configuration constants
pub fn validate_tool_name(name: &str) -> Result<(), ValidationError> {
    if name.len() > MAX_TOOL_NAME_LENGTH {
        return Err(ValidationError::ToolNameTooLong(
            name.to_string(),
            MAX_TOOL_NAME_LENGTH,
        ));
    }
    Ok(())
}

pub fn validate_method_name(name: &str) -> Result<(), ValidationError> {
    if name.len() > MAX_METHOD_NAME_LENGTH {
        return Err(ValidationError::MethodNameTooLong(
            name.to_string(),
            MAX_METHOD_NAME_LENGTH,
        ));
    }
    Ok(())
}

pub fn validate_uri(uri: &str) -> Result<(), ValidationError> {
    if uri.len() > MAX_URI_LENGTH {
        return Err(ValidationError::UriTooLong(uri.to_string(), MAX_URI_LENGTH));
    }
    Ok(())
}

pub fn validate_error_message(message: &str) -> Result<(), ValidationError> {
    if message.len() > MAX_ERROR_MESSAGE_LENGTH {
        return Err(ValidationError::ErrorMessageTooLong(
            message.to_string(),
            MAX_ERROR_MESSAGE_LENGTH,
        ));
    }
    Ok(())
}

pub fn validate_request_id_string(id: &str) -> Result<(), ValidationError> {
    if id.len() > MAX_REQUEST_ID_LENGTH {
        return Err(ValidationError::RequestIdTooLong(
            id.to_string(),
            MAX_REQUEST_ID_LENGTH,
        ));
    }
    Ok(())
}

pub fn validate_request_id_number(id: i64) -> Result<(), ValidationError> {
    if !(MIN_REQUEST_ID_NUMBER..=MAX_REQUEST_ID_NUMBER).contains(&id) {
        return Err(ValidationError::RequestIdNumberOutOfRange(
            id,
            MIN_REQUEST_ID_NUMBER,
            MAX_REQUEST_ID_NUMBER,
        ));
    }
    Ok(())
}

pub fn validate_token_length(length: usize) -> Result<(), ValidationError> {
    if length > MAX_TOKEN_LENGTH {
        return Err(ValidationError::TokenTooLong(length, MAX_TOKEN_LENGTH));
    }
    Ok(())
}

pub fn validate_log_message_length(length: usize) -> Result<(), ValidationError> {
    if length > MAX_LOG_MESSAGE_LENGTH {
        return Err(ValidationError::LogMessageTooLong(
            length,
            MAX_LOG_MESSAGE_LENGTH,
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_constants() {
        assert_eq!(JSONRPC_VERSION, "2.0");
    }

    #[test]
    fn test_validation_functions() {
        // Test valid inputs
        assert!(validate_tool_name("valid_tool").is_ok());
        assert!(validate_method_name("valid_method").is_ok());
        assert!(validate_uri("file:///valid/path").is_ok());
        assert!(validate_error_message("valid error").is_ok());
        assert!(validate_request_id_string("valid_id").is_ok());
        assert!(validate_request_id_number(0).is_ok());
        assert!(validate_token_length(100).is_ok());
        assert!(validate_log_message_length(100).is_ok());

        // Test invalid inputs
        let long_tool_name = "a".repeat(MAX_TOOL_NAME_LENGTH + 1);
        assert!(validate_tool_name(&long_tool_name).is_err());

        let long_method_name = "a".repeat(MAX_METHOD_NAME_LENGTH + 1);
        assert!(validate_method_name(&long_method_name).is_err());

        let long_uri = "a".repeat(MAX_URI_LENGTH + 1);
        assert!(validate_uri(&long_uri).is_err());

        let long_error = "a".repeat(MAX_ERROR_MESSAGE_LENGTH + 1);
        assert!(validate_error_message(&long_error).is_err());

        let long_id = "a".repeat(MAX_REQUEST_ID_LENGTH + 1);
        assert!(validate_request_id_string(&long_id).is_err());

        assert!(validate_request_id_number(MIN_REQUEST_ID_NUMBER - 1).is_err());
        assert!(validate_request_id_number(MAX_REQUEST_ID_NUMBER + 1).is_err());

        assert!(validate_token_length(MAX_TOKEN_LENGTH + 1).is_err());
        assert!(validate_log_message_length(MAX_LOG_MESSAGE_LENGTH + 1).is_err());
    }
}
