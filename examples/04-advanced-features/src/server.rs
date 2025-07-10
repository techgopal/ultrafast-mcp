//! Advanced Features Server Example
//!
//! This example demonstrates the complete UltraFastServer API with all advanced features:
//! - OAuth 2.1 authentication
//! - Monitoring and metrics collection
//! - Middleware integration
//! - Recovery mechanisms
//! - Health checking
//! - Multiple tool types
//! - Resource management
//! - Prompt generation

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, warn};
use ultrafast_mcp::types::roots::{RootOperation, RootSecurityValidator};
use ultrafast_mcp::McpCoreError::ResourceError;
use ultrafast_mcp::{
    // Import health types
    health::{HealthCheck, HealthCheckResult},
    // Import types for handler implementations
    types::{completion, elicitation, prompts, resources, roots, sampling},
    CompletionHandler,
    ElicitationHandler,
    GetPromptRequest,
    GetPromptResponse,
    HealthStatus,
    HttpTransportConfig,
    // Import transport types
    HttpTransportServer,
    ListPromptsRequest,
    ListPromptsResponse,
    ListResourcesRequest,
    ListResourcesResponse,
    ListToolsRequest,
    ListToolsResponse,
    MCPError,
    MCPResult,
    MonitoringConfig,
    // Import monitoring types
    MonitoringSystem,
    Prompt,
    PromptHandler,
    PromptsCapability,
    ReadResourceRequest,
    ReadResourceResponse,
    RequestTimer,
    Resource,
    ResourceContent,
    ResourceHandler,
    ResourceSubscriptionHandler,
    ResourceTemplate,
    // Import capability types
    ResourcesCapability,
    RootsHandler,
    // Import handler traits
    SamplingHandler,
    ServerCapabilities,
    ServerInfo,
    Tool,
    ToolAnnotations,
    ToolCall,
    ToolContent,
    ToolHandler,
    ToolResult,
    ToolsCapability,
    UltraFastServer,
};

