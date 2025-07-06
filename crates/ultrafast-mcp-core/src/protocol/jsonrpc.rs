use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Configuration constants for JSON-RPC protocol
pub mod config {
    /// Maximum length for request ID strings
    pub const MAX_REQUEST_ID_LENGTH: usize = 1000;
    /// Minimum value for numeric request IDs
    pub const MIN_REQUEST_ID_NUMBER: i64 = -999_999_999;
    /// Maximum value for numeric request IDs
    pub const MAX_REQUEST_ID_NUMBER: i64 = 999_999_999;
    /// JSON-RPC version string
    pub const JSONRPC_VERSION: &str = "2.0";
}

/// Standard JSON-RPC 2.0 error codes
pub mod error_codes {
    /// Parse error (invalid JSON)
    pub const PARSE_ERROR: i32 = -32700;
    /// Invalid request (malformed request)
    pub const INVALID_REQUEST: i32 = -32600;
    /// Method not found
    pub const METHOD_NOT_FOUND: i32 = -32601;
    /// Invalid parameters
    pub const INVALID_PARAMS: i32 = -32602;
    /// Internal error
    pub const INTERNAL_ERROR: i32 = -32603;
    /// Server error (implementation-defined)
    pub const SERVER_ERROR_START: i32 = -32000;
    pub const SERVER_ERROR_END: i32 = -32099;
}

/// MCP-specific error codes
pub mod mcp_error_codes {
    /// Initialization failed
    pub const INITIALIZATION_FAILED: i32 = -32000;
    /// Capability not supported
    pub const CAPABILITY_NOT_SUPPORTED: i32 = -32001;
    /// Resource not found
    pub const RESOURCE_NOT_FOUND: i32 = -32002;
    /// Tool execution error
    pub const TOOL_EXECUTION_ERROR: i32 = -32003;
    /// Invalid URI
    pub const INVALID_URI: i32 = -32004;
    /// Access denied
    pub const ACCESS_DENIED: i32 = -32005;
    /// Request timeout
    pub const REQUEST_TIMEOUT: i32 = -32006;
    /// Protocol version not supported
    pub const PROTOCOL_VERSION_NOT_SUPPORTED: i32 = -32007;
}

/// JSON-RPC 2.0 request ID can be string or number (null is not supported in MCP)
/// 
/// This enum represents the request ID field in JSON-RPC messages. According to the
/// JSON-RPC 2.0 specification, request IDs can be strings, numbers, or null. However,
/// the MCP specification does not support null request IDs, so this implementation
/// only supports strings and numbers.
/// 
/// # Examples
/// 
/// ```rust
/// use ultrafast_mcp_core::protocol::jsonrpc::RequestId;
/// 
/// // String-based request ID
/// let string_id = RequestId::string("request-123");
/// 
/// // Number-based request ID
/// let number_id = RequestId::number(42);
/// 
/// // Validation
/// assert!(string_id.validate().is_ok());
/// assert!(number_id.validate().is_ok());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    /// String-based request ID
    String(String),
    /// Number-based request ID
    Number(i64),
}

impl RequestId {
    /// Create a new string-based request ID
    pub fn string(s: impl Into<String>) -> Self {
        Self::String(s.into())
    }

    /// Create a new number-based request ID
    pub fn number(n: i64) -> Self {
        Self::Number(n)
    }

    /// Validate the request ID
    pub fn validate(&self) -> Result<(), crate::error::ProtocolError> {
        match self {
            RequestId::String(s) => {
                if s.is_empty() {
                    return Err(crate::error::ProtocolError::InvalidRequestId(
                        "Request ID string cannot be empty".to_string(),
                    ));
                }
                // Relaxed length check - allow reasonable lengths up to configurable limit
                if s.len() > config::MAX_REQUEST_ID_LENGTH {
                    return Err(crate::error::ProtocolError::InvalidRequestId(
                        format!("Request ID string too long (max {} characters)", config::MAX_REQUEST_ID_LENGTH),
                    ));
                }
            }
            RequestId::Number(n) => {
                // Reasonable range check - allow numbers within practical JSON-RPC limits
                // This prevents overflow issues and keeps IDs manageable
                if *n < config::MIN_REQUEST_ID_NUMBER || *n > config::MAX_REQUEST_ID_NUMBER {
                    return Err(crate::error::ProtocolError::InvalidRequestId(
                        format!("Request ID number out of range ({} to {})", config::MIN_REQUEST_ID_NUMBER, config::MAX_REQUEST_ID_NUMBER),
                    ));
                }
            }
        }
        Ok(())
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestId::String(s) => write!(f, "{s}"),
            RequestId::Number(n) => write!(f, "{n}"),
        }
    }
}

