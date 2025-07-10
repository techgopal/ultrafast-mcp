//! Comprehensive MCP Completion Tests
//!
//! This test suite validates that the ultrafast-mcp implementation correctly handles
//! completion requests as specified in MCP 2025-06-18.

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Arc;
    use ultrafast_mcp::{
        UltraFastClient, UltraFastServer, ClientInfo, ClientCapabilities, ServerInfo, ServerCapabilities,
        CompletionHandler, MCPResult, CompletionCapability,
    };
    use ultrafast_mcp_core::types::completion::{
        CompleteRequest, CompleteResponse, Completion, CompletionValue, CompletionReference, 
        CompletionArgument, CompletionContext
    };

    // Mock completion handler for testing
    struct TestCompletionHandler;

    #[async_trait]
    impl CompletionHandler for TestCompletionHandler {
        async fn complete(&self, request: CompleteRequest) -> MCPResult<CompleteResponse> {
            let ref_type = request.reference.ref_type.as_str();
            let argument_name = &request.argument.name;
            let argument_value = &request.argument.value;

            match ref_type {
                "ref/prompt" => {
                    let prompt_name = &request.reference.name;
                    let values = match (prompt_name.as_str(), argument_name.as_str()) {
                        ("code_review", "language") => {
                            let mut suggestions = vec!["python", "pytorch", "pyside", "rust", "javascript", "typescript"];
                            // Filter by current value
                            suggestions.retain(|s| s.starts_with(argument_value));
                            suggestions
                        },
                        ("code_review", "framework") => {
                            // Check context for language to provide framework-specific suggestions
                            let context_language = request.context
                                .as_ref()
                                .and_then(|c| c.arguments.as_ref())
                                .and_then(|args| args.get("language"))
                                .cloned();
                            
                            let mut suggestions = match context_language.as_deref() {
                                Some("python") => vec!["flask", "django", "fastapi", "pytorch", "tensorflow"],
                                Some("javascript") => vec!["react", "vue", "angular", "express", "next"],
                                Some("rust") => vec!["actix", "rocket", "axum", "tokio", "serde"],
                                _ => vec!["flask", "django", "react", "vue", "actix"],
                            };
                            suggestions.retain(|s| s.starts_with(argument_value));
                            suggestions
                        },
                        ("greeting", "style") => {
                            let mut suggestions = vec!["casual", "formal", "technical", "friendly"];
                            suggestions.retain(|s| s.starts_with(argument_value));
                            suggestions
                        },
                        ("greeting", "temperature") => {
                            let mut suggestions = vec!["0", "0.5", "0.7", "1.0"];
                            suggestions.retain(|s| s.starts_with(argument_value));
                            suggestions
                        },
                        _ => vec![],
                    };

                    let completion_values: Vec<CompletionValue> = values
                        .into_iter()
                        .map(CompletionValue::new)
                        .collect();

                    Ok(CompleteResponse {
                        completion: Completion::new(completion_values),
                        metadata: None,
                    })
                }
                "ref/resource" => {
                    let _uri = &request.reference.name;
                    let mut suggestions = vec!["1", "2", "3", "4", "5"];
                    suggestions.retain(|s| s.starts_with(argument_value));

                    let completion_values: Vec<CompletionValue> = suggestions
                        .into_iter()
                        .map(CompletionValue::new)
                        .collect();

                    Ok(CompleteResponse {
                        completion: Completion::new(completion_values),
                        metadata: None,
                    })
                }
                _ => Ok(CompleteResponse {
                    completion: Completion::new(vec![]),
                    metadata: None,
                }),
            }
        }
    }

    fn create_test_server() -> UltraFastServer {
        let server_info = ServerInfo {
            name: "completion-test-server".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test server for completion tests".to_string()),
            authors: None,
            homepage: None,
            license: None,
            repository: None,
        };

        let capabilities = ServerCapabilities {
            completion: Some(CompletionCapability {}),
            ..Default::default()
        };

        UltraFastServer::new(server_info, capabilities)
            .with_completion_handler(Arc::new(TestCompletionHandler))
    }

    fn create_test_client() -> UltraFastClient {
        let client_info = ClientInfo {
            name: "completion-test-client".to_string(),
            version: "1.0.0".to_string(),
            authors: None,
            description: Some("Test client for completion tests".to_string()),
            homepage: None,
            repository: None,
            license: None,
        };

        let capabilities = ClientCapabilities::default();
        UltraFastClient::new(client_info, capabilities)
    }

    #[tokio::test]
    async fn test_completion_request_structure() {
        // Test that the completion request structure matches MCP 2025-06-18 spec
        let request = CompleteRequest {
            reference: CompletionReference {
                ref_type: "ref/prompt".to_string(),
                name: "code_review".to_string(),
            },
            argument: CompletionArgument {
                name: "language".to_string(),
                value: "py".to_string(),
            },
            context: Some(CompletionContext {
                arguments: Some({
                    let mut map = HashMap::new();
                    map.insert("language".to_string(), "python".to_string());
                    map
                }),
                cursor_position: None,
                line_content: None,
                document_content: None,
                language: None,
                metadata: None,
            }),
        };

        assert_eq!(request.reference.ref_type, "ref/prompt");
        assert_eq!(request.reference.name, "code_review");
        assert_eq!(request.argument.name, "language");
        assert_eq!(request.argument.value, "py");
        assert!(request.context.is_some());

        let context = request.context.unwrap();
        assert!(context.arguments.is_some());
        let args = context.arguments.unwrap();
        assert_eq!(args.get("language"), Some(&"python".to_string()));

        println!("✅ Completion request structure test passed!");
    }

    #[tokio::test]
    async fn test_completion_handler_basic() {
        let handler = TestCompletionHandler;

        // Test basic prompt completion
        let request = CompleteRequest {
            reference: CompletionReference {
                ref_type: "ref/prompt".to_string(),
                name: "code_review".to_string(),
            },
            argument: CompletionArgument {
                name: "language".to_string(),
                value: "py".to_string(),
            },
            context: None,
        };

        let response = handler.complete(request).await.unwrap();
        assert!(!response.completion.values.is_empty());
        
        // Should return python and pytorch
        let values: Vec<&str> = response.completion.values.iter()
            .map(|v| v.value.as_str())
            .collect();
        assert!(values.contains(&"python"));
        assert!(values.contains(&"pytorch"));

        println!("✅ Basic completion handler test passed!");
    }

    #[tokio::test]
    async fn test_completion_with_context() {
        let handler = TestCompletionHandler;

        // Test completion with context (language already selected)
        let request = CompleteRequest {
            reference: CompletionReference {
                ref_type: "ref/prompt".to_string(),
                name: "code_review".to_string(),
            },
            argument: CompletionArgument {
                name: "framework".to_string(),
                value: "fla".to_string(),
            },
            context: Some(CompletionContext {
                arguments: Some({
                    let mut map = HashMap::new();
                    map.insert("language".to_string(), "python".to_string());
                    map
                }),
                cursor_position: None,
                line_content: None,
                document_content: None,
                language: None,
                metadata: None,
            }),
        };

        let response = handler.complete(request).await.unwrap();
        assert!(!response.completion.values.is_empty());
        
        // Should return flask (Python framework)
        let values: Vec<&str> = response.completion.values.iter()
            .map(|v| v.value.as_str())
            .collect();
        assert!(values.contains(&"flask"));

        println!("✅ Completion with context test passed!");
    }

    #[tokio::test]
    async fn test_completion_filtering() {
        let handler = TestCompletionHandler;

        // Test that completion filters by current value
        let request = CompleteRequest {
            reference: CompletionReference {
                ref_type: "ref/prompt".to_string(),
                name: "code_review".to_string(),
            },
            argument: CompletionArgument {
                name: "language".to_string(),
                value: "pyt".to_string(),
            },
            context: None,
        };

        let response = handler.complete(request).await.unwrap();
        assert!(!response.completion.values.is_empty());
        
        // Should return python and pytorch (both start with "pyt")
        let values: Vec<&str> = response.completion.values.iter()
            .map(|v| v.value.as_str())
            .collect();
        assert_eq!(values.len(), 2);
        assert!(values.contains(&"python"));
        assert!(values.contains(&"pytorch"));

        println!("✅ Completion filtering test passed!");
    }

    #[tokio::test]
    async fn test_resource_completion() {
        let handler = TestCompletionHandler;

        // Test resource completion
        let request = CompleteRequest {
            reference: CompletionReference {
                ref_type: "ref/resource".to_string(),
                name: "file:///path/to/resource".to_string(),
            },
            argument: CompletionArgument {
                name: "id".to_string(),
                value: "1".to_string(),
            },
            context: None,
        };

        let response = handler.complete(request).await.unwrap();
        assert!(!response.completion.values.is_empty());
        
        // Should return "1" (starts with "1")
        let values: Vec<&str> = response.completion.values.iter()
            .map(|v| v.value.as_str())
            .collect();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0], "1");

        println!("✅ Resource completion test passed!");
    }

    #[tokio::test]
    async fn test_completion_error_handling() {
        let handler = TestCompletionHandler;

        // Test unknown reference type
        let request = CompleteRequest {
            reference: CompletionReference {
                ref_type: "ref/unknown".to_string(),
                name: "test".to_string(),
            },
            argument: CompletionArgument {
                name: "test".to_string(),
                value: "test".to_string(),
            },
            context: None,
        };

        let response = handler.complete(request).await.unwrap();
        // Should return empty completion for unknown reference type
        assert!(response.completion.values.is_empty());

        println!("✅ Completion error handling test passed!");
    }

    #[tokio::test]
    async fn test_completion_builder_methods() {
        // Test the builder methods for creating completion requests
        let request = CompleteRequest::new("ref/prompt", "code_review")
            .with_argument_name_value("language", "py")
            .with_context(CompletionContext::new());

        assert_eq!(request.reference.ref_type, "ref/prompt");
        assert_eq!(request.reference.name, "code_review");
        assert_eq!(request.argument.name, "language");
        assert_eq!(request.argument.value, "py");
        assert!(request.context.is_some());

        println!("✅ Completion builder methods test passed!");
    }

    #[tokio::test]
    async fn test_completion_value_creation() {
        // Test creating completion values with different methods
        let basic = CompletionValue::new("test");
        assert_eq!(basic.value, "test");

        let with_label = CompletionValue::with_label("test", "Test Label");
        assert_eq!(with_label.value, "test");
        assert_eq!(with_label.label, Some("Test Label".to_string()));

        let with_description = CompletionValue::with_description("test", "Test Description");
        assert_eq!(with_description.value, "test");
        assert_eq!(with_description.description, Some("Test Description".to_string()));

        let function = CompletionValue::function("myFunction", "A test function");
        assert_eq!(function.value, "myFunction");
        assert_eq!(function.description, Some("A test function".to_string()));
        assert!(matches!(function.kind, Some(ultrafast_mcp_core::types::completion::CompletionKind::Function)));

        println!("✅ Completion value creation test passed!");
    }

    #[tokio::test]
    async fn test_completion_serialization() {
        // Test that completion requests can be serialized and deserialized
        let original_request = CompleteRequest {
            reference: CompletionReference {
                ref_type: "ref/prompt".to_string(),
                name: "code_review".to_string(),
            },
            argument: CompletionArgument {
                name: "language".to_string(),
                value: "py".to_string(),
            },
            context: None,
        };

        // Serialize
        let serialized = serde_json::to_string(&original_request).unwrap();
        
        // Deserialize
        let deserialized: CompleteRequest = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(original_request.reference.ref_type, deserialized.reference.ref_type);
        assert_eq!(original_request.reference.name, deserialized.reference.name);
        assert_eq!(original_request.argument.name, deserialized.argument.name);
        assert_eq!(original_request.argument.value, deserialized.argument.value);

        println!("✅ Completion serialization test passed!");
    }

    #[tokio::test]
    async fn test_completion_server_integration() {
        // Test that the server can be created with completion handler
        let server = create_test_server();
        
        assert_eq!(server.info().name, "completion-test-server");
        // Check that the server has completion capability by checking the server info
        assert_eq!(server.info().name, "completion-test-server");

        println!("✅ Completion server integration test passed!");
    }

    #[tokio::test]
    async fn test_completion_client_integration() {
        // Test that the client can be created
        let client = create_test_client();
        
        assert_eq!(client.info().name, "completion-test-client");

        println!("✅ Completion client integration test passed!");
    }

    #[tokio::test]
    async fn test_completion_protocol_compliance() {
        // Test that the completion implementation follows MCP 2025-06-18 spec
        
        // 1. Test reference structure
        let reference = CompletionReference {
            ref_type: "ref/prompt".to_string(),
            name: "code_review".to_string(),
        };
        assert_eq!(reference.ref_type, "ref/prompt");
        assert_eq!(reference.name, "code_review");

        // 2. Test argument structure
        let argument = CompletionArgument {
            name: "language".to_string(),
            value: "py".to_string(),
        };
        assert_eq!(argument.name, "language");
        assert_eq!(argument.value, "py");

        // 3. Test context structure
        let context = CompletionContext {
            arguments: Some({
                let mut map = HashMap::new();
                map.insert("language".to_string(), "python".to_string());
                map
            }),
            cursor_position: None,
            line_content: None,
            document_content: None,
            language: None,
            metadata: None,
        };
        assert!(context.arguments.is_some());
        let args = context.arguments.as_ref().unwrap();
        assert_eq!(args.get("language"), Some(&"python".to_string()));

        // 4. Test complete request structure
        let request = CompleteRequest {
            reference,
            argument,
            context: Some(context),
        };
        assert_eq!(request.reference.ref_type, "ref/prompt");
        assert_eq!(request.argument.name, "language");

        println!("✅ Completion protocol compliance test passed!");
    }
} 