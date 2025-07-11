# Everything MCP Server Example (Streamable HTTP)

This example demonstrates a complete MCP server implementation using Streamable HTTP transport that implements all handler traits and provides comprehensive MCP 2025-06-18 features including progress notifications, cancellation support, and resource subscriptions.

## Features

### Core MCP Implementation
- **Complete Handler Implementation**: Implements all MCP handler traits
- **Tool Handler**: Provides 11 different tools demonstrating various MCP capabilities
- **Resource Handler**: Provides 100 test resources with pagination support
- **Prompt Handler**: Provides 3 different prompts including resource-embedded prompts
- **Streamable HTTP Transport**: Uses HTTP with Server-Sent Events (SSE)
- **Monitoring Dashboard**: Includes web-based monitoring at port 8081
- **Health Checks**: Built-in health monitoring
- **Session Management**: Proper session handling for HTTP connections

### New MCP 2025-06-18 Features
- **Progress Notifications**: Long-running operations with progress tracking
- **Cancellation Support**: Cancellable operations that can be interrupted
- **Resource Subscriptions**: Subscribe to resource changes and receive notifications
- **Enhanced Completion**: Advanced completion and elicitation handlers
- **Comprehensive Notifications**: Support for all MCP notification types
- **Resource Templates**: Dynamic resource discovery with templates
- **Logging Integration**: Full logging support with level controls

### HTTP-Specific Features
- **CORS Support**: Enabled for cross-origin requests
- **Real-time Monitoring**: Web-based dashboard for server monitoring
- **Streamable Transport**: Server-Sent Events for real-time communication
- **Protocol Version**: Full MCP 2025-06-18 protocol support

## Available Tools

1. **echo** - Echoes back the input message
2. **add** - Adds two numbers together
3. **longRunningOperation** - Demonstrates progress tracking over multiple steps
4. **cancellableOperation** - Shows cancellation support with periodic checks
5. **notificationDemo** - Demonstrates various MCP notification types
6. **printEnv** - Prints environment variables for debugging
7. **sampleLLM** - Simulates LLM sampling functionality
8. **getTinyImage** - Returns a tiny test image with multiple content types
9. **annotatedMessage** - Demonstrates message annotations and metadata
10. **getResourceReference** - Returns resource references for client usage
11. **getResourceLinks** - Returns multiple resource links with descriptions

## Available Resources

- **100 Test Resources**: `test://static/resource/1` through `test://static/resource/100`
- **Pagination Support**: Resources are returned with cursor-based pagination
- **Resource Templates**: Dynamic resource discovery with `test://static/resource/{id}`
- **Content Variety**: Mix of text and binary content types

## Available Prompts

1. **simple_prompt** - A basic prompt without arguments
2. **complex_prompt** - A prompt with temperature and style arguments
3. **resource_prompt** - A prompt that embeds resource references

## Running the Server

```bash
# Build the server
cargo build --release

# Run the server (will start on 0.0.0.0:8080)
./target/release/server
```

The server will start on:
- **MCP Server**: `http://127.0.0.1:8080`
- **Monitoring Dashboard**: `http://127.0.0.1:8081`

## Running the Client

```bash
# Build the client
cargo build --release

# Run the client (will connect to server via HTTP)
./target/release/client
```

## Testing with MCP Inspector

You can test this server with the MCP Inspector by configuring it to connect to:
```
http://127.0.0.1:8080
```

### Using MCP Inspector with Config File

You can also launch the MCP Inspector with a configuration that connects to the HTTP server:

```bash
npx @modelcontextprotocol/inspector --config mcp-inspector-config.json --server everything-server-streamable-http
```

### Testing Progress and Cancellation

1. **Progress Tracking**:
   - Use the `longRunningOperation` tool with custom duration and steps
   - Monitor progress messages in the server console
   - Observe real-time progress updates

2. **Cancellation Support**:
   - Use the `cancellableOperation` tool with long duration
   - Send cancellation requests during execution
   - See how the operation handles cancellation gracefully

3. **Notification Demo**:
   - Use the `notificationDemo` tool with different notification types
   - Observe how various MCP notifications would be sent
   - Test resource subscription and update notifications

## Testing with curl

You can test the server directly with curl for HTTP requests:

```bash
# Initialize a session
curl -X POST http://127.0.0.1:8080 \
  -H "Content-Type: application/json" \
  -H "X-MCP-Session-ID: test-session" \
  -d '{
    "jsonrpc": "2.0",
    "method": "initialize",
    "params": {
      "capabilities": {},
      "clientInfo": {"name": "test-client", "version": "1.0.0"},
      "protocolVersion": "2025-06-18"
    },
    "id": 1
  }'

# List tools
curl -X POST http://127.0.0.1:8080 \
  -H "Content-Type: application/json" \
  -H "X-MCP-Session-ID: test-session" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/list",
    "params": {},
    "id": 2
  }'

# Call echo tool
curl -X POST http://127.0.0.1:8080 \
  -H "Content-Type: application/json" \
  -H "X-MCP-Session-ID: test-session" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
      "name": "echo",
      "arguments": {"message": "Hello from curl!"}
    },
    "id": 3
  }'
```

## Handler Types Implemented

1. **ToolHandler**: `EchoToolHandler` - Provides an echo tool
2. **ResourceHandler**: `DummyResourceHandler` - Provides dummy resources
3. **PromptHandler**: `DummyPromptHandler` - Provides hello prompts
4. **SamplingHandler**: `DummySamplingHandler` - Stub implementation
5. **CompletionHandler**: `DummyCompletionHandler` - Stub implementation
6. **RootsHandler**: `DummyRootsHandler` - Stub implementation
7. **ElicitationHandler**: `DummyElicitationHandler` - Stub implementation
8. **ResourceSubscriptionHandler**: `DummySubscriptionHandler` - Stub implementation

## Available Tools

- **echo**: Returns the input message (takes a `message` parameter)

## Available Resources

- **file:///dummy.txt**: A dummy text resource

## Available Prompts

- **hello-prompt**: A simple hello prompt

## HTTP Endpoints

- **POST /**: JSON-RPC messages (direct calls)
- **GET /**: Server-Sent Events (SSE) for streaming
- **DELETE /**: Session termination

## Architecture

This example shows how to:
- Implement all MCP handler traits
- Use the builder pattern for server configuration
- Handle tool calls with proper argument parsing
- Provide resources and prompts
- Use Streamable HTTP transport for communication
- Enable monitoring and health checks
- Handle session management
- Support both direct JSON-RPC and SSE streaming

The server demonstrates the complete MCP protocol implementation with all handler types over HTTP, making it a comprehensive example for understanding the full MCP server capabilities with modern web transport. 