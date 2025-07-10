# Basic Echo Example - Subprocess Transport

This example demonstrates how to use UltraFast MCP with **subprocess transport**, where the MCP server runs as a separate process and communicates via STDIO.

## Overview

The subprocess transport pattern is ideal for:
- **Language isolation**: Run servers in different languages/environments
- **Process isolation**: Separate server crashes don't affect the client
- **Resource management**: Independent memory and resource allocation
- **Deployment flexibility**: Deploy servers as standalone executables

## Architecture

```
┌─────────────────┐    STDIO    ┌─────────────────┐
│   MCP Client    │ ──────────► │   MCP Server    │
│   (Rust)        │             │   (Subprocess)  │
└─────────────────┘             └─────────────────┘
```

## Components

### 1. Echo Server (`basic-echo-server`)

A standalone MCP server that:
- Runs as a subprocess via STDIO
- Implements a simple `echo` tool
- Returns messages with timestamps and metadata
- Tracks echo count across calls

**Features:**
- ✅ MCP 2025-06-18 protocol compliance
- ✅ STDIO transport
- ✅ Tool implementation with validation
- ✅ Error handling and logging
- ✅ Stateful echo counter

### 2. Echo Client (`basic-echo-client`)

A client that demonstrates subprocess transport by:
- Spawning the server as a subprocess
- Establishing STDIO communication
- Calling the echo tool multiple times
- Graceful shutdown and cleanup

**Features:**
- ✅ Subprocess spawning and management
- ✅ STDIO transport setup
- ✅ Multiple tool calls
- ✅ Error handling
- ✅ Process lifecycle management

## Quick Start

### Prerequisites

- Rust toolchain (latest stable)
- UltraFast MCP workspace built

### Running the Example

1. **Build the example:**
   ```bash
   cd ultrafast-mcp/examples/01-basic-echo
   cargo build --release
   ```

2. **Run the subprocess client:**
   ```bash
   cargo run --release --bin basic-echo-client
   ```

3. **Expected output:**
   ```
   🚀 Starting Basic Echo MCP Client (Subprocess)
   🔧 Spawning echo server as subprocess...
   ✅ Server process spawned (PID: 12345)
   🔌 Connecting client to subprocess server...
   ✅ Connected to subprocess server
   📋 Listing available tools...
   Found 1 tools:
     - echo: Echo back a message with timestamp and metadata
   🔧 Calling echo tool (attempt 1)...
   📤 Echo response 1:
   {
     "message": "Hello from UltraFast MCP Client! (attempt 1)",
     "timestamp": "2024-01-15T10:30:45.123Z",
     "echo_count": 1,
     "server_id": "echo-server-12345"
   }
   🎉 Basic echo subprocess transport test completed successfully!
   ```

## API Reference

### Echo Tool

**Name:** `echo`

**Description:** Echo back a message with timestamp and metadata

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "message": {
      "type": "string",
      "description": "Message to echo back (max 1000 characters, optional - defaults to 'Hello, World!')",
      "maxLength": 1000,
      "default": "Hello, World!"
    }
  }
}
```

**Output:**
```json
{
  "message": "User's message",
  "timestamp": "2024-01-15T10:30:45.123Z",
  "echo_count": 1,
  "server_id": "echo-server-12345"
}
```

## Implementation Details

### Server Implementation

The server uses the standard UltraFast MCP server pattern:

```rust
// Create server with tool handler
let server = UltraFastServer::new(server_info, capabilities)
    .with_tool_handler(Arc::new(EchoToolHandler::new()));

// Run with STDIO transport
server.run_stdio().await?;
```

### Client Implementation

The client demonstrates subprocess transport:

```rust
// Spawn server as subprocess
let mut server_process = Command::new("cargo")
    .args(&["run", "--release", "--bin", "basic-echo-server"])
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;

// Create STDIO transport from pipes
let transport = StdioTransport::from_stdio(
    server_process.stdin.take().unwrap(),
    server_process.stdout.take().unwrap(),
    server_process.stderr.take().unwrap(),
).await?;

// Connect client to transport
client.connect(Box::new(transport)).await?;
```

## Benefits of Subprocess Transport

### 1. **Language Flexibility**
- Run servers in any language that supports STDIO
- Mix and match languages in your MCP ecosystem
- Leverage language-specific libraries and tools

### 2. **Process Isolation**
- Server crashes don't affect the client
- Independent memory management
- Separate resource allocation

### 3. **Deployment Options**
- Deploy servers as standalone executables
- Container-friendly architecture
- Easy integration with existing systems

### 4. **Development Workflow**
- Independent development and testing
- Language-specific tooling and debugging
- Clear separation of concerns

## Error Handling

The example includes comprehensive error handling:

- **Process spawning errors**: Invalid commands, missing executables
- **Transport errors**: Pipe failures, communication issues
- **Protocol errors**: Invalid messages, timeouts
- **Tool errors**: Invalid arguments, server-side failures

## Performance Considerations

- **Process startup overhead**: ~10-50ms for simple servers
- **STDIO communication**: Very low latency for local processes
- **Memory usage**: Separate memory spaces for client and server
- **Resource cleanup**: Automatic cleanup when processes exit

## Next Steps

This example provides a foundation for:

1. **Multi-language MCP servers**: Implement servers in Python, Node.js, Go, etc.
2. **Containerized deployment**: Package servers as Docker containers
3. **Load balancing**: Run multiple server instances
4. **Advanced error handling**: Implement retry logic and circuit breakers
5. **Monitoring and observability**: Add metrics and health checks

## Troubleshooting

### Common Issues

1. **Server not found**: Ensure the server binary is built and accessible
2. **Permission denied**: Check file permissions and PATH
3. **Communication errors**: Verify STDIO pipes are properly configured
4. **Protocol errors**: Check MCP protocol version compatibility

### Debug Mode

Enable debug logging:

```bash
RUST_LOG=debug cargo run --release --bin basic-echo-client
```

This will show detailed communication between client and server.

## License

MIT OR Apache-2.0 