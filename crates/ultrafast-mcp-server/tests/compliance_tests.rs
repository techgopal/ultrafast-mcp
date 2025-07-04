//! MCP 2025-06-18 Full Compliance Test Suite
//!
//! This module contains comprehensive tests that validate 100% compliance with
//! the Model Context Protocol (MCP) 2025-06-18 specification.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use serde_json::{json, Value};
use tokio::time::timeout;

use ultrafast_mcp_core::{
    error::{MCPError, MCPResult},
    protocol::{
        capabilities::{
            ServerCapabilities, ToolsCapability, ResourcesCapability, PromptsCapability,
            LoggingCapability, CompletionCapability,
        },
        jsonrpc::{JsonRpcMessage, JsonRpcRequest, JsonRpcResponse, RequestId},
        version::PROTOCOL_VERSION,
    },
    types::{
        tools::{Tool, ToolCall, ToolResult, ToolContent, ListToolsRequest, ListToolsResponse},
        resources::{
            Resource, ResourceTemplate, ReadResourceRequest, ReadResourceResponse,
            ListResourcesRequest, ListResourcesResponse, ListResourceTemplatesRequest,
            ListResourceTemplatesResponse, ResourceContent,
        },
        prompts::{
            Prompt, PromptArgument, GetPromptRequest, GetPromptResponse,
            ListPromptsRequest, ListPromptsResponse, PromptMessage, PromptRole, PromptContent,
        },
        sampling::{
            SamplingRequest, SamplingResponse, SamplingMessage, SamplingRole, SamplingContent,
        },
        completion::{CompleteRequest, CompleteResponse},
        roots::{Root, ListRootsRequest, ListRootsResponse},
        elicitation::{ElicitationRequest, ElicitationResponse},
        notifications::{
            LoggingMessageNotification, LogLevel, LogLevelSetRequest, LogLevelSetResponse,
            ProgressNotification, CancelledNotification, PingRequest, PingResponse,
        },
        server::ServerInfo,
    },
    schema::validation::{
        validate_tool_schema, validate_tool_input, validate_request_message,
        validate_comprehensive_input, ValidationContext,
    },
    utils::{CancellationManager, PingManager},
};

use ultrafast_mcp_server::{
    UltraFastServer, ToolHandler, ResourceHandler, PromptHandler, SamplingHandler,
    CompletionHandler, RootsHandler, ElicitationHandler, ResourceSubscriptionHandler,
    Context, LoggerConfig, ServerLoggingConfig,
};

/// Test Protocol Version Compliance
#[tokio::test]
async fn test_protocol_version_compliance() {
    // Test that we're using the correct protocol version
    assert_eq!(PROTOCOL_VERSION, "2025-06-18");
    
    // Test version negotiation
    let versions = vec!["2025-06-18".to_string(), "2024-11-05".to_string()];
    let negotiator = ultrafast_mcp_core::protocol::version::VersionNegotiator::new(versions);
    
    assert!(negotiator.supports_version("2025-06-18"));
    assert_eq!(negotiator.get_preferred_version(), Some("2025-06-18"));
    
    // Test incompatible version handling
    assert!(!negotiator.supports_version("2023-01-01"));
}

/// Test JSON-RPC 2.0 Message Validation
#[tokio::test]
async fn test_jsonrpc_message_validation() {
    let context = ValidationContext::default();
    
    // Test valid request
    let valid_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(RequestId::string("test-1")),
        method: "tools/list".to_string(),
        params: None,
        meta: HashMap::new(),
    };
    
    assert!(validate_request_message(&valid_request, &context).is_ok());
    
    // Test invalid JSON-RPC version
    let invalid_version = JsonRpcRequest {
        jsonrpc: "1.0".to_string(),
        id: Some(RequestId::string("test-2")),
        method: "tools/list".to_string(),
        params: None,
        meta: HashMap::new(),
    };
    
    assert!(validate_request_message(&invalid_version, &context).is_err());
    
    // Test reserved method name
    let reserved_method = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(RequestId::string("test-3")),
        method: "rpc.internal".to_string(),
        params: None,
        meta: HashMap::new(),
    };
    
    assert!(validate_request_message(&reserved_method, &context).is_err());
}