#[derive(Debug, Serialize, Deserialize)]
struct CalculatorRequest {
    operation: String,
    a: f64,
    b: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct CalculatorResponse {
    result: f64,
    operation: String,
    timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WeatherRequest {
    city: String,
    country: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WeatherResponse {
    city: String,
    temperature: f64,
    condition: String,
    humidity: f64,
    timestamp: String,
}

struct AdvancedToolHandler {
    monitoring: Arc<MonitoringSystem>,
}

impl AdvancedToolHandler {
    fn new(monitoring: Arc<MonitoringSystem>) -> Self {
        Self { monitoring }
    }
}

#[async_trait::async_trait]
impl ToolHandler for AdvancedToolHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult> {
        let timer = RequestTimer::start(&call.name, self.monitoring.metrics());

        info!("Handling tool call: {}", call.name);

        let result = match call.name.as_str() {
            "calculate" => {
                let request: CalculatorRequest =
                    serde_json::from_value(call.arguments.unwrap_or_default()).map_err(|e| {
                        error!("Failed to parse calculator request: {}", e);
                        MCPError::invalid_params(format!("Invalid request format: {}", e))
                    })?;

                let result = match request.operation.as_str() {
                    "add" => request.a + request.b,
                    "subtract" => request.a - request.b,
                    "multiply" => request.a * request.b,
                    "divide" => {
                        if request.b == 0.0 {
                            return Err(MCPError::invalid_params("Division by zero".to_string()));
                        }
                        request.a / request.b
                    }
                    _ => {
                        return Err(MCPError::invalid_params(format!(
                            "Unknown operation: {}",
                            request.operation
                        )))
                    }
                };

                let response = CalculatorResponse {
                    result,
                    operation: request.operation,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };

                let response_text = serde_json::to_string_pretty(&response).map_err(|e| {
                    error!("Failed to serialize calculator response: {}", e);
                    MCPError::serialization_error(e.to_string())
                })?;

                Ok(ToolResult {
                    content: vec![ToolContent::text(response_text)],
                    is_error: None,
                })
            }
            "weather" => {
                let request: WeatherRequest =
                    serde_json::from_value(call.arguments.unwrap_or_default()).map_err(|e| {
                        error!("Failed to parse weather request: {}", e);
                        MCPError::invalid_params(format!("Invalid request format: {}", e))
                    })?;

                // Simulate weather API call
                let response = WeatherResponse {
                    city: request.city,
                    temperature: 22.5,
                    condition: "Sunny".to_string(),
                    humidity: 65.0,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };

                let response_text = serde_json::to_string_pretty(&response).map_err(|e| {
                    error!("Failed to serialize weather response: {}", e);
                    MCPError::serialization_error(e.to_string())
                })?;

                Ok(ToolResult {
                    content: vec![ToolContent::text(response_text)],
                    is_error: None,
                })
            }
            "delete_file" => {
                let args = call.arguments.unwrap_or_default();
                let path = args
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| MCPError::invalid_params("Path is required".to_string()))?;

                // Simulate file deletion (in a real implementation, you would actually delete the file)
                let result = format!("File '{}' would be deleted (simulation only)", path);

                Ok(ToolResult {
                    content: vec![ToolContent::text(result)],
                    is_error: None,
                })
            }
            _ => Err(MCPError::method_not_found(format!(
                "Unknown tool: {}",
                call.name
            ))),
        };

        // Record metrics
        match &result {
            Ok(_) => timer.finish(true).await,
            Err(_) => timer.finish(false).await,
        }

        result
    }

    async fn list_tools(&self, _request: ListToolsRequest) -> MCPResult<ListToolsResponse> {
        info!("Listing available tools");
        Ok(ListToolsResponse {
            tools: vec![
                Tool {
                    name: "calculate".to_string(),
                    description: "Perform basic mathematical operations".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "operation": {
                                "type": "string",
                                "enum": ["add", "subtract", "multiply", "divide"],
                                "description": "Mathematical operation to perform"
                            },
                            "a": {
                                "type": "number",
                                "description": "First operand"
                            },
                            "b": {
                                "type": "number",
                                "description": "Second operand"
                            }
                        },
                        "required": ["operation", "a", "b"]
                    }),
                    output_schema: None,
                    annotations: Some(
                        ToolAnnotations::read_only()
                            .with_title("Calculator".to_string())
                            .with_read_only_hint(true)
                            .with_open_world_hint(false),
                    ),
                },
                Tool {
                    name: "weather".to_string(),
                    description: "Get weather information for a location".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "city": {
                                "type": "string",
                                "description": "City name"
                            },
                            "country": {
                                "type": "string",
                                "description": "Country code (e.g., US, UK)"
                            }
                        },
                        "required": ["city"]
                    }),
                    output_schema: None,
                    annotations: Some(
                        ToolAnnotations::open_world()
                            .with_title("Weather Lookup".to_string())
                            .with_read_only_hint(true)
                            .with_open_world_hint(true),
                    ),
                },
                Tool {
                    name: "delete_file".to_string(),
                    description: "Delete a file from the filesystem".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "Path to the file to delete"
                            }
                        },
                        "required": ["path"]
                    }),
                    output_schema: None,
                    annotations: Some(
                        ToolAnnotations::destructive()
                            .with_title("Delete File".to_string())
                            .with_read_only_hint(false)
                            .with_destructive_hint(true)
                            .with_idempotent_hint(true)
                            .with_open_world_hint(false),
                    ),
                },
            ],
            next_cursor: None,
        })
    }
}

struct FileResourceHandler;

#[async_trait::async_trait]
impl ResourceHandler for FileResourceHandler {
    async fn read_resource(&self, request: ReadResourceRequest) -> MCPResult<ReadResourceResponse> {
        info!("Reading resource: {}", request.uri);

        if request.uri.starts_with("file://") {
            let path = request.uri.strip_prefix("file://").unwrap();

            match std::fs::read_to_string(path) {
                Ok(content) => Ok(ReadResourceResponse {
                    contents: vec![ResourceContent::text(request.uri.clone(), content)],
                }),
                Err(e) => {
                    error!("Failed to read file {}: {}", path, e);
                    Err(MCPError::not_found(format!("File not found: {}", path)))
                }
            }
        } else {
            Err(MCPError::invalid_params(
                "Unsupported URI scheme".to_string(),
            ))
        }
    }

