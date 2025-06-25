# Server API Reference

Complete reference for the **ULTRAFAST_MCP** server API, including configuration, handlers, and advanced features.

## 🖥️ UltraFastServer

The main server type that provides a high-level, ergonomic API for building MCP servers.

### Basic Usage

```rust
use ultrafast_mcp::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let server = UltraFastServer::new("My MCP Server")
        .with_protocol_version("2025-06-18")
        .with_capabilities(ServerCapabilities {
            tools: Some(ToolsCapability { list_changed: true }),
            resources: Some(ResourcesCapability { subscribe: true, list_changed: true }),
            prompts: Some(PromptsCapability { list_changed: true }),
            logging: Some(LoggingCapability {}),
            ..Default::default()
        });
    
    server.run_stdio().await?;
    Ok(())
}
```

### Constructor

```rust
impl UltraFastServer {
    pub fn new(name: &str) -> UltraFastServerBuilder;
    
    pub fn new_with_info(info: ServerInfo, capabilities: ServerCapabilities) -> Self;
}
```

### Builder Pattern

```rust
let server = UltraFastServer::new("My Server")
    .with_protocol_version("2025-06-18")
    .with_capabilities(capabilities)
    .with_tool_handler(Arc::new(MyToolHandler))
    .with_resource_handler(Arc::new(MyResourceHandler))
    .with_prompt_handler(Arc::new(MyPromptHandler))
    .with_sampling_handler(Arc::new(MySamplingHandler))
    .with_completion_handler(Arc::new(MyCompletionHandler))
    .with_roots_handler(Arc::new(MyRootsHandler))
    .with_elicitation_handler(Arc::new(MyElicitationHandler))
    .with_subscription_handler(Arc::new(MySubscriptionHandler))
    .with_monitoring_config(monitoring_config)
    .build()?;
```

## 🛠️ Tool Handlers

### ToolHandler Trait

```rust
#[async_trait]
pub trait ToolHandler: Send + Sync {
    async fn handle_tool_call(&self, call: ToolCall) -> McpResult<ToolResult>;
    async fn list_tools(&self, request: ListToolsRequest) -> McpResult<ListToolsResponse>;
}
```

### Simple Tool Implementation

```rust
use ultrafast_mcp::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct CalculatorRequest {
    operation: String,
    a: f64,
    b: f64,
}

#[derive(Serialize)]
struct CalculatorResponse {
    result: f64,
}

pub struct CalculatorHandler;

#[async_trait]
impl ToolHandler for CalculatorHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> McpResult<ToolResult> {
        match call.name.as_str() {
            "calculate" => {
                let request: CalculatorRequest = serde_json::from_value(call.arguments)?;
                let result = match request.operation.as_str() {
                    "add" => request.a + request.b,
                    "subtract" => request.a - request.b,
                    "multiply" => request.a * request.b,
                    "divide" => {
                        if request.b == 0.0 {
                            return Err(McpError::invalid_params("Division by zero"));
                        }
                        request.a / request.b
                    }
                    _ => return Err(McpError::invalid_params("Unknown operation")),
                };
                
                Ok(ToolResult {
                    content: vec![ToolContent::Text {
                        type_: "text".to_string(),
                        text: serde_json::to_value(CalculatorResponse { result })?,
                    }],
                })
            }
            _ => Err(McpError::method_not_found(format!("Unknown tool: {}", call.name))),
        }
    }
    
    async fn list_tools(&self, _request: ListToolsRequest) -> McpResult<ListToolsResponse> {
        Ok(ListToolsResponse {
            tools: vec![Tool {
                name: "calculate".to_string(),
                description: Some("Perform basic arithmetic operations".to_string()),
                input_schema: Some(json!({
                    "type": "object",
                    "properties": {
                        "operation": {"type": "string", "enum": ["add", "subtract", "multiply", "divide"]},
                        "a": {"type": "number"},
                        "b": {"type": "number"}
                    },
                    "required": ["operation", "a", "b"]
                })),
                ..Default::default()
            }],
        })
    }
}
```

### Advanced Tool with Progress Tracking