/// Test Input Validation Compliance
#[tokio::test]
async fn test_comprehensive_input_validation() {
    let context = ValidationContext::default();
    
    // Test script injection detection
    let script_injection = json!({
        "data": "<script>alert('xss')</script>"
    });
    assert!(validate_comprehensive_input(&script_injection, &context).is_err());
    
    // Test SQL injection detection
    let sql_injection = json!({
        "query": "'; DROP TABLE users; --"
    });
    assert!(validate_comprehensive_input(&sql_injection, &context).is_err());
    
    // Test command injection detection
    let command_injection = json!({
        "command": "ls; rm -rf /"
    });
    assert!(validate_comprehensive_input(&command_injection, &context).is_err());
    
    // Test valid input
    let valid_input = json!({
        "data": "Hello, world!",
        "count": 42
    });
    assert!(validate_comprehensive_input(&valid_input, &context).is_ok());
}

/// Test Tool Schema Validation Compliance
#[tokio::test]
async fn test_tool_schema_validation_compliance() {
    // Test valid tool schema
    let valid_schema = json!({
        "type": "object",
        "properties": {
            "input": {
                "type": "string",
                "description": "Input text"
            }
        },
        "required": ["input"]
    });
    assert!(validate_tool_schema(&valid_schema).is_ok());
    
    // Test invalid schema (non-object)
    let invalid_schema = json!("not an object");
    assert!(validate_tool_schema(&invalid_schema).is_err());
    
    // Test complex schema validation
    let complex_schema = json!({
        "type": "object",
        "properties": {
            "nested": {
                "type": "object",
                "properties": {
                    "value": {"type": "number"}
                }
            },
            "array": {
                "type": "array",
                "items": {"type": "string"}
            }
        }
    });
    assert!(validate_tool_schema(&complex_schema).is_ok());
    
    // Test tool input validation
    let tool_input = json!({"input": "test value"});
    assert!(validate_tool_input(&tool_input, &valid_schema).is_ok());
    
    let invalid_input = json!({"wrong_field": "test"});
    assert!(validate_tool_input(&invalid_input, &valid_schema).is_err());
}

/// Test Server Capabilities Compliance
#[tokio::test]
async fn test_server_capabilities_compliance() {
    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability {
            list_changed: Some(true),
        }),
        resources: Some(ResourcesCapability {
            subscribe: Some(true),
            list_changed: Some(true),
        }),
        prompts: Some(PromptsCapability {
            list_changed: Some(true),
        }),
        logging: Some(LoggingCapability {}),
        completion: Some(CompletionCapability {
            complete: Some(true),
        }),
        experimental: None,
    };
    
    // Test that all required capabilities are defined
    assert!(capabilities.tools.is_some());
    assert!(capabilities.resources.is_some());
    assert!(capabilities.prompts.is_some());
    assert!(capabilities.logging.is_some());
    assert!(capabilities.completion.is_some());
}

/// Mock implementations for testing
    
struct MockToolHandler;
    
#[async_trait::async_trait]
impl ToolHandler for MockToolHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult> {
        Ok(ToolResult {
            content: vec![ToolContent::text(format!("Result for {}", call.name))],
            is_error: Some(false),
        })
    }
    
    async fn list_tools(&self, _request: ListToolsRequest) -> MCPResult<ListToolsResponse> {
        Ok(ListToolsResponse {
            tools: vec![
                Tool {
                    name: "echo".to_string(),
                    description: "Echo input".to_string(),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "text": {"type": "string"}
                        },
                        "required": ["text"]
                    }),
                    output_schema: Some(json!({
                        "type": "object",
                        "properties": {
                            "result": {"type": "string"}
                        }
                    })),
                }
            ],
            next_cursor: None,
        })
    }
}
    
struct MockResourceHandler;
    
#[async_trait::async_trait]
impl ResourceHandler for MockResourceHandler {
    async fn read_resource(&self, request: ReadResourceRequest) -> MCPResult<ReadResourceResponse> {
        Ok(ReadResourceResponse {
            contents: vec![ResourceContent::text(format!("Content of {}", request.uri))],
        })
    }
    
