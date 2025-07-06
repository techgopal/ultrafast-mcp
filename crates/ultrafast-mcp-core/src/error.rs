//! # Error Handling Module
//!
//! Comprehensive error handling system for the UltraFast MCP Core crate.
//!
//! This module provides a hierarchical error system that covers all aspects of MCP
//! operations, from protocol-level errors to transport and application-specific issues.
//! The error types are designed to provide detailed context for debugging and
//! recovery while maintaining type safety and ergonomic usage.
//!
//! ## Overview
//!
//! The error system is built around the `MCPError` enum, which serves as the canonical
//! error type for all MCP operations. It provides:
//!
//! - **Hierarchical Error Types**: Categorized errors for different failure modes
//! - **Rich Context**: Detailed error messages with actionable information
//! - **Automatic Conversion**: Seamless integration with standard library and third-party errors
//! - **JSON-RPC Compliance**: Proper error codes and messages for protocol compliance
//! - **Recovery Information**: Error types that help with error recovery and handling
//!
//! ## Error Categories
//!
//! ### Protocol Errors (`ProtocolError`)
//! Errors related to the MCP protocol itself:
//! - Invalid JSON-RPC version or format
//! - Method not found or unsupported
//! - Invalid parameters or request structure
//! - Protocol initialization failures
//! - Capability negotiation issues
//!
//! ### Transport Errors (`TransportError`)
//! Errors related to communication and transport:
//! - Connection failures and timeouts
//! - Send/receive operation failures
//! - Network-level issues
//! - Transport protocol violations
//!
//! ### Tool Errors (`ToolError`)
//! Errors specific to tool execution:
//! - Tool not found or unavailable
//! - Tool execution failures
//! - Invalid tool input parameters
//! - Schema validation failures
//!
//! ### Resource Errors (`ResourceError`)
//! Errors related to resource operations:
//! - Resource not found or inaccessible
//! - Access permission issues
//! - Invalid resource URIs
//! - Content type mismatches
//!
//! ## Usage Examples
//!
//! ### Basic Error Handling
//!
//! ```rust
//! use ultrafast_mcp_core::{MCPError, MCPResult};
//!
//! fn process_tool_call(tool_name: &str) -> MCPResult<String> {
//!     match tool_name {
//!         "valid_tool" => Ok("Success".to_string()),
//!         _ => Err(MCPError::method_not_found(
//!             format!("Tool '{}' not found", tool_name)
//!         )),
//!     }
//! }
//! ```
//!
//! ### Error Conversion
//!
//! ```rust
//! use ultrafast_mcp_core::{MCPError, MCPResult};
//! use std::io;
//!
//! fn read_file(path: &str) -> MCPResult<String> {
//!     std::fs::read_to_string(path)
//!         .map_err(|e| MCPError::from(e)) // Automatic conversion from io::Error
//! }
//!
//! fn parse_json(data: &str) -> MCPResult<serde_json::Value> {
//!     serde_json::from_str(data)
//!         .map_err(|e| MCPError::from(e)) // Automatic conversion from serde_json::Error
//! }
//! ```
//!
//! ### Custom Error Context
//!
//! ```rust
//! use ultrafast_mcp_core::{MCPError, MCPResult};
//!
//! fn validate_user_input(input: &str) -> MCPResult<()> {
//!     if input.is_empty() {
//!         return Err(MCPError::invalid_params(
//!             "User input cannot be empty".to_string()
//!         ));
//!     }
//!
//!     if input.len() > 1000 {
//!         return Err(MCPError::invalid_params(
//!             "User input exceeds maximum length of 1000 characters".to_string()
//!         ));
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Error Recovery
//!
//! ```rust
//! use ultrafast_mcp_core::{MCPError, MCPResult};
//! use ultrafast_mcp_core::error::{ProtocolError, TransportError};
//! // Dummy perform_operation for example
//! fn perform_operation() -> MCPResult<String> {
//!     Err(MCPError::Transport(TransportError::ConnectionFailed("network error".to_string())))
//! }
//!
//! fn robust_operation() -> MCPResult<String> {
//!     match perform_operation() {
//!         Ok(result) => Ok(result),
//!         Err(MCPError::Transport(_transport_err)) => {
//!             // Retry on transport errors
//!             // tracing::warn!("Transport error, retrying: {}", transport_err);
//!             perform_operation()
//!         }
//!         Err(MCPError::Protocol(ProtocolError::RequestTimeout)) => {
//!             // Handle timeout specifically
//!             Err(MCPError::internal_error("Operation timed out after retries".to_string()))
//!         }
//!         Err(e) => Err(e), // Pass through other errors
//!     }
//! }
//! ```
//!
//! ## Error Codes
//!
//! The module defines standard JSON-RPC error codes and MCP-specific extensions:
//!
//! ### Standard JSON-RPC Codes
//! - `-32700`: Parse error (invalid JSON)
//! - `-32600`: Invalid request (malformed request)
//! - `-32601`: Method not found
//! - `-32602`: Invalid parameters
//! - `-32603`: Internal error
//!
//! ### MCP-Specific Codes
//! - `-32000`: Initialization failed
//! - `-32001`: Capability not supported
//! - `-32002`: Resource not found
//! - `-32003`: Tool execution error
//! - `-32004`: Invalid URI
//! - `-32005`: Access denied
//!
//! ## Best Practices
//!
//! ### Error Creation
//! - Use the convenience constructors for common error types
//! - Provide descriptive error messages with context
//! - Include relevant parameters in error messages
//! - Use appropriate error categories for different failure modes
//!
//! ### Error Handling
//! - Handle errors at the appropriate level
//! - Provide fallback behavior where possible
//! - Log errors with sufficient context for debugging
//! - Convert errors to user-friendly messages when appropriate
//!
//! ### Error Propagation
//! - Use `?` operator for automatic error propagation
//! - Add context when converting between error types
//! - Preserve original error information when possible
//! - Consider error recovery strategies
//!
//! ## Thread Safety
//!
//! All error types in this module are designed to be thread-safe:
//! - Error types implement `Send + Sync`
//! - Error conversion operations are thread-safe
//! - No mutable global state is used
//!
//! ## Performance Considerations
//!
//! - Error types use efficient string storage
//! - Error conversion is optimized for common cases
//! - Minimal allocations in error creation paths
//! - Lazy error message formatting where appropriate

