# Tools

**Tools** are the core feature of MCP that enable function execution with model-controlled safety. ULTRAFAST_MCP provides an ergonomic, type-safe API for implementing tools with automatic schema generation, progress tracking, and comprehensive error handling.

## 🛠️ Overview

Tools in MCP allow servers to expose functions that can be called by clients, typically under the control of an LLM. This enables:

- **Function execution** with structured input/output
- **Model-controlled safety** with human-in-the-loop approval
- **JSON Schema validation** for type safety
- **Progress tracking** for long-running operations
- **Resource integration** with embedded resources

## 🚀 Quick Start

### Basic Tool Implementation

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
                        "operation": {
                            "type": "string",
                            "enum": ["add", "subtract", "multiply", "divide"]
                        },
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

### Registering Tools

```rust
let server = UltraFastServer::new("Calculator Server")
    .with_capabilities(ServerCapabilities {
        tools: Some(ToolsCapability { list_changed: true }),
        ..Default::default()
    })
    .with_tool_handler(Arc::new(CalculatorHandler))
    .build()?;
```

## 📋 Tool Structure

### Tool Definition

```rust
pub struct Tool {
    pub name: String,                    // Unique tool identifier
    pub description: Option<String>,     // Human-readable description
    pub input_schema: Option<Value>,     // JSON Schema for input validation
    pub input_schema_uri: Option<String>, // URI to input schema
    pub output_schema: Option<Value>,    // JSON Schema for output validation
    pub output_schema_uri: Option<String>, // URI to output schema
    pub is_required: Option<bool>,       // Whether tool is required
    pub is_dangerous: Option<bool>,      // Whether tool requires approval
}
```

### Tool Call Request

```rust
pub struct ToolCall {
    pub name: String,           // Tool name to call
    pub arguments: Value,       // Tool arguments (JSON)
    pub id: Option<String>,     // Optional call ID
}
```

### Tool Result

```rust
pub struct ToolResult {
    pub content: Vec<ToolContent>, // Tool output content
    pub is_error: Option<bool>,    // Whether result represents an error
}
```

### Tool Content Types

```rust
pub enum ToolContent {
    Text {
        type_: String,          // Content type (e.g., "text")
        text: Value,            // Text content
    },
    Image {
        type_: String,          // Content type (e.g., "image")
        image: ImageContent,    // Image data
    },
    Audio {
        type_: String,          // Content type (e.g., "audio")
        audio: AudioContent,    // Audio data
    },
    Video {
        type_: String,          // Content type (e.g., "video")
        video: VideoContent,    // Video data
    },
    EmbeddedResource {
        type_: String,          // Content type (e.g., "embedded_resource")
        resource: ResourceReference, // Resource reference
    },
}
```

## 🔧 Advanced Tool Patterns

### 1. Progress Tracking

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
                        text: json!({
                            "status": "success",
                            "output_path": request.output_path,
                            "processed_bytes": processed.len()
                        }),
                    }],
                })
            }
            _ => Err(McpError::method_not_found(format!("Unknown tool: {}", call.name))),
        }
    }
}
```

### 2. Resource Integration

```rust
pub struct DocumentAnalyzerHandler;

#[async_trait]
impl ToolHandler for DocumentAnalyzerHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> McpResult<ToolResult> {
        match call.name.as_str() {
            "analyze_document" => {
                let request: AnalyzeDocumentRequest = serde_json::from_value(call.arguments)?;
                
                // Read document from resource
                let document_content = read_resource(&request.document_uri).await?;
                
                // Analyze document
                let analysis = analyze_document(&document_content).await?;
                
                // Create analysis report as embedded resource
                let report_uri = format!("file:///tmp/analysis_{}.json", uuid::Uuid::new_v4());
                write_resource(&report_uri, &analysis).await?;
                
                Ok(ToolResult {
                    content: vec![
                        ToolContent::Text {
                            type_: "text".to_string(),
                            text: json!({
                                "status": "success",
                                "summary": analysis.summary,
                                "word_count": analysis.word_count,
                                "sentiment": analysis.sentiment
                            }),
                        },
                        ToolContent::EmbeddedResource {
                            type_: "embedded_resource".to_string(),
                            resource: ResourceReference {
                                uri: report_uri,
                                mime_type: "application/json".to_string(),
                                name: Some("Detailed Analysis Report".to_string()),
                            },
                        },
                    ],
                })
            }
            _ => Err(McpError::method_not_found(format!("Unknown tool: {}", call.name))),
        }
    }
}
```

### 3. Multi-Modal Content

```rust
pub struct ImageProcessorHandler;

