# Advanced Features Example

This example demonstrates the new `UltraFastServer` and `UltraFastClient` APIs with comprehensive MCP features: tools, resources, and prompts.

## Overview

The example consists of:
- **Server**: An MCP server that provides advanced tools, resources, and prompts
- **Client**: A client that connects to the server and tests all advanced features

## Features Demonstrated

- Creating an MCP server using `UltraFastServer` with multiple capabilities
- Implementing comprehensive tool handlers with the `ToolHandler` trait
- Implementing resource handlers with the `ResourceHandler` trait
- Implementing prompt handlers with the `PromptHandler` trait
- Advanced data generation and processing tools
- Text analysis with sentiment detection
- Dynamic resource serving
- Prompt generation with arguments
- Creating an MCP client using `UltraFastClient`
- Testing all MCP features end-to-end
- Error handling and validation

## Running the Example

### 1. Build the Example

```bash
cd examples/04-advanced-features
cargo build
```

### 2. Run the Server

In one terminal:

```bash
cargo run --bin server
```

The server will start and wait for connections on stdio.

### 3. Run the Client

In another terminal:

```bash
cargo run --bin client
```

The client will connect to the server and test all advanced features.

## Expected Output

### Server Output
```
🚀 Starting Advanced Features Server
✅ Server created successfully
📡 Starting server on stdio transport
📨 Received tool call: generate_data
🔢 Generating data: count=5, type=Some("mixed")
✅ Generated 5 items of type mixed
📨 Received tool call: process_data
⚙️ Processing data: operation=sum, count=10
✅ Processed data in 2ms
📨 Received tool call: analyze_text
📝 Analyzing text: type=Some("full"), length=156
✅ Analyzed text: 25 words, 156 characters
```

### Client Output
```
🚀 Starting Advanced Features Client
✅ Client created successfully
🔗 Connected to advanced features server
📋 Listing available tools...
🔧 Tool: generate_data - Generate sample data of various types
🔧 Tool: process_data - Process data with various operations
🔧 Tool: analyze_text - Analyze text content with various metrics
🔢 Testing generate_data tool...
📥 Generate data response received:
📄 Response: {"data":[{"type":"number","value":42},{"type":"string","value":"Text 1"},...],"count":5,"data_type":"mixed","generated_at":"2024-01-01T12:00:00Z","seed":12345}
⚙️ Testing process_data tool...
📥 Process data response received:
📄 Response: {"result":{"sum":55.0,"count":10},"operation":"sum","input_count":10,"processed_at":"2024-01-01T12:00:01Z","processing_time_ms":2}
📝 Testing analyze_text tool...
📥 Analyze text response received:
📄 Response: {"text":"This is a wonderful example...","word_count":25,"character_count":156,"analysis_type":"full","sentiment":"positive","keywords":["wonderful","example","analysis","features","capabilities"],"analysis_time":"2024-01-01T12:00:02Z"}
📄 Listing available resources...
📄 Resource: server://status - Current server status and information
📄 Resource: server://info - Detailed server information and capabilities
📄 Resource: data://sample - Sample data for testing and demonstration
💬 Listing available prompts...
💬 Prompt: greeting - Generate a friendly greeting
💬 Prompt: analysis - Generate text analysis prompts
💬 Prompt: generation - Generate data generation prompts
✅ All advanced features tests completed successfully!
```

## Code Structure

### Server (`src/server.rs`)

The server demonstrates:
- Creating an `UltraFastServer` with comprehensive capabilities
- Implementing multiple trait handlers: `ToolHandler`, `ResourceHandler`, `PromptHandler`
- Advanced data processing and generation
- Text analysis with sentiment detection
- Dynamic resource serving
- Prompt generation with argument handling

### Client (`src/client.rs`)

The client demonstrates:
- Creating an `UltraFastClient` with stdio transport
- Testing all MCP features systematically
- Tool calling with complex arguments
- Resource reading and listing
- Prompt generation and retrieval
- Error handling and validation