impl From<String> for RequestId {
    fn from(s: String) -> Self {
        RequestId::String(s)
    }
}

impl From<i64> for RequestId {
    fn from(n: i64) -> Self {
        RequestId::Number(n)
    }
}

impl From<u64> for RequestId {
    fn from(n: u64) -> Self {
        RequestId::Number(n as i64)
    }
}

/// JSON-RPC 2.0 Request
/// 
/// Represents a JSON-RPC 2.0 request message. This struct contains all the required
/// fields for a valid JSON-RPC request according to the specification.
/// 
/// # Fields
/// 
/// - `jsonrpc`: The JSON-RPC version string (always "2.0")
/// - `method`: The name of the method to be invoked
/// - `params`: Optional parameters for the method
/// - `id`: Optional request ID (null for notifications)
/// - `meta`: Additional metadata fields (flattened)
/// 
/// # Examples
/// 
/// ```rust
/// use ultrafast_mcp_core::protocol::jsonrpc::{JsonRpcRequest, RequestId};
/// use serde_json::json;
/// 
/// // Create a request with parameters
/// let request = JsonRpcRequest::new(
///     "tools/call".to_string(),
///     Some(json!({"name": "echo", "arguments": {"message": "hello"}})),
///     Some(RequestId::number(1))
/// );
/// 
/// // Create a notification (no ID)
/// let notification = JsonRpcRequest::notification(
///     "initialized".to_string(),
///     None
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC version string (always "2.0")
    pub jsonrpc: String,
    /// The name of the method to be invoked
    pub method: String,
    /// Optional parameters for the method
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    /// Optional request ID (null for notifications)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RequestId>,
    /// Additional metadata fields (flattened)
    #[serde(flatten)]
    pub meta: HashMap<String, Value>,
}

impl JsonRpcRequest {
    pub fn new(method: String, params: Option<Value>, id: Option<RequestId>) -> Self {
        Self {
            jsonrpc: config::JSONRPC_VERSION.to_string(),
            method,
            params,
            id,
            meta: HashMap::new(),
        }
    }

    pub fn notification(method: String, params: Option<Value>) -> Self {
        Self {
            jsonrpc: config::JSONRPC_VERSION.to_string(),
            method,
            params,
            id: None,
            meta: HashMap::new(),
        }
    }

    pub fn is_notification(&self) -> bool {
        self.id.is_none()
    }

    pub fn with_meta(mut self, key: String, value: Value) -> Self {
        self.meta.insert(key, value);
        self
    }
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: Option<RequestId>,
    #[serde(flatten)]
    pub meta: HashMap<String, Value>,
}

impl JsonRpcResponse {
    pub fn success(result: Value, id: Option<RequestId>) -> Self {
        Self {
            jsonrpc: config::JSONRPC_VERSION.to_string(),
            result: Some(result),
            error: None,
            id,
            meta: HashMap::new(),
        }
    }

    pub fn error(error: JsonRpcError, id: Option<RequestId>) -> Self {
        Self {
            jsonrpc: config::JSONRPC_VERSION.to_string(),
            result: None,
            error: Some(error),
            id,
            meta: HashMap::new(),
        }
    }

    pub fn with_meta(mut self, key: String, value: Value) -> Self {
        self.meta.insert(key, value);
        self
    }
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    pub fn new(code: i32, message: String) -> Self {
        Self {
            code,
            message,
            data: None,
        }
    }

    pub fn with_data(mut self, data: Value) -> Self {
        self.data = Some(data);
        self
    }