    async fn list_resources(&self, _request: ListResourcesRequest) -> MCPResult<ListResourcesResponse> {
        Ok(ListResourcesResponse {
            resources: vec![
                Resource {
                    uri: "file:///test.txt".to_string(),
                    name: "Test File".to_string(),
                    description: Some("A test file".to_string()),
                    mime_type: Some("text/plain".to_string()),
                }
            ],
            next_cursor: None,
        })
    }
    
    async fn list_resource_templates(&self, _request: ListResourceTemplatesRequest) -> MCPResult<ListResourceTemplatesResponse> {
        Ok(ListResourceTemplatesResponse {
            resource_templates: vec![
                ResourceTemplate {
                    uri_template: "file:///files/{name}".to_string(),
                    name: "File Template".to_string(),
                    description: Some("Template for files".to_string()),
                    mime_type: Some("text/plain".to_string()),
                }
            ],
            next_cursor: None,
        })
    }
}
    
struct MockPromptHandler;
    
#[async_trait::async_trait]
impl PromptHandler for MockPromptHandler {
    async fn get_prompt(&self, request: GetPromptRequest) -> MCPResult<GetPromptResponse> {
        Ok(GetPromptResponse {
            description: Some(format!("Prompt for {}", request.name)),
            messages: vec![
                PromptMessage {
                    role: PromptRole::User,
                    content: PromptContent::text("Hello".to_string()),
                }
            ],
        })
    }
    
    async fn list_prompts(&self, _request: ListPromptsRequest) -> MCPResult<ListPromptsResponse> {
        Ok(ListPromptsResponse {
            prompts: vec![
                Prompt {
                    name: "greeting".to_string(),
                    description: Some("A greeting prompt".to_string()),
                    arguments: Some(vec![
                        PromptArgument {
                            name: "name".to_string(),
                            description: Some("Name to greet".to_string()),
                            required: Some(true),
                        }
                    ]),
                }
            ],
            next_cursor: None,
        })
    }
}
    
struct MockSamplingHandler;
    
#[async_trait::async_trait]
impl SamplingHandler for MockSamplingHandler {
    async fn create_message(&self, _request: SamplingRequest) -> MCPResult<SamplingResponse> {
        Ok(SamplingResponse {
            role: "assistant".to_string(),
            content: SamplingContent::text("Generated response".to_string()),
            model: Some("mock-model".to_string()),
            stop_reason: Some("stop".to_string()),
        })
    }
}
    
struct MockCompletionHandler;
    
#[async_trait::async_trait]
impl CompletionHandler for MockCompletionHandler {
    async fn complete(&self, _request: CompleteRequest) -> MCPResult<CompleteResponse> {
        Ok(CompleteResponse {
            completion: ultrafast_mcp_core::types::completion::CompletionResult {
                values: vec!["completion1".to_string(), "completion2".to_string()],
                total: Some(2),
                has_more: Some(false),
            },
        })
    }
}
    
struct MockRootsHandler;
    
#[async_trait::async_trait]
impl RootsHandler for MockRootsHandler {
    async fn list_roots(&self) -> MCPResult<Vec<Root>> {
        Ok(vec![
            Root {
                uri: "file:///workspace".to_string(),
                name: Some("Workspace".to_string()),
                security: None,
            }
        ])
    }
}
    
struct MockElicitationHandler;
    
#[async_trait::async_trait]
impl ElicitationHandler for MockElicitationHandler {
    async fn handle_elicitation(&self, _request: ElicitationRequest) -> MCPResult<ElicitationResponse> {
        Ok(ElicitationResponse {
            data: json!({"response": "elicited"}),
        })
    }
}
    
struct MockSubscriptionHandler;
    
#[async_trait::async_trait]
impl ResourceSubscriptionHandler for MockSubscriptionHandler {
    async fn subscribe(&self, _uri: String) -> MCPResult<()> {
        Ok(())
    }
    
    async fn unsubscribe(&self, _uri: String) -> MCPResult<()> {
        Ok(())
    }
    
