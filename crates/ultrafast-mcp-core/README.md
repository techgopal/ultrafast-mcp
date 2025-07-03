# ultrafast-mcp-core

Core MCP protocol implementation for ULTRAFAST_MCP.

This crate provides the fundamental types, traits, and utilities for implementing the Model Context Protocol (MCP) in Rust.

## Features

- **Protocol Types**: Complete implementation of MCP message types and structures
- **JSON-RPC**: Built-in JSON-RPC 2.0 support
- **Error Handling**: Comprehensive error types and handling
- **Async Support**: Full async/await support with Tokio
- **Schema Validation**: JSON Schema generation and validation
- **UUID Support**: Built-in UUID handling for resource identification

## Usage

```rust
use ultrafast_mcp_core::{
    types::{Client, Server},
    protocol::messages::*,
    error::McpError,
};

// Create MCP client
let client = Client::new();

// Create MCP server
let server = Server::new();
```

## Dependencies

- `serde` - Serialization/deserialization
- `tokio` - Async runtime
- `uuid` - Unique identifier generation
- `schemars` - JSON Schema generation

## License

MIT OR Apache-2.0 