    // Standard JSON-RPC error constructors
    pub fn parse_error(message: Option<String>) -> Self {
        Self::new(
            error_codes::PARSE_ERROR,
            message.unwrap_or_else(|| "Parse error".to_string()),
        )
    }

    pub fn invalid_request(message: Option<String>) -> Self {
        Self::new(
            error_codes::INVALID_REQUEST,
            message.unwrap_or_else(|| "Invalid Request".to_string()),
        )
    }

    pub fn method_not_found(method: String) -> Self {
        Self::new(
            error_codes::METHOD_NOT_FOUND,
            format!("Method not found: {}", method),
        )
    }

    pub fn invalid_params(message: Option<String>) -> Self {
        Self::new(
            error_codes::INVALID_PARAMS,
            message.unwrap_or_else(|| "Invalid params".to_string()),
        )
    }

    pub fn internal_error(message: Option<String>) -> Self {
        Self::new(
            error_codes::INTERNAL_ERROR,
            message.unwrap_or_else(|| "Internal error".to_string()),
        )
    }

    // MCP-specific error constructors
    pub fn initialization_failed(message: String) -> Self {
        Self::new(mcp_error_codes::INITIALIZATION_FAILED, message)
    }

    pub fn capability_not_supported(capability: String) -> Self {
        Self::new(
            mcp_error_codes::CAPABILITY_NOT_SUPPORTED,
            format!("Capability not supported: {}", capability),
        )
    }

    pub fn resource_not_found(uri: String) -> Self {
        Self::new(
            mcp_error_codes::RESOURCE_NOT_FOUND,
            format!("Resource not found: {}", uri),
        )
    }

    pub fn tool_execution_error(tool_name: String, error: String) -> Self {
        Self::new(
            mcp_error_codes::TOOL_EXECUTION_ERROR,
            format!("Tool execution failed for '{}': {}", tool_name, error),
        )
    }

    pub fn invalid_uri(uri: String) -> Self {
        Self::new(
            mcp_error_codes::INVALID_URI,
            format!("Invalid URI: {}", uri),
        )
    }

    pub fn access_denied(resource: String) -> Self {
        Self::new(
            mcp_error_codes::ACCESS_DENIED,
            format!("Access denied: {}", resource),
        )
    }

    pub fn request_timeout() -> Self {
        Self::new(
            mcp_error_codes::REQUEST_TIMEOUT,
            "Request timeout".to_string(),
        )
    }

    pub fn protocol_version_not_supported(version: String) -> Self {
        Self::new(
            mcp_error_codes::PROTOCOL_VERSION_NOT_SUPPORTED,
            format!("Protocol version not supported: {}", version),
        )
    }
}

/// JSON-RPC 2.0 Message (Request, Response, or Notification)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    Request(JsonRpcRequest),
    Response(JsonRpcResponse),
    Notification(JsonRpcRequest), // Notification is a request without an ID
}

impl JsonRpcMessage {
    pub fn get_id(&self) -> Option<&RequestId> {
        match self {
            JsonRpcMessage::Request(req) => req.id.as_ref(),
            JsonRpcMessage::Response(res) => res.id.as_ref(),
            JsonRpcMessage::Notification(_) => None,
        }
    }

    pub fn is_notification(&self) -> bool {
        match self {
            JsonRpcMessage::Request(req) => req.is_notification(),
            JsonRpcMessage::Response(_) => false,
            JsonRpcMessage::Notification(_) => true,
        }
    }
}

