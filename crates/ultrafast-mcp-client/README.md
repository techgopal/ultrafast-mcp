# ultrafast-mcp-client

Client implementation for UltraFast MCP.

This crate provides a high-performance MCP client implementation with support for multiple transports and easy-to-use APIs.

## Features

- **High Performance**: Optimized for high-throughput MCP operations
- **Multiple Transports**: Support for stdio and HTTP transports
- **Easy-to-Use API**: Simple and ergonomic client interface
- **Async Support**: Full async/await support with Tokio
- **Connection Management**: Automatic connection handling and retry logic
- **Resource Discovery**: Built-in resource discovery and management
- **Error Handling**: Comprehensive error handling and recovery

## Usage

```rust
use ultrafast_mcp_client::{Client, ClientBuilder};
use ultrafast_mcp_core::types::Client as McpClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ClientBuilder::new()
        .with_transport("stdio")
        .with_server_url("http://localhost:8080")
        .build()?;
    
    // Connect to server
    client.connect().await?;
    
    // Use client for MCP operations
    let resources = client.list_resources().await?;
    
    Ok(())
}
```

## Features

- `http` - Enables HTTP transport support

## Dependencies

- `ultrafast-mcp-core` - Core MCP types and protocol
- `ultrafast-mcp-transport` - Transport layer
- `tokio` - Async runtime
- `tracing` - Logging and tracing

## License

MIT OR Apache-2.0 