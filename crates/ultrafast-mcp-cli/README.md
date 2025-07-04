# ultrafast-mcp-cli

Command-line interface for ULTRAFAST MCP.

This crate provides a comprehensive CLI tool for managing and interacting with MCP servers and clients.

## Features

- **Server Management**: Start, stop, and manage MCP servers
- **Client Operations**: Connect to and interact with MCP servers
- **Development Tools**: Code generation and validation utilities
- **Configuration**: Manage server and client configurations
- **Monitoring**: Real-time monitoring and debugging
- **Authentication**: OAuth and token-based authentication support

## Installation

```bash
cargo install ultrafast-mcp-cli
```

## Usage

### Basic Commands

```bash
# Start a server
mcp server start --config server.toml

# Connect to a server
mcp client connect http://localhost:8080

# List available resources
mcp client resources

# Generate code from schema
mcp generate --schema schema.json --output src/generated.rs
```

### Development Commands

```bash
# Validate configuration
mcp validate config.toml

# Run tests
mcp test --server test-server

# Monitor server performance
mcp monitor --server http://localhost:8080
```

## Features

- `auth` - Enables OAuth authentication support
- `monitoring` - Enables monitoring and metrics collection

## Dependencies

- `clap` - Command-line argument parsing
- `tokio` - Async runtime
- `serde` - Configuration serialization
- `tracing` - Logging and tracing

## License

MIT OR Apache-2.0 