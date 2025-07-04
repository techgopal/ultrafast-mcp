# ultrafast-mcp-macros

Procedural macros for the ULTRAFAST MCP implementation.

This crate provides procedural macros to simplify MCP server and client development with compile-time code generation.

## Features

- **Server Macros**: Easy server handler registration and setup
- **Client Macros**: Simplified client method generation
- **Type Derivation**: Automatic trait implementations
- **Compile-time Validation**: Early error detection
- **Code Generation**: Reduced boilerplate code
- **IDE Support**: Full IDE integration and autocompletion

## Usage

### Server Macros

```rust
use ultrafast_mcp_macros::server_handler;

#[server_handler]
struct MyHandler {
    // Handler implementation
}

#[tokio::main]
async fn main() {
    let server = Server::new()
        .with_handler(MyHandler::new())
        .run()
        .await;
}
```

### Client Macros

```rust
use ultrafast_mcp_macros::client_methods;

#[client_methods]
trait MyClient {
    async fn get_resource(&self, id: &str) -> Result<Resource, Error>;
}
```

## Dependencies

- `proc-macro2` - Procedural macro support
- `quote` - Code generation
- `syn` - Rust code parsing
- `serde_json` - JSON serialization

## License

MIT OR Apache-2.0 