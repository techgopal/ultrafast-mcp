use thiserror::Error;

/// MCPError is the canonical error type for all MCP operations.
///
/// This is the preferred error type to use in all public APIs and user code.
pub type MCPResult<T> = Result<T, MCPError>;

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

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

impl MCPError {
    // Convenience constructors for backward compatibility
    pub fn invalid_params(msg: String) -> Self {
        Self::Protocol(ProtocolError::InvalidParams(msg))
    }

    pub fn method_not_found(msg: String) -> Self {
        Self::Protocol(ProtocolError::MethodNotFound(msg))
    }

    pub fn not_found(msg: String) -> Self {
        Self::Protocol(ProtocolError::NotFound(msg))
    }

    pub fn invalid_request(msg: String) -> Self {
        Self::Protocol(ProtocolError::InvalidRequest(msg))
    }

    pub fn invalid_response(msg: String) -> Self {
        Self::Protocol(ProtocolError::InvalidResponse(msg))
    }

    pub fn serialization_error(msg: String) -> Self {
        Self::Other(anyhow::anyhow!("Serialization error: {}", msg))
    }

    pub fn transport_error(msg: String) -> Self {
        Self::Transport(TransportError::ConnectionFailed(msg))
    }

    pub fn request_timeout() -> Self {
        Self::Protocol(ProtocolError::RequestTimeout)
    }

    pub fn internal_error(msg: String) -> Self {
        Self::Protocol(ProtocolError::InternalError(msg))
    }
}

