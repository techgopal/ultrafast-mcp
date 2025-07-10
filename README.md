# UltraFast MCP 🚀

**High-performance, ergonomic Model Context Protocol (MCP) implementation in Rust**

[![Crates.io](https://img.shields.io/crates/v/ultrafast-mcp)](https://crates.io/crates/ultrafast-mcp)
[![Documentation](https://img.shields.io/badge/docs-docs.rs-blue.svg)](https://docs.rs/ultrafast-mcp)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/techgopal/ultrafast-mcp/blob/main/LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![MCP](https://img.shields.io/badge/MCP-2025--06--18-green.svg)](https://modelcontextprotocol.io)

> **UltraFast MCP** is a high-performance, developer-friendly MCP framework in the Rust ecosystem. Built with performance, safety, and ergonomics in mind, it enables robust MCP servers and clients with minimal boilerplate while maintaining full MCP 2025-06-18 specification compliance.

## ⚠️ Release Candidate Status

This is **Release Candidate 2 (v20250618.1.0-rc.2)** of UltraFast MCP. While the framework is feature-complete and well-tested, it should be considered **pre-production** software. We recommend thorough testing in your environment before deploying to production.

## 🏗️ Architecture Overview

UltraFast MCP is built as a modular, high-performance framework with the following core components:

### Core Crates

- **`ultrafast-mcp-core`**: Foundation types, protocol implementation, and utilities
- **`ultrafast-mcp-server`**: High-performance server implementation with handler traits
- **`ultrafast-mcp-client`**: Async client with connection management and retry logic
- **`ultrafast-mcp-transport`**: Transport layer with STDIO and HTTP support
- **`ultrafast-mcp-auth`**: OAuth 2.1 authentication with PKCE support
- **`ultrafast-mcp-monitoring`**: Metrics, health checks, and OpenTelemetry integration
- **`ultrafast-mcp-cli`**: Command-line tools for development and testing
- **`ultrafast-mcp-test-utils`**: Testing utilities and fixtures

### Key Design Principles

- **Type Safety**: Compile-time guarantees for protocol compliance
- **Async-First**: Built on `tokio` for high-performance async operations
- **Modular Design**: Independent crates for different concerns
- **Production Ready**: Comprehensive error handling, logging, and monitoring
- **Developer Experience**: Ergonomic APIs with minimal boilerplate

## ✨ Features

### 🎯 **Core Protocol Support**
- **Complete MCP 2025-06-18 Implementation**: Full specification compliance
- **Tools**: Function execution with JSON Schema validation
- **Resources**: URI-based resource management with templates
- **Prompts**: Template-based prompt system with arguments
- **Sampling**: Server-initiated LLM completions
- **Roots**: Filesystem boundary management
- **Elicitation**: User input collection and validation
- **Completion**: Argument autocompletion system

### 🚀 **Performance & Reliability**
- **High-Performance Transport**: Streamable HTTP with connection pooling
- **Async/Await**: Non-blocking I/O with `tokio` integration
- **Connection Recovery**: Automatic reconnection with exponential backoff
- **Request Timeouts**: Configurable timeout management
- **Memory Safety**: Rust's ownership system prevents common bugs

### 🛡️ **Security & Authentication**
- **OAuth 2.1**: Complete OAuth implementation with PKCE
- **Token Management**: Secure token storage and validation
- **Session Management**: Secure session handling
- **CSRF Protection**: State parameter validation
- **Scope Management**: Fine-grained permission control

### 🔧 **Developer Experience**
- **Ergonomic APIs**: Simple, intuitive interfaces
- **Type-Safe Schemas**: Automatic JSON Schema generation
- **Comprehensive CLI**: Project scaffolding and development tools
- **Rich Examples**: 5+ working examples with full documentation
- **Testing Utilities**: Comprehensive test support

### 📊 **Observability**
- **Metrics Collection**: Request, transport, and system metrics
- **Health Checking**: Application and system health monitoring
- **Distributed Tracing**: OpenTelemetry integration
- **Structured Logging**: RFC 5424 compliant logging
- **Performance Monitoring**: Response times and throughput tracking

## 📦 Installation

### Quick Start

```bash
# Create a new MCP server project
cargo new my-mcp-server
cd my-mcp-server

# Add UltraFast MCP with HTTP transport and OAuth
cargo add ultrafast-mcp --features="http,oauth"
```

### Feature Flags

```toml
[dependencies]
ultrafast-mcp = { version = "20250618.1.0-rc.2", features = [
    "http",               # HTTP/HTTPS transport
    "oauth",              # OAuth 2.1 authentication
    "monitoring",         # OpenTelemetry observability
    "full"                # All features enabled
] }
```

**Note:** STDIO transport and JSON Schema support are always included by default.

### Convenience Features

```bash
# Web server with authentication
cargo add ultrafast-mcp --features="http,oauth"

# All features enabled
cargo add ultrafast-mcp --features="full"
```

## 🚀 Quick Start

### Create Your First MCP Server

```rust
use ultrafast_mcp::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize)]
struct GreetRequest {
    name: String,
    greeting: Option<String>,
}

#[derive(Serialize)]
struct GreetResponse {
    message: String,
    timestamp: String,
}

// Implement the tool handler
struct GreetToolHandler;

#[async_trait::async_trait]
impl ToolHandler for GreetToolHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult> {
        match call.name.as_str() {
            "greet" => {
                // Parse the arguments
                let args: GreetRequest = serde_json::from_value(
                    call.arguments.unwrap_or_default()
                )?;

                // Generate the response
                let greeting = args.greeting.unwrap_or_else(|| "Hello".to_string());
                let message = format!("{}, {}!", greeting, args.name);

                Ok(ToolResult {
                    content: vec![ToolContent::text(message)],
                    is_error: Some(false),
                })
            }
            _ => Err(MCPError::method_not_found(
                format!("Unknown tool: {}", call.name)
            )),
        }
    }

    async fn list_tools(&self, _request: ListToolsRequest) -> MCPResult<ListToolsResponse> {
        Ok(ListToolsResponse {
            tools: vec![Tool {
                name: "greet".to_string(),
                description: "Greet a person by name".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "greeting": {"type": "string", "default": "Hello"}
                    },
                    "required": ["name"]
                }),
                output_schema: None,
            }],
            next_cursor: None,
        })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create server configuration
    let server_info = ServerInfo {
        name: "greeting-server".to_string(),
        version: "1.0.0".to_string(),
        description: Some("A simple greeting server".to_string()),
        authors: None,
        homepage: None,
        license: None,
        repository: None,
    };

    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability { list_changed: Some(true) }),
        ..Default::default()
    };

    // Create and configure the server
    let server = UltraFastServer::new(server_info, capabilities)
        .with_tool_handler(Arc::new(GreetToolHandler));

    // Start the server with STDIO transport
    server.run_stdio().await?;

    Ok(())
}
```

### Create Your First MCP Client

```rust
use ultrafast_mcp::prelude::*;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create client configuration
    let client_info = ClientInfo {
        name: "greeting-client".to_string(),
        version: "1.0.0".to_string(),
        authors: None,
        description: Some("A simple greeting client".to_string()),
        homepage: None,
        repository: None,
        license: None,
    };

    let capabilities = ClientCapabilities::default();

    // Create the client
    let client = UltraFastClient::new(client_info, capabilities);

    // Connect to the server using STDIO
    client.connect_stdio().await?;

    // Call a tool
    let tool_call = ToolCall {
        name: "greet".to_string(),
        arguments: Some(json!({
            "name": "Alice",
            "greeting": "Hello there"
        })),
    };

    let result = client.call_tool(tool_call).await?;
    println!("Server response: {:?}", result);

    // Disconnect
    client.disconnect().await?;

    Ok(())
}
```

## 🔧 Advanced Examples

### HTTP Server with OAuth

```rust
use ultrafast_mcp::prelude::*;
use std::sync::Arc;

struct SecureToolHandler;

#[async_trait::async_trait]
impl ToolHandler for SecureToolHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult> {
        // Your secure tool implementation
        Ok(ToolResult {
            content: vec![ToolContent::text("Secure operation completed".to_string())],
            is_error: Some(false),
        })
    }

    async fn list_tools(&self, _request: ListToolsRequest) -> MCPResult<ListToolsResponse> {
        Ok(ListToolsResponse {
            tools: vec![Tool {
                name: "secure_operation".to_string(),
                description: "Perform a secure operation".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "data": {"type": "string"}
                    },
                    "required": ["data"]
                }),
                output_schema: None,
            }],
            next_cursor: None,
        })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server_info = ServerInfo {
        name: "secure-server".to_string(),
        version: "1.0.0".to_string(),
        description: Some("A secure MCP server".to_string()),
        authors: None,
        homepage: None,
        license: None,
        repository: None,
    };

    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability { list_changed: Some(true) }),
        ..Default::default()
    };

    let server = UltraFastServer::new(server_info, capabilities)
        .with_tool_handler(Arc::new(SecureToolHandler));

    // Run with HTTP transport
    server.run_streamable_http("127.0.0.1", 8080).await?;

    Ok(())
}
```

### Resource Management

```rust
use ultrafast_mcp::prelude::*;
use std::sync::Arc;

struct FileResourceHandler;

#[async_trait::async_trait]
impl ResourceHandler for FileResourceHandler {
    async fn read_resource(&self, request: ReadResourceRequest) -> MCPResult<ReadResourceResponse> {
        // Implement file reading logic
        let content = std::fs::read_to_string(&request.uri)?;
        
        Ok(ReadResourceResponse {
            contents: vec![ResourceContent::text(
                request.uri,
                content
            )],
        })
    }

    async fn list_resources(&self, _request: ListResourcesRequest) -> MCPResult<ListResourcesResponse> {
        Ok(ListResourcesResponse {
            resources: vec![],
            next_cursor: None,
        })
    }

    async fn list_resource_templates(&self, _request: ListResourceTemplatesRequest) -> MCPResult<ListResourceTemplatesResponse> {
        Ok(ListResourceTemplatesResponse {
            resource_templates: vec![],
            next_cursor: None,
        })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server_info = ServerInfo {
        name: "file-server".to_string(),
        version: "1.0.0".to_string(),
        description: Some("A file resource server".to_string()),
        authors: None,
        homepage: None,
        license: None,
        repository: None,
    };

    let capabilities = ServerCapabilities {
        resources: Some(ResourcesCapability { list_changed: Some(true) }),
        ..Default::default()
    };

    let server = UltraFastServer::new(server_info, capabilities)
        .with_resource_handler(Arc::new(FileResourceHandler));

    server.run_stdio().await?;

    Ok(())
}
```

## 🛠️ Development Tools

### CLI Commands

```bash
# Install the CLI
cargo install ultrafast-mcp-cli

# Initialize a new project
mcp init my-server --template server

# Start development server
mcp dev --watch

# Test connections
mcp test --server http://localhost:8080

# Validate schemas
mcp validate

# Generate shell completions
mcp completions bash > ~/.bash_completion
```

### Project Structure

```
my-mcp-server/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── tools.rs
│   │   └── resources.rs
│   └── config.rs
├── tests/
│   └── integration_tests.rs
└── README.md
```

## 📊 Monitoring & Observability

### Basic Monitoring Setup

```rust
use ultrafast_mcp_monitoring::{MonitoringSystem, MonitoringConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize monitoring
    let monitoring = MonitoringSystem::init(MonitoringConfig::default()).await?;
    
    // Use monitoring in your application
    let metrics = monitoring.metrics();
    let timer = RequestTimer::start("tools/call", metrics.clone());
    
    // ... perform your operation ...
    
    // Record the request completion
    timer.finish(true).await;
    
    Ok(())
}
```

### Health Checks

```rust
use ultrafast_mcp_monitoring::{HealthChecker, HealthStatus};

async fn check_health(health_checker: &HealthChecker) {
    match health_checker.check_all().await {
        HealthStatus::Healthy => println!("All systems healthy"),
        HealthStatus::Degraded(warnings) => {
            println!("System degraded: {:?}", warnings);
        }
        HealthStatus::Unhealthy(errors) => {
            println!("System unhealthy: {:?}", errors);
        }
    }
}
```

## 🔐 Authentication

### OAuth 2.1 Setup

```rust
use ultrafast_mcp_auth::{OAuthClient, OAuthConfig, generate_pkce_params};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = OAuthConfig {
        client_id: "your-client-id".to_string(),
        client_secret: "your-client-secret".to_string(),
        auth_url: "https://auth.example.com/oauth/authorize".to_string(),
        token_url: "https://auth.example.com/oauth/token".to_string(),
        redirect_uri: "http://localhost:8080/callback".to_string(),
        scopes: vec!["read".to_string(), "write".to_string()],
    };

    let client = OAuthClient::from_config(config);
    let pkce_params = generate_pkce_params()?;

    // Create authorization URL
    let auth_url = client.get_authorization_url_with_pkce("state", pkce_params.clone()).await?;
    println!("Authorization URL: {}", auth_url);

    Ok(())
}
```

## 🧪 Testing

### Integration Tests

```rust
use ultrafast_mcp_test_utils::*;
use ultrafast_mcp::prelude::*;

#[tokio::test]
async fn test_tool_execution() {
    // Create test server
    let server = create_test_server().await;
    
    // Create test client
    let client = create_test_client().await;
    
    // Test tool call
    let result = client.call_tool(ToolCall {
        name: "test_tool".to_string(),
        arguments: Some(serde_json::json!({"input": "test"})),
    }).await;
    
    assert!(result.is_ok());
}
```

## 📚 Examples

The repository includes comprehensive examples:

- **01-basic-echo**: Simple echo server with HTTP transport
- **02-file-operations**: File reading and writing tools
- **03-http-server**: HTTP server with OAuth authentication
- **04-advanced-features**: Advanced features demonstration
- **05-lifecycle-compliance**: Lifecycle management examples

Run examples with:

```bash
# Run basic echo server
cargo run --example basic-echo-server

# Run HTTP server
cargo run --example http-server

# Run client
cargo run --example basic-echo-client
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/your-repo/ultrafast-mcp
cd ultrafast-mcp

# Install dependencies
cargo build

# Run tests
cargo test

# Run benchmarks
cargo bench

# Check formatting
cargo fmt

# Run clippy
cargo clippy
```

## 📄 License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

## 🙏 Acknowledgments

- [Model Context Protocol](https://modelcontextprotocol.io) for the specification
- [Tokio](https://tokio.rs) for the async runtime
- [Serde](https://serde.rs) for serialization
- [Tracing](https://tracing.rs) for observability

## 📞 Support

- **Documentation**: [docs.rs/ultrafast-mcp](https://docs.rs/ultrafast-mcp)
- **Issues**: [GitHub Issues](https://github.com/your-repo/ultrafast-mcp/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-repo/ultrafast-mcp/discussions)

---

**UltraFast MCP** - Making MCP development fast, safe, and enjoyable! 🚀