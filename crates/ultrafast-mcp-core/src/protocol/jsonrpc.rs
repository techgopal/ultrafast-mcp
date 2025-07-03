use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// JSON-RPC 2.0 request ID can be string, number, or null
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    String(String),
    Number(i64),
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
            if req.method.starts_with("rpc.") && !req.method.starts_with("rpc.") {
                return Err(crate::error::ProtocolError::MethodNotFound(format!(
                    "Reserved method name: {}",
                    req.method
                )));
            }
        }
        JsonRpcMessage::Response(res) => {
            if res.jsonrpc != "2.0" {
                return Err(crate::error::ProtocolError::InvalidVersion(
                    res.jsonrpc.clone(),
                ));
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
}
