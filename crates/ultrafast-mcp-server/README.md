# ultrafast-mcp-server

Server implementation for UltraFast MCP.

This crate provides a high-performance MCP server implementation with support for multiple transports and extensible handlers.

## Features

- **High Performance**: Optimized for high-throughput MCP operations
- **Multiple Transports**: Support for stdio and HTTP transports
- **Extensible Handlers**: Easy to extend with custom handlers
- **Async Support**: Full async/await support with Tokio
- **Monitoring**: Built-in monitoring and observability
- **Context Management**: Efficient context and session management
- **Resource Management**: Comprehensive resource lifecycle management

## Usage

```rust
use ultrafast_mcp_server::{Server, ServerBuilder};
use ultrafast_mcp_core::types::Server as McpServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = ServerBuilder::new()
        .with_transport("stdio")
        .with_handler(MyHandler::new())
        .build()?;
    
    server.run().await?;
    Ok(())
}
```

## Features

- `monitoring` - Enables monitoring and metrics collection
- `http` - Enables HTTP transport support

## Dependencies

- `ultrafast-mcp-core` - Core MCP types and protocol
- `ultrafast-mcp-transport` - Transport layer
- `tokio` - Async runtime
- `tracing` - Logging and tracing

## License

MIT OR Apache-2.0 