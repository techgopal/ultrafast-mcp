# ultrafast-mcp-transport

Transport layer implementation for UltraFast MCP.

This crate provides transport abstractions for MCP communication, supporting both stdio and HTTP transports.

## Features

- **Stdio Transport**: Standard input/output transport for local communication
- **HTTP Transport**: HTTP/HTTPS transport for remote communication
- **Async Support**: Full async/await support with Tokio
- **Middleware Support**: Extensible middleware system
- **Authentication**: Built-in authentication support
- **Rate Limiting**: Configurable rate limiting
- **Connection Pooling**: Efficient connection management

## Usage

### Stdio Transport (Default)

```rust
use ultrafast_mcp_transport::stdio::StdioTransport;

let transport = StdioTransport::new();
```

### HTTP Transport

```rust
use ultrafast_mcp_transport::http::{HttpClient, HttpServer};

// HTTP Client
let client = HttpClient::new("http://localhost:8080");

// HTTP Server
let server = HttpServer::new("127.0.0.1:8080");
```

## Features

- `stdio` - Standard input/output transport (default)
- `http` - HTTP/HTTPS transport with full HTTP stack

## Dependencies

- `tokio` - Async runtime
- `axum` - HTTP framework
- `reqwest` - HTTP client
- `tower` - Middleware framework
- `hyper` - HTTP implementation

## License

MIT OR Apache-2.0 