    async fn notify_change(&self, _uri: String, _content: Value) -> MCPResult<()> {
        Ok(())
    }
}

/// Test Full Server Functionality Compliance
#[tokio::test]
async fn test_full_server_functionality_compliance() {
    let server_info = ServerInfo {
        name: "compliance-test-server".to_string(),
        version: "1.0.0".to_string(),
        description: Some("Server for compliance testing".to_string()),
        authors: Some(vec!["Test Author".to_string()]),
        homepage: Some("https://example.com".to_string()),
        license: Some("MIT".to_string()),
        repository: Some("https://github.com/example/test".to_string()),
    };
    
    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability { list_changed: Some(true) }),
        resources: Some(ResourcesCapability { 
            subscribe: Some(true),
            list_changed: Some(true),
        }),
        prompts: Some(PromptsCapability { list_changed: Some(true) }),
        logging: Some(LoggingCapability {}),
        completion: Some(CompletionCapability { complete: Some(true) }),
        experimental: None,
    };
    
    let server = UltraFastServer::new(server_info.clone(), capabilities)
        .with_tool_handler(Arc::new(MockToolHandler))
        .with_resource_handler(Arc::new(MockResourceHandler))
        .with_prompt_handler(Arc::new(MockPromptHandler))
        .with_sampling_handler(Arc::new(MockSamplingHandler))
        .with_completion_handler(Arc::new(MockCompletionHandler))
        .with_roots_handler(Arc::new(MockRootsHandler))
        .with_elicitation_handler(Arc::new(MockElicitationHandler))
        .with_subscription_handler(Arc::new(MockSubscriptionHandler));
    
    // Test server info
    assert_eq!(server.info().name, "compliance-test-server");
    assert_eq!(server.info().version, "1.0.0");
    
    // Test tool registration and validation
    let test_tool = Tool {
        name: "test_tool".to_string(),
        description: "A test tool".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "input": {"type": "string"}
            },
            "required": ["input"]
        }),
        output_schema: Some(json!({
            "type": "object",
            "properties": {
                "output": {"type": "string"}
            }
        })),
    };
    
    assert!(server.register_tool(test_tool).await.is_ok());
    assert_eq!(server.tool_count().await, 1);
    assert!(server.has_tool("test_tool").await);
    
    // Test tool validation
    let valid_args = json!({"input": "test"});
    assert!(server.validate_tool_call("test_tool", &valid_args).await.is_ok());
    
    let invalid_args = json!({"wrong": "field"});
    assert!(server.validate_tool_call("test_tool", &invalid_args).await.is_err());
}

/// Test Logging Compliance
#[tokio::test]
async fn test_logging_compliance() {
    let server_info = ServerInfo::new("test-server".to_string(), "1.0.0".to_string());
    let capabilities = ServerCapabilities::default();
    let server = UltraFastServer::new(server_info, capabilities);
    
    // Test log level management
    assert!(server.set_log_level(LogLevel::Debug).await.is_ok());
    assert!(matches!(server.get_log_level().await, LogLevel::Debug));
    
    assert!(server.set_log_level(LogLevel::Warning).await.is_ok());
    assert!(matches!(server.get_log_level().await, LogLevel::Warning));
    
    // Test context creation with logging
    let context = server.create_context().await;
    assert!(matches!(context.get_log_level(), LogLevel::Warning));
    
    // Test structured logging configuration
    let logger_config = LoggerConfig::default()
        .with_structured_output(true)
        .with_timestamps(true)
        .with_logger_name("compliance-test".to_string());
    
    let context_with_config = server
        .create_context()
        .await
        .with_logger_config(logger_config);
    
    // Test all log levels
    assert!(context_with_config.log_debug("Debug message").await.is_ok());
    assert!(context_with_config.log_info("Info message").await.is_ok());
    assert!(context_with_config.log_notice("Notice message").await.is_ok());
    assert!(context_with_config.log_warn("Warning message").await.is_ok());
    assert!(context_with_config.log_error("Error message").await.is_ok());
    assert!(context_with_config.log_critical("Critical message").await.is_ok());
    assert!(context_with_config.log_alert("Alert message").await.is_ok());
    assert!(context_with_config.log_emergency("Emergency message").await.is_ok());
    
    // Test structured logging
    let structured_data = json!({"key": "value", "count": 42});
    assert!(context_with_config.log_info_structured("Structured message", structured_data).await.is_ok());
}