/// Validate JSON-RPC 2.0 message format
pub fn validate_jsonrpc_message(
    message: &JsonRpcMessage,
) -> Result<(), crate::error::ProtocolError> {
    match message {
        JsonRpcMessage::Request(req) => {
            if req.jsonrpc != "2.0" {
                return Err(crate::error::ProtocolError::InvalidVersion(
                    req.jsonrpc.clone(),
                ));
            }

            if req.method.is_empty() {
                return Err(crate::error::ProtocolError::MethodNotFound(
                    "Empty method name".to_string(),
                ));
            }

            // Check for reserved method names (starting with "rpc.")
            if req.method.starts_with("rpc.") {
                return Err(crate::error::ProtocolError::MethodNotFound(format!(
                    "Reserved method name: {}",
                    req.method
                )));
            }

            // Validate request ID if present
            if let Some(ref id) = req.id {
                id.validate()?;
            }
        }
        JsonRpcMessage::Response(res) => {
            if res.jsonrpc != "2.0" {
                return Err(crate::error::ProtocolError::InvalidVersion(
                    res.jsonrpc.clone(),
                ));
            }

            // Validate response ID if present
            if let Some(ref id) = res.id {
                id.validate()?;
            }

            // Response must have either result or error, but not both
            match (&res.result, &res.error) {
                (Some(_), Some(_)) => {
                    return Err(crate::error::ProtocolError::InvalidParams(
                        "Response cannot have both result and error".to_string(),
                    ));
                }
                (None, None) => {
                    return Err(crate::error::ProtocolError::InvalidParams(
                        "Response must have either result or error".to_string(),
                    ));
                }
                _ => {}
            }
        }
        JsonRpcMessage::Notification(notif) => {
            if notif.jsonrpc != "2.0" {
                return Err(crate::error::ProtocolError::InvalidVersion(
                    notif.jsonrpc.clone(),
                ));
            }

            if notif.method.is_empty() {
                return Err(crate::error::ProtocolError::MethodNotFound(
                    "Empty method name".to_string(),
                ));
            }

            // Notification must not have an ID
            if notif.id.is_some() {
                return Err(crate::error::ProtocolError::InvalidParams(
                    "Notification must not have an ID".to_string(),
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let req = JsonRpcRequest::new(
            "test_method".to_string(),
            Some(serde_json::json!({"param": "value"})),
            Some(RequestId::Number(1)),
        );

        let json = serde_json::to_string(&req).unwrap();
        let deserialized: JsonRpcRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(req.method, deserialized.method);
        assert_eq!(req.id, deserialized.id);
    }

    #[test]
    fn test_notification() {
        let notif = JsonRpcRequest::notification("test_notification".to_string(), None);

        assert!(notif.is_notification());
        assert_eq!(notif.id, None);
    }

    #[test]
    fn test_response_success() {
        let resp = JsonRpcResponse::success(
            serde_json::json!({"success": true}),
            Some(RequestId::String("test".to_string())),
        );

        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_response_error() {
        let error = JsonRpcError::new(-32602, "Invalid params".to_string());
        let resp = JsonRpcResponse::error(error, Some(RequestId::Number(1)));

        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
    }

    #[test]
    fn test_request_id_validation() {
        // Valid request IDs
        assert!(RequestId::string("valid_id").validate().is_ok());
        assert!(RequestId::number(123).validate().is_ok());
        assert!(RequestId::number(-123).validate().is_ok());

        // Invalid request IDs
        assert!(RequestId::string("").validate().is_err()); // Empty string
        assert!(RequestId::number(1000000000).validate().is_err()); // Too large
        assert!(RequestId::number(-1000000000).validate().is_err()); // Too small
    }

    #[test]
    fn test_request_id_to_string() {
        assert_eq!(RequestId::string("test").to_string(), "test");
        assert_eq!(RequestId::number(123).to_string(), "123");
        assert_eq!(RequestId::number(-123).to_string(), "-123");
    }

    #[test]
    fn test_jsonrpc_message_validation() {
        // Valid request
        let valid_req = JsonRpcRequest::new(
            "test_method".to_string(),
            None,
            Some(RequestId::string("valid_id")),
        );
        assert!(validate_jsonrpc_message(&JsonRpcMessage::Request(valid_req)).is_ok());

        // Invalid request with reserved method name
        let invalid_req = JsonRpcRequest::new(
            "rpc.test".to_string(),
            None,
            Some(RequestId::string("valid_id")),
        );
        assert!(validate_jsonrpc_message(&JsonRpcMessage::Request(invalid_req)).is_err());

        // Invalid request with empty method name
        let empty_method_req =
            JsonRpcRequest::new("".to_string(), None, Some(RequestId::string("valid_id")));
        assert!(validate_jsonrpc_message(&JsonRpcMessage::Request(empty_method_req)).is_err());
    }
}