```rust
pub struct FileProcessorHandler;

#[async_trait]
impl ToolHandler for FileProcessorHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> McpResult<ToolResult> {
        match call.name.as_str() {
            "process_file" => {
                let request: ProcessFileRequest = serde_json::from_value(call.arguments)?;
                
                // Create context for progress tracking
                let ctx = Context::new()
                    .with_request_id(call.id.clone())
                    .with_metadata("file_path".to_string(), json!(request.file_path));
                
                // Process file with progress updates
                ctx.progress("Reading file...", 0.1, Some(1.0)).await?;
                let content = tokio::fs::read_to_string(&request.file_path).await?;
                
                ctx.progress("Processing content...", 0.5, Some(1.0)).await?;
                let processed = process_content(&content, &request.operations).await?;
                
                ctx.progress("Writing result...", 0.9, Some(1.0)).await?;
                tokio::fs::write(&request.output_path, processed).await?;
                
                ctx.progress("Complete", 1.0, Some(1.0)).await?;
                ctx.log_info(&format!("Processed file: {}", request.file_path)).await?;
                
                Ok(ToolResult {
                    content: vec![ToolContent::Text {
                        type_: "text".to_string(),
                        text: json!({"status": "success", "output_path": request.output_path}),
                    }],
                })
            }
            _ => Err(McpError::method_not_found(format!("Unknown tool: {}", call.name))),
        }
    }
    
    async fn list_tools(&self, _request: ListToolsRequest) -> McpResult<ListToolsResponse> {
        Ok(ListToolsResponse {
            tools: vec![Tool {
                name: "process_file".to_string(),
                description: Some("Process files with various operations".to_string()),
                input_schema: Some(json!({
                    "type": "object",
                    "properties": {
                        "file_path": {"type": "string"},
                        "output_path": {"type": "string"},
                        "operations": {"type": "array", "items": {"type": "string"}}
                    },
                    "required": ["file_path", "output_path", "operations"]
                })),
                ..Default::default()
            }],
        })
    }
}
```

## 📁 Resource Handlers

### ResourceHandler Trait

```rust
#[async_trait]
pub trait ResourceHandler: Send + Sync {
    async fn read_resource(&self, request: ReadResourceRequest) -> McpResult<ReadResourceResponse>;
    async fn list_resources(&self, request: ListResourcesRequest) -> McpResult<ListResourcesResponse>;
    async fn list_resource_templates(&self, request: ListResourceTemplatesRequest) -> McpResult<ListResourceTemplatesResponse>;
}
```

### File System Resource Handler

```rust
pub struct FileSystemResourceHandler {
    root_path: PathBuf,
}

#[async_trait]
impl ResourceHandler for FileSystemResourceHandler {
    async fn read_resource(&self, request: ReadResourceRequest) -> McpResult<ReadResourceResponse> {
        let uri = request.uri;
        
        if uri.starts_with("file://") {
            let path = uri.strip_prefix("file://").unwrap();
            let full_path = self.root_path.join(path);
            
            if !full_path.starts_with(&self.root_path) {
                return Err(McpError::invalid_params("Path traversal not allowed"));
            }
            
            if full_path.is_file() {
                let content = tokio::fs::read(&full_path).await?;
                let mime_type = mime_guess::from_path(&full_path)
                    .first_or_octet_stream()
                    .to_string();
                
                Ok(ReadResourceResponse {
                    contents: vec![ResourceContent::Binary {
                        uri: uri.clone(),
                        mime_type,
                        data: content,
                    }],
                })
            } else {
                Err(McpError::resource_not_found(uri))
            }
        } else {
            Err(McpError::invalid_params("Unsupported URI scheme"))
        }
    }
    
    async fn list_resources(&self, request: ListResourcesRequest) -> McpResult<ListResourcesResponse> {
        let uri = request.uri;
        
        if uri.starts_with("file://") {
            let path = uri.strip_prefix("file://").unwrap();
            let full_path = self.root_path.join(path);
            
            if !full_path.starts_with(&self.root_path) {
                return Err(McpError::invalid_params("Path traversal not allowed"));
            }
            
            if full_path.is_dir() {
                let mut resources = Vec::new();
                let mut entries = tokio::fs::read_dir(&full_path).await?;
                
                while let Some(entry) = entries.next_entry().await? {
                    let file_type = entry.file_type().await?;
                    let name = entry.file_name().to_string_lossy().to_string();
                    
                    resources.push(Resource {
                        uri: format!("file://{}/{}", path, name),
                        name: Some(name),
                        description: None,
                        mime_type: if file_type.is_file() {
                            Some(mime_guess::from_path(entry.path())
                                .first_or_octet_stream()
                                .to_string())
                        } else {
                            None
                        },
                    });
                }
                
                Ok(ListResourcesResponse {
                    resources,
                    continuation_token: None,
                })
            } else {
                Err(McpError::resource_not_found(uri))
            }
        } else {
            Err(McpError::invalid_params("Unsupported URI scheme"))
        }
    }
    
    async fn list_resource_templates(&self, _request: ListResourceTemplatesRequest) -> McpResult<ListResourceTemplatesResponse> {
        Ok(ListResourceTemplatesResponse {
            resource_templates: vec![ResourceTemplate {
                uri: "file://{path}".to_string(),
                name: Some("File System Resource".to_string()),
                description: Some("Access files in the file system".to_string()),
            }],
        })
    }
}
```

