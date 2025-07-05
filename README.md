# UltraFast MCP 🚀

**High-performance, ergonomic Model Context Protocol (MCP) implementation in Rust**

[![Crates.io](https://img.shields.io/crates/v/ultrafast-mcp)](https://crates.io/crates/ultrafast-mcp)
[![Documentation](https://img.shields.io/badge/docs-docs.rs-blue.svg)](https://docs.rs/ultrafast-mcp)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/techgopal/ultrafast-mcp/blob/main/LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![MCP](https://img.shields.io/badge/MCP-2025--06--18-green.svg)](https://modelcontextprotocol.io)

> **UltraFast MCP** is a high-performance, developer-friendly MCP framework in the Rust ecosystem. Built with performance, safety, and ergonomics in mind, it enables robust MCP servers and clients with minimal boilerplate while maintaining full MCP 2025-06-18 specification compliance.

## ⚠️ Release Candidate Status

This is **Release Candidate 2 (v202506018.1.0-rc.2)** of UltraFast MCP. While the framework is feature-complete and well-tested, it should be considered **pre-production** software. We recommend thorough testing in your environment before deploying to production.

## ✨ Features

### 🎯 **Developer Experience**
- **Ergonomic APIs** with minimal boilerplate
- **Type-safe** with automatic schema generation
- **Async-first** design with `tokio` integration
- **Comprehensive CLI** with project scaffolding
- **5 working examples** with full documentation

### 🔧 **Complete Feature Set**
- **Tools**: Function execution with JSON Schema validation
- **Resources**: URI-based resource management with templates
- **Prompts**: Template-based prompt system with arguments
- **Sampling**: Server-initiated LLM completions
- **Roots**: Filesystem boundary management
- **Elicitation**: User input collection
- **Logging**: RFC 5424 compliant structured logging
- **Completion**: Argument autocompletion system

### 🛡️ **Security & Authentication**
- **OAuth 2.1** with PKCE and dynamic client registration
- **Streamable HTTP transport** with session management
- **Comprehensive error handling** and recovery
- **Memory safety** guaranteed by Rust

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
ultrafast-mcp = { version = "202506018.1.0-rc.2", features = [
    "http",               # HTTP/HTTPS transport
    "oauth",              # OAuth 2.1 authentication
    "monitoring",         # OpenTelemetry observability
    "full"                # All features enabled
] }
```

**Note:** stdio transport and JSON Schema support are always included by default.

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

#[derive(Deserialize)]
struct GreetRequest {
    name: String,
}

#[derive(Serialize)]
struct GreetResponse {
    message: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let server = UltraFastServer::new("My MCP Server")
        .with_protocol_version("2025-06-18")
        .with_capabilities(ServerCapabilities {
            tools: Some(ToolsCapability { list_changed: true }),
            ..Default::default()
        });
    
    // Add a simple greeting tool
    server.tool("greet", |request: GreetRequest, ctx: Context| async move {
        ctx.progress("Processing greeting...", 0.5, Some(1.0)).await?;
        ctx.log_info(&format!("Greeting requested for {}", request.name)).await?;
        
        Ok(GreetResponse {
            message: format!("Hello, {}! Welcome to UltraFast MCP!", request.name),
        })
    })
    .description("Greet a user by name")
    .output_schema::<GreetResponse>();
    
    // Run the server
    server.run_stdio().await?;
    Ok(())
}
```

### Create Your First MCP Client

```rust
use ultrafast_mcp::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let client = UltraFastClient::connect(Transport::Stdio {
        command: "cargo".into(),
        args: vec!["run", "--bin", "server"].into(),
    }).await?;
    
    // Initialize the client
    client.initialize(ClientCapabilities {
        tools: Some(ToolsCapability { list_changed: true }),
        ..Default::default()
    }).await?;
    
    // Call the greeting tool
    let response: GreetResponse = client.call_tool("greet")
        .arg("name", "Alice")
        .with_progress(|progress, total, message| {
            println!("Progress: {}/{:?} - {}", progress, total, message.unwrap_or_default());
        })
        .await?;
    
    println!("Server says: {}", response.message);
    Ok(())
}
```

## 🔧 Advanced Examples

### File Operations Server

```rust
use ultrafast_mcp::prelude::*;
use std::path::PathBuf;

#[derive(Deserialize)]
struct ReadFileRequest {
    path: String,
}

#[derive(Serialize)]
struct ReadFileResponse {
    content: String,
    size: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let server = UltraFastServer::new("File Operations Server")
        .with_capabilities(ServerCapabilities {
            tools: Some(ToolsCapability { list_changed: true }),
            resources: Some(ResourcesCapability { list_changed: true }),
            ..Default::default()
        });
    
    // Add file reading tool
    server.tool("read_file", |request: ReadFileRequest, ctx: Context| async move {
        let path = PathBuf::from(&request.path);
        
        // Validate path is within allowed roots
        ctx.validate_path(&path)?;
        
        ctx.progress("Reading file...", 0.0, Some(1.0)).await?;
        
        let content = tokio::fs::read_to_string(&path).await?;
        let metadata = tokio::fs::metadata(&path).await?;
        
        ctx.progress("File read complete", 1.0, Some(1.0)).await?;
        
        Ok(ReadFileResponse {
            content,
            size: metadata.len(),
        })
    })
    .description("Read a file from the filesystem")
    .output_schema::<ReadFileResponse>();
    
    // Add resource for file listing
    server.resource("file://{path}", |uri: Uri, ctx: Context| async move {
        let path = uri.path().trim_start_matches('/');
        let path_buf = PathBuf::from(path);
        
        ctx.validate_path(&path_buf)?;
        
        if path_buf.is_file() {
            let content = tokio::fs::read_to_string(&path_buf).await?;
            Ok(ResourceContent::Text(content))
        } else if path_buf.is_dir() {
            let entries = tokio::fs::read_dir(&path_buf).await?;
            let files: Vec<String> = entries
                .map(|entry| entry.map(|e| e.file_name().to_string_lossy().to_string()))
                .collect::<Result<Vec<_>, _>>()?;
            
            Ok(ResourceContent::Text(files.join("\n")))
        } else {
            Err(Error::ResourceNotFound(uri))
        }
    });
    
    server.run_stdio().await?;
    Ok(())
}
```

### HTTP Server with Authentication

```rust
use ultrafast_mcp::prelude::*;
use ultrafast_mcp::auth::OAuthConfig;

#[tokio::main]
async fn main() -> Result<()> {
    let server = UltraFastServer::new("HTTP MCP Server")
        .with_capabilities(ServerCapabilities {
            tools: Some(ToolsCapability { list_changed: true }),
            prompts: Some(PromptsCapability { list_changed: true }),
            ..Default::default()
        });
    
    // Add OAuth configuration
    let oauth_config = OAuthConfig {
        client_id: "your-client-id".into(),
        client_secret: Some("your-client-secret".into()),
        auth_url: "https://accounts.google.com/oauth/authorize".into(),
        token_url: "https://oauth2.googleapis.com/token".into(),
        scopes: vec!["https://www.googleapis.com/auth/userinfo.profile".into()],
        redirect_uri: "http://localhost:8080/callback".into(),
    };
    
    // Add authenticated tool
    server.tool("get_user_info", |_: (), ctx: Context| async move {
        // Verify authentication
        let token = ctx.get_auth_token()?;
        
        // Make authenticated request
        let client = reqwest::Client::new();
        let response = client
            .get("https://www.googleapis.com/oauth2/v2/userinfo")
            .bearer_auth(token)
            .send()
            .await?;
        
        let user_info: serde_json::Value = response.json().await?;
        
        Ok(user_info)
    })
    .description("Get authenticated user information")
    .requires_auth(true);
    
    // Run HTTP server
    server.run_http("127.0.0.1:8080", Some(oauth_config)).await?;
    Ok(())
}
```

### Advanced Features Example

```rust
use ultrafast_mcp::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct SearchRequest {
    query: String,
    limit: Option<u32>,
}

#[derive(Serialize)]
struct SearchResult {
    title: String,
    url: String,
    snippet: String,
}

#[derive(Serialize)]
struct SearchResponse {
    results: Vec<SearchResult>,
    total: u32,
}

#[tokio::main]
async fn main() -> Result<()> {
    let server = UltraFastServer::new("Advanced Features Server")
        .with_capabilities(ServerCapabilities {
            tools: Some(ToolsCapability { list_changed: true }),
            prompts: Some(PromptsCapability { list_changed: true }),
            sampling: Some(SamplingCapability { list_changed: true }),
            ..Default::default()
        });
    
    // Add search tool with progress tracking
    server.tool("search", |request: SearchRequest, ctx: Context| async move {
        let limit = request.limit.unwrap_or(10);
        
        ctx.progress("Starting search...", 0.0, Some(1.0)).await?;
        ctx.log_info(&format!("Searching for: {}", request.query)).await?;
        
        // Simulate search with progress updates
        for i in 0..limit {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            ctx.progress("Searching...", (i + 1) as f64 / limit as f64, Some(1.0)).await?;
        }
        
        // Simulate results
        let results = (0..limit)
            .map(|i| SearchResult {
                title: format!("Result {}", i + 1),
                url: format!("https://example.com/result-{}", i + 1),
                snippet: format!("This is result {} for query '{}'", i + 1, request.query),
            })
            .collect();
        
        ctx.progress("Search complete", 1.0, Some(1.0)).await?;
        
        Ok(SearchResponse {
            results,
            total: limit,
        })
    })
    .description("Search for information")
    .output_schema::<SearchResponse>();
    
    // Add prompt template
    server.prompt("search_prompt", |args: serde_json::Value, ctx: Context| async move {
        let query = args["query"].as_str().unwrap_or("");
        let limit = args["limit"].as_u64().unwrap_or(10);
        
        let prompt = format!(
            "Please search for information about '{}' and return up to {} results. " \
            "Format the results as a structured list with titles, URLs, and snippets.",
            query, limit
        );
        
        Ok(Prompt {
            messages: vec![
                PromptMessage::User { content: vec![PromptContent::Text(prompt)] }
            ],
        })
    })
    .description("Generate a search prompt")
    .input_schema(json!({
        "type": "object",
        "properties": {
            "query": { "type": "string" },
            "limit": { "type": "integer", "minimum": 1, "maximum": 100 }
        },
        "required": ["query"]
    }));
    
    // Add sampling capability
    server.sampling("complete_search", |request: SamplingRequest, ctx: Context| async move {
        let prompt = request.prompt.messages.first()
            .and_then(|msg| msg.content.first())
            .and_then(|content| {
                if let PromptContent::Text(text) = content {
                    Some(text.clone())
                } else {
                    None
                }
            })
            .ok_or(Error::InvalidRequest("No text content found".into()))?;
        
        // Simulate LLM completion
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        let completion = format!("Based on the search query, here are the relevant results:\n\n{}", prompt);
        
        Ok(SamplingResponse {
            content: vec![PromptContent::Text(completion)],
            stop_reason: Some("end_turn".into()),
        })
    })
    .description("Complete search queries with LLM");
    
    server.run_stdio().await?;
    Ok(())
}
```

## 🏗️ Architecture

### Core Components

```
ultrafast-mcp/              # Main crate with unified APIs
├── ultrafast-mcp-core/     # Core protocol implementation
├── ultrafast-mcp-server/   # Server-side implementation
├── ultrafast-mcp-client/   # Client-side implementation
├── ultrafast-mcp-transport/# Transport layer (stdio/HTTP)
├── ultrafast-mcp-auth/     # OAuth 2.1 authentication
├── ultrafast-mcp-cli/      # Command-line interface
├── ultrafast-mcp-monitoring/# Observability and metrics
└── ultrafast-mcp-macros/   # Procedural macros
```

### Transport Layer

#### Streamable HTTP (Primary)
- **Single endpoint** (`/mcp`) for all operations
- **Optional SSE upgrade** for streaming when needed
- **Stateless architecture** for horizontal scaling
- **Session management** with secure session IDs
- **High performance** with connection pooling

#### stdio Transport
- **Subprocess communication** for local tools
- **Newline-delimited JSON-RPC** messages
- **Bidirectional communication** with proper lifecycle
- **stderr logging** support

## 📚 Documentation

### API Documentation
- **[Main Documentation](https://docs.rs/ultrafast-mcp)** - Complete API reference
- **[Core Types](https://docs.rs/ultrafast-mcp-core)** - Protocol definitions and types
- **[Server API](https://docs.rs/ultrafast-mcp-server)** - Server implementation details
- **[Client API](https://docs.rs/ultrafast-mcp-client)** - Client implementation details
- **[Transport Layer](https://docs.rs/ultrafast-mcp-transport)** - Transport implementations
- **[Authentication](https://docs.rs/ultrafast-mcp-auth)** - OAuth 2.1 implementation
- **[CLI Tools](https://docs.rs/ultrafast-mcp-cli)** - Command-line interface
- **[Monitoring](https://docs.rs/ultrafast-mcp-monitoring)** - Observability features

### Local Documentation
```bash
# Generate and open local documentation
cargo doc --open

# Generate documentation for specific crates
cargo doc --package ultrafast-mcp --open
cargo doc --package ultrafast-mcp-server --open
```

## 📚 Examples

### 1. Basic Echo Server
```bash
cd examples/01-basic-echo
cargo run --bin server  # Terminal 1
cargo run --bin client  # Terminal 2
```

### 2. File Operations Server
```bash
cd examples/02-file-operations
cargo run --bin server  # Terminal 1
cargo run --bin client  # Terminal 2
```

### 3. HTTP Server with Authentication
```bash
cd examples/03-http-server
cargo run --bin server  # Terminal 1
cargo run --bin client  # Terminal 2
```

### 4. Advanced Features
```bash
cd examples/04-advanced-features
cargo run --bin server  # Terminal 1
cargo run --bin client  # Terminal 2
```

## 🛠️ CLI Tools

### Project Management

```bash
# Initialize a new MCP project
mcp init my-project

# Generate project scaffolding
mcp generate server --name my-server

# Run development server with hot reload
mcp dev --port 8080

# Build the project
mcp build --release

# Test MCP connections
mcp test --endpoint http://localhost:8080/mcp

# Validate schemas and configurations
mcp validate --config config.toml
```

### Server Management

```bash
# Start a server
mcp server start --config server.toml

# Check server health
mcp server health --endpoint http://localhost:8080

# View server logs
mcp server logs --follow
```

### Client Management

```bash
# Connect to a server
mcp client connect --endpoint http://localhost:8080/mcp

# List available tools
mcp client tools

# Call a tool
mcp client call-tool greet --arg name=Alice
```

## 🔒 Security

### OAuth 2.1 Authentication

```rust
let client = UltraFastClient::connect(Transport::Streamable {
    url: "https://api.example.com/mcp".into(),
    auth: Some(AuthConfig::OAuth {
        client_id: "my-client".into(),
        scopes: vec!["read".into(), "write".into()],
        redirect_uri: "http://localhost:8080/callback".into(),
    }),
}).await?;
```

### Security Features
- **PKCE**: Authorization code protection (RFC 7636)
- **Dynamic Client Registration**: RFC 7591 compliance
- **Resource Indicators**: RFC 8707 token audience binding
- **HTTPS Enforcement**: TLS 1.2+ required
- **Token Validation**: JWT token verification
- **Rate Limiting**: Protection against abuse

## 📊 Monitoring & Observability

### OpenTelemetry Integration

```rust
let server = UltraFastServer::new("My Server")
    .with_monitoring_config(MonitoringConfig {
        tracing: Some(TracingConfig {
            endpoint: "http://localhost:14268/api/traces".into(),
            service_name: "my-mcp-server".into(),
        }),
        metrics: Some(MetricsConfig {
            endpoint: "http://localhost:9090".into(),
        }),
    });
```

### Available Metrics
- **Request/Response Latency**
- **Throughput (requests/second)**
- **Error Rates**
- **Resource Usage** (CPU, Memory, Network)
- **Connection Pool Status**
- **Authentication Success/Failure Rates**

## 🚀 Performance

### Performance Optimizations
- **Zero-copy serialization** with `serde` and `bytes`
- **SIMD-optimized JSON parsing**
- **Connection pooling** for HTTP transports
- **Stateless architecture** for horizontal scaling
- **Async-first design** with `tokio` integration

## 📋 MCP 2025-06-18 Compliance

### ✅ Complete Specification Support

- **Base Protocol**: JSON-RPC 2.0, lifecycle management, capability negotiation
- **Transport Layer**: stdio, Streamable HTTP, session management
- **Authorization**: OAuth 2.1 with full RFC compliance
- **Server Features**: Tools, Resources, Prompts, Logging, Completion
- **Client Features**: Sampling, Roots, Elicitation
- **Utilities**: Progress tracking, cancellation, pagination, ping/pong

### Compliance Checklist
- ✅ JSON-RPC 2.0 message format
- ✅ Three-phase lifecycle (Initialize → Operation → Shutdown)
- ✅ Capability negotiation for optional features
- ✅ Version negotiation with fallback support
- ✅ OAuth 2.1 authorization framework
- ✅ Resource templates with RFC 6570 URI templates
- ✅ Structured logging with RFC 5424 compliance
- ✅ Progress tracking with token-based system
- ✅ Request cancellation with race condition handling

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/techgopal/ultrafast-mcp.git
cd ultrafast-mcp

# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Run examples
cargo run --example basic-echo-server
```

### Project Structure

```
mcp/
├── crates/                 # Core library crates
│   ├── ultrafast-mcp/      # Main crate
│   ├── ultrafast-mcp-core/ # Protocol implementation
│   ├── ultrafast-mcp-server/ # Server implementation
│   ├── ultrafast-mcp-client/ # Client implementation
│   ├── ultrafast-mcp-transport/ # Transport layer
│   ├── ultrafast-mcp-auth/ # Authentication
│   ├── ultrafast-mcp-cli/  # Command-line interface
│   ├── ultrafast-mcp-monitoring/ # Observability
│   └── ultrafast-mcp-macros/ # Procedural macros
├── examples/               # Working examples
│   ├── 01-basic-echo/      # Basic server/client
│   ├── 02-file-operations/ # File system operations
│   ├── 03-http-server/     # HTTP operations
│   └── 04-advanced-features/ # Complete feature set
├── tests/                  # Integration tests
├── benches/                # Performance benchmarks
└── docs/                   # Documentation
```

## 📄 License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

## 🙏 Acknowledgments

- **FastMCP**: Inspiration for the ergonomic API design
- **MCP Community**: For the excellent Model Context Protocol specification
- **Rust Ecosystem**: For the amazing tools and libraries that make this possible
- **OpenTelemetry**: For the comprehensive observability framework

## 📞 Support

- **Documentation**: [https://docs.rs/ultrafast-mcp](https://docs.rs/ultrafast-mcp)
- **Issues**: [GitHub Issues](https://github.com/techgopal/ultrafast-mcp/issues)
- **Discussions**: [GitHub Discussions](https://github.com/techgopal/ultrafast-mcp/discussions)
- **Email**: team@ultrafast-mcp.com

---

**UltraFast MCP** - A high-performance, developer-friendly MCP framework in Rust. 🚀 