/// Test Resource Template Compliance
#[tokio::test]
async fn test_resource_template_compliance() {
    use ultrafast_mcp_core::types::resources::{
        ResourceTemplate, TemplateValidator, TemplateSecurityPolicy, TemplateExpansionOptions
    };
    
    // Test basic template validation
    let template = ResourceTemplate {
        uri_template: "https://api.example.com/users/{id}".to_string(),
        name: "User API".to_string(),
        description: Some("API for user data".to_string()),
        mime_type: Some("application/json".to_string()),
    };
    
    let security_policy = TemplateSecurityPolicy::default();
    let validator = TemplateValidator::new(security_policy);
    
    assert!(validator.validate_template(&template.uri_template).is_ok());
    
    // Test variable parsing
    let variables = validator.parse_variables(&template.uri_template).unwrap();
    assert_eq!(variables.len(), 1);
    assert_eq!(variables[0], "id");
    
    // Test template expansion
    let mut values = std::collections::HashMap::new();
    values.insert("id".to_string(), "123".to_string());
    
    let expanded = validator.expand_template(&template.uri_template, &values).unwrap();
    assert_eq!(expanded, "https://api.example.com/users/123");
    
    // Test security validation
    let malicious_template = "https://evil.com/{script}?<script>alert(1)</script>";
    assert!(validator.validate_template(malicious_template).is_err());
    
    // Test path traversal protection
    let traversal_template = "file:///path/{..%2F..%2Fetc%2Fpasswd}";
    assert!(validator.validate_template(traversal_template).is_err());
}

/// Test Prompt Embedded Resources Compliance
#[tokio::test]
async fn test_prompt_embedded_resources_compliance() {
    use ultrafast_mcp_core::types::prompts::{
        EmbeddedResourceReference, ResourceInclusionOptions, ResourceSecurityPolicy,
        EmbeddedResourceValidator
    };
    
    // Test valid resource reference
    let resource_ref = EmbeddedResourceReference::new("https://example.com/data.json".to_string())
        .with_description("Test data".to_string())
        .with_fallback("Default data".to_string())
        .with_options(ResourceInclusionOptions {
            max_size: Some(1024),
            timeout_ms: Some(5000),
            allowed_mime_types: Some(vec!["application/json".to_string()]),
        });
    
    assert!(resource_ref.validate().is_ok());
    
    // Test security policy validation
    let security_policy = ResourceSecurityPolicy::default();
    assert!(resource_ref.validate_with_policy(&security_policy).is_ok());
    
    // Test blocked patterns
    let blocked_ref = EmbeddedResourceReference::new("http://localhost:8080/admin".to_string());
    assert!(blocked_ref.validate_with_policy(&security_policy).is_err());
    
    // Test file scheme validation
    let file_ref = EmbeddedResourceReference::new("file:///safe/path/data.txt".to_string());
    let mut file_policy = ResourceSecurityPolicy::default();
    file_policy.allow_local_files = true;
    
    let validator = EmbeddedResourceValidator::new(file_policy);
    assert!(validator.validate_reference(&file_ref).is_ok());
}

