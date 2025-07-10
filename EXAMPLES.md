# UltraFast MCP Examples Guide

This guide provides comprehensive examples demonstrating how to use the UltraFast MCP framework for building servers and clients.

## Table of Contents

1. [Basic Examples](#basic-examples)
2. [Server Examples](#server-examples)
3. [Client Examples](#client-examples)
4. [Advanced Examples](#advanced-examples)
5. [Integration Examples](#integration-examples)
6. [Real-World Examples](#real-world-examples)

## Basic Examples

### 1. Simple Echo Server

A basic server that echoes back messages with timestamps.

```rust
use ultrafast_mcp::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
struct EchoRequest {
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct EchoResponse {
    message: String,
    timestamp: String,
}

struct EchoToolHandler;

#[async_trait::async_trait]
impl ToolHandler for EchoToolHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult> {
        match call.name.as_str() {
            "echo" => {
                let args: EchoRequest = serde_json::from_value(
                    call.arguments.unwrap_or_default()
                )?;

                let response = EchoResponse {
                    message: args.message,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };

                let response_text = serde_json::to_string_pretty(&response)?;

                Ok(ToolResult {
                    content: vec![ToolContent::text(response_text)],
                    is_error: Some(false),
                })
            }
            _ => Err(MCPError::method_not_found(
                format!("Unknown tool: {}", call.name)
            )),
        }
    }

    async fn list_tools(&self, _request: ListToolsRequest) -> MCPResult<ListToolsResponse> {
        Ok(ListToolsResponse {
            tools: vec![Tool {
                name: "echo".to_string(),
                description: "Echo back a message with timestamp".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "message": {
                            "type": "string",
                            "description": "Message to echo back",
                            "maxLength": 1000
                        }
                    },
                    "required": ["message"]
                }),
                output_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "message": {"type": "string"},
                        "timestamp": {"type": "string"}
                    },
                    "required": ["message", "timestamp"]
                })),
                annotations: None,
            }],
            next_cursor: None,
        })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server_info = ServerInfo {
        name: "echo-server".to_string(),
        version: "1.0.0".to_string(),
        description: Some("A simple echo server".to_string()),
        authors: None,
        homepage: None,
        license: None,
        repository: None,
    };

    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability { list_changed: Some(true) }),
        ..Default::default()
    };

    let server = UltraFastServer::new(server_info, capabilities)
        .with_tool_handler(Arc::new(EchoToolHandler));

    server.run_stdio().await?;
    Ok(())
}
```

### 2. Simple Echo Client

A client that connects to the echo server and calls the echo tool.

```rust
use ultrafast_mcp::prelude::*;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client_info = ClientInfo {
        name: "echo-client".to_string(),
        version: "1.0.0".to_string(),
        authors: None,
        description: Some("A simple echo client".to_string()),
        homepage: None,
        repository: None,
        license: None,
    };

    let capabilities = ClientCapabilities::default();

    let client = UltraFastClient::new(client_info, capabilities);

    // Connect to the server using STDIO
    client.connect_stdio().await?;

    // Call the echo tool
    let tool_call = ToolCall {
        name: "echo".to_string(),
        arguments: Some(json!({
            "message": "Hello, UltraFast MCP!"
        })),
    };

    let result = client.call_tool(tool_call).await?;
    println!("Server response: {:?}", result);

    // Disconnect
    client.disconnect().await?;

    Ok(())
}
```

## Server Examples

### 3. Calculator Server

A server that provides mathematical operations.

```rust
use ultrafast_mcp::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
struct CalculateRequest {
    operation: String,
    a: f64,
    b: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct CalculateResponse {
    result: f64,
    operation: String,
}

struct CalculatorHandler;

#[async_trait::async_trait]
impl ToolHandler for CalculatorHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult> {
        match call.name.as_str() {
            "calculate" => {
                let args: CalculateRequest = serde_json::from_value(
                    call.arguments.unwrap_or_default()
                )?;

                let result = match args.operation.as_str() {
                    "add" => args.a + args.b,
                    "subtract" => args.a - args.b,
                    "multiply" => args.a * args.b,
                    "divide" => {
                        if args.b == 0.0 {
                            return Err(MCPError::invalid_params(
                                "Division by zero".to_string()
                            ));
                        }
                        args.a / args.b
                    }
                    _ => return Err(MCPError::invalid_params(
                        format!("Unknown operation: {}", args.operation)
                    )),
                };

                let response = CalculateResponse {
                    result,
                    operation: args.operation,
                };

                let response_text = serde_json::to_string_pretty(&response)?;

                Ok(ToolResult {
                    content: vec![ToolContent::text(response_text)],
                    is_error: Some(false),
                })
            }
            _ => Err(MCPError::method_not_found(
                format!("Unknown tool: {}", call.name)
            )),
        }
    }

    async fn list_tools(&self, _request: ListToolsRequest) -> MCPResult<ListToolsResponse> {
        Ok(ListToolsResponse {
            tools: vec![Tool {
                name: "calculate".to_string(),
                description: "Perform mathematical operations".to_string(),
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
                output_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "result": {"type": "number"},
                        "operation": {"type": "string"}
                    },
                    "required": ["result", "operation"]
                })),
                annotations: None,
            }],
            next_cursor: None,
        })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server_info = ServerInfo {
        name: "calculator-server".to_string(),
        version: "1.0.0".to_string(),
        description: Some("A mathematical calculator server".to_string()),
        authors: None,
        homepage: None,
        license: None,
        repository: None,
    };

    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability { list_changed: Some(true) }),
        ..Default::default()
    };

    let server = UltraFastServer::new(server_info, capabilities)
        .with_tool_handler(Arc::new(CalculatorHandler));

    server.run_stdio().await?;
    Ok(())
}
```

### 4. File Operations Server

A server that provides file system operations.

```rust
use ultrafast_mcp::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
struct ReadFileRequest {
    path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReadFileResponse {
    content: String,
    size: u64,
    path: String,
}

struct FileHandler;

#[async_trait::async_trait]
impl ToolHandler for FileHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult> {
        match call.name.as_str() {
            "read_file" => {
                let args: ReadFileRequest = serde_json::from_value(
                    call.arguments.unwrap_or_default()
                )?;

                let path = PathBuf::from(&args.path);
                
                // Basic security check
                if !path.is_relative() {
                    return Err(MCPError::invalid_params(
                        "Only relative paths are allowed".to_string()
                    ));
                }

                let content = tokio::fs::read_to_string(&path).await
                    .map_err(|e| MCPError::not_found(
                        format!("File not found: {}", e)
                    ))?;

                let metadata = tokio::fs::metadata(&path).await
                    .map_err(|e| MCPError::internal_error(
                        format!("Failed to get file metadata: {}", e)
                    ))?;

                let response = ReadFileResponse {
                    content,
                    size: metadata.len(),
                    path: args.path,
                };

                let response_text = serde_json::to_string_pretty(&response)?;

                Ok(ToolResult {
                    content: vec![ToolContent::text(response_text)],
                    is_error: Some(false),
                })
            }
            _ => Err(MCPError::method_not_found(
                format!("Unknown tool: {}", call.name)
            )),
        }
    }

    async fn list_tools(&self, _request: ListToolsRequest) -> MCPResult<ListToolsResponse> {
        Ok(ListToolsResponse {
            tools: vec![Tool {
                name: "read_file".to_string(),
                description: "Read a file from the filesystem".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the file to read"
                        }
                    },
                    "required": ["path"]
                }),
                output_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "content": {"type": "string"},
                        "size": {"type": "integer"},
                        "path": {"type": "string"}
                    },
                    "required": ["content", "size", "path"]
                })),
                annotations: None,
            }],
            next_cursor: None,
        })
    }
}

#[async_trait::async_trait]
impl ResourceHandler for FileHandler {
    async fn read_resource(&self, request: ReadResourceRequest) -> MCPResult<ReadResourceResponse> {
        let path = request.uri.trim_start_matches("file://");
        let content = tokio::fs::read_to_string(path).await
            .map_err(|e| MCPError::not_found(
                format!("File not found: {}", e)
            ))?;

        Ok(ReadResourceResponse {
            contents: vec![ResourceContent::text(
                request.uri,
                content
            )],
        })
    }

    async fn list_resources(&self, _request: ListResourcesRequest) -> MCPResult<ListResourcesResponse> {
        Ok(ListResourcesResponse {
            resources: vec![],
            next_cursor: None,
        })
    }

    async fn list_resource_templates(&self, _request: ListResourceTemplatesRequest) -> MCPResult<ListResourceTemplatesResponse> {
        Ok(ListResourceTemplatesResponse {
            resource_templates: vec![],
            next_cursor: None,
        })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server_info = ServerInfo {
        name: "file-server".to_string(),
        version: "1.0.0".to_string(),
        description: Some("A file operations server".to_string()),
        authors: None,
        homepage: None,
        license: None,
        repository: None,
    };

    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability { list_changed: Some(true) }),
        resources: Some(ResourcesCapability { list_changed: Some(true) }),
        ..Default::default()
    };

    let handler = Arc::new(FileHandler);
    let server = UltraFastServer::new(server_info, capabilities)
        .with_tool_handler(handler.clone())
        .with_resource_handler(handler);

    server.run_stdio().await?;
    Ok(())
}
```

### 5. HTTP Server with OAuth

A server that runs over HTTP with OAuth authentication.

```rust
use ultrafast_mcp::prelude::*;
use std::sync::Arc;

struct SecureHandler;

#[async_trait::async_trait]
impl ToolHandler for SecureHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult> {
        match call.name.as_str() {
            "secure_operation" => {
                // In a real implementation, you would verify authentication here
                Ok(ToolResult {
                    content: vec![ToolContent::text(
                        "Secure operation completed successfully".to_string()
                    )],
                    is_error: Some(false),
                })
            }
            _ => Err(MCPError::method_not_found(
                format!("Unknown tool: {}", call.name)
            )),
        }
    }

    async fn list_tools(&self, _request: ListToolsRequest) -> MCPResult<ListToolsResponse> {
        Ok(ListToolsResponse {
            tools: vec![Tool {
                name: "secure_operation".to_string(),
                description: "Perform a secure operation".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "data": {
                            "type": "string",
                            "description": "Data to process"
                        }
                    },
                    "required": ["data"]
                }),
                output_schema: None,
                annotations: None,
            }],
            next_cursor: None,
        })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info,ultrafast_mcp=debug")
        .init();

    let server_info = ServerInfo {
        name: "secure-server".to_string(),
        version: "1.0.0".to_string(),
        description: Some("A secure HTTP server".to_string()),
        authors: None,
        homepage: None,
        license: None,
        repository: None,
    };

    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability { list_changed: Some(true) }),
        ..Default::default()
    };

    let server = UltraFastServer::new(server_info, capabilities)
        .with_tool_handler(Arc::new(SecureHandler));

    println!("Starting secure server on http://127.0.0.1:8080");
    server.run_streamable_http("127.0.0.1", 8080).await?;

    Ok(())
}
```

## Client Examples

### 6. Interactive Client

A client that provides an interactive interface for calling tools.

```rust
use ultrafast_mcp::prelude::*;
use serde_json::json;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client_info = ClientInfo {
        name: "interactive-client".to_string(),
        version: "1.0.0".to_string(),
        authors: None,
        description: Some("An interactive MCP client".to_string()),
        homepage: None,
        repository: None,
        license: None,
    };

    let capabilities = ClientCapabilities::default();

    let client = UltraFastClient::new(client_info, capabilities);

    // Connect to the server
    println!("Connecting to server...");
    client.connect_stdio().await?;
    println!("Connected!");

    // Get server info
    if let Some(info) = client.get_server_info().await {
        println!("Server: {} v{}", info.name, info.version);
        if let Some(desc) = info.description {
            println!("Description: {}", desc);
        }
    }

    // List available tools
    let tools_response = client.list_tools_default().await?;
    println!("\nAvailable tools:");
    for tool in &tools_response.tools {
        println!("  - {}: {}", tool.name, tool.description);
    }

    // Interactive loop
    loop {
        print!("\nEnter tool name (or 'quit' to exit): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let tool_name = input.trim();

        if tool_name == "quit" {
            break;
        }

        if tool_name.is_empty() {
            continue;
        }

        // Find the tool
        let tool = tools_response.tools.iter()
            .find(|t| t.name == tool_name);

        match tool {
            Some(tool) => {
                println!("Calling tool: {}", tool.name);
                
                // For simplicity, we'll use empty arguments
                let tool_call = ToolCall {
                    name: tool_name.to_string(),
                    arguments: Some(json!({})),
                };

                match client.call_tool(tool_call).await {
                    Ok(result) => {
                        println!("Result:");
                        for content in &result.content {
                            if let ToolContent::Text { text } = content {
                                println!("{}", text);
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
            }
            None => {
                println!("Tool '{}' not found", tool_name);
            }
        }
    }

    println!("Disconnecting...");
    client.disconnect().await?;
    println!("Disconnected!");

    Ok(())
}
```

### 7. Batch Processing Client

A client that processes multiple operations in batch.

```rust
use ultrafast_mcp::prelude::*;
use serde_json::json;
use std::time::Instant;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client_info = ClientInfo {
        name: "batch-client".to_string(),
        version: "1.0.0".to_string(),
        authors: None,
        description: Some("A batch processing client".to_string()),
        homepage: None,
        repository: None,
        license: None,
    };

    let capabilities = ClientCapabilities::default();

    let client = UltraFastClient::new(client_info, capabilities);

    // Connect to the server
    client.connect_stdio().await?;
    println!("Connected to server");

    // Define batch operations
    let operations = vec![
        ("echo", json!({"message": "Hello 1"})),
        ("echo", json!({"message": "Hello 2"})),
        ("echo", json!({"message": "Hello 3"})),
        ("echo", json!({"message": "Hello 4"})),
        ("echo", json!({"message": "Hello 5"})),
    ];

    println!("Processing {} operations...", operations.len());
    let start = Instant::now();

    // Process operations concurrently
    let handles: Vec<_> = operations
        .into_iter()
        .map(|(name, args)| {
            let client = &client;
            tokio::spawn(async move {
                let tool_call = ToolCall {
                    name: name.to_string(),
                    arguments: Some(args),
                };
                client.call_tool(tool_call).await
            })
        })
        .collect();

    // Wait for all operations to complete
    let results = futures::future::join_all(handles).await;

    let duration = start.elapsed();
    println!("Completed in {:?}", duration);

    // Process results
    let mut success_count = 0;
    let mut error_count = 0;

    for (i, result) in results.into_iter().enumerate() {
        match result {
            Ok(Ok(tool_result)) => {
                success_count += 1;
                println!("Operation {}: Success", i + 1);
                for content in &tool_result.content {
                    if let ToolContent::Text { text } = content {
                        println!("  Response: {}", text);
                    }
                }
            }
            Ok(Err(e)) => {
                error_count += 1;
                println!("Operation {}: Error - {}", i + 1, e);
            }
            Err(e) => {
                error_count += 1;
                println!("Operation {}: Join Error - {}", i + 1, e);
            }
        }
    }

    println!("\nSummary:");
    println!("  Successful: {}", success_count);
    println!("  Failed: {}", error_count);
    println!("  Total time: {:?}", duration);

    // Disconnect
    client.disconnect().await?;
    println!("Disconnected");

    Ok(())
}
```

## Advanced Examples

### 8. Server with Multiple Handlers

A server that implements multiple handler types.

```rust
use ultrafast_mcp::prelude::*;
use std::sync::Arc;

// Tool Handler
struct MultiToolHandler;

#[async_trait::async_trait]
impl ToolHandler for MultiToolHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult> {
        match call.name.as_str() {
            "hello" => {
                Ok(ToolResult {
                    content: vec![ToolContent::text("Hello, World!".to_string())],
                    is_error: Some(false),
                })
            }
            _ => Err(MCPError::method_not_found(
                format!("Unknown tool: {}", call.name)
            )),
        }
    }

    async fn list_tools(&self, _request: ListToolsRequest) -> MCPResult<ListToolsResponse> {
        Ok(ListToolsResponse {
            tools: vec![Tool {
                name: "hello".to_string(),
                description: "Say hello".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
                output_schema: None,
                annotations: None,
            }],
            next_cursor: None,
        })
    }
}

// Resource Handler
struct MultiResourceHandler;

#[async_trait::async_trait]
impl ResourceHandler for MultiResourceHandler {
    async fn read_resource(&self, request: ReadResourceRequest) -> MCPResult<ReadResourceResponse> {
        if request.uri == "memory://status" {
            Ok(ReadResourceResponse {
                contents: vec![ResourceContent::text(
                    request.uri,
                    "Server is running".to_string()
                )],
            })
        } else {
            Err(MCPError::not_found(
                format!("Resource not found: {}", request.uri)
            ))
        }
    }

    async fn list_resources(&self, _request: ListResourcesRequest) -> MCPResult<ListResourcesResponse> {
        Ok(ListResourcesResponse {
            resources: vec![Resource {
                uri: "memory://status".to_string(),
                name: "Server Status".to_string(),
                description: Some("Current server status".to_string()),
                mime_type: Some("text/plain".to_string()),
            }],
            next_cursor: None,
        })
    }

    async fn list_resource_templates(&self, _request: ListResourceTemplatesRequest) -> MCPResult<ListResourceTemplatesResponse> {
        Ok(ListResourceTemplatesResponse {
            resource_templates: vec![],
            next_cursor: None,
        })
    }
}

// Prompt Handler
struct MultiPromptHandler;

#[async_trait::async_trait]
impl PromptHandler for MultiPromptHandler {
    async fn get_prompt(&self, request: GetPromptRequest) -> MCPResult<GetPromptResponse> {
        match request.name.as_str() {
            "greeting" => {
                Ok(GetPromptResponse {
                    messages: vec![PromptMessage::system(
                        PromptContent::text("You are a helpful assistant.".to_string())
                    )],
                    description: Some("A greeting prompt".to_string()),
                })
            }
            _ => Err(MCPError::not_found(
                format!("Prompt not found: {}", request.name)
            )),
        }
    }

    async fn list_prompts(&self, _request: ListPromptsRequest) -> MCPResult<ListPromptsResponse> {
        Ok(ListPromptsResponse {
            prompts: vec![Prompt {
                name: "greeting".to_string(),
                description: Some("A greeting prompt".to_string()),
                arguments: None,
            }],
            next_cursor: None,
        })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server_info = ServerInfo {
        name: "multi-handler-server".to_string(),
        version: "1.0.0".to_string(),
        description: Some("A server with multiple handlers".to_string()),
        authors: None,
        homepage: None,
        license: None,
        repository: None,
    };

    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability { list_changed: Some(true) }),
        resources: Some(ResourcesCapability { list_changed: Some(true) }),
        prompts: Some(PromptsCapability { list_changed: Some(true) }),
        ..Default::default()
    };

    let server = UltraFastServer::new(server_info, capabilities)
        .with_tool_handler(Arc::new(MultiToolHandler))
        .with_resource_handler(Arc::new(MultiResourceHandler))
        .with_prompt_handler(Arc::new(MultiPromptHandler));

    server.run_stdio().await?;
    Ok(())
}
```

### 9. Server with Monitoring

A server that includes monitoring and observability features.

```rust
use ultrafast_mcp::prelude::*;
use ultrafast_mcp_monitoring::{MonitoringSystem, MonitoringConfig, RequestTimer};
use std::sync::Arc;

struct MonitoredHandler {
    monitoring: Arc<MonitoringSystem>,
}

impl MonitoredHandler {
    fn new(monitoring: Arc<MonitoringSystem>) -> Self {
        Self { monitoring }
    }
}

#[async_trait::async_trait]
impl ToolHandler for MonitoredHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult> {
        let timer = RequestTimer::start("tools/call", self.monitoring.metrics().clone());

        let result = match call.name.as_str() {
            "monitored_operation" => {
                // Simulate some work
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                
                Ok(ToolResult {
                    content: vec![ToolContent::text(
                        "Monitored operation completed".to_string()
                    )],
                    is_error: Some(false),
                })
            }
            _ => Err(MCPError::method_not_found(
                format!("Unknown tool: {}", call.name)
            )),
        };

        // Record the operation result
        match &result {
            Ok(_) => timer.finish(true).await,
            Err(_) => timer.finish(false).await,
        }

        result
    }

    async fn list_tools(&self, _request: ListToolsRequest) -> MCPResult<ListToolsResponse> {
        Ok(ListToolsResponse {
            tools: vec![Tool {
                name: "monitored_operation".to_string(),
                description: "A monitored operation".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
                output_schema: None,
                annotations: None,
            }],
            next_cursor: None,
        })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize monitoring
    let monitoring = Arc::new(MonitoringSystem::init(MonitoringConfig::default()).await?);
    
    // Start monitoring HTTP server
    let monitoring_addr = "127.0.0.1:9091".parse()?;
    let monitoring_clone = monitoring.clone();
    tokio::spawn(async move {
        if let Err(e) = monitoring_clone.start_http_server(monitoring_addr).await {
            eprintln!("Failed to start monitoring server: {}", e);
        }
    });

    let server_info = ServerInfo {
        name: "monitored-server".to_string(),
        version: "1.0.0".to_string(),
        description: Some("A monitored server".to_string()),
        authors: None,
        homepage: None,
        license: None,
        repository: None,
    };

    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability { list_changed: Some(true) }),
        ..Default::default()
    };

    let server = UltraFastServer::new(server_info, capabilities)
        .with_tool_handler(Arc::new(MonitoredHandler::new(monitoring)));

    println!("Starting monitored server...");
    println!("Monitoring available at http://127.0.0.1:9091");
    
    server.run_stdio().await?;
    Ok(())
}
```

## Integration Examples

### 10. Complete Integration Test

A comprehensive integration test showing server-client communication.

```rust
use ultrafast_mcp::prelude::*;
use ultrafast_mcp_test_utils::*;
use std::sync::Arc;

#[tokio::test]
async fn test_complete_integration() -> anyhow::Result<()> {
    // Create test server
    let server_info = ServerInfo {
        name: "test-server".to_string(),
        version: "1.0.0".to_string(),
        description: Some("Test server".to_string()),
        authors: None,
        homepage: None,
        license: None,
        repository: None,
    };

    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability { list_changed: Some(true) }),
        resources: Some(ResourcesCapability { list_changed: Some(true) }),
        ..Default::default()
    };

    let server = UltraFastServer::new(server_info, capabilities)
        .with_tool_handler(Arc::new(TestToolHandler))
        .with_resource_handler(Arc::new(TestResourceHandler));

    // Start server in background
    let server_handle = tokio::spawn(async move {
        server.run_stdio().await
    });

    // Give server time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Create client
    let client_info = ClientInfo {
        name: "test-client".to_string(),
        version: "1.0.0".to_string(),
        authors: None,
        description: Some("Test client".to_string()),
        homepage: None,
        repository: None,
        license: None,
    };

    let capabilities = ClientCapabilities::default();
    let client = UltraFastClient::new(client_info, capabilities);

    // Connect to server
    client.connect_stdio().await?;

    // Test server info
    let server_info = client.get_server_info().await;
    assert!(server_info.is_some());
    assert_eq!(server_info.unwrap().name, "test-server");

    // Test tool listing
    let tools_response = client.list_tools_default().await?;
    assert_eq!(tools_response.tools.len(), 1);
    assert_eq!(tools_response.tools[0].name, "test_tool");

    // Test tool call
    let tool_call = ToolCall {
        name: "test_tool".to_string(),
        arguments: Some(serde_json::json!({"input": "test"})),
    };

    let result = client.call_tool(tool_call).await?;
    assert!(!result.is_error.unwrap_or(true));

    // Test resource listing
    let resources_response = client.list_resources(ListResourcesRequest {
        uri: None,
        name: None,
        mime_type: None,
        cursor: None,
    }).await?;
    assert_eq!(resources_response.resources.len(), 1);

    // Disconnect client
    client.disconnect().await?;

    // Stop server
    server_handle.abort();
    let _ = server_handle.await;

    Ok(())
}

struct TestToolHandler;

#[async_trait::async_trait]
impl ToolHandler for TestToolHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult> {
        match call.name.as_str() {
            "test_tool" => {
                Ok(ToolResult {
                    content: vec![ToolContent::text("Test successful".to_string())],
                    is_error: Some(false),
                })
            }
            _ => Err(MCPError::method_not_found(
                format!("Unknown tool: {}", call.name)
            )),
        }
    }

    async fn list_tools(&self, _request: ListToolsRequest) -> MCPResult<ListToolsResponse> {
        Ok(ListToolsResponse {
            tools: vec![Tool {
                name: "test_tool".to_string(),
                description: "A test tool".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "input": {"type": "string"}
                    },
                    "required": ["input"]
                }),
                output_schema: None,
                annotations: None,
            }],
            next_cursor: None,
        })
    }
}

struct TestResourceHandler;

#[async_trait::async_trait]
impl ResourceHandler for TestResourceHandler {
    async fn read_resource(&self, request: ReadResourceRequest) -> MCPResult<ReadResourceResponse> {
        Ok(ReadResourceResponse {
            contents: vec![ResourceContent::text(
                request.uri,
                "Test resource content".to_string()
            )],
        })
    }

    async fn list_resources(&self, _request: ListResourcesRequest) -> MCPResult<ListResourcesResponse> {
        Ok(ListResourcesResponse {
            resources: vec![Resource {
                uri: "test://resource".to_string(),
                name: "Test Resource".to_string(),
                description: Some("A test resource".to_string()),
                mime_type: Some("text/plain".to_string()),
            }],
            next_cursor: None,
        })
    }

    async fn list_resource_templates(&self, _request: ListResourceTemplatesRequest) -> MCPResult<ListResourceTemplatesResponse> {
        Ok(ListResourceTemplatesResponse {
            resource_templates: vec![],
            next_cursor: None,
        })
    }
}
```

## Real-World Examples

### 11. Weather API Server

A server that integrates with a weather API.

```rust
use ultrafast_mcp::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
struct WeatherRequest {
    city: String,
    country: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WeatherResponse {
    city: String,
    temperature: f64,
    description: String,
    humidity: u8,
    wind_speed: f64,
}

struct WeatherHandler;

#[async_trait::async_trait]
impl ToolHandler for WeatherHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult> {
        match call.name.as_str() {
            "get_weather" => {
                let args: WeatherRequest = serde_json::from_value(
                    call.arguments.unwrap_or_default()
                )?;

                // In a real implementation, you would call a weather API here
                // For this example, we'll return mock data
                let response = WeatherResponse {
                    city: args.city,
                    temperature: 22.5,
                    description: "Partly cloudy".to_string(),
                    humidity: 65,
                    wind_speed: 12.3,
                };

                let response_text = serde_json::to_string_pretty(&response)?;

                Ok(ToolResult {
                    content: vec![ToolContent::text(response_text)],
                    is_error: Some(false),
                })
            }
            _ => Err(MCPError::method_not_found(
                format!("Unknown tool: {}", call.name)
            )),
        }
    }

    async fn list_tools(&self, _request: ListToolsRequest) -> MCPResult<ListToolsResponse> {
        Ok(ListToolsResponse {
            tools: vec![Tool {
                name: "get_weather".to_string(),
                description: "Get weather information for a city".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "city": {
                            "type": "string",
                            "description": "City name"
                        },
                        "country": {
                            "type": "string",
                            "description": "Country code (optional)"
                        }
                    },
                    "required": ["city"]
                }),
                output_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "city": {"type": "string"},
                        "temperature": {"type": "number"},
                        "description": {"type": "string"},
                        "humidity": {"type": "integer"},
                        "wind_speed": {"type": "number"}
                    },
                    "required": ["city", "temperature", "description", "humidity", "wind_speed"]
                })),
                annotations: None,
            }],
            next_cursor: None,
        })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server_info = ServerInfo {
        name: "weather-server".to_string(),
        version: "1.0.0".to_string(),
        description: Some("A weather information server".to_string()),
        authors: None,
        homepage: None,
        license: None,
        repository: None,
    };

    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability { list_changed: Some(true) }),
        ..Default::default()
    };

    let server = UltraFastServer::new(server_info, capabilities)
        .with_tool_handler(Arc::new(WeatherHandler));

    server.run_stdio().await?;
    Ok(())
}
```

### 12. Database Query Server

A server that provides database query capabilities.

```rust
use ultrafast_mcp::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
struct QueryRequest {
    sql: String,
    parameters: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct QueryResponse {
    rows: Vec<serde_json::Value>,
    row_count: usize,
    execution_time_ms: u64,
}

struct DatabaseHandler;

#[async_trait::async_trait]
impl ToolHandler for DatabaseHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult> {
        match call.name.as_str() {
            "execute_query" => {
                let args: QueryRequest = serde_json::from_value(
                    call.arguments.unwrap_or_default()
                )?;

                let start = std::time::Instant::now();

                // In a real implementation, you would execute the SQL query here
                // For this example, we'll return mock data
                let mock_rows = vec![
                    json!({"id": 1, "name": "Alice", "email": "alice@example.com"}),
                    json!({"id": 2, "name": "Bob", "email": "bob@example.com"}),
                ];

                let execution_time = start.elapsed().as_millis() as u64;

                let response = QueryResponse {
                    rows: mock_rows,
                    row_count: 2,
                    execution_time_ms: execution_time,
                };

                let response_text = serde_json::to_string_pretty(&response)?;

                Ok(ToolResult {
                    content: vec![ToolContent::text(response_text)],
                    is_error: Some(false),
                })
            }
            _ => Err(MCPError::method_not_found(
                format!("Unknown tool: {}", call.name)
            )),
        }
    }

    async fn list_tools(&self, _request: ListToolsRequest) -> MCPResult<ListToolsResponse> {
        Ok(ListToolsResponse {
            tools: vec![Tool {
                name: "execute_query".to_string(),
                description: "Execute a SQL query".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "sql": {
                            "type": "string",
                            "description": "SQL query to execute"
                        },
                        "parameters": {
                            "type": "array",
                            "items": {"type": "object"},
                            "description": "Query parameters (optional)"
                        }
                    },
                    "required": ["sql"]
                }),
                output_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "rows": {
                            "type": "array",
                            "items": {"type": "object"}
                        },
                        "row_count": {"type": "integer"},
                        "execution_time_ms": {"type": "integer"}
                    },
                    "required": ["rows", "row_count", "execution_time_ms"]
                })),
                annotations: None,
            }],
            next_cursor: None,
        })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server_info = ServerInfo {
        name: "database-server".to_string(),
        version: "1.0.0".to_string(),
        description: Some("A database query server".to_string()),
        authors: None,
        homepage: None,
        license: None,
        repository: None,
    };

    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability { list_changed: Some(true) }),
        ..Default::default()
    };

    let server = UltraFastServer::new(server_info, capabilities)
        .with_tool_handler(Arc::new(DatabaseHandler));

    server.run_stdio().await?;
    Ok(())
}
```

## Running the Examples

### Building Examples

```bash
# Build all examples
cargo build --examples

# Build specific example
cargo build --example basic-echo-server
```

### Running Examples

```bash
# Run server examples
cargo run --example basic-echo-server
cargo run --example calculator-server
cargo run --example file-server

# Run client examples
cargo run --example basic-echo-client
cargo run --example interactive-client
```

### Testing with MCP Inspector

1. Build your server example:
   ```bash
   cargo build --release --example basic-echo-server
   ```

2. Start MCP Inspector:
   ```bash
   mcp-inspector
   ```

3. Connect to your server using the web interface

4. Test all available tools and resources

## Best Practices

### Error Handling

- Always provide meaningful error messages
- Use appropriate error types for different scenarios
- Include context in error responses

### Performance

- Use async/await for I/O operations
- Implement proper connection pooling
- Add monitoring for production deployments

### Security

- Validate all inputs
- Implement proper authentication for HTTP servers
- Use HTTPS in production

### Testing

- Write comprehensive unit tests
- Include integration tests
- Test error scenarios

### Documentation

- Document all tools and their parameters
- Provide clear examples
- Include usage instructions

---

This examples guide demonstrates the key features and capabilities of the UltraFast MCP framework. For more advanced usage patterns and real-world scenarios, refer to the API documentation and development guide. 