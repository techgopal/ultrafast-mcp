//! Error handling for UltraFast MCP Core

use thiserror::Error;

/// MCPResult is the canonical result type for all MCP operations.
pub type MCPResult<T> = Result<T, MCPError>;

/// Main error type for MCP operations
#[derive(Debug, Error)]
pub enum MCPError {
    #[error("Protocol error: {0}")]
    Protocol(#[from] ProtocolError),

    #[error("Transport error: {0}")]
    Transport(#[from] TransportError),

    #[error("Tool execution error: {0}")]
    ToolExecution(#[from] ToolError),

    #[error("Resource error: {0}")]
    Resource(#[from] ResourceError),

    #[error("Authentication error: {0}")]
    Authentication(#[from] AuthenticationError),

    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("Rate limiting error: {0}")]
    RateLimit(#[from] RateLimitError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

impl MCPError {
    pub fn invalid_params(msg: String) -> Self {
        MCPError::Protocol(ProtocolError::InvalidParams(msg))
    }

    pub fn method_not_found(msg: String) -> Self {
        MCPError::Protocol(ProtocolError::MethodNotFound(msg))
    }

    pub fn not_found(msg: String) -> Self {
        MCPError::Protocol(ProtocolError::NotFound(msg))
    }

    pub fn invalid_request(msg: String) -> Self {
        MCPError::Protocol(ProtocolError::InvalidRequest(msg))
    }

    pub fn invalid_response(msg: String) -> Self {
        MCPError::Protocol(ProtocolError::InvalidResponse(msg))
    }

    pub fn serialization_error(msg: String) -> Self {
        MCPError::Protocol(ProtocolError::SerializationError(msg))
    }

    pub fn transport_error(msg: String) -> Self {
        MCPError::Transport(TransportError::ConnectionFailed(msg))
    }

    pub fn request_timeout() -> Self {
        MCPError::Protocol(ProtocolError::RequestTimeout)
    }

    pub fn internal_error(msg: String) -> Self {
        MCPError::Protocol(ProtocolError::InternalError(msg))
    }
}

/// Protocol-related errors
#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Invalid JSON-RPC version: {0}")]
    InvalidVersion(String),

    #[error("Invalid request ID: {0}")]
    InvalidRequestId(String),

    #[error("Method not found: {0}")]
    MethodNotFound(String),

    #[error("Invalid parameters: {0}")]
    InvalidParams(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Request timeout")]
    RequestTimeout,

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Capability not supported: {0}")]
    CapabilityNotSupported(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Connection closed")]
    ConnectionClosed,

    #[error("Transport error: {0}")]
    TransportError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Authentication error: {0}")]
    AuthenticationError(String),
}

/// Transport-related errors
#[derive(Debug, Error)]
pub enum TransportError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Connection closed")]
    ConnectionClosed,

    #[error("Send failed: {0}")]
    SendFailed(String),

    #[error("Receive failed: {0}")]
    ReceiveFailed(String),
}

/// Tool execution errors
#[derive(Debug, Error)]
pub enum ToolError {
    #[error("Tool not found: {0}")]
    NotFound(String),

    #[error("Tool execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Schema validation failed: {0}")]
    SchemaValidation(String),
}

/// Resource-related errors
#[derive(Debug, Error)]
pub enum ResourceError {
    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Access denied: {0}")]
    AccessDenied(String),

    #[error("Invalid URI: {0}")]
    InvalidUri(String),

    #[error("Content type mismatch: expected {expected}, got {actual}")]
    ContentTypeMismatch { expected: String, actual: String },
}

/// Authentication errors
#[derive(Debug, Error)]
pub enum AuthenticationError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Token expired")]
    TokenExpired,

    #[error("Insufficient permissions for resource: {resource}")]
    InsufficientPermissions { resource: String },

    #[error("OAuth error: {error} - {description}")]
    OAuthError { error: String, description: String },
}

/// Validation errors
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Schema validation failed for field '{field}': {details}")]
    SchemaValidation { field: String, details: String },

    #[error("Required field '{field}' is missing")]
    RequiredField { field: String },

    #[error("Invalid format for field '{field}': expected {expected}")]
    InvalidFormat { field: String, expected: String },

    #[error("Value for field '{field}' is out of range: {actual} (expected {min}..{max})")]
    ValueOutOfRange { field: String, min: String, max: String, actual: String },
}

/// Rate limiting errors
#[derive(Debug, Error)]
pub enum RateLimitError {
    #[error("Too many requests. Retry after {retry_after}ms. Limit: {limit}")]
    TooManyRequests { retry_after: u64, limit: u32 },

    #[error("Quota exceeded: {quota} requests per {period}")]
    QuotaExceeded { quota: u32, period: String },
}

/// Standard JSON-RPC error codes
pub mod error_codes {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;

    // MCP-specific error codes
    pub const INITIALIZATION_FAILED: i32 = -32000;
    pub const CAPABILITY_NOT_SUPPORTED: i32 = -32001;
    pub const RESOURCE_NOT_FOUND: i32 = -32002;
    pub const TOOL_EXECUTION_ERROR: i32 = -32003;
    pub const INVALID_URI: i32 = -32004;
    pub const ACCESS_DENIED: i32 = -32005;
    pub const AUTHENTICATION_ERROR: i32 = -32006;
    pub const VALIDATION_ERROR: i32 = -32007;
    pub const RATE_LIMIT_ERROR: i32 = -32008;
}

impl From<crate::protocol::jsonrpc::JsonRpcError> for MCPError {
    fn from(err: crate::protocol::jsonrpc::JsonRpcError) -> Self {
        match err.code {
            error_codes::PARSE_ERROR => MCPError::Protocol(ProtocolError::SerializationError(err.message)),
            error_codes::INVALID_REQUEST => MCPError::Protocol(ProtocolError::InvalidRequest(err.message)),
            error_codes::METHOD_NOT_FOUND => MCPError::Protocol(ProtocolError::MethodNotFound(err.message)),
            error_codes::INVALID_PARAMS => MCPError::Protocol(ProtocolError::InvalidParams(err.message)),
            error_codes::INTERNAL_ERROR => MCPError::Protocol(ProtocolError::InternalError(err.message)),
            error_codes::INITIALIZATION_FAILED => MCPError::Protocol(ProtocolError::InitializationFailed(err.message)),
            error_codes::CAPABILITY_NOT_SUPPORTED => MCPError::Protocol(ProtocolError::CapabilityNotSupported(err.message)),
            error_codes::RESOURCE_NOT_FOUND => MCPError::Resource(ResourceError::NotFound(err.message)),
            error_codes::TOOL_EXECUTION_ERROR => MCPError::ToolExecution(ToolError::ExecutionFailed(err.message)),
            error_codes::INVALID_URI => MCPError::Resource(ResourceError::InvalidUri(err.message)),
            error_codes::ACCESS_DENIED => MCPError::Resource(ResourceError::AccessDenied(err.message)),
            error_codes::AUTHENTICATION_ERROR => MCPError::Authentication(AuthenticationError::InvalidCredentials),
            error_codes::VALIDATION_ERROR => MCPError::Validation(ValidationError::SchemaValidation {
                field: "unknown".to_string(),
                details: err.message,
            }),
            error_codes::RATE_LIMIT_ERROR => MCPError::RateLimit(RateLimitError::TooManyRequests {
                retry_after: 0,
                limit: 0,
            }),
            _ => MCPError::Protocol(ProtocolError::InternalError(err.message)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_variant_conversion() {
        let error = MCPError::invalid_params("test".to_string());
        assert!(matches!(error, MCPError::Protocol(ProtocolError::InvalidParams(_))));

        let error = MCPError::method_not_found("test".to_string());
        assert!(matches!(error, MCPError::Protocol(ProtocolError::MethodNotFound(_))));
    }

    #[test]
    fn test_error_creation() {
        let error = MCPError::internal_error("test error".to_string());
        assert!(matches!(error, MCPError::Protocol(ProtocolError::InternalError(_))));
    }
}