use thiserror::Error;
use std::collections::HashMap;

/// MCPResult is the canonical result type for all MCP operations.
///
/// This is the preferred result type to use in all public APIs and user code.
/// It provides a consistent error handling experience across the entire MCP ecosystem.
pub type MCPResult<T> = Result<T, MCPError>;

/// Enhanced error context for better debugging and recovery
#[derive(Debug, Clone, Default)]
pub struct ErrorContext {
    /// Additional context information
    pub context: HashMap<String, String>,
    /// Suggested recovery actions
    pub recovery_hints: Vec<String>,
    /// Whether this error is retryable
    pub retryable: bool,
    /// Suggested retry delay in milliseconds
    pub retry_delay_ms: Option<u64>,
    /// Error correlation ID for tracking
    pub correlation_id: Option<String>,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new() -> Self {
        Self::default()
    }

    /// Add context information
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    /// Add recovery hint
    pub fn with_recovery_hint(mut self, hint: impl Into<String>) -> Self {
        self.recovery_hints.push(hint.into());
        self
    }

    /// Mark as retryable
    pub fn retryable(mut self, delay_ms: u64) -> Self {
        self.retryable = true;
        self.retry_delay_ms = Some(delay_ms);
        self
    }

    /// Set correlation ID
    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }
}

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
    /// Create an invalid parameters error
    pub fn invalid_params(msg: String) -> Self {
        MCPError::Protocol(ProtocolError::InvalidParams(msg))
    }

    /// Create a method not found error
    pub fn method_not_found(msg: String) -> Self {
        MCPError::Protocol(ProtocolError::MethodNotFound(msg))
    }

    /// Create a not found error
    pub fn not_found(msg: String) -> Self {
        MCPError::Protocol(ProtocolError::NotFound(msg))
    }

    /// Create an invalid request error
    pub fn invalid_request(msg: String) -> Self {
        MCPError::Protocol(ProtocolError::InvalidRequest(msg))
    }

    /// Create an invalid response error
    pub fn invalid_response(msg: String) -> Self {
        MCPError::Protocol(ProtocolError::InvalidResponse(msg))
    }

    /// Create a serialization error
    pub fn serialization_error(msg: String) -> Self {
        MCPError::Protocol(ProtocolError::SerializationError(msg))
    }

    /// Create a transport error
    pub fn transport_error(msg: String) -> Self {
        MCPError::Transport(TransportError::ConnectionFailed(msg))
    }

    /// Create a request timeout error
    pub fn request_timeout() -> Self {
        MCPError::Protocol(ProtocolError::RequestTimeout)
    }

    /// Create an internal error
    pub fn internal_error(msg: String) -> Self {
        MCPError::Protocol(ProtocolError::InternalError(msg))
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            MCPError::Transport(TransportError::ConnectionFailed(_))
                | MCPError::Transport(TransportError::SendFailed(_))
                | MCPError::Transport(TransportError::ReceiveFailed(_))
                | MCPError::Protocol(ProtocolError::RequestTimeout)
                | MCPError::RateLimit(_)
        )
    }

    /// Get suggested retry delay in milliseconds
    pub fn retry_delay_ms(&self) -> Option<u64> {
        match self {
            MCPError::Transport(_) => Some(1000), // 1 second for transport errors
            MCPError::Protocol(ProtocolError::RequestTimeout) => Some(5000), // 5 seconds for timeouts
            MCPError::RateLimit(RateLimitError::TooManyRequests { retry_after, .. }) => {
                Some(*retry_after)
            }
            _ => None,
        }
    }

    /// Get error context for debugging
    pub fn context(&self) -> ErrorContext {
        let mut ctx = ErrorContext::new();
        
        match self {
            MCPError::Protocol(err) => {
                ctx = ctx.with_context("error_type".to_string(), "protocol".to_string());
                match err {
                    ProtocolError::MethodNotFound(method) => {
                        ctx = ctx
                            .with_context("method".to_string(), method.clone())
                            .with_recovery_hint("Check if the method name is correct")
                            .with_recovery_hint("Verify the method is supported by this server");
                    }
                    ProtocolError::InvalidParams(details) => {
                        ctx = ctx
                            .with_context("details".to_string(), details.clone())
                            .with_recovery_hint("Check parameter types and values")
                            .with_recovery_hint("Refer to the method documentation");
                    }
                    ProtocolError::RequestTimeout => {
                        ctx = ctx
                            .retryable(5000)
                            .with_recovery_hint("The operation may succeed on retry")
                            .with_recovery_hint("Consider increasing timeout settings");
                    }
                    _ => {}
                }
            }
            MCPError::Transport(err) => {
                ctx = ctx.with_context("error_type".to_string(), "transport".to_string());
                match err {
                    TransportError::ConnectionFailed(reason) => {
                        ctx = ctx
                            .with_context("reason".to_string(), reason.clone())
                            .retryable(1000)
                            .with_recovery_hint("Check network connectivity")
                            .with_recovery_hint("Verify server is running");
                    }
                    TransportError::SendFailed(reason) => {
                        ctx = ctx
                            .with_context("reason".to_string(), reason.clone())
                            .retryable(1000)
                            .with_recovery_hint("Check connection stability");
                    }
                    TransportError::ReceiveFailed(reason) => {
                        ctx = ctx
                            .with_context("reason".to_string(), reason.clone())
                            .retryable(1000)
                            .with_recovery_hint("Check connection stability");
                    }
                    TransportError::ConnectionClosed => {
                        ctx = ctx
                            .retryable(1000)
                            .with_recovery_hint("Connection was closed, try reconnecting");
                    }
                }
            }
            MCPError::ToolExecution(err) => {
                ctx = ctx.with_context("error_type".to_string(), "tool_execution".to_string());
                match err {
                    ToolError::NotFound(tool) => {
                        ctx = ctx
                            .with_context("tool".to_string(), tool.clone())
                            .with_recovery_hint("Check if the tool is available")
                            .with_recovery_hint("Verify tool name spelling");
                    }
                    ToolError::ExecutionFailed(reason) => {
                        ctx = ctx
                            .with_context("reason".to_string(), reason.clone())
                            .with_recovery_hint("Check tool dependencies")
                            .with_recovery_hint("Verify tool configuration");
                    }
                    ToolError::InvalidInput(details) => {
                        ctx = ctx
                            .with_context("details".to_string(), details.clone())
                            .with_recovery_hint("Check input parameter values")
                            .with_recovery_hint("Refer to tool documentation");
                    }
                    ToolError::SchemaValidation(details) => {
                        ctx = ctx
                            .with_context("details".to_string(), details.clone())
                            .with_recovery_hint("Check input schema compliance")
                            .with_recovery_hint("Verify parameter types");
                    }
                }
            }
            MCPError::Resource(err) => {
                ctx = ctx.with_context("error_type".to_string(), "resource".to_string());
                match err {
                    ResourceError::NotFound(uri) => {
                        ctx = ctx
                            .with_context("uri".to_string(), uri.clone())
                            .with_recovery_hint("Check if the resource exists")
                            .with_recovery_hint("Verify the URI is correct");
                    }
                    ResourceError::AccessDenied(reason) => {
                        ctx = ctx
                            .with_context("reason".to_string(), reason.clone())
                            .with_recovery_hint("Check access permissions")
                            .with_recovery_hint("Verify authentication");
                    }
                    ResourceError::InvalidUri(uri) => {
                        ctx = ctx
                            .with_context("uri".to_string(), uri.clone())
                            .with_recovery_hint("Check URI format")
                            .with_recovery_hint("Verify URI scheme is supported");
                    }
                    ResourceError::ContentTypeMismatch { expected, actual } => {
                        ctx = ctx
                            .with_context("expected".to_string(), expected.clone())
                            .with_context("actual".to_string(), actual.clone())
                            .with_recovery_hint("Check content type handling")
                            .with_recovery_hint("Verify resource format");
                    }
                }
            }
            MCPError::Authentication(err) => {
                ctx = ctx.with_context("error_type".to_string(), "authentication".to_string());
                match err {
                    AuthenticationError::InvalidCredentials => {
                        ctx = ctx
                            .with_recovery_hint("Check username and password")
                            .with_recovery_hint("Verify authentication method");
                    }
                    AuthenticationError::TokenExpired => {
                        ctx = ctx
                            .retryable(0) // Immediate retry with new token
                            .with_recovery_hint("Refresh authentication token")
                            .with_recovery_hint("Re-authenticate with the server");
                    }
                    AuthenticationError::InsufficientPermissions { resource } => {
                        ctx = ctx
                            .with_context("resource".to_string(), resource.clone())
                            .with_recovery_hint("Check user permissions")
                            .with_recovery_hint("Contact administrator for access");
                    }
                    AuthenticationError::OAuthError { error, description } => {
                        ctx = ctx
                            .with_context("oauth_error".to_string(), error.clone())
                            .with_context("description".to_string(), description.clone())
                            .with_recovery_hint("Check OAuth configuration")
                            .with_recovery_hint("Verify OAuth provider settings");
                    }
                }
            }
            MCPError::Validation(err) => {
                ctx = ctx.with_context("error_type".to_string(), "validation".to_string());
                match err {
                    ValidationError::SchemaValidation { field, details } => {
                        ctx = ctx
                            .with_context("field".to_string(), field.clone())
                            .with_context("details".to_string(), details.clone())
                            .with_recovery_hint("Check field format and constraints")
                            .with_recovery_hint("Refer to schema documentation");
                    }
                    ValidationError::RequiredField { field } => {
                        ctx = ctx
                            .with_context("field".to_string(), field.clone())
                            .with_recovery_hint("Provide the required field")
                            .with_recovery_hint("Check field name spelling");
                    }
                    ValidationError::InvalidFormat { field, expected } => {
                        ctx = ctx
                            .with_context("field".to_string(), field.clone())
                            .with_context("expected".to_string(), expected.clone())
                            .with_recovery_hint("Check field format")
                            .with_recovery_hint("Follow the expected pattern");
                    }
                    ValidationError::ValueOutOfRange { field, min, max, actual } => {
                        ctx = ctx
                            .with_context("field".to_string(), field.clone())
                            .with_context("min".to_string(), min.to_string())
                            .with_context("max".to_string(), max.to_string())
                            .with_context("actual".to_string(), actual.to_string())
                            .with_recovery_hint("Use a value within the allowed range")
                            .with_recovery_hint("Check field constraints");
                    }
                }
            }
            MCPError::RateLimit(err) => {
                ctx = ctx.with_context("error_type".to_string(), "rate_limit".to_string());
                match err {
                    RateLimitError::TooManyRequests { retry_after, limit } => {
                        ctx = ctx
                            .with_context("retry_after".to_string(), retry_after.to_string())
                            .with_context("limit".to_string(), limit.to_string())
                            .retryable(*retry_after)
                            .with_recovery_hint("Wait before making more requests")
                            .with_recovery_hint("Consider reducing request frequency");
                    }
                    RateLimitError::QuotaExceeded { quota, period } => {
                        ctx = ctx
                            .with_context("quota".to_string(), quota.to_string())
                            .with_context("period".to_string(), period.to_string())
                            .with_recovery_hint("Check usage limits")
                            .with_recovery_hint("Consider upgrading quota");
                    }
                }
            }
            _ => {
                ctx = ctx.with_context("error_type".to_string(), "other".to_string());
            }
        }
        
        ctx
    }
}