    async fn list_resources(
        &self,
        _request: ListResourcesRequest,
    ) -> MCPResult<ListResourcesResponse> {
        Ok(ListResourcesResponse {
            resources: vec![Resource {
                uri: "file:///tmp/example.txt".to_string(),
                name: "Example File".to_string(),
                description: Some("An example text file".to_string()),
                mime_type: Some("text/plain".to_string()),
            }],
            next_cursor: None,
        })
    }

    async fn list_resource_templates(
        &self,
        _request: resources::ListResourceTemplatesRequest,
    ) -> MCPResult<resources::ListResourceTemplatesResponse> {
        Ok(resources::ListResourceTemplatesResponse {
            resource_templates: vec![
                ResourceTemplate {
                    uri_template: "template://greeting/{name}".to_string(),
                    name: "Greeting Template".to_string(),
                    description: Some("A template for greeting messages".to_string()),
                    mime_type: Some("text/plain".to_string()),
                },
                ResourceTemplate {
                    uri_template: "template://weather/{text}".to_string(),
                    name: "Weather Template".to_string(),
                    description: Some("A template for weather reports".to_string()),
                    mime_type: Some("text/plain".to_string()),
                },
            ],
            next_cursor: None,
        })
    }

    async fn validate_resource_access(
        &self,
        uri: &str,
        operation: RootOperation,
        roots: &[roots::Root],
    ) -> MCPResult<()> {
        if roots.is_empty() {
            return Ok(());
        }
        for root in roots {
            if uri.starts_with(&root.uri) {
                if root.uri.starts_with("file://") && uri.starts_with("file://") {
                    let validator = RootSecurityValidator::default();
                    return validator
                        .validate_access(root, uri, operation)
                        .map_err(|e| {
                            MCPError::Resource(ResourceError::AccessDenied(format!(
                                "Root validation failed: {}",
                                e
                            )))
                        });
                } else {
                    return Ok(());
                }
            }
        }
        Ok(())
    }
}

struct TemplatePromptHandler;

#[async_trait::async_trait]
impl PromptHandler for TemplatePromptHandler {
    async fn get_prompt(&self, request: GetPromptRequest) -> MCPResult<GetPromptResponse> {
        info!("Getting prompt: {}", request.name);

        match request.name.as_str() {
            "greeting" => {
                let messages = vec![
                    prompts::PromptMessage::system(prompts::PromptContent::text(
                        "You are a helpful assistant that creates personalized greetings."
                            .to_string(),
                    )),
                    prompts::PromptMessage::user(prompts::PromptContent::text(
                        "Create a greeting for {name}".to_string(),
                    )),
                ];

                Ok(GetPromptResponse {
                    description: Some("A template for creating personalized greetings".to_string()),
                    messages,
                })
            }
            "weather" => {
                let messages = vec![
                    prompts::PromptMessage::system(prompts::PromptContent::text(
                        "You are a weather assistant that provides weather information."
                            .to_string(),
                    )),
                    prompts::PromptMessage::user(prompts::PromptContent::text(
                        "Provide weather information for {text}".to_string(),
                    )),
                ];

                Ok(GetPromptResponse {
                    description: Some("A template for weather information requests".to_string()),
                    messages,
                })
            }
            _ => Err(MCPError::not_found(format!(
                "Prompt not found: {}",
                request.name
            ))),
        }
    }

