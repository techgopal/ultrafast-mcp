use crate::protocol::constants::{
    JSONRPC_VERSION, MAX_REQUEST_ID_LENGTH, MAX_REQUEST_ID_NUMBER, MIN_REQUEST_ID_NUMBER,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;
use std::collections::HashMap;

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

/// JSON-RPC 2.0 request ID can be string or number
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
                if s.len() > MAX_REQUEST_ID_LENGTH {
                    return Err(crate::error::ProtocolError::InvalidRequestId(format!(
                        "Request ID string too long (max {MAX_REQUEST_ID_LENGTH} characters)"
                    )));
                }
            }
            RequestId::Number(n) => {
                if *n < MIN_REQUEST_ID_NUMBER || *n > MAX_REQUEST_ID_NUMBER {
                    return Err(crate::error::ProtocolError::InvalidRequestId(format!(
                        "Request ID number out of range ({MIN_REQUEST_ID_NUMBER} to {MAX_REQUEST_ID_NUMBER})"
                    )));
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

/// JSON-RPC 2.0 Request with optimized memory usage
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcRequest {
    /// JSON-RPC version string (always "2.0")
    #[serde(rename = "jsonrpc")]
    pub jsonrpc: Cow<'static, str>,
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
            jsonrpc: Cow::Borrowed(JSONRPC_VERSION),
            method,
            params,
            id,
            meta: HashMap::new(),
        }
    }

    pub fn notification(method: String, params: Option<Value>) -> Self {
        Self::new(method, params, None)
    }

    pub fn is_notification(&self) -> bool {
        self.id.is_none()
    }

    pub fn with_meta(mut self, key: String, value: Value) -> Self {
        self.meta.insert(key, value);
        self
    }
}

/// JSON-RPC 2.0 Response with optimized memory usage
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcResponse {
    #[serde(rename = "jsonrpc")]
    pub jsonrpc: Cow<'static, str>,
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
            jsonrpc: Cow::Borrowed(JSONRPC_VERSION),
            result: Some(result),
            error: None,
            id,
            meta: HashMap::new(),
        }
    }

    pub fn error(error: JsonRpcError, id: Option<RequestId>) -> Self {
        Self {
            jsonrpc: Cow::Borrowed(JSONRPC_VERSION),
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

/// JSON-RPC 2.0 Error with improved error handling and optimized memory usage
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

    pub fn parse_error(message: Option<String>) -> Self {
        let msg = message.unwrap_or_else(|| "Parse error".to_string());
        Self::new(error_codes::PARSE_ERROR, msg)
    }

    pub fn invalid_request(message: Option<String>) -> Self {
        let msg = message.unwrap_or_else(|| "Invalid request".to_string());
        Self::new(error_codes::INVALID_REQUEST, msg)
    }

    pub fn method_not_found(method: String) -> Self {
        Self::new(
            error_codes::METHOD_NOT_FOUND,
            format!("Method not found: {method}"),
        )
    }

    pub fn invalid_params(message: Option<String>) -> Self {
        let msg = message.unwrap_or_else(|| "Invalid parameters".to_string());
        Self::new(error_codes::INVALID_PARAMS, msg)
    }

    pub fn internal_error(message: Option<String>) -> Self {
        let msg = message.unwrap_or_else(|| "Internal error".to_string());
        Self::new(error_codes::INTERNAL_ERROR, msg)
    }

    pub fn initialization_failed(message: String) -> Self {
        Self::new(mcp_error_codes::INITIALIZATION_FAILED, message)
    }

    pub fn capability_not_supported(capability: String) -> Self {
        Self::new(
            mcp_error_codes::CAPABILITY_NOT_SUPPORTED,
            format!("Capability not supported: {capability}"),
        )
    }

    pub fn resource_not_found(uri: String) -> Self {
        Self::new(
            mcp_error_codes::RESOURCE_NOT_FOUND,
            format!("Resource not found: {uri}"),
        )
    }

    pub fn tool_execution_error(tool_name: String, error: String) -> Self {
        Self::new(
            mcp_error_codes::TOOL_EXECUTION_ERROR,
            format!("Tool execution error for '{tool_name}': {error}"),
        )
    }

    pub fn invalid_uri(uri: String) -> Self {
        Self::new(mcp_error_codes::INVALID_URI, format!("Invalid URI: {uri}"))
    }

    pub fn access_denied(resource: String) -> Self {
        Self::new(
            mcp_error_codes::ACCESS_DENIED,
            format!("Access denied to resource: {resource}"),
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
            format!("Protocol version not supported: {version}"),
        )
    }
}

/// JSON-RPC 2.0 Message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
            JsonRpcMessage::Response(resp) => resp.id.as_ref(),
            JsonRpcMessage::Notification(_) => None,
        }
    }

    pub fn is_notification(&self) -> bool {
        matches!(self, JsonRpcMessage::Notification(_))
    }
}