### HTTP Resource Handler

```rust
pub struct HttpResourceHandler {
    client: reqwest::Client,
}

#[async_trait]
impl ResourceHandler for HttpResourceHandler {
    async fn read_resource(&self, request: ReadResourceRequest) -> McpResult<ReadResourceResponse> {
        let uri = request.uri;
        
        if uri.starts_with("https://") || uri.starts_with("http://") {
            let response = self.client.get(&uri).send().await?;
            
            if response.status().is_success() {
                let content = response.bytes().await?;
                let mime_type = response.headers()
                    .get("content-type")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("application/octet-stream")
                    .to_string();
                
                Ok(ReadResourceResponse {
                    contents: vec![ResourceContent::Binary {
                        uri: uri.clone(),
                        mime_type,
                        data: content.to_vec(),
                    }],
                })
            } else {
                Err(McpError::resource_not_found(uri))
            }
        } else {
            Err(McpError::invalid_params("Unsupported URI scheme"))
        }
    }
    
    async fn list_resources(&self, _request: ListResourcesRequest) -> McpResult<ListResourcesResponse> {
        // HTTP resources don't support listing
        Err(McpError::method_not_supported("HTTP resources do not support listing"))
    }
    
    async fn list_resource_templates(&self, _request: ListResourceTemplatesRequest) -> McpResult<ListResourceTemplatesResponse> {
        Ok(ListResourceTemplatesResponse {
            resource_templates: vec![ResourceTemplate {
                uri: "https://{domain}/{path}".to_string(),
                name: Some("HTTP Resource".to_string()),
                description: Some("Access HTTP resources".to_string()),
            }],
        })
    }
}
```

## 💬 Prompt Handlers

### PromptHandler Trait

```rust
#[async_trait]
pub trait PromptHandler: Send + Sync {
    async fn get_prompt(&self, request: GetPromptRequest) -> McpResult<GetPromptResponse>;
    async fn list_prompts(&self, request: ListPromptsRequest) -> McpResult<ListPromptsResponse>;
}
```

### Code Review Prompt Handler

```rust
pub struct CodeReviewPromptHandler;

#[async_trait]
impl PromptHandler for CodeReviewPromptHandler {
    async fn get_prompt(&self, request: GetPromptRequest) -> McpResult<GetPromptResponse> {
        match request.name.as_str() {
            "code_review" => {
                let arguments = request.arguments.unwrap_or_default();
                let code = arguments.get("code")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing 'code' argument"))?;
                
                let language = arguments.get("language")
                    .and_then(|v| v.as_str())
                    .unwrap_or_else(|| detect_language(code));
                
                let prompt = PromptMessages::new()
                    .system("You are an expert code reviewer. Focus on security, performance, and maintainability.")
                    .user(&format!("Please review this {} code:\n\n```{}\n{}\n```", language, language, code))
                    .with_context("Provide specific, actionable feedback");
                
                Ok(GetPromptResponse {
                    prompt: Prompt {
                        name: "code_review".to_string(),
                        description: Some("Review code for best practices".to_string()),
                        arguments: vec![
                            PromptArgument {
                                name: "code".to_string(),
                                description: "The code to review".to_string(),
                                required: true,
                                schema: Some(json!({"type": "string"})),
                            },
                            PromptArgument {
                                name: "language".to_string(),
                                description: "Programming language (auto-detected if not provided)".to_string(),
                                required: false,
                                schema: Some(json!({"type": "string"})),
                            },
                        ],
                    },
                    messages: prompt,
                })
            }
            _ => Err(McpError::method_not_found(format!("Unknown prompt: {}", request.name))),
        }
    }
    
    async fn list_prompts(&self, _request: ListPromptsRequest) -> McpResult<ListPromptsResponse> {
        Ok(ListPromptsResponse {
            prompts: vec![Prompt {
                name: "code_review".to_string(),
                description: Some("Review code for best practices".to_string()),
                arguments: vec![
                    PromptArgument {
                        name: "code".to_string(),
                        description: "The code to review".to_string(),
                        required: true,
                        schema: Some(json!({"type": "string"})),
                    },
                    PromptArgument {
                        name: "language".to_string(),
                        description: "Programming language".to_string(),
                        required: false,
                        schema: Some(json!({"type": "string"})),
                    },
                ],
            }],
        })
    }
}

fn detect_language(code: &str) -> &'static str {
    // Simple language detection based on file extensions or code patterns
    if code.contains("fn ") && code.contains("->") {
        "rust"
    } else if code.contains("def ") && code.contains(":"):
        "python"
    } else if code.contains("function ") && code.contains("{"):
        "javascript"
    } else {
        "text"
    }
}
```