## Available Tools

### 1. `generate_data`
Generates sample data of various types.

**Parameters:**
- `count` (integer, optional): Number of items to generate (default: 10)
- `data_type` (string, optional): Type of data to generate - "numbers", "strings", "objects", "mixed" (default: mixed)
- `seed` (integer, optional): Random seed for reproducible generation

**Response:**
```json
{
  "data": [
    {"type": "number", "value": 42},
    {"type": "string", "value": "Text 1"},
    {"type": "boolean", "value": true},
    {"type": "object", "value": {"id": 1, "name": "Object 1"}}
  ],
  "count": 4,
  "data_type": "mixed",
  "generated_at": "2024-01-01T12:00:00Z",
  "seed": 12345
}
```

### 2. `process_data`
Processes data with various operations.

**Parameters:**
- `data` (array, required): Array of data to process
- `operation` (string, required): Operation to perform - "sum", "average", "count", "filter", "sort"
- `parameters` (object, optional): Additional parameters for the operation

**Response:**
```json
{
  "result": {"sum": 55.0, "count": 10},
  "operation": "sum",
  "input_count": 10,
  "processed_at": "2024-01-01T12:00:01Z",
  "processing_time_ms": 2
}
```

### 3. `analyze_text`
Analyzes text content with various metrics.

**Parameters:**
- `text` (string, required): Text to analyze
- `analysis_type` (string, optional): Type of analysis - "basic", "sentiment", "keywords", "full" (default: basic)

**Response:**
```json
{
  "text": "This is a wonderful example...",
  "word_count": 25,
  "character_count": 156,
  "analysis_type": "full",
  "sentiment": "positive",
  "keywords": ["wonderful", "example", "analysis"],
  "analysis_time": "2024-01-01T12:00:02Z"
}
```

## Available Resources

### 1. `server://status`
Current server status and information.

**Content:**
```json
{
  "server_name": "Advanced Features Server",
  "status": "running",
  "uptime": 1703875200,
  "version": "1.0.0",
  "features": ["tools", "resources", "prompts"]
}
```

### 2. `server://info`
Detailed server information and capabilities.

**Content:**
```json
{
  "name": "Advanced Features Server",
  "description": "Demonstrates advanced MCP features",
  "capabilities": {
    "tools": true,
    "resources": true,
    "prompts": true
  },
  "available_tools": ["generate_data", "process_data", "analyze_text"],
  "available_resources": ["server://status", "server://info", "data://sample"],
  "available_prompts": ["greeting", "analysis", "generation"]
}
```

### 3. `data://sample`
Sample data for testing and demonstration.

**Content:**
```json
{
  "items": [
    {"id": 1, "name": "Sample Item 1", "value": 100},
    {"id": 2, "name": "Sample Item 2", "value": 200},
    {"id": 3, "name": "Sample Item 3", "value": 300}
  ],
  "metadata": {
    "generated_at": "2024-01-01T12:00:00Z",
    "count": 3
  }
}
```

## Available Prompts

### 1. `greeting`
Generate a friendly greeting.

**Arguments:**
- `name` (string, optional): Name of the person to greet

**Messages:**
```
[system]: You are a helpful assistant that provides friendly greetings.
[user]: Please greet UltraFastClient in a friendly way.
```

### 2. `analysis`
Generate text analysis prompts.

**Arguments:**
- `text` (string, optional): Text to analyze

**Messages:**
```
[system]: You are an expert text analyst. Provide detailed analysis of the given text.
[user]: Please analyze this text: 'This is a sample text for analysis.'
```

### 3. `generation`
Generate data generation prompts.

**Arguments:**
- `count` (integer, optional): Number of items to generate
- `type` (string, optional): Type of data to generate

**Messages:**
```
[system]: You are a data generation expert. Generate sample data based on the user's request.
[user]: Please generate 3 items of numbers data.
```

## Key API Usage