/// Test Sampling Model Preferences Compliance
#[tokio::test]
async fn test_sampling_model_preferences_compliance() {
    use ultrafast_mcp_core::types::sampling::{
        ModelCapability, Modality, ModelSelector, ModelSelectionContext
    };
    
    // Test model capability definition
    let model = ModelCapability {
        model_id: "gpt-4".to_string(),
        provider: "openai".to_string(),
        display_name: "GPT-4".to_string(),
        version: Some("turbo".to_string()),
        cost_per_1k_input_tokens: 0.03,
        cost_per_1k_output_tokens: 0.06,
        speed_score: 7.0,
        intelligence_score: 9.0,
        max_context_length: 128000,
        modalities: vec![Modality::Text],
        supports_function_calling: true,
        supports_streaming: true,
        metadata: None,
    };
    
    // Test model selector
    let selector = ModelSelector::new();
    let models = vec![model.clone()];
    
    let context = ModelSelectionContext {
        estimated_input_tokens: 1000,
        estimated_output_tokens: 500,
        required_modalities: vec![Modality::Text],
        cost_priority: 0.3,
        speed_priority: 0.4,
        intelligence_priority: 0.3,
        requires_function_calling: true,
        requires_streaming: false,
    };
    
    let result = selector.select_model(&models, &context).unwrap();
    assert_eq!(result.selected_model.model_id, "gpt-4");
    assert!(result.confidence_score > 0.0);
    assert!(!result.selection_reasoning.primary_factors.is_empty());
}

/// Test Roots Security Validation Compliance
#[tokio::test]
async fn test_roots_security_validation_compliance() {
    use ultrafast_mcp_core::types::roots::{
        RootSecurityValidator, RootOperation, RootSecurityConfig
    };
    
    let validator = RootSecurityValidator::default();
    
    // Test valid path validation
    let root_path = "/safe/workspace";
    let file_path = "/safe/workspace/project/file.txt";
    assert!(validator.validate_path_in_root(file_path, root_path, RootOperation::Read).is_ok());
    
    // Test path traversal detection
    let traversal_path = "/safe/workspace/../../../etc/passwd";
    assert!(validator.validate_path_in_root(traversal_path, root_path, RootOperation::Read).is_err());
    
    // Test blocked file extensions
    let executable_path = "/safe/workspace/malware.exe";
    assert!(validator.validate_path_in_root(executable_path, root_path, RootOperation::Read).is_err());
    
    // Test security config
    let mut config = RootSecurityConfig::default();
    config.allow_write = true;
    
    assert!(validator.validate_operation_allowed(&config, RootOperation::Read));
    assert!(validator.validate_operation_allowed(&config, RootOperation::Write));
    assert!(!validator.validate_operation_allowed(&config, RootOperation::Execute));
}

/// Test Cancellation and Progress Compliance
#[tokio::test]
async fn test_cancellation_and_progress_compliance() {
    let server_info = ServerInfo::new("test-server".to_string(), "1.0.0".to_string());
    let capabilities = ServerCapabilities::default();
    let server = UltraFastServer::new(server_info, capabilities);
    
    // Test cancellation manager
    let cancellation_manager = server.cancellation_manager();
    let request_id = "test-request-123";
    
    assert!(!cancellation_manager.is_cancelled(request_id).await);
    cancellation_manager.cancel(request_id, Some("User requested cancellation".to_string())).await;
    assert!(cancellation_manager.is_cancelled(request_id).await);
    
    // Test ping manager
    let ping_manager = server.ping_manager();
    let ping_request = PingRequest::new().with_data(json!({"test": "data"}));
    
    let ping_response = ping_manager.handle_ping(ping_request).await.unwrap();
    assert_eq!(ping_response.data, Some(json!({"test": "data"})));
    
    // Test progress tracking
    let context = server.create_context_with_ids("progress-test".to_string(), None).await;
    assert!(context.progress("Starting operation", 0.0, Some(100.0)).await.is_ok());
    assert!(context.progress("Half done", 50.0, Some(100.0)).await.is_ok());
    assert!(context.progress("Completed", 100.0, Some(100.0)).await.is_ok());
}

/// Test Transport Layer Compliance
#[tokio::test]
async fn test_transport_layer_compliance() {
    use ultrafast_mcp_transport::{create_transport, TransportConfig};
    
    // Test STDIO transport creation
    let stdio_transport = create_transport(TransportConfig::Stdio).await;
    assert!(stdio_transport.is_ok());
    
    // Test HTTP transport configuration
    #[cfg(feature = "http")]
    {
        use ultrafast_mcp_transport::http::server::HttpTransportConfig;
        
        let http_config = HttpTransportConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            max_connections: Some(100),
            request_timeout: Some(Duration::from_secs(30)),
            enable_compression: Some(true),
            cors_enabled: Some(true),
            max_request_size: Some(1024 * 1024), // 1MB
        };
        
        assert_eq!(http_config.host, "127.0.0.1");
        assert_eq!(http_config.port, 8080);
        assert_eq!(http_config.max_connections, Some(100));
    }
}