## 🧠 Sampling Handlers

### SamplingHandler Trait

```rust
#[async_trait]
pub trait SamplingHandler: Send + Sync {
    async fn create_message(&self, request: CreateMessageRequest) -> McpResult<CreateMessageResponse>;
}
```

### LLM Integration Handler

```rust
pub struct LlmSamplingHandler {
    llm_client: Arc<dyn LlmClient>,
}

#[async_trait]
impl SamplingHandler for LlmSamplingHandler {
    async fn create_message(&self, request: CreateMessageRequest) -> McpResult<CreateMessageResponse> {
        // Convert MCP messages to LLM format
        let messages = request.messages.iter().map(|msg| {
            match msg.role.as_str() {
                "user" => LlmMessage::User { content: msg.content.clone() },
                "assistant" => LlmMessage::Assistant { content: msg.content.clone() },
                "system" => LlmMessage::System { content: msg.content.clone() },
                _ => LlmMessage::User { content: msg.content.clone() },
            }
        }).collect();
        
        // Generate response using LLM
        let response = self.llm_client.generate(
            messages,
            request.model_preferences,
            request.max_tokens,
        ).await?;
        
        Ok(CreateMessageResponse {
            role: "assistant".to_string(),
            content: response.content,
            model: response.model_used,
            stop_reason: response.stop_reason,
        })
    }
}

#[async_trait]
trait LlmClient: Send + Sync {
    async fn generate(
        &self,
        messages: Vec<LlmMessage>,
        model_preferences: Option<ModelPreferences>,
        max_tokens: Option<u32>,
    ) -> Result<LlmResponse>;
}

enum LlmMessage {
    User { content: String },
    Assistant { content: String },
    System { content: String },
}

struct LlmResponse {
    content: String,
    model_used: String,
    stop_reason: String,
}
```

## 🔍 Completion Handlers

### CompletionHandler Trait

```rust
#[async_trait]
pub trait CompletionHandler: Send + Sync {
    async fn complete(&self, request: CompleteRequest) -> McpResult<CompleteResponse>;
}
```

### Argument Completion Handler

```rust
pub struct ArgumentCompletionHandler;

#[async_trait]
impl CompletionHandler for ArgumentCompletionHandler {
    async fn complete(&self, request: CompleteRequest) -> McpResult<CompleteResponse> {
        match request.prompt_name.as_str() {
            "code_review" => {
                match request.argument_name.as_str() {
                    "language" => {
                        let suggestions = vec![
                            "rust", "python", "javascript", "typescript", 
                            "go", "java", "c++", "c#", "php", "ruby"
                        ];
                        
                        let filtered = suggestions.iter()
                            .filter(|s| s.starts_with(&request.partial))
                            .map(|s| s.to_string())
                            .collect();
                        
                        Ok(CompleteResponse {
                            suggestions: filtered,
                        })
                    }
                    _ => Ok(CompleteResponse { suggestions: vec![] }),
                }
            }
            _ => Ok(CompleteResponse { suggestions: vec![] }),
        }
    }
}
```

## 📂 Roots Handlers

### RootsHandler Trait

```rust
#[async_trait]
pub trait RootsHandler: Send + Sync {
    async fn list_roots(&self) -> McpResult<Vec<Root>>;
}
```

