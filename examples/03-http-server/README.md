# HTTP Server Example

This example demonstrates the new `UltraFastServer` and `UltraFastClient` APIs with comprehensive HTTP operations.

## Overview

The example consists of:
- **Server**: An MCP server that provides HTTP operations (GET, POST, status checks, info)
- **Client**: A client that connects to the server and tests various HTTP operations

## Features Demonstrated

- Creating an MCP server using `UltraFastServer` with HTTP tools
- Implementing HTTP client functionality with the `ToolHandler` trait
- HTTP operations: GET, POST, status checks, info retrieval
- Custom headers and timeout handling
- Error handling for network operations
- Creating an MCP client using `UltraFastClient`
- Testing HTTP operations with real APIs
- Response time tracking and metadata

## Running the Example

### 1. Build the Example

```bash
cd examples/03-http-server
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

The client will connect to the server and perform various HTTP operations.

## Expected Output

### Server Output
```
🚀 Starting HTTP Server
✅ Server created successfully
📡 Starting server on stdio transport
📨 Received tool call: http_get
🌐 Making HTTP GET request to: https://jsonplaceholder.typicode.com/posts/1
✅ HTTP GET completed in 245ms with status 200
📨 Received tool call: http_post
🌐 Making HTTP POST request to: https://jsonplaceholder.typicode.com/posts
✅ HTTP POST completed in 312ms with status 201
```

### Client Output
```
🚀 Starting HTTP Client
✅ Client created successfully
🔗 Connected to HTTP server
📋 Listing available tools...
🔧 Tool: http_get - Perform an HTTP GET request
🔧 Tool: http_post - Perform an HTTP POST request
🔧 Tool: http_status - Check the status of an HTTP endpoint
🔧 Tool: http_info - Get information about an HTTP request
🌐 Testing http_get tool with JSONPlaceholder API...
📥 HTTP GET response received:
📄 Response: {"url":"https://jsonplaceholder.typicode.com/posts/1","status_code":200,"headers":{"content-type":"application/json; charset=utf-8",...},"body":"{\"userId\":1,\"id\":1,\"title\":\"sunt aut facere repellat provident occaecati excepturi optio reprehenderit\",\"body\":\"quia et suscipit suscipit recusandae consequuntur expedita et cum reprehenderit molestiae ut ut quas totam nostrum rerum est autem sunt rem eveniet architecto\"}","response_time_ms":245}
🌐 Testing http_post tool with JSONPlaceholder API...
📥 HTTP POST response received:
📄 Response: {"url":"https://jsonplaceholder.typicode.com/posts","status_code":201,"headers":{"content-type":"application/json; charset=utf-8",...},"body":"{\"title\":\"Test Post\",\"body\":\"This is a test post from UltraFastClient\",\"userId\":1,\"id\":101}","response_time_ms":312}
🔍 Testing http_status tool with Google...
📥 HTTP status response received:
📄 Response: {"url":"https://www.google.com","status_code":200,"is_online":true,"response_time_ms":156,"timestamp":"2024-01-01T12:00:00Z"}
✅ All HTTP operations tests completed successfully!
```

## Code Structure

### Server (`src/server.rs`)

The server demonstrates:
- Creating an `UltraFastServer` with HTTP operation capabilities
- Implementing a comprehensive `ToolHandler` with HTTP tools
- HTTP client operations with proper error handling
- Response time tracking and metadata extraction
- Custom headers and timeout support

### Client (`src/client.rs`)

The client demonstrates:
- Creating an `UltraFastClient` with stdio transport
- Testing various HTTP operations with real APIs
- Custom headers and timeout testing
- Error handling for network failures
- End-to-end HTTP workflow testing

## Available Tools

### 1. `http_get`
Performs an HTTP GET request.

**Parameters:**
- `url` (string, required): URL to make the GET request to
- `headers` (object, optional): Custom headers to include in the request
- `timeout` (integer, optional): Request timeout in seconds (default: 30)

**Response:**
```json
{
  "url": "https://example.com",
  "status_code": 200,
  "headers": {
    "content-type": "application/json",
    "server": "nginx"
  },
  "body": "Response body content",
  "response_time_ms": 245
}
```

### 2. `http_post`
Performs an HTTP POST request.

**Parameters:**
- `url` (string, required): URL to make the POST request to
- `headers` (object, optional): Custom headers to include in the request
- `body` (string, optional): Body content for the POST request
- `timeout` (integer, optional): Request timeout in seconds (default: 30)

**Response:**
```json
{
  "url": "https://example.com/api",
  "status_code": 201,
  "headers": {
    "content-type": "application/json",
    "location": "/api/resource/123"
  },
  "body": "Response body content",
  "response_time_ms": 312
}
```

### 3. `http_status`
Checks the status of an HTTP endpoint.

**Parameters:**
- `url` (string, required): URL to check the status of

**Response:**
```json
{
  "url": "https://example.com",
  "status_code": 200,
  "is_online": true,
  "response_time_ms": 156,
  "timestamp": "2024-01-01T12:00:00Z"
}
```

### 4. `http_info`
Gets information about an HTTP request using HEAD method.

**Parameters:**
- `url` (string, required): URL to get information about

**Response:**
```json
{
  "url": "https://example.com",
  "method": "HEAD",
  "headers": {
    "content-type": "text/html",
    "content-length": "1234",
    "server": "nginx"
  },
  "body_size": 1234,
  "timestamp": "2024-01-01T12:00:00Z"
}
```

## Key API Usage

### Server Creation with HTTP Tools
```rust
let server = UltraFastServer::new(
    ServerInfo { /* ... */ },
    ServerCapabilities {
        tools: Some(ToolsCapability { list_changed: Some(true) }),
        resources: Some(ResourcesCapability { /* ... */ }),
        ..Default::default()
    }
)
.with_tool_handler(Arc::new(HttpOperationsHandler))
.build()?;
```

### HTTP Tool Handler Implementation
```rust
#[async_trait::async_trait]
impl ultrafast_mcp::ToolHandler for HttpOperationsHandler {
    async fn handle_tool_call(&self, call: ultrafast_mcp::ToolCall) -> ultrafast_mcp::McpResult<ultrafast_mcp::ToolResult> {
        match call.name.as_str() {
            "http_get" => self.handle_http_get(request).await,
            "http_post" => self.handle_http_post(request).await,
            "http_status" => self.handle_http_status(request).await,
            "http_info" => self.handle_http_info(request).await,
            _ => Err(ultrafast_mcp::McpError::method_not_found(format!("Unknown tool: {}", call.name))),
        }
    }
}
```

### Client HTTP Operations
```rust
// HTTP GET request
let get_result = client.call_tool("http_get", json!({
    "url": "https://api.example.com/data",
    "headers": {
        "Authorization": "Bearer token123",
        "Accept": "application/json"
    },
    "timeout": 30
})).await?;