#[async_trait]
impl ToolHandler for ImageProcessorHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> McpResult<ToolResult> {
        match call.name.as_str() {
            "process_image" => {
                let request: ProcessImageRequest = serde_json::from_value(call.arguments)?;
                
                // Load image
                let image_data = load_image(&request.image_path).await?;
                
                // Process image
                let processed_image = apply_filters(&image_data, &request.filters).await?;
                
                // Save processed image
                let output_path = format!("/tmp/processed_{}", uuid::Uuid::new_v4());
                save_image(&processed_image, &output_path).await?;
                
                Ok(ToolResult {
                    content: vec![
                        ToolContent::Text {
                            type_: "text".to_string(),
                            text: json!({
                                "status": "success",
                                "output_path": output_path,
                                "filters_applied": request.filters
                            }),
                        },
                        ToolContent::Image {
                            type_: "image".to_string(),
                            image: ImageContent {
                                uri: format!("file://{}", output_path),
                                mime_type: "image/png".to_string(),
                                alt_text: Some("Processed image".to_string()),
                            },
                        },
                    ],
                })
            }
            _ => Err(McpError::method_not_found(format!("Unknown tool: {}", call.name))),
        }
    }
}
```

### 4. Error Handling

```rust
pub struct RobustToolHandler;

#[async_trait]
impl ToolHandler for RobustToolHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> McpResult<ToolResult> {
        match call.name.as_str() {
            "robust_operation" => {
                let result = self.perform_robust_operation(call).await;
                
                match result {
                    Ok(success_result) => Ok(ToolResult {
                        content: vec![ToolContent::Text {
                            type_: "text".to_string(),
                            text: serde_json::to_value(success_result)?,
                        }],
                        is_error: Some(false),
                    }),
                    Err(error) => {
                        // Log the error
                        tracing::error!("Tool operation failed: {:?}", error);
                        
                        // Return structured error
                        Ok(ToolResult {
                            content: vec![ToolContent::Text {
                                type_: "text".to_string(),
                                text: json!({
                                    "error": error.to_string(),
                                    "error_type": "operation_failed",
                                    "suggestions": ["Check input parameters", "Verify system resources"]
                                }),
                            }],
                            is_error: Some(true),
                        })
                    }
                }
            }
            _ => Err(McpError::method_not_found(format!("Unknown tool: {}", call.name))),
        }
    }
    
    async fn perform_robust_operation(&self, call: ToolCall) -> Result<OperationResult> {
        // Implement robust operation with retries, timeouts, etc.
        let request: RobustOperationRequest = serde_json::from_value(call.arguments)?;
        
        // Add retry logic
        let mut attempts = 0;
        let max_attempts = 3;
        
        while attempts < max_attempts {
            match self.try_operation(&request).await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    attempts += 1;
                    if attempts >= max_attempts {
                        return Err(error);
                    }
                    
                    // Exponential backoff
                    tokio::time::sleep(Duration::from_millis(100 * 2_u64.pow(attempts))).await;
                }
            }
        }
        
        Err(anyhow::anyhow!("Operation failed after {} attempts", max_attempts))
    }
}
```

## 📊 Schema Generation

### Automatic Schema Generation

ULTRAFAST_MCP can automatically generate JSON schemas from Rust types:

```rust
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema)]
struct WeatherRequest {
    #[schemars(description = "City name for weather lookup")]
    city: String,
    