### Server Creation with Multiple Handlers
```rust
let server = UltraFastServer::new(
    ServerInfo { /* ... */ },
    ServerCapabilities {
        tools: Some(ToolsCapability { list_changed: Some(true) }),
        resources: Some(ResourcesCapability { 
            list_changed: Some(true),
            subscribe: Some(true)
        }),
        prompts: Some(PromptsCapability { list_changed: Some(true) }),
        ..Default::default()
    }
)
.with_tool_handler(Arc::new(AdvancedFeaturesHandler))
.with_resource_handler(Arc::new(AdvancedFeaturesResourceHandler))
.with_prompt_handler(Arc::new(AdvancedFeaturesPromptHandler))
.build()?;
```

### Multiple Trait Implementations
```rust
#[async_trait::async_trait]
impl ultrafast_mcp::ToolHandler for AdvancedFeaturesHandler {
    async fn handle_tool_call(&self, call: ultrafast_mcp::ToolCall) -> ultrafast_mcp::McpResult<ultrafast_mcp::ToolResult> {
        match call.name.as_str() {
            "generate_data" => self.handle_generate_data(request).await,
            "process_data" => self.handle_process_data(request).await,
            "analyze_text" => self.handle_analyze_text(request).await,
            _ => Err(ultrafast_mcp::McpError::method_not_found(format!("Unknown tool: {}", call.name))),
        }
    }
}

#[async_trait::async_trait]
impl ultrafast_mcp::ResourceHandler for AdvancedFeaturesResourceHandler {
    async fn read_resource(&self, request: ultrafast_mcp::ReadResourceRequest) -> ultrafast_mcp::McpResult<ultrafast_mcp::ReadResourceResponse> {
        match request.uri.as_str() {
            "server://status" => self.handle_server_status().await,
            "server://info" => self.handle_server_info().await,
            "data://sample" => self.handle_sample_data().await,
            _ => Err(ultrafast_mcp::McpError::not_found(format!("Resource not found: {}", request.uri))),
        }
    }
}

#[async_trait::async_trait]
impl ultrafast_mcp::PromptHandler for AdvancedFeaturesPromptHandler {
    async fn get_prompt(&self, request: ultrafast_mcp::GetPromptRequest) -> ultrafast_mcp::McpResult<ultrafast_mcp::GetPromptResponse> {
        match request.name.as_str() {
            "greeting" => self.handle_greeting_prompt(request).await,
            "analysis" => self.handle_analysis_prompt(request).await,
            "generation" => self.handle_generation_prompt(request).await,
            _ => Err(ultrafast_mcp::McpError::not_found(format!("Prompt not found: {}", request.name))),
        }
    }
}
```

### Client Advanced Operations
```rust
// Tool calling with complex arguments
let generate_result = client.call_tool("generate_data", json!({
    "count": 5,
    "data_type": "mixed",
    "seed": 12345
})).await?;

// Resource reading
let status_result = client.read_resource("server://status").await?;

// Prompt generation
let greeting_result = client.get_prompt("greeting", json!({
    "name": "UltraFastClient"
})).await?;
```

## Error Handling

The example demonstrates comprehensive error handling:
- Invalid tool arguments
- Non-existent resources
- Non-existent prompts
- Data processing errors
- Network/connection errors

## Advanced Features

### Data Generation
- Multiple data types (numbers, strings, objects, mixed)
- Reproducible generation with seeds
- Configurable output counts

### Data Processing
- Multiple operations (sum, average, count, filter, sort)
- Performance timing
- Error handling for invalid operations

### Text Analysis
- Basic metrics (word count, character count)
- Sentiment analysis
- Keyword extraction
- Configurable analysis types

### Resource Management
- Dynamic resource generation
- JSON content serving
- Metadata inclusion

### Prompt Generation
- Context-aware prompts
- Argument handling
- System and user message generation

This example provides a comprehensive demonstration of building a full-featured MCP server with all major capabilities using the new ergonomic APIs. 