### File System Roots Handler

```rust
pub struct FileSystemRootsHandler {
    allowed_paths: Vec<PathBuf>,
}

#[async_trait]
impl RootsHandler for FileSystemRootsHandler {
    async fn list_roots(&self) -> McpResult<Vec<Root>> {
        let mut roots = Vec::new();
        
        for path in &self.allowed_paths {
            if path.exists() && path.is_dir() {
                roots.push(Root {
                    uri: format!("file://{}", path.display()),
                    name: Some(path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string()),
                });
            }
        }
        
        Ok(roots)
    }
}
```

## ❓ Elicitation Handlers

### ElicitationHandler Trait

```rust
#[async_trait]
pub trait ElicitationHandler: Send + Sync {
    async fn handle_elicitation(&self, request: ElicitationRequest) -> McpResult<ElicitationResponse>;
}
```

### User Input Handler

```rust
pub struct UserInputHandler;

#[async_trait]
impl ElicitationHandler for UserInputHandler {
    async fn handle_elicitation(&self, request: ElicitationRequest) -> McpResult<ElicitationResponse> {
        match request.elicitation_type.as_str() {
            "user_confirmation" => {
                // In a real implementation, this would prompt the user
                // For now, we'll simulate user approval
                Ok(ElicitationResponse {
                    response: json!({"approved": true}),
                })
            }
            "user_input" => {
                // Simulate user input
                Ok(ElicitationResponse {
                    response: json!({"input": "User provided input"}),
                })
            }
            _ => Err(McpError::invalid_params("Unknown elicitation type")),
        }
    }
}
```

## 🔔 Subscription Handlers

### ResourceSubscriptionHandler Trait

```rust
#[async_trait]
pub trait ResourceSubscriptionHandler: Send + Sync {
    async fn subscribe(&self, uri: String) -> McpResult<()>;
    async fn unsubscribe(&self, uri: String) -> McpResult<()>;
    async fn notify_change(&self, uri: String, content: serde_json::Value) -> McpResult<()>;
}
```

### File System Subscription Handler

```rust
use tokio::sync::broadcast;
use std::collections::HashMap;

pub struct FileSystemSubscriptionHandler {
    subscriptions: Arc<RwLock<HashMap<String, broadcast::Sender<serde_json::Value>>>>,
    watcher: Arc<FileWatcher>,
}

#[async_trait]
impl ResourceSubscriptionHandler for FileSystemSubscriptionHandler {
    async fn subscribe(&self, uri: String) -> McpResult<()> {
        if uri.starts_with("file://") {
            let path = uri.strip_prefix("file://").unwrap();
            let (tx, _rx) = broadcast::channel(100);
            
            self.subscriptions.write().await.insert(uri.clone(), tx);
            self.watcher.watch_path(path).await?;
            
            Ok(())
        } else {
            Err(McpError::invalid_params("Unsupported URI scheme"))
        }
    }
    
    async fn unsubscribe(&self, uri: String) -> McpResult<()> {
        self.subscriptions.write().await.remove(&uri);
        Ok(())
    }
    
    async fn notify_change(&self, uri: String, content: serde_json::Value) -> McpResult<()> {
        if let Some(tx) = self.subscriptions.read().await.get(&uri) {
            let _ = tx.send(content);
        }
        Ok(())
    }
}
```

## 🚀 Transport Configuration

### stdio Transport

```rust
// Default transport for local development
server.run_stdio().await?;
```

### HTTP Transport

```rust
// HTTP server with custom configuration
server.run_http("127.0.0.1", 8080).await?;

// Or with custom HTTP configuration
let http_config = HttpTransportConfig {
    host: "0.0.0.0".to_string(),
    port: 8080,
    session_timeout_secs: 300,
    max_message_retries: 3,
    cors_enabled: true,
    auth_required: false,
    protocol_version: "2025-06-18".to_string(),
    enable_streamable_http: true,
    enable_legacy_endpoints: false,
    rate_limit_config: RateLimitConfig::default(),
    connection_pool_config: PoolConfig::default(),
    request_timeout: Duration::from_secs(30),
    max_request_size: 1024 * 1024,
    enable_compression: true,
};

server.run_with_http_config(http_config).await?;
```

### Custom Transport

```rust
// Use a custom transport implementation
let custom_transport = Box::new(MyCustomTransport::new());
server.run_with_transport(custom_transport).await?;
```

