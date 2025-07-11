# Basic Echo Example - Transport Choice Demo

This example demonstrates UltraFast MCP's flexible transport layer by implementing a simple echo server and client that can work with both STDIO and Streamable HTTP transports.

## Features

- **Dual Transport Support**: Choose between STDIO (subprocess) and Streamable HTTP (network) transports
- **Command Line Interface**: Easy-to-use CLI with transport selection
- **Comprehensive Demo**: Shows all transport options in action
- **Real-world Patterns**: Demonstrates proper server/client lifecycle management

## Quick Start

### Run the Comprehensive Demo

```bash
# Demo both transports
cargo run --bin basic-echo-demo

# Demo STDIO transport only
cargo run --bin basic-echo-demo -- stdio

# Demo HTTP transport only
cargo run --bin basic-echo-demo -- http
```

### Run Individual Components

#### Server

```bash
# Start STDIO server (subprocess mode)
cargo run --bin basic-echo-server -- stdio

# Start HTTP server (network mode)
cargo run --bin basic-echo-server -- http --host 127.0.0.1 --port 8080
```

#### Client

```bash
# Connect to STDIO server (spawns server automatically)
cargo run --bin basic-echo-client -- stdio --spawn-server

# Connect to HTTP server
cargo run --bin basic-echo-client -- http --url http://127.0.0.1:8080
```

## Transport Comparison

| Feature | STDIO Transport | Streamable HTTP Transport |
|---------|----------------|---------------------------|
| **Use Case** | Subprocess communication | Network communication |
| **Connection** | Parent-child process | Client-server over network |
| **Performance** | Very fast (no network overhead) | Network-dependent |
| **Deployment** | Local execution | Remote/cloud deployment |
| **Security** | Process isolation | Network security required |
| **Scalability** | Single client per server | Multiple clients per server |

## Architecture

### Server Components

- **EchoToolHandler**: Implements the echo tool with transport awareness
- **Transport Selection**: Command-line argument parsing for transport choice
- **Configuration**: Transport-specific setup (STDIO vs HTTP)

### Client Components

- **Transport Connection**: Automatic transport detection and connection
- **Server Management**: Optional subprocess spawning for STDIO
- **Error Handling**: Robust error handling for both transports

### Demo Components

- **Comprehensive Testing**: Tests all transport combinations
- **Lifecycle Management**: Proper server/client startup/shutdown
- **Real-world Scenarios**: Demonstrates practical usage patterns

## Code Examples

### Server Implementation

```rust
// Transport selection via CLI
let args = Args::parse();

// Run with chosen transport
match args.transport {
    TransportType::Stdio => {
        server.run_stdio().await?;
    }
    TransportType::Http => {
        let config = HttpTransportConfig { /* ... */ };
        server.run_streamable_http_with_config(config).await?;
    }
}
```

### Client Implementation

```rust
// Connect based on transport type
match args.transport {
    TransportType::Stdio => {
        client.connect_stdio().await?;
    }
    TransportType::Http => {
        client.connect_streamable_http(&args.url).await?;
    }
}
```

## Echo Tool

The example implements a simple echo tool that:

- **Accepts Messages**: Takes a message parameter (optional, defaults to "Hello, World!")
- **Validates Input**: Ensures message is not empty and under 1000 characters
- **Adds Metadata**: Includes timestamp, echo counter, server ID, and transport type
- **Returns JSON**: Structured response with all metadata

### Tool Schema

```json
{
  "name": "echo",
  "description": "Echo back a message with timestamp and metadata",
  "input_schema": {
    "type": "object",
    "properties": {
      "message": {
        "type": "string",
        "description": "Message to echo back (max 1000 characters, optional)",
        "maxLength": 1000,
        "default": "Hello, World!"
      }
    }
  }
}
```

### Example Response

```json
{
  "message": "Hello from UltraFast MCP!",
  "timestamp": "2024-01-15T10:30:00Z",
  "echo_count": 42,
  "server_id": "echo-server-12345",
  "transport": "Http"
}
```

## Development

### Building

```bash
# Build all components
cargo build --release

# Build specific component
cargo build --release --bin basic-echo-server
cargo build --release --bin basic-echo-client
cargo build --release --bin basic-echo-demo
```

### Testing

```bash
# Run the comprehensive demo
cargo run --bin basic-echo-demo

# Test individual transports
cargo run --bin basic-echo-demo -- stdio
cargo run --bin basic-echo-demo -- http
```

### Debugging

```bash
# Enable debug logging
RUST_LOG=debug cargo run --bin basic-echo-server -- stdio

# Run with specific log level
RUST_LOG=ultrafast_mcp=debug cargo run --bin basic-echo-client -- http
```

## Integration Examples

### With External Tools

```bash
# Use with curl (HTTP transport)
curl -X POST http://127.0.0.1:8080/tools/call \
  -H "Content-Type: application/json" \
  -d '{"name": "echo", "arguments": {"message": "Hello from curl!"}}'

# Use with subprocess (STDIO transport)
echo '{"jsonrpc": "2.0", "method": "tools/call", "params": {"name": "echo", "arguments": {"message": "Hello!"}}}' | \
  cargo run --bin basic-echo-server -- stdio
```

### With Other MCP Clients

This server is compatible with any MCP client that supports STDIO or Streamable HTTP transports, including:

- Claude Desktop
- MCP Inspector
- Custom MCP clients

## Troubleshooting

### Common Issues

1. **Port Already in Use**: Change the port with `--port 8081`
2. **Permission Denied**: Ensure you have permission to bind to the specified port
3. **Connection Refused**: Make sure the server is running before connecting the client
4. **Subprocess Spawn Failed**: Ensure the server binary is built and accessible

### Debug Commands

```bash
# Check if server is running (HTTP)
curl http://127.0.0.1:8080/health

# Check server logs
RUST_LOG=debug cargo run --bin basic-echo-server -- http 2>&1 | tee server.log

# Test connection manually
nc -v 127.0.0.1 8080
```

## Next Steps

After understanding this example, explore:

1. **Advanced Examples**: Check out other examples in the `examples/` directory
2. **Custom Tools**: Implement your own tools following the same pattern
3. **Authentication**: Add OAuth or API key authentication
4. **Monitoring**: Enable metrics and health checks
5. **Production Deployment**: Configure for production environments

## Contributing

This example serves as a foundation for understanding UltraFast MCP's transport layer. Feel free to:

- Add new transport types
- Implement more complex tools
- Add authentication examples
- Improve error handling
- Add performance benchmarks

## License

This example is part of the UltraFast MCP project and is licensed under the same terms as the main project. 