// HTTP POST request
let post_result = client.call_tool("http_post", json!({
    "url": "https://api.example.com/create",
    "headers": {
        "Content-Type": "application/json"
    },
    "body": r#"{"name": "Test", "value": 123}"#,
    "timeout": 30
})).await?;

// HTTP status check
let status_result = client.call_tool("http_status", json!({
    "url": "https://example.com"
})).await?;

// HTTP info retrieval
let info_result = client.call_tool("http_info", json!({
    "url": "https://example.com"
})).await?;
```

## Error Handling

The example demonstrates comprehensive error handling:
- Network connection failures
- Timeout errors
- Invalid URLs
- HTTP error status codes
- Response parsing errors

## Network Considerations

The HTTP server includes several network features:
- Configurable timeouts
- Custom header support
- Response time tracking
- Status code handling
- Error recovery

## Security Considerations

The HTTP server includes basic safety measures:
- URL validation
- Timeout limits
- Error message sanitization
- Request size limits (can be extended)

## Testing with Real APIs

The client demonstrates testing with various public APIs:
- **JSONPlaceholder**: For testing GET and POST operations
- **Google**: For testing status checks
- **GitHub API**: For testing info retrieval
- **HTTPBin**: For testing custom headers and responses

This example provides a comprehensive demonstration of building an MCP server with HTTP client capabilities using the new ergonomic APIs. 