impl From<crate::protocol::jsonrpc::JsonRpcError> for MCPError {
    fn from(err: crate::protocol::jsonrpc::JsonRpcError) -> Self {
        match err.code {
            error_codes::PARSE_ERROR => MCPError::Protocol(ProtocolError::SerializationError(
                err.message
            )),
            error_codes::INVALID_REQUEST => MCPError::Protocol(ProtocolError::InvalidRequest(
                err.message
            )),
            error_codes::METHOD_NOT_FOUND => MCPError::Protocol(ProtocolError::MethodNotFound(
                err.message
            )),
            error_codes::INVALID_PARAMS => MCPError::Protocol(ProtocolError::InvalidParams(
                err.message
            )),
            error_codes::INTERNAL_ERROR => MCPError::Protocol(ProtocolError::InternalError(
                err.message
            )),
            _ => MCPError::Protocol(ProtocolError::InternalError(
                err.message
            )),
        }
    }
}

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

#[derive(Debug, Error)]
pub enum RateLimitError {
    #[error("Too many requests. Retry after {retry_after}ms. Limit: {limit}")]
    TooManyRequests { retry_after: u64, limit: u32 },

    #[error("Quota exceeded: {quota} requests per {period}")]
    QuotaExceeded { quota: u32, period: String },
}

