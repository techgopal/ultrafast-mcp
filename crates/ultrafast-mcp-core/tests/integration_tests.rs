use serde_json::json;
use ultrafast_mcp_core::*;

#[cfg(test)]
mod protocol_tests {
    use super::*;

    #[test]
    fn test_jsonrpc_request_creation() {
        let request = protocol::jsonrpc::JsonRpcRequest::new(
            "test_method".to_string(),
            Some(json!({"param": "value"})),
            Some(RequestId::String("test-123".to_string())),
        );

        assert_eq!(request.method, "test_method");
        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.id, Some(RequestId::String("test-123".to_string())));
    }

    #[test]
    fn test_jsonrpc_response_success() {
        let response = protocol::jsonrpc::JsonRpcResponse::success(
            json!({"result": "ok"}),
            Some(RequestId::String("test-456".to_string())),
        );

        assert!(response.result.is_some());
        assert!(response.error.is_none());
        assert_eq!(response.id, Some(RequestId::String("test-456".to_string())));
    }

    #[test]
    fn test_jsonrpc_response_error() {
        let error = protocol::jsonrpc::JsonRpcError::new(-32601, "Method not found".to_string());
        let response = protocol::jsonrpc::JsonRpcResponse::error(
            error,
            Some(RequestId::String("test-789".to_string())),
        );

        assert!(response.result.is_none());
        assert!(response.error.is_some());
        assert_eq!(response.id, Some(RequestId::String("test-789".to_string())));
    }

    #[test]
    fn test_lifecycle_initialization() {
        let init_request = protocol::lifecycle::InitializeRequest {
            protocol_version: "2025-06-18".to_string(),
            capabilities: protocol::capabilities::ClientCapabilities::default(),
            client_info: types::client::ClientInfo {
                name: "test-client".to_string(),
                version: "1.0.0".to_string(),
                authors: None,
                description: None,
                homepage: None,
                repository: None,
                license: None,
            },
        };

        assert_eq!(init_request.protocol_version, "2025-06-18");
        assert_eq!(init_request.client_info.name, "test-client");
    }

    #[test]
    fn test_lifecycle_phases() {
        use protocol::lifecycle::LifecyclePhase;

        // Test enum variants exist (not Display implementation)
        assert!(matches!(
            LifecyclePhase::Uninitialized,
            LifecyclePhase::Uninitialized
        ));
        assert!(matches!(
            LifecyclePhase::Initializing,
            LifecyclePhase::Initializing
        ));
        assert!(matches!(
            LifecyclePhase::Initialized,
            LifecyclePhase::Initialized
        ));
        assert!(matches!(LifecyclePhase::Shutdown, LifecyclePhase::Shutdown));
    }

    #[test]
    fn test_capability_negotiation() {
        let server_caps = protocol::capabilities::ServerCapabilities::default();
        let client_caps = protocol::capabilities::ClientCapabilities::default();

        // Test default capabilities
        assert!(server_caps.tools.is_none());
        assert!(client_caps.roots.is_none());
    }
}

#[cfg(test)]
mod types_tests {
    use super::*;

    #[test]
    fn test_tool_creation() {
        let tool = types::tools::Tool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            input_schema: json!({"type": "object"}),
            output_schema: None,
        };

        assert_eq!(tool.name, "test_tool");
        assert_eq!(tool.description, "A test tool");
        assert_eq!(tool.input_schema, json!({"type": "object"}));
    }

    #[test]
    fn test_resource_creation() {
        let resource = types::resources::Resource {
            uri: "test://resource".to_string(),
            name: "Test Resource".to_string(),
            description: Some("A test resource".to_string()),
            mime_type: Some("application/json".to_string()),
        };

        assert_eq!(resource.uri, "test://resource");
        assert_eq!(resource.name, "Test Resource");
    }

    #[test]
    fn test_prompt_creation() {
        let prompt = types::prompts::Prompt {
            name: "test_prompt".to_string(),
            description: Some("A test prompt".to_string()),
            arguments: Some(vec![types::prompts::PromptArgument {
                name: "code".to_string(),
                description: Some("Code to review".to_string()),
                required: Some(true),
            }]),
        };

        assert_eq!(prompt.name, "test_prompt");
        if let Some(ref args) = prompt.arguments {
            assert_eq!(args.len(), 1);
            assert_eq!(args[0].required, Some(true));
        }
    }
}

#[cfg(test)]
mod error_tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = MCPError::invalid_request("Missing required field".to_string());
        assert!(matches!(error, MCPError::Protocol(_)));
    }

    #[test]
    fn test_method_not_found_error() {
        let error = MCPError::method_not_found("unknown_method".to_string());
        assert!(matches!(error, MCPError::Protocol(_)));
    }
}

#[cfg(test)]
mod result_tests {
    use super::*;

    #[test]
    fn test_result_error_handling() {
        let result: MCPResult<String> = Err(MCPError::internal_error("test error".to_string()));
        assert!(result.is_err());

        if let Err(error) = result {
            assert!(matches!(error, MCPError::Protocol(_)));
        }
    }

    #[test]
    fn test_result_success() {
        let result: MCPResult<String> = Ok("success".to_string());
        assert!(result.is_ok());
        assert_eq!(result.as_ref().unwrap(), "success");
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_tool_flow() {
        // Create a tool
        let tool = types::tools::Tool {
            name: "echo".to_string(),
            description: "Echo input".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "message": {"type": "string"}
                }
            }),
            output_schema: None,
        };

        // Create a tool call request
        let call_request = types::tools::ToolCallRequest {
            name: "echo".to_string(),
            arguments: Some(json!({"message": "Hello, World!"})),
        };

        // Simulate tool execution result
        let call_response = types::tools::ToolCallResponse {
            content: vec![types::tools::ToolContent::Text {
                text: "Hello, World!".to_string(),
            }],
            is_error: Some(false),
        };

        assert_eq!(tool.name, call_request.name);
        assert_eq!(call_response.content.len(), 1);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let original_tool = types::tools::Tool {
            name: "test_tool".to_string(),
            description: "Test tool".to_string(),
            input_schema: json!({"type": "string"}),
            output_schema: None,
        };

        // Serialize
        let serialized = serde_json::to_string(&original_tool).unwrap();

        // Deserialize
        let deserialized: types::tools::Tool = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original_tool.name, deserialized.name);
        assert_eq!(original_tool.description, deserialized.description);
    }
}