/// Validate JSON-RPC message format with improved error handling
pub fn validate_jsonrpc_message(
    message: &JsonRpcMessage,
) -> Result<(), crate::error::ProtocolError> {
    match message {
        JsonRpcMessage::Request(request) => {
            if request.jsonrpc != JSONRPC_VERSION {
                return Err(crate::error::ProtocolError::InvalidVersion(format!(
                    "Expected JSON-RPC version {}, got {}",
                    JSONRPC_VERSION, request.jsonrpc
                )));
            }

            if request.method.is_empty() {
                return Err(crate::error::ProtocolError::InvalidRequest(
                    "Method name cannot be empty".to_string(),
                ));
            }

            if let Some(ref id) = request.id {
                id.validate()?;
            }
        }
        JsonRpcMessage::Response(response) => {
            if response.jsonrpc != JSONRPC_VERSION {
                return Err(crate::error::ProtocolError::InvalidVersion(format!(
                    "Expected JSON-RPC version {}, got {}",
                    JSONRPC_VERSION, response.jsonrpc
                )));
            }

            if response.result.is_some() && response.error.is_some() {
                return Err(crate::error::ProtocolError::InvalidResponse(
                    "Response cannot have both result and error".to_string(),
                ));
            }

            if response.result.is_none() && response.error.is_none() {
                return Err(crate::error::ProtocolError::InvalidResponse(
                    "Response must have either result or error".to_string(),
                ));
            }

            if let Some(ref id) = response.id {
                id.validate()?;
            }
        }
        JsonRpcMessage::Notification(notification) => {
            if notification.jsonrpc != JSONRPC_VERSION {
                return Err(crate::error::ProtocolError::InvalidVersion(format!(
                    "Expected JSON-RPC version {}, got {}",
                    JSONRPC_VERSION, notification.jsonrpc
                )));
            }

            if notification.method.is_empty() {
                return Err(crate::error::ProtocolError::InvalidRequest(
                    "Method name cannot be empty".to_string(),
                ));
            }

            if notification.id.is_some() {
                return Err(crate::error::ProtocolError::InvalidRequest(
                    "Notification cannot have an ID".to_string(),
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
        let request = JsonRpcRequest::new(
            "test_method".to_string(),
            Some(serde_json::json!({"param": "value"})),
            Some(RequestId::number(1)),
        );

        let serialized = serde_json::to_string(&request).expect("Failed to serialize request");
        let deserialized: JsonRpcRequest = serde_json::from_str(&serialized).expect("Failed to deserialize request");

        assert_eq!(request, deserialized);
    }

    #[test]
    fn test_notification() {
        let notification = JsonRpcRequest::notification(
            "test_notification".to_string(),
            Some(serde_json::json!({"data": "value"})),
        );

        assert!(notification.is_notification());
        assert_eq!(notification.id, None);
    }

    #[test]
    fn test_response_success() {
        let response = JsonRpcResponse::success(
            serde_json::json!({"result": "success"}),
            Some(RequestId::number(1)),
        );

        assert!(response.result.is_some());
        assert!(response.error.is_none());
        assert_eq!(response.id, Some(RequestId::number(1)));
    }

    #[test]
    fn test_response_error() {
        let error = JsonRpcError::method_not_found("unknown_method".to_string());
        let response = JsonRpcResponse::error(error, Some(RequestId::number(1)));

        assert!(response.result.is_none());
        assert!(response.error.is_some());
        assert_eq!(response.id, Some(RequestId::number(1)));
    }

    #[test]
    fn test_request_id_validation() {
        // Valid string ID
        let valid_string = RequestId::string("valid_id");
        assert!(valid_string.validate().is_ok());

        // Valid number ID
        let valid_number = RequestId::number(123);
        assert!(valid_number.validate().is_ok());

        // Invalid empty string
        let invalid_string = RequestId::string("");
        assert!(invalid_string.validate().is_err());

        // Invalid number out of range
        let invalid_number = RequestId::number(9999999999);
        assert!(invalid_number.validate().is_err());
    }

    #[test]
    fn test_request_id_to_string() {
        let string_id = RequestId::string("test_id");
        assert_eq!(string_id.to_string(), "test_id");

        let number_id = RequestId::number(123);
        assert_eq!(number_id.to_string(), "123");
    }

    #[test]
    fn test_jsonrpc_message_validation() {
        // Valid request
        let valid_request = JsonRpcMessage::Request(JsonRpcRequest::new(
            "test_method".to_string(),
            None,
            Some(RequestId::number(1)),
        ));
        assert!(validate_jsonrpc_message(&valid_request).is_ok());

        // Invalid request with empty method
        let invalid_request = JsonRpcMessage::Request(JsonRpcRequest::new(
            "".to_string(),
            None,
            Some(RequestId::number(1)),
        ));
        assert!(validate_jsonrpc_message(&invalid_request).is_err());
    }
}