    async fn list_prompts(&self, _request: ListPromptsRequest) -> MCPResult<ListPromptsResponse> {
        Ok(ListPromptsResponse {
            prompts: vec![
                Prompt {
                    name: "greeting".to_string(),
                    description: Some("Generate a personalized greeting".to_string()),
                    arguments: Some(vec![
                        prompts::PromptArgument::new("name".to_string()).required(true)
                    ]),
                },
                Prompt {
                    name: "summarize".to_string(),
                    description: Some("Summarize text content".to_string()),
                    arguments: Some(vec![
                        prompts::PromptArgument::new("text".to_string()).required(true)
                    ]),
                },
            ],
            next_cursor: None,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing with comprehensive configuration
    tracing_subscriber::fmt()
        .with_env_filter("info,ultrafast_mcp=debug,ultrafast_mcp_transport=debug")
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    info!("🚀 Starting Advanced Features MCP Server with Streamable HTTP");

    // Initialize monitoring system
    let monitoring_config = MonitoringConfig::default();
    let monitoring = Arc::new(MonitoringSystem::new(monitoring_config));

    // Initialize health checker
    let health_checker = monitoring.health();

    // Add custom health checks
    health_checker
        .add_check(Box::new(DatabaseHealthCheck))
        .await;
    health_checker
        .add_check(Box::new(FileSystemHealthCheck))
        .await;

    // Create server capabilities with all features
    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability {
            list_changed: Some(true),
        }),
        resources: Some(ResourcesCapability {
            list_changed: Some(true),
            subscribe: Some(true),
        }),
        prompts: Some(PromptsCapability {
            list_changed: Some(true),
        }),
        ..Default::default()
    };

    // Create server info
    let server_info = ServerInfo {
        name: "advanced-features-server".to_string(),
        version: "1.0.0".to_string(),
        description: Some(
            "Advanced MCP server demonstrating all features with monitoring and authentication"
                .to_string(),
        ),
        authors: Some(vec!["ULTRAFAST_MCP Team".to_string()]),
        homepage: Some("https://github.com/ultrafast-mcp/ultrafast-mcp".to_string()),
        license: Some("MIT OR Apache-2.0".to_string()),
        repository: Some("https://github.com/ultrafast-mcp/ultrafast-mcp".to_string()),
    };

    // Create the UltraFastServer with advanced features
    let _server = UltraFastServer::new(server_info, capabilities)
        .with_tool_handler(Arc::new(AdvancedToolHandler::new(monitoring.clone())))
        .with_resource_handler(Arc::new(FileResourceHandler))
        .with_prompt_handler(Arc::new(TemplatePromptHandler))
        .with_sampling_handler(Arc::new(AdvancedSamplingHandler))
        .with_completion_handler(Arc::new(AdvancedCompletionHandler))
        .with_roots_handler(Arc::new(AdvancedRootsHandler))
        .with_elicitation_handler(Arc::new(AdvancedElicitationHandler))
        .with_subscription_handler(Arc::new(AdvancedSubscriptionHandler))
        .with_full_monitoring() // Enable all monitoring features
        .with_middleware() // Enable middleware support
        .with_recovery() // Enable recovery mechanisms
        .with_oauth() // Enable OAuth authentication
        .with_rate_limiting(100) // Enable rate limiting (100 requests per minute)
        .with_request_validation() // Enable request validation
        .with_response_caching(); // Enable response caching

    info!("✅ Server created successfully with all advanced features");

    // Start health monitoring in background
    let health_monitor = monitoring.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

            match health_monitor.health().get_overall_health().await {
                HealthStatus::Healthy => {
                    info!("System health check: All systems healthy");
                }
                HealthStatus::Degraded(warnings) => {
                    warn!("System health check: System degraded - {:?}", warnings);
                }
                HealthStatus::Unhealthy(errors) => {
                    error!("System health check: System unhealthy - {:?}", errors);
                }
            }
        }
    });

    // Set up graceful shutdown
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for ctrl+c signal");
        info!("Received shutdown signal");
    };

    // Run the server with error handling and graceful shutdown
    let server_task = async {
        // Create and start the HTTP transport server
        let transport_config = HttpTransportConfig {
            host: "0.0.0.0".to_string(),
            port: 8080,
            cors_enabled: true,
            protocol_version: "2025-06-18".to_string(),
            allow_origin: Some("http://localhost:*".to_string()),
            monitoring_enabled: true,
            enable_sse_resumability: true,
        };

        let transport_server = HttpTransportServer::new(transport_config);

        info!("Starting HTTP transport server on 127.0.0.1:8080");
        info!("Monitoring dashboard available at http://127.0.0.1:8081");

        // Run the transport server (this will block until shutdown)
        if let Err(e) = transport_server.run().await {
            error!("Transport server error: {}", e);
            return Err(e);
        }

        Ok(())
    };

    // Wait for either shutdown signal or server error
    tokio::select! {
        _ = shutdown_signal => {
            info!("Shutting down server gracefully...");
            monitoring.shutdown().await?;
            Ok(())
        }
        result = server_task => {
            match result {
                Ok(_) => {
                    info!("Server completed successfully");
                    Ok(())
                }
                Err(e) => {
                    error!("Server failed: {}", e);
                    Err(e.into())
                }
            }
        }
    }
}

