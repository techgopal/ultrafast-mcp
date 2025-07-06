# Everything MCP Server Example (Streamable HTTP)

This example demonstrates a complete MCP server implementation using Streamable HTTP transport that implements all handler traits and provides at least one tool, resource, and prompt.

## Features

- **Complete Handler Implementation**: Implements all MCP handler traits
- **Tool Handler**: Provides an `echo` tool that returns the input message
- **Resource Handler**: Provides a dummy resource for demonstration
- **Prompt Handler**: Provides a hello prompt for demonstration
- **Streamable HTTP Transport**: Uses HTTP with Server-Sent Events (SSE)
- **Monitoring Dashboard**: Includes web-based monitoring at port 8081
- **Health Checks**: Built-in health monitoring
- **Session Management**: Proper session handling for HTTP connections

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

## Testing with curl

You can test the server directly with curl:

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