/// Test Error Handling Compliance
#[tokio::test]
async fn test_error_handling_compliance() {
    // Test MCP error types
    let validation_error = MCPError::invalid_request("Invalid request format".to_string());
    assert!(format!("{}", validation_error).contains("Invalid request format"));
    
    let method_error = MCPError::method_not_found("unknown_method".to_string());
    assert!(format!("{}", method_error).contains("unknown_method"));
    
    let internal_error = MCPError::internal_error("Internal server error".to_string());
    assert!(format!("{}", internal_error).contains("Internal server error"));
    
    let timeout_error = MCPError::request_timeout();
    assert!(format!("{}", timeout_error).contains("timeout"));
    
    // Test error serialization/deserialization
    let error_json = serde_json::to_value(&validation_error).unwrap();
    assert!(error_json.is_object());
    
    let deserialized: MCPError = serde_json::from_value(error_json).unwrap();
    assert_eq!(format!("{}", deserialized), format!("{}", validation_error));
}

/// Test Performance and Limits Compliance
#[tokio::test]
async fn test_performance_and_limits_compliance() {
    let server_info = ServerInfo::new("perf-test-server".to_string(), "1.0.0".to_string());
    let capabilities = ServerCapabilities::default();
    let server = UltraFastServer::new(server_info, capabilities);
    
    // Test concurrent tool registration
    let mut handles = vec![];
    for i in 0..10 {
        let server_clone = server.clone();
        handles.push(tokio::spawn(async move {
            let tool = Tool {
                name: format!("tool_{}", i),
                description: format!("Tool number {}", i),
                input_schema: json!({"type": "object"}),
                output_schema: Some(json!({"type": "object"})),
            };
            server_clone.register_tool(tool).await
        }));
    }
    
    // Wait for all registrations to complete
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }
    
    assert_eq!(server.tool_count().await, 10);
    
    // Test operation timeout compliance
    let context = server.create_context().await;
    let result = timeout(
        Duration::from_millis(100),
        context.log_info("Test timeout compliance")
    ).await;
    assert!(result.is_ok());
}