## 📊 Monitoring Configuration

### Basic Monitoring

```rust
let server = UltraFastServer::new("My Server")
    .with_default_monitoring()
    .build()?;
```

### Custom Monitoring

```rust
let monitoring_config = MonitoringConfig {
    tracing: Some(TracingConfig {
        endpoint: "http://localhost:14268/api/traces".into(),
        service_name: "my-mcp-server".into(),
    }),
    metrics: Some(MetricsConfig {
        endpoint: "http://localhost:9090".into(),
    }),
    health: Some(HealthConfig {
        enabled: true,
        port: 8081,
    }),
};

let server = UltraFastServer::new("My Server")
    .with_monitoring_config(monitoring_config)
    .build()?;
```

## 🔧 Advanced Features

### Server Composition

```rust
let server = UltraFastServer::new("Composed Server")
    .with_federation_config(FederationConfig {
        servers: HashMap::new(),
        routing_strategy: RoutingStrategy::CapabilityBased,
        health_check_interval: Duration::from_secs(30),
    })
    .build()?;
```

### Middleware Integration

```rust
let server = UltraFastServer::new("My Server")
    .with_middleware(Arc::new(LoggingMiddleware))
    .with_middleware(Arc::new(AuthMiddleware))
    .with_middleware(Arc::new(RateLimitMiddleware))
    .build()?;
```

### Custom Error Handling

```rust
impl From<MyCustomError> for McpError {
    fn from(error: MyCustomError) -> Self {
        match error {
            MyCustomError::NotFound => McpError::resource_not_found("Resource not found".to_string()),
            MyCustomError::InvalidInput => McpError::invalid_params("Invalid input".to_string()),
            MyCustomError::Internal => McpError::internal_error("Internal server error".to_string()),
        }
    }
}
```

## 📋 Complete Example

```rust
use ultrafast_mcp::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct WeatherRequest {
    city: String,
}

#[derive(Serialize)]
struct WeatherResponse {
    temperature: f64,
    conditions: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let server = UltraFastServer::new("Weather MCP Server")
        .with_protocol_version("2025-06-18")
        .with_capabilities(ServerCapabilities {
            tools: Some(ToolsCapability { list_changed: true }),
            resources: Some(ResourcesCapability { subscribe: true, list_changed: true }),
            prompts: Some(PromptsCapability { list_changed: true }),
            logging: Some(LoggingCapability {}),
            ..Default::default()
        })
        .with_tool_handler(Arc::new(WeatherToolHandler))
        .with_resource_handler(Arc::new(WeatherResourceHandler))
        .with_prompt_handler(Arc::new(WeatherPromptHandler))
        .with_monitoring_config(MonitoringConfig::default())
        .build()?;
    
    // Run with HTTP transport
    server.run_http("127.0.0.1", 8080).await?;
    Ok(())
}

pub struct WeatherToolHandler;

#[async_trait]
impl ToolHandler for WeatherToolHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> McpResult<ToolResult> {
        match call.name.as_str() {
            "get_weather" => {
                let request: WeatherRequest = serde_json::from_value(call.arguments)?;
                
                // Simulate weather API call
                let weather = get_weather_data(&request.city).await?;
                
                Ok(ToolResult {
                    content: vec![ToolContent::Text {
                        type_: "text".to_string(),
                        text: serde_json::to_value(weather)?,
                    }],
                })
            }
            _ => Err(McpError::method_not_found(format!("Unknown tool: {}", call.name))),
        }
    }
    
    async fn list_tools(&self, _request: ListToolsRequest) -> McpResult<ListToolsResponse> {
        Ok(ListToolsResponse {
            tools: vec![Tool {
                name: "get_weather".to_string(),
                description: Some("Get current weather for a city".to_string()),
                input_schema: Some(json!({
                    "type": "object",
                    "properties": {
                        "city": {"type": "string"}
                    },
                    "required": ["city"]
                })),
                ..Default::default()
            }],
        })
    }
}

async fn get_weather_data(city: &str) -> Result<WeatherResponse> {
    // Simulate API call
    Ok(WeatherResponse {
        temperature: 22.5,
        conditions: "Sunny".to_string(),
    })
}
```

This comprehensive server API reference covers all aspects of building MCP servers with **ULTRAFAST_MCP**, from basic usage to advanced features and production deployment. 🚀 