pub mod error_codes {
    // Standard JSON-RPC error codes
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

impl From<MCPError> for crate::protocol::jsonrpc::JsonRpcError {
    fn from(err: MCPError) -> Self {
        let (code, message) = match err {
            MCPError::Protocol(ProtocolError::InvalidVersion(msg)) => {
                (error_codes::INVALID_REQUEST, msg)
            }
            MCPError::Protocol(ProtocolError::InvalidRequestId(msg)) => {
                (error_codes::INVALID_REQUEST, msg)
            }
            MCPError::Protocol(ProtocolError::MethodNotFound(msg)) => {
                (error_codes::METHOD_NOT_FOUND, msg)
            }
            MCPError::Protocol(ProtocolError::InvalidParams(msg)) => {
                (error_codes::INVALID_PARAMS, msg)
            }
            MCPError::Protocol(ProtocolError::InvalidRequest(msg)) => {
                (error_codes::INVALID_REQUEST, msg)
            }
            MCPError::Protocol(ProtocolError::InvalidResponse(msg)) => {
                (error_codes::INTERNAL_ERROR, msg)
            }
            MCPError::Protocol(ProtocolError::RequestTimeout) => {
                (error_codes::INTERNAL_ERROR, "Request timeout".to_string())
            }
            MCPError::Protocol(ProtocolError::InternalError(msg)) => {
                (error_codes::INTERNAL_ERROR, msg)
            }
            MCPError::Protocol(ProtocolError::InitializationFailed(msg)) => {
                (error_codes::INITIALIZATION_FAILED, msg)
            }
            MCPError::Protocol(ProtocolError::CapabilityNotSupported(msg)) => {
                (error_codes::CAPABILITY_NOT_SUPPORTED, msg)
            }
            MCPError::Protocol(ProtocolError::NotFound(msg)) => {
                (error_codes::RESOURCE_NOT_FOUND, msg)
            }
            MCPError::Protocol(ProtocolError::ConnectionClosed) => {
                (error_codes::INTERNAL_ERROR, "Connection closed".to_string())
            }
            MCPError::Protocol(ProtocolError::TransportError(msg)) => {
                (error_codes::INTERNAL_ERROR, msg)
            }
            MCPError::Protocol(ProtocolError::SerializationError(msg)) => {
                (error_codes::PARSE_ERROR, msg)
            }
            MCPError::Protocol(ProtocolError::AuthenticationError(msg)) => {
                (error_codes::AUTHENTICATION_ERROR, msg)
            }
            MCPError::Transport(TransportError::ConnectionFailed(msg)) => {
                (error_codes::INTERNAL_ERROR, msg)
            }
            MCPError::Transport(TransportError::ConnectionClosed) => {
                (error_codes::INTERNAL_ERROR, "Connection closed".to_string())
            }
            MCPError::Transport(TransportError::SendFailed(msg)) => {
                (error_codes::INTERNAL_ERROR, msg)
            }
            MCPError::Transport(TransportError::ReceiveFailed(msg)) => {
                (error_codes::INTERNAL_ERROR, msg)
            }
            MCPError::ToolExecution(ToolError::NotFound(msg)) => {
                (error_codes::TOOL_EXECUTION_ERROR, msg)
            }
            MCPError::ToolExecution(ToolError::ExecutionFailed(msg)) => {
                (error_codes::TOOL_EXECUTION_ERROR, msg)
            }
            MCPError::ToolExecution(ToolError::InvalidInput(msg)) => {
                (error_codes::INVALID_PARAMS, msg)
            }
            MCPError::ToolExecution(ToolError::SchemaValidation(msg)) => {
                (error_codes::INVALID_PARAMS, msg)
            }
            MCPError::Resource(ResourceError::NotFound(msg)) => {
                (error_codes::RESOURCE_NOT_FOUND, msg)
            }
            MCPError::Resource(ResourceError::AccessDenied(msg)) => {
                (error_codes::ACCESS_DENIED, msg)
            }
            MCPError::Resource(ResourceError::InvalidUri(msg)) => {
                (error_codes::INVALID_URI, msg)
            }
            MCPError::Resource(ResourceError::ContentTypeMismatch { expected, actual }) => {
                (error_codes::INTERNAL_ERROR, format!("Content type mismatch: expected {}, got {}", expected, actual))
            }
            MCPError::Authentication(AuthenticationError::InvalidCredentials) => {
                (error_codes::AUTHENTICATION_ERROR, "Invalid credentials".to_string())
            }
            MCPError::Authentication(AuthenticationError::TokenExpired) => {
                (error_codes::AUTHENTICATION_ERROR, "Token expired".to_string())
            }
            MCPError::Authentication(AuthenticationError::InsufficientPermissions { resource }) => {
                (error_codes::ACCESS_DENIED, format!("Insufficient permissions for resource: {}", resource))
            }
            MCPError::Authentication(AuthenticationError::OAuthError { error, description }) => {
                (error_codes::AUTHENTICATION_ERROR, format!("OAuth error: {} - {}", error, description))
            }
            MCPError::Validation(ValidationError::SchemaValidation { field, details }) => {
                (error_codes::VALIDATION_ERROR, format!("Schema validation failed for field '{}': {}", field, details))
            }
            MCPError::Validation(ValidationError::RequiredField { field }) => {
                (error_codes::INVALID_PARAMS, format!("Required field '{}' is missing", field))
            }
            MCPError::Validation(ValidationError::InvalidFormat { field, expected }) => {
                (error_codes::INVALID_PARAMS, format!("Invalid format for field '{}': expected {}", field, expected))
            }
            MCPError::Validation(ValidationError::ValueOutOfRange { field, min, max, actual }) => {
                (error_codes::INVALID_PARAMS, format!("Value for field '{}' is out of range: {} (expected {}..{})", field, actual, min, max))
            }
            MCPError::RateLimit(RateLimitError::TooManyRequests { retry_after, limit }) => {
                (error_codes::RATE_LIMIT_ERROR, format!("Too many requests. Retry after {}ms. Limit: {}", retry_after, limit))
            }
            MCPError::RateLimit(RateLimitError::QuotaExceeded { quota, period }) => {
                (error_codes::RATE_LIMIT_ERROR, format!("Quota exceeded: {} requests per {}", quota, period))
            }
            MCPError::Serialization(err) => {
                (error_codes::PARSE_ERROR, err.to_string())
            }
            MCPError::Io(err) => {
                (error_codes::INTERNAL_ERROR, err.to_string())
            }
            MCPError::Other(err) => {
                (error_codes::INTERNAL_ERROR, err.to_string())
            }
        };

        crate::protocol::jsonrpc::JsonRpcError {
            code,
            message,
            data: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_variant_conversion() {
        let protocol_err = ProtocolError::MethodNotFound("test_method".to_string());
        let mcp_err = MCPError::Protocol(protocol_err);
        let jsonrpc_err: crate::protocol::jsonrpc::JsonRpcError = mcp_err.into();
        
        assert_eq!(jsonrpc_err.code, error_codes::METHOD_NOT_FOUND);
        assert_eq!(jsonrpc_err.message, Some("test_method".to_string()));
    }

    #[test]
    fn test_error_context() {
        let err = MCPError::Protocol(ProtocolError::MethodNotFound("test_method".to_string()));
        let ctx = err.context();
        
        assert_eq!(ctx.context.get("error_type"), Some(&"protocol".to_string()));
        assert_eq!(ctx.context.get("method"), Some(&"test_method".to_string()));
        assert!(!ctx.recovery_hints.is_empty());
        assert!(!ctx.retryable);
    }

    #[test]
    fn test_retryable_errors() {
        let transport_err = MCPError::Transport(TransportError::ConnectionFailed("test".to_string()));
        assert!(transport_err.is_retryable());
        assert_eq!(transport_err.retry_delay_ms(), Some(1000));

        let timeout_err = MCPError::Protocol(ProtocolError::RequestTimeout);
        assert!(timeout_err.is_retryable());
        assert_eq!(timeout_err.retry_delay_ms(), Some(5000));

        let validation_err = MCPError::Validation(ValidationError::RequiredField { field: "test".to_string() });
        assert!(!validation_err.is_retryable());
        assert_eq!(validation_err.retry_delay_ms(), None);
    }

    #[test]
    fn test_error_context_builder() {
        let ctx = ErrorContext::new()
            .with_context("key".to_string(), "value".to_string())
            .with_recovery_hint("test hint".to_string())
            .retryable(5000)
            .with_correlation_id("test-id".to_string());

        assert_eq!(ctx.context.get("key"), Some(&"value".to_string()));
        assert_eq!(ctx.recovery_hints, vec!["test hint".to_string()]);
        assert!(ctx.retryable);
        assert_eq!(ctx.retry_delay_ms, Some(5000));
        assert_eq!(ctx.correlation_id, Some("test-id".to_string()));
    }
}