    #[schemars(description = "Temperature unit (celsius/fahrenheit)")]
    unit: Option<String>,
    
    #[schemars(description = "Include detailed forecast")]
    detailed: Option<bool>,
}

#[derive(Serialize, JsonSchema)]
struct WeatherResponse {
    temperature: f64,
    conditions: String,
    humidity: Option<f64>,
    wind_speed: Option<f64>,
}

pub struct WeatherHandler;

#[async_trait]
impl ToolHandler for WeatherHandler {
    async fn list_tools(&self, _request: ListToolsRequest) -> McpResult<ListToolsResponse> {
        Ok(ListToolsResponse {
            tools: vec![Tool {
                name: "get_weather".to_string(),
                description: Some("Get current weather for a city".to_string()),
                input_schema: Some(serde_json::to_value(WeatherRequest::schema())?),
                output_schema: Some(serde_json::to_value(WeatherResponse::schema())?),
                ..Default::default()
            }],
        })
    }
}
```

### Custom Schema Validation

```rust
pub struct ValidatedToolHandler;

#[async_trait]
impl ToolHandler for ValidatedToolHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> McpResult<ToolResult> {
        // Validate input against schema
        let schema = self.get_tool_schema(&call.name)?;
        validate_against_schema(&call.arguments, &schema)?;
        
        // Process tool call
        let result = self.process_tool_call(call).await?;
        
        // Validate output against schema
        let output_schema = self.get_tool_output_schema(&call.name)?;
        validate_against_schema(&result, &output_schema)?;
        
        Ok(ToolResult {
            content: vec![ToolContent::Text {
                type_: "text".to_string(),
                text: result,
            }],
        })
    }
}
```

## 🔒 Security Considerations

### 1. Input Validation

```rust
pub struct SecureToolHandler;

#[async_trait]
impl ToolHandler for SecureToolHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> McpResult<ToolResult> {
        // Validate and sanitize input
        let sanitized_args = self.sanitize_input(call.arguments)?;
        
        // Check permissions
        self.check_permissions(&call.name, &sanitized_args).await?;
        
        // Rate limiting
        self.check_rate_limit(&call.name).await?;
        
        // Process tool call
        let result = self.process_tool_call(call.name, sanitized_args).await?;
        
        Ok(ToolResult {
            content: vec![ToolContent::Text {
                type_: "text".to_string(),
                text: result,
            }],
        })
    }
    
    fn sanitize_input(&self, args: Value) -> Result<Value> {
        // Remove potentially dangerous fields
        let mut sanitized = args.clone();
        if let Some(obj) = sanitized.as_object_mut() {
            obj.remove("__proto__");
            obj.remove("constructor");
            obj.remove("prototype");
        }
        
        // Validate string lengths
        if let Some(obj) = sanitized.as_object() {
            for (key, value) in obj {
                if let Some(s) = value.as_str() {
                    if s.len() > 10000 {
                        return Err(anyhow::anyhow!("Input too long: {}", key));
                    }
                }
            }
        }
        
        Ok(sanitized)
    }
}
```

### 2. Dangerous Operations

```rust
pub struct DangerousToolHandler;

#[async_trait]
impl ToolHandler for DangerousToolHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> McpResult<ToolResult> {
        match call.name.as_str() {
            "delete_file" => {
                // Check if operation requires approval
                if self.requires_approval(&call.name) {
                    let approved = self.request_approval(&call).await?;
                    if !approved {
                        return Err(McpError::permission_denied("Operation not approved"));
                    }
                }
                
                // Perform dangerous operation
                self.perform_dangerous_operation(call).await
            }
            _ => Err(McpError::method_not_found(format!("Unknown tool: {}", call.name))),
        }
    }
}
```

## 📈 Performance Optimization

### 1. Async Processing

```rust
pub struct AsyncToolHandler {
    thread_pool: Arc<ThreadPool>,
}