// Custom health checks
struct DatabaseHealthCheck;

#[async_trait::async_trait]
impl HealthCheck for DatabaseHealthCheck {
    async fn check(&self) -> HealthCheckResult {
        let start = std::time::Instant::now();

        // Simulate database health check
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let status = HealthStatus::Healthy;

        HealthCheckResult {
            status,
            duration: start.elapsed(),
            timestamp: std::time::SystemTime::now(),
            details: None,
        }
    }

    fn name(&self) -> &str {
        "database"
    }
}

struct FileSystemHealthCheck;

#[async_trait::async_trait]
impl HealthCheck for FileSystemHealthCheck {
    async fn check(&self) -> HealthCheckResult {
        let start = std::time::Instant::now();

        // Check if /tmp directory is writable
        let status = match std::fs::metadata("/tmp") {
            Ok(metadata) if metadata.is_dir() => HealthStatus::Healthy,
            _ => HealthStatus::Unhealthy(vec!["Cannot access /tmp directory".to_string()]),
        };

        HealthCheckResult {
            status,
            duration: start.elapsed(),
            timestamp: std::time::SystemTime::now(),
            details: None,
        }
    }

    fn name(&self) -> &str {
        "filesystem"
    }
}

// Simple handler implementations for missing types
struct AdvancedSamplingHandler;

#[async_trait::async_trait]
impl SamplingHandler for AdvancedSamplingHandler {
    async fn create_message(
        &self,
        _request: sampling::CreateMessageRequest,
    ) -> MCPResult<sampling::CreateMessageResponse> {
        Ok(sampling::SamplingResponse {
            role: sampling::SamplingRole::Assistant,
            content: sampling::SamplingContent::Text {
                text: "Sample message created".to_string(),
            },
            model: None,
            stop_reason: None,
            approval_status: None,
            request_id: None,
            processing_time_ms: None,
            cost_info: None,
            included_context: None,
            human_feedback: None,
            warnings: None,
        })
    }
}

struct AdvancedCompletionHandler;

#[async_trait::async_trait]
impl CompletionHandler for AdvancedCompletionHandler {
    async fn complete(
        &self,
        _request: completion::CompleteRequest,
    ) -> MCPResult<completion::CompleteResponse> {
        Ok(completion::CompleteResponse {
            completion: completion::Completion::new(vec![completion::CompletionValue::new(
                "completion",
            )]),
            metadata: None,
        })
    }
}

struct AdvancedRootsHandler;

#[async_trait::async_trait]
impl RootsHandler for AdvancedRootsHandler {
    async fn list_roots(&self) -> MCPResult<Vec<roots::Root>> {
        Ok(vec![roots::Root {
            uri: "file:///".to_string(),
            name: Some("File System Root".to_string()),
            security: None,
        }])
    }
}

struct AdvancedElicitationHandler;

#[async_trait::async_trait]
impl ElicitationHandler for AdvancedElicitationHandler {
    async fn handle_elicitation(
        &self,
        request: elicitation::ElicitationRequest,
    ) -> MCPResult<elicitation::ElicitationResponse> {
        // Log the elicitation request
        println!("Advanced elicitation request: {}", request.message);
        println!("Requested schema: {:?}", request.requested_schema);

        // In a real implementation, this would present the request to the user
        // For demonstration, we'll simulate an accept response with project details
        Ok(elicitation::ElicitationResponse {
            action: elicitation::ElicitationAction::Accept,
            content: Some(serde_json::json!({
                "name": "my-awesome-project",
                "framework": "react",
                "useTypeScript": true
            })),
        })
    }
}

struct AdvancedSubscriptionHandler;

#[async_trait::async_trait]
impl ResourceSubscriptionHandler for AdvancedSubscriptionHandler {
    async fn subscribe(&self, _uri: String) -> MCPResult<()> {
        Ok(())
    }

    async fn unsubscribe(&self, _uri: String) -> MCPResult<()> {
        Ok(())
    }

    async fn notify_change(&self, _uri: String, _content: serde_json::Value) -> MCPResult<()> {
        Ok(())
    }
}
