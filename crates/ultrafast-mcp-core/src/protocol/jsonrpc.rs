use serde::{Deserialize, Serialize};
use serde_json::Value;
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

/// JSON-RPC 2.0 request ID can be string or number (null is not supported in MCP)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    String(String),
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
                // Check for reasonable length
                if s.len() > 100 {
                    return Err(crate::error::ProtocolError::InvalidRequestId(
                        "Request ID string too long".to_string(),
                    ));
                }
            }
            RequestId::Number(n) => {
                // Check for reasonable range
                if *n < -999999999 || *n > 999999999 {
                    return Err(crate::error::ProtocolError::InvalidRequestId(
                        "Request ID number out of reasonable range".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Get the request ID as a string representation
    pub fn to_string(&self) -> String {
        match self {
            RequestId::String(s) => s.clone(),
            RequestId::Number(n) => n.to_string(),
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RequestId>,
    #[serde(flatten)]
    pub meta: HashMap<String, Value>,
}

impl JsonRpcRequest {
    pub fn new(method: String, params: Option<Value>, id: Option<RequestId>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method,
            params,
            id,
            meta: HashMap::new(),
        }
    }

    pub fn notification(method: String, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
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
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
            meta: HashMap::new(),
        }
    }

    pub fn error(error: JsonRpcError, id: Option<RequestId>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
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
        let empty_method_req = JsonRpcRequest::new(
            "".to_string(),
            None,
            Some(RequestId::string("valid_id")),
        );
        assert!(validate_jsonrpc_message(&JsonRpcMessage::Request(empty_method_req)).is_err());
    }
}