#[async_trait]
impl ToolHandler for AsyncToolHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> McpResult<ToolResult> {
        match call.name.as_str() {
            "cpu_intensive_task" => {
                // Offload CPU-intensive work to thread pool
                let result = self.thread_pool.spawn_ok(async move {
                    perform_cpu_intensive_task(call.arguments).await
                }).await?;
                
                Ok(ToolResult {
                    content: vec![ToolContent::Text {
                        type_: "text".to_string(),
                        text: result,
                    }],
                })
            }
            _ => Err(McpError::method_not_found(format!("Unknown tool: {}", call.name))),
        }
    }
}
```

### 2. Caching

```rust
pub struct CachedToolHandler {
    cache: Arc<DashMap<String, (Value, Instant)>>,
    cache_ttl: Duration,
}

#[async_trait]
impl ToolHandler for CachedToolHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> McpResult<ToolResult> {
        // Check cache first
        let cache_key = format!("{}:{}", call.name, serde_json::to_string(&call.arguments)?);
        
        if let Some((cached_result, timestamp)) = self.cache.get(&cache_key) {
            if timestamp.elapsed() < self.cache_ttl {
                return Ok(ToolResult {
                    content: vec![ToolContent::Text {
                        type_: "text".to_string(),
                        text: cached_result.clone(),
                    }],
                });
            }
        }
        
        // Perform operation
        let result = self.perform_operation(call).await?;
        
        // Cache result
        self.cache.insert(cache_key, (result.clone(), Instant::now()));
        
        Ok(ToolResult {
            content: vec![ToolContent::Text {
                type_: "text".to_string(),
                text: result,
            }],
        })
    }
}
```

## 🔄 Tool Lifecycle

### 1. Tool Registration

```rust
impl UltraFastServer {
    pub fn register_tool<T>(&mut self, name: &str, handler: T) -> &mut Self
    where
        T: ToolHandler + 'static,
    {
        self.tool_handlers.insert(name.to_string(), Arc::new(handler));
        self
    }
    
    pub fn register_tool_with_schema<T>(
        &mut self,
        name: &str,
        handler: T,
        input_schema: Value,
        output_schema: Value,
    ) -> &mut Self
    where
        T: ToolHandler + 'static,
    {
        let tool = Tool {
            name: name.to_string(),
            input_schema: Some(input_schema),
            output_schema: Some(output_schema),
            ..Default::default()
        };
        
        self.tools.insert(name.to_string(), tool);
        self.register_tool(name, handler)
    }
}
```

### 2. Tool Discovery

```rust
pub struct ToolDiscoveryHandler;

#[async_trait]
impl ToolHandler for ToolDiscoveryHandler {
    async fn list_tools(&self, request: ListToolsRequest) -> McpResult<ListToolsResponse> {
        let mut tools = Vec::new();
        
        // Add built-in tools
        tools.push(Tool {
            name: "list_tools".to_string(),
            description: Some("List available tools".to_string()),
            ..Default::default()
        });
        
        // Add dynamic tools
        for tool in self.discover_dynamic_tools().await? {
            tools.push(tool);
        }
        
        // Apply pagination
        let (start, end) = self.apply_pagination(&tools, &request)?;
        let paginated_tools = tools[start..end].to_vec();
        
        Ok(ListToolsResponse {
            tools: paginated_tools,
            continuation_token: if end < tools.len() {
                Some(end.to_string())
            } else {
                None
            },
        })
    }
}
```

## 📋 Best Practices

### 1. Error Handling

- **Use structured errors** with error codes and messages
- **Provide helpful error messages** with suggestions
- **Log errors** for debugging and monitoring
- **Handle timeouts** gracefully
- **Validate input** thoroughly

### 2. Performance

- **Use async operations** for I/O-bound tasks
- **Implement caching** for expensive operations
- **Add progress tracking** for long-running operations
- **Use connection pooling** for external services
- **Optimize memory usage** with efficient data structures

### 3. Security

- **Validate and sanitize** all input
- **Implement rate limiting** to prevent abuse
- **Use authentication** for sensitive operations
- **Require approval** for dangerous operations
- **Log security events** for monitoring

### 4. User Experience

- **Provide clear descriptions** for tools
- **Use consistent naming** conventions
- **Include examples** in tool descriptions
- **Support pagination** for large result sets
- **Add progress indicators** for long operations

## 🎯 Complete Example

```rust
use ultrafast_mcp::prelude::*;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema)]
struct DataAnalysisRequest {
    #[schemars(description = "Data file path to analyze")]
    file_path: String,
    