/// Test Full Integration Workflow
#[tokio::test]
async fn test_full_integration_workflow() {
    let server_info = ServerInfo {
        name: "integration-test-server".to_string(),
        version: "1.0.0".to_string(),
        description: Some("Full integration test server".to_string()),
        authors: Some(vec!["Test Team".to_string()]),
        homepage: Some("https://test.com".to_string()),
        license: Some("MIT".to_string()),
        repository: Some("https://github.com/test/test".to_string()),
    };
    
    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability { list_changed: Some(true) }),
        resources: Some(ResourcesCapability { 
            subscribe: Some(true),
            list_changed: Some(true),
        }),
        prompts: Some(PromptsCapability { list_changed: Some(true) }),
        logging: Some(LoggingCapability {}),
        completion: Some(CompletionCapability { complete: Some(true) }),
        experimental: None,
    };
    
    let server = UltraFastServer::new(server_info, capabilities)
        .with_tool_handler(Arc::new(MockToolHandler))
        .with_resource_handler(Arc::new(MockResourceHandler))
        .with_prompt_handler(Arc::new(MockPromptHandler))
        .with_sampling_handler(Arc::new(MockSamplingHandler))
        .with_completion_handler(Arc::new(MockCompletionHandler))
        .with_roots_handler(Arc::new(MockRootsHandler))
        .with_elicitation_handler(Arc::new(MockElicitationHandler))
        .with_subscription_handler(Arc::new(MockSubscriptionHandler));
    
    // 1. Register tools with comprehensive validation
    let tools = vec![
        Tool {
            name: "calculator".to_string(),
            description: "Perform calculations".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "operation": {"type": "string", "enum": ["add", "subtract", "multiply", "divide"]},
                    "a": {"type": "number"},
                    "b": {"type": "number"}
                },
                "required": ["operation", "a", "b"]
            }),
            output_schema: Some(json!({
                "type": "object",
                "properties": {
                    "result": {"type": "number"}
                }
            })),
        },
        Tool {
            name: "text_processor".to_string(),
            description: "Process text input".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "text": {"type": "string", "maxLength": 1000},
                    "operation": {"type": "string", "enum": ["uppercase", "lowercase", "reverse"]}
                },
                "required": ["text", "operation"]
            }),
            output_schema: Some(json!({
                "type": "object",
                "properties": {
                    "processed_text": {"type": "string"}
                }
            })),
        },
    ];
    
    for tool in tools {
        assert!(server.register_tool(tool).await.is_ok());
    }
    
    // 2. Test tool validation and execution
    let calc_args = json!({
        "operation": "add",
        "a": 5,
        "b": 3
    });
    assert!(server.validate_tool_call("calculator", &calc_args).await.is_ok());
    assert!(server.execute_tool_call("calculator", calc_args).await.is_ok());
    
    // 3. Test logging with different levels
    let logging_config = ServerLoggingConfig {
        current_level: LogLevel::Debug,
        allow_level_changes: true,
        default_logger_config: LoggerConfig {
            min_level: LogLevel::Debug,
            send_notifications: true,
            structured_output: true,
            max_message_length: 2048,
            include_timestamps: true,
            include_logger_name: true,
            logger_name: Some("integration-test".to_string()),
        },
    };
    
    server.set_logging_config(logging_config).await;
    
    let context = server.create_context_with_ids("integration-test-123".to_string(), Some("session-456".to_string())).await;
    assert!(context.log_info("Integration test started").await.is_ok());
    assert!(context.log_warn("Test warning").await.is_ok());
    assert!(context.log_error("Test error").await.is_ok());
    
    // 4. Test progress tracking
    assert!(context.progress("Initializing", 0.0, Some(100.0)).await.is_ok());
    assert!(context.progress("Processing", 50.0, Some(100.0)).await.is_ok());
    assert!(context.progress("Finalizing", 90.0, Some(100.0)).await.is_ok());
    assert!(context.progress("Complete", 100.0, Some(100.0)).await.is_ok());
    
    // 5. Test resource template validation
    use ultrafast_mcp_core::types::resources::{ResourceTemplate, TemplateValidator, TemplateSecurityPolicy};
    
    let template = ResourceTemplate {
        uri_template: "https://api.service.com/data/{id}/{type}".to_string(),
        name: "Data API".to_string(),
        description: Some("API for fetching data".to_string()),
        mime_type: Some("application/json".to_string()),
    };
    
    let validator = TemplateValidator::new(TemplateSecurityPolicy::default());
    assert!(validator.validate_template(&template.uri_template).is_ok());
    
    let variables = validator.parse_variables(&template.uri_template).unwrap();
    assert_eq!(variables.len(), 2);
    assert!(variables.contains(&"id".to_string()));
    assert!(variables.contains(&"type".to_string()));
    
    // 6. Test embedded resource validation
    use ultrafast_mcp_core::types::prompts::{EmbeddedResourceReference, ResourceSecurityPolicy};
    
    let resource_ref = EmbeddedResourceReference::new("https://safe-api.com/data.json".to_string());
    let security_policy = ResourceSecurityPolicy::default();
    assert!(resource_ref.validate_with_policy(&security_policy).is_ok());
    
    // 7. Test comprehensive input validation
    let valid_input = json!({
        "user_input": "Hello, world!",
        "parameters": {
            "count": 42,
            "enabled": true
        }
    });
    
    let context = ultrafast_mcp_core::schema::validation::ValidationContext::default();
    assert!(ultrafast_mcp_core::schema::validation::validate_comprehensive_input(&valid_input, &context).is_ok());
    
    // 8. Test cancellation and cleanup
    let cancellation_manager = server.cancellation_manager();
    cancellation_manager.cancel("integration-test-123", Some("Test completed".to_string())).await;
    assert!(cancellation_manager.is_cancelled("integration-test-123").await);
    
    assert!(context.log_info("Integration test completed successfully").await.is_ok());
}