impl From<crate::protocol::jsonrpc::JsonRpcError> for MCPError {
    fn from(err: crate::protocol::jsonrpc::JsonRpcError) -> Self {
        Self::Protocol(ProtocolError::InvalidResponse(err.message))
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

// JSON-RPC error codes
pub mod error_codes {
    // Standard JSON-RPC errors
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
}

impl From<MCPError> for crate::protocol::JsonRpcError {
    fn from(err: MCPError) -> Self {
        use crate::protocol::JsonRpcError;

        match err {
            MCPError::Protocol(ProtocolError::InvalidVersion(version)) => JsonRpcError::new(
                error_codes::INVALID_REQUEST,
                format!("Invalid JSON-RPC version: {}", version),
            ),
            MCPError::Protocol(ProtocolError::InvalidRequestId(id)) => JsonRpcError::new(
                error_codes::INVALID_REQUEST,
                format!("Invalid request ID: {}", id),
            ),
            MCPError::Protocol(ProtocolError::ConnectionClosed) => JsonRpcError::new(
                error_codes::INTERNAL_ERROR,
                "Connection closed".to_string(),
            ),
            MCPError::Protocol(ProtocolError::TransportError(msg)) => JsonRpcError::new(
                error_codes::INTERNAL_ERROR,
                format!("Transport error: {}", msg),
            ),
            MCPError::Protocol(ProtocolError::SerializationError(msg)) => JsonRpcError::new(
                error_codes::PARSE_ERROR,
                format!("Serialization error: {}", msg),
            ),
            MCPError::Protocol(ProtocolError::MethodNotFound(method)) => JsonRpcError::new(
                error_codes::METHOD_NOT_FOUND,
                format!("Method not found: {}", method),
            ),
            MCPError::Protocol(ProtocolError::InvalidParams(msg)) => JsonRpcError::new(
                error_codes::INVALID_PARAMS,
                format!("Invalid params: {}", msg),
            ),
            MCPError::Protocol(ProtocolError::InitializationFailed(msg)) => JsonRpcError::new(
                error_codes::INITIALIZATION_FAILED,
                format!("Initialization failed: {}", msg),
            ),
            MCPError::Protocol(ProtocolError::CapabilityNotSupported(cap)) => JsonRpcError::new(
                error_codes::CAPABILITY_NOT_SUPPORTED,
                format!("Capability not supported: {}", cap),
            ),
            MCPError::Protocol(ProtocolError::NotFound(msg)) => JsonRpcError::new(
                error_codes::RESOURCE_NOT_FOUND,
                format!("Not found: {}", msg),
            ),
            MCPError::Protocol(ProtocolError::InvalidRequest(msg)) => JsonRpcError::new(
                error_codes::INVALID_REQUEST,
                format!("Invalid request: {}", msg),
            ),
            MCPError::Protocol(ProtocolError::InvalidResponse(msg)) => JsonRpcError::new(
                error_codes::INVALID_REQUEST,
                format!("Invalid response: {}", msg),
            ),
            MCPError::Protocol(ProtocolError::RequestTimeout) => JsonRpcError::new(
                error_codes::INTERNAL_ERROR,
                "Request timeout".to_string(),
            ),
            MCPError::Protocol(ProtocolError::InternalError(msg)) => JsonRpcError::new(
                error_codes::INTERNAL_ERROR,
                format!("Internal error: {}", msg),
            ),
            MCPError::Protocol(ProtocolError::AuthenticationError(msg)) => JsonRpcError::new(
                error_codes::ACCESS_DENIED,
                format!("Authentication error: {}", msg),
            ),
            MCPError::ToolExecution(ToolError::NotFound(tool)) => JsonRpcError::new(
                error_codes::METHOD_NOT_FOUND,
                format!("Tool not found: {}", tool),
            ),
            MCPError::ToolExecution(ToolError::ExecutionFailed(msg)) => JsonRpcError::new(
                error_codes::TOOL_EXECUTION_ERROR,
                format!("Tool execution failed: {}", msg),
            ),
            MCPError::ToolExecution(ToolError::InvalidInput(msg)) => JsonRpcError::new(
                error_codes::INVALID_PARAMS,
                format!("Invalid tool input: {}", msg),
            ),
            MCPError::ToolExecution(ToolError::SchemaValidation(msg)) => JsonRpcError::new(
                error_codes::INVALID_PARAMS,
                format!("Schema validation failed: {}", msg),
            ),
            MCPError::Resource(ResourceError::NotFound(uri)) => JsonRpcError::new(
                error_codes::RESOURCE_NOT_FOUND,
                format!("Resource not found: {}", uri),
            ),
            MCPError::Resource(ResourceError::AccessDenied(msg)) => JsonRpcError::new(
                error_codes::ACCESS_DENIED,
                format!("Access denied: {}", msg),
            ),
            MCPError::Resource(ResourceError::InvalidUri(uri)) => {
                JsonRpcError::new(error_codes::INVALID_URI, format!("Invalid URI: {}", uri))
            }
            MCPError::Resource(ResourceError::ContentTypeMismatch { expected, actual }) => {
                JsonRpcError::new(
                    error_codes::INVALID_REQUEST,
                    format!("Content type mismatch: expected {}, got {}", expected, actual),
                )
            }
            MCPError::Transport(TransportError::ConnectionFailed(msg)) => JsonRpcError::new(
                error_codes::INTERNAL_ERROR,
                format!("Connection failed: {}", msg),
            ),
            MCPError::Transport(TransportError::ConnectionClosed) => JsonRpcError::new(
                error_codes::INTERNAL_ERROR,
                "Connection closed".to_string(),
            ),
            MCPError::Transport(TransportError::SendFailed(msg)) => JsonRpcError::new(
                error_codes::INTERNAL_ERROR,
                format!("Send failed: {}", msg),
            ),
            MCPError::Transport(TransportError::ReceiveFailed(msg)) => JsonRpcError::new(
                error_codes::INTERNAL_ERROR,
                format!("Receive failed: {}", msg),
            ),
            MCPError::Serialization(e) => JsonRpcError::new(
                error_codes::PARSE_ERROR,
                format!("Serialization error: {}", e),
            ),
            MCPError::Io(e) => JsonRpcError::new(
                error_codes::INTERNAL_ERROR,
                format!("IO error: {}", e),
            ),
            MCPError::Other(e) => JsonRpcError::new(
                error_codes::INTERNAL_ERROR,
                format!("Other error: {}", e),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::JsonRpcError;

    #[test]
    fn test_error_variant_conversion() {
        let errors = vec![
            MCPError::method_not_found("test_method".to_string()),
            MCPError::invalid_params("bad params".to_string()),
            MCPError::not_found("missing resource".to_string()),
            MCPError::internal_error("internal".to_string()),
        ];
        for err in errors {
            let rpc: JsonRpcError = err.into();
            assert!(!rpc.message.is_empty());
        }
    }
}
