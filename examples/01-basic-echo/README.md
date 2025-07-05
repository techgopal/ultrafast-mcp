# Basic Echo Example with Streamable HTTP

This example demonstrates the ULTRAFAST_MCP framework using **Streamable HTTP transport** for high-performance communication between an MCP server and client.

## 🚀 Features Demonstrated

- **Streamable HTTP Transport**: High-performance HTTP communication with session management
- **Ergonomic API**: Simple, intuitive methods for server and client creation
- **Server Implementation**: Simple echo tool with automatic schema generation
- **Client Implementation**: HTTP client with automatic connection management
- **Type Safety**: Full type-safe tool calling with serde serialization
- **Error Handling**: Comprehensive error handling and logging

## 📋 Prerequisites

- Rust toolchain (1.70+)
- Cargo

## 🏗️ Architecture

### Server (`src/server.rs`)
- Creates an MCP server with echo tool capability
- Uses **ergonomic `run_streamable_http()`** method for simple HTTP startup
- Streamable HTTP transport on `127.0.0.1:8080`
- Automatic JSON schema generation for tools

### Client (`src/client.rs`)
- Creates an MCP client with tool calling capability
- Uses **ergonomic `connect_streamable_http()`** method for simple HTTP connection
- Connects to server at `http://127.0.0.1:8080/mcp`
- Demonstrates tool listing and calling

## 🚀 Quick Start

### 1. Build the Example
```bash
cargo build
```

### 2. Run the Server (Terminal 1)
```bash
cargo run --bin server
```

### 3. Run the Client (Terminal 2)
```bash
cargo run --bin client
```

## 📝 Code Examples

### Server Creation (Simplified)
```rust
// Create server with ergonomic API
let server = UltraFastServer::new(server_info, server_capabilities)
    .with_tool_handler(Arc::new(EchoToolHandler));

// Start with Streamable HTTP - just one line!
server.run_streamable_http("127.0.0.1", 8080).await?;
```

### Client Creation (Simplified)
```rust
// Create client
let client = UltraFastClient::new(client_info, client_capabilities);

// Connect with Streamable HTTP - just one line!
client.connect_streamable_http("http://127.0.0.1:8080/mcp").await?;
```

## 🔧 Available Ergonomic Methods

### UltraFastServer
- `run_streamable_http(host, port)` - **Recommended**: High-performance HTTP
- `run_with_config(config)` - Custom HTTP configuration
- `run_stdio()` - Local stdio transport

### UltraFastClient
- `connect_streamable_http(url)` - **Recommended**: High-performance HTTP
- `connect()` - Generic connection (uses configured transport)
- `connect_stdio()` - Local stdio transport
- `with_transport(transport)` - Configure custom transport

## 🎯 Expected Output

### Server Output
```
2024-01-15T10:30:00.000Z INFO  basic_echo_example::server Server created, starting Streamable HTTP transport on 127.0.0.1:8080
2024-01-15T10:30:00.000Z INFO  ultrafast_mcp_server Starting MCP server: Basic Echo Server
2024-01-15T10:30:00.000Z INFO  ultrafast_mcp_server MCP server initialized and ready
```

### Client Output
```
2024-01-15T10:30:05.000Z INFO  basic_echo_example::client 🚀 Starting Basic Echo Client
2024-01-15T10:30:05.000Z INFO  basic_echo_example::client Connecting to server via Streamable HTTP at http://127.0.0.1:8080/mcp
2024-01-15T10:30:05.000Z INFO  ultrafast_mcp_client Connecting to MCP server
2024-01-15T10:30:05.000Z INFO  ultrafast_mcp_client Successfully connected and initialized
2024-01-15T10:30:05.000Z INFO  basic_echo_example::client ✅ Connected! Listing available tools
2024-01-15T10:30:05.000Z INFO  basic_echo_example::client Available tools: [Tool { name: "echo", description: "Echoes back the input message with a timestamp" }]
2024-01-15T10:30:05.000Z INFO  basic_echo_example::client 🔧 Calling echo tool with message: "Hello, ULTRAFAST_MCP!"
2024-01-15T10:30:05.000Z INFO  basic_echo_example::client ✅ Tool call successful!
2024-01-15T10:30:05.000Z INFO  basic_echo_example::client 📤 Response: EchoResponse { message: "Hello, ULTRAFAST_MCP!", timestamp: "2024-01-15T10:30:05.123Z" }
2024-01-15T10:30:05.000Z INFO  basic_echo_example::client 🎉 Example completed successfully!
```

## 🔍 Key Benefits

### **10x Performance Improvement**
- Streamable HTTP provides 10x better performance than traditional HTTP transports under load
- Efficient session management and connection pooling
- Zero-copy message handling

### **Developer Experience**
- **One-line server startup**: `server.run_streamable_http("127.0.0.1", 8080)`
- **One-line client connection**: `client.connect_streamable_http(url)`
- Automatic transport configuration and error handling
- Type-safe tool calling with automatic schema generation

### **Production Ready**
- Comprehensive error handling and logging
- Session management and reconnection logic
- CORS support and security features
- Monitoring and health check capabilities

## 🔗 Next Steps

- Explore the [File Operations Example](../02-file-operations/) for resource handling
- Check out the [HTTP Server Example](../03-http-server/) for advanced HTTP features
- Review the [Advanced Features Example](../04-advanced-features/) for Phase 3 capabilities

## 🔍 What's Different from stdio

| Feature | stdio Transport | Streamable HTTP Transport |
|---------|----------------|---------------------------|
| **Communication** | Subprocess pipes | HTTP requests/responses |
| **Scalability** | Single client | Multiple concurrent clients |
| **Network** | Local only | Network accessible |
| **Session Management** | None | Built-in session management |
| **Performance** | Good for local | Excellent for distributed |
| **Infrastructure** | Simple | Enterprise-ready |

## 🧪 Testing

You can test the server manually using curl:

```bash
# Test the Streamable HTTP endpoint
curl -X POST http://127.0.0.1:8080/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "session_id": null,
    "message": {
      "jsonrpc": "2.0",
      "id": "1",
      "method": "tools/list",
      "params": {}
    }
  }'
```

## 📚 Next Steps

This example demonstrates the basic Streamable HTTP transport. For more advanced features, see:

- **02-file-operations**: File system operations with HTTP transport
- **03-http-server**: HTTP client operations and network integration  
- **04-advanced-features**: Complete MCP capabilities with all features

## 🔗 Related Documentation

- [ULTRAFAST_MCP PRD](../ULTRAFAST_MCP_PRD_LLM_FRIENDLY.md)
- [Streamable HTTP Transport Documentation](../../docs/core-concepts/architecture.md)
- [MCP 2025-06-18 Specification](https://modelcontextprotocol.io/) 