    #[schemars(description = "Analysis type")]
    analysis_type: String,
    
    #[schemars(description = "Output format")]
    output_format: Option<String>,
}

#[derive(Serialize, JsonSchema)]
struct DataAnalysisResponse {
    summary: String,
    statistics: Value,
    output_file: String,
}

pub struct DataAnalysisHandler {
    cache: Arc<DashMap<String, (Value, Instant)>>,
}

#[async_trait]
impl ToolHandler for DataAnalysisHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> McpResult<ToolResult> {
        match call.name.as_str() {
            "analyze_data" => {
                let request: DataAnalysisRequest = serde_json::from_value(call.arguments)?;
                
                // Create context for progress tracking
                let ctx = Context::new()
                    .with_request_id(call.id.clone())
                    .with_metadata("file_path".to_string(), json!(request.file_path));
                
                // Check cache
                let cache_key = format!("{}:{}:{}", request.file_path, request.analysis_type, request.output_format.unwrap_or_default());
                if let Some((cached_result, timestamp)) = self.cache.get(&cache_key) {
                    if timestamp.elapsed() < Duration::from_hours(1) {
                        ctx.log_info("Returning cached result").await?;
                        return Ok(ToolResult {
                            content: vec![ToolContent::Text {
                                type_: "text".to_string(),
                                text: cached_result.clone(),
                            }],
                        });
                    }
                }
                
                // Perform analysis with progress updates
                ctx.progress("Reading data file...", 0.1, Some(1.0)).await?;
                let data = read_data_file(&request.file_path).await?;
                
                ctx.progress("Performing analysis...", 0.5, Some(1.0)).await?;
                let analysis = perform_analysis(&data, &request.analysis_type).await?;
                
                ctx.progress("Generating output...", 0.8, Some(1.0)).await?;
                let output_file = generate_output(&analysis, &request.output_format).await?;
                
                ctx.progress("Complete", 1.0, Some(1.0)).await?;
                ctx.log_info(&format!("Analysis completed for: {}", request.file_path)).await?;
                
                let result = DataAnalysisResponse {
                    summary: analysis.summary,
                    statistics: analysis.statistics,
                    output_file: output_file.clone(),
                };
                
                let result_json = serde_json::to_value(result)?;
                
                // Cache result
                self.cache.insert(cache_key, (result_json.clone(), Instant::now()));
                
                Ok(ToolResult {
                    content: vec![
                        ToolContent::Text {
                            type_: "text".to_string(),
                            text: result_json,
                        },
                        ToolContent::EmbeddedResource {
                            type_: "embedded_resource".to_string(),
                            resource: ResourceReference {
                                uri: format!("file://{}", output_file),
                                mime_type: "application/json".to_string(),
                                name: Some("Analysis Results".to_string()),
                            },
                        },
                    ],
                })
            }
            _ => Err(McpError::method_not_found(format!("Unknown tool: {}", call.name))),
        }
    }
    
    async fn list_tools(&self, _request: ListToolsRequest) -> McpResult<ListToolsResponse> {
        Ok(ListToolsResponse {
            tools: vec![Tool {
                name: "analyze_data".to_string(),
                description: Some("Perform data analysis on files".to_string()),
                input_schema: Some(serde_json::to_value(DataAnalysisRequest::schema())?),
                output_schema: Some(serde_json::to_value(DataAnalysisResponse::schema())?),
                ..Default::default()
            }],
        })
    }
}
```

This comprehensive guide covers all aspects of implementing tools in **ULTRAFAST_MCP**, from basic usage to advanced patterns and best practices. 🚀 