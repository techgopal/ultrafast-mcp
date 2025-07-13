# UltraFast MCP 🚀

**High-performance, ergonomic Model Context Protocol (MCP) implementation in Rust**

[![Crates.io](https://img.shields.io/crates/v/ultrafast-mcp)](https://crates.io/crates/ultrafast-mcp)
[![Documentation](https://img.shields.io/badge/docs-docs.rs-blue.svg)](https://docs.rs/ultrafast-mcp)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/techgopal/ultrafast-mcp/blob/main/LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![MCP](https://img.shields.io/badge/MCP-2025--06--18-green.svg)](https://modelcontextprotocol.io)

> **UltraFast MCP** is a high-performance, developer-friendly MCP framework in the Rust ecosystem. Built with performance, safety, and ergonomics in mind, it enables robust MCP servers and clients with minimal boilerplate while maintaining full MCP 2025-06-18 specification compliance.

## 📋 Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Crates](#crates)
- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Examples](#examples)
- [API Reference](#api-reference)
- [Contributing](#contributing)
- [License](#license)

## 🎯 Overview

UltraFast MCP is designed to be the definitive Rust implementation of the Model Context Protocol, providing:

- **🚀 High Performance**: Optimized for throughput and low latency with async/await
- **🛡️ Type Safety**: Compile-time guarantees for protocol compliance
- **🎨 Ergonomic APIs**: Simple, intuitive interfaces with minimal boilerplate
- **📦 Modular Design**: Independent crates for different concerns
- **🔧 Production Ready**: Comprehensive error handling, logging, and monitoring
- **🔐 Security First**: OAuth 2.1, PKCE, and secure token management
- **📊 Observability**: Metrics, health checks, and distributed tracing

## 🏗️ Architecture

UltraFast MCP follows a modular architecture with clear separation of concerns:

```text
┌─────────────────────────────────────────────────────────────┐
│                    UltraFast MCP                            │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │    CLI      │  │  Monitoring │  │    Auth     │        │
│  │   Tools     │  │  & Metrics  │  │   OAuth     │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │   Server    │  │   Client    │  │  Transport  │        │
│  │  Handler    │  │  Connection │  │   Layer     │        │
│  │  System     │  │  Management │  │  HTTP/STDIO │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
├─────────────────────────────────────────────────────────────┤
│                    Core Protocol                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │   Types     │  │  Protocol   │  │  Utilities  │        │
│  │  & Traits   │  │  Messages   │  │  & Helpers  │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
└─────────────────────────────────────────────────────────────┘
```

## 📦 Crates

UltraFast MCP is organized into specialized crates, each focusing on specific functionality:

### 🎯 Core Crates

#### `ultrafast-mcp-core` - Foundation Layer
**Purpose**: Core protocol implementation and foundational types

**Key Components**:
- **Protocol Types**: Complete MCP 2025-06-18 type definitions
- **Message Handling**: JSON-RPC message serialization/deserialization
- **Schema Generation**: Automatic JSON Schema generation and validation
- **Error Handling**: Comprehensive error types and result handling
- **Utilities**: URI handling, pagination, progress tracking, and identifiers

**Features**:
- Full MCP specification compliance
- Type-safe protocol implementation
- Automatic schema generation
- Comprehensive error handling
- Utility functions for common operations

#### `ultrafast-mcp-server` - Server Implementation
**Purpose**: High-performance server with ergonomic handler system

**Key Components**:
- **UltraFastServer**: Main server implementation with fluent API
- **Handler Traits**: ToolHandler, ResourceHandler, PromptHandler, etc.
- **State Management**: Thread-safe server state and context management
- **Lifecycle Management**: Connection initialization, shutdown, and state transitions
- **Capability Negotiation**: Feature discovery and negotiation

**Features**:
- Ergonomic server creation and configuration
- Trait-based handler system for extensibility
- Comprehensive state management
- Built-in timeout and error handling
- Support for all MCP capabilities

#### `ultrafast-mcp-client` - Client Implementation
**Purpose**: Async client with connection management and retry logic

**Key Components**:
- **UltraFastClient**: Main client implementation with async/await
- **Connection Management**: Automatic connection handling and recovery
- **Request Management**: Pending request tracking and timeout handling
- **State Management**: Client state transitions and capability checking
- **Elicitation Handling**: User input collection and validation

**Features**:
- Async/await client API
- Automatic connection recovery
- Request timeout management
- State-aware operations
- Elicitation support

#### `ultrafast-mcp-transport` - Transport Layer
**Purpose**: Flexible transport layer with multiple protocols

**Key Components**:
- **Transport Trait**: Abstract transport interface
- **STDIO Transport**: Local communication with minimal overhead
- **HTTP Transport**: Web-based communication with session management
- **Streamable HTTP**: High-performance HTTP transport (recommended)
- **Recovery System**: Automatic reconnection with exponential backoff

**Features**:
- Multiple transport protocols (STDIO, HTTP, Streamable HTTP)
- Connection pooling and management
- Automatic recovery and retry logic
- Health monitoring and diagnostics
- Extensible transport architecture

### 🔐 Authentication & Security

#### `ultrafast-mcp-auth` - Authentication System
**Purpose**: Comprehensive authentication and authorization support

**Key Components**:
- **OAuth 2.1**: Complete OAuth implementation with PKCE
- **Token Management**: Secure token storage, validation, and rotation
- **PKCE Support**: Proof Key for Code Exchange for enhanced security
- **Session Management**: Secure session handling and management
- **Validation**: Comprehensive token and credential validation

**Features**:
- OAuth 2.1 authorization code flow
- PKCE for public client security
- JWT token validation
- Automatic token refresh
- CSRF protection with state validation

### 📊 Monitoring & Observability

#### `ultrafast-mcp-monitoring` - Monitoring System
**Purpose**: Comprehensive monitoring and observability

**Key Components**:
- **Metrics Collection**: Request, transport, and system metrics
- **Health Checking**: Application and system health monitoring
- **Distributed Tracing**: OpenTelemetry integration
- **Performance Monitoring**: Response times and throughput tracking
- **Exporters**: Prometheus, JSON, and custom metric exporters

**Features**:
- Real-time metrics collection
- Custom health checks
- OpenTelemetry tracing
- Performance monitoring
- Multiple export formats

### 🛠️ Development Tools

#### `ultrafast-mcp-cli` - Command Line Interface
**Purpose**: Development tools and project management

**Key Components**:
- **Project Management**: Initialize, build, and manage MCP projects
- **Development Tools**: Development servers, hot reloading, and debugging
- **Testing Utilities**: Connection testing, schema validation, and integration tests
- **Code Generation**: Scaffolding, templates, and boilerplate generation
- **Configuration Management**: Server and client configuration management

**Features**:
- Project scaffolding and templates
- Development server with hot reloading
- Testing and validation tools
- Code generation utilities
- Configuration management

#### `ultrafast-mcp-test-utils` - Testing Utilities
**Purpose**: Common test fixtures and utilities

**Key Components**:
- **Test Fixtures**: Pre-configured test data and scenarios
- **Mock Implementations**: Mock handlers and transport implementations
- **Assertions**: Test assertions and validation helpers
- **Test Utilities**: Common testing patterns and utilities

**Features**:
- Reusable test fixtures
- Mock implementations
- Test assertions
- Common testing patterns

### 🎯 Main Crate

#### `ultrafast-mcp` - Primary API
**Purpose**: Convenient re-exports and feature management

**Key Components**:
- **Re-exports**: Convenient access to all major types and traits
- **Feature Management**: Feature flag organization and combinations
- **Prelude Module**: Common imports for quick development
- **Documentation**: Comprehensive examples and usage patterns

**Features**:
- Convenient re-exports
- Feature flag combinations
- Comprehensive documentation
- Quick start examples

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
ultrafast-mcp = { version = "202506018.1.0", features = [
    "http",                    # HTTP/HTTPS transport
    "oauth",                   # OAuth 2.1 authentication
    "monitoring-full",         # Complete monitoring suite
    "full"                     # All features enabled
] }
```

**Note:** No features are enabled by default for minimal footprint.

### Available Features

#### **Core Features**
- `core` - Basic MCP functionality (types, traits, utilities)
- `stdio` - STDIO transport support (includes core functionality)
- `http` - HTTP/HTTPS transport support (includes stdio fallback + core functionality)

#### **Authentication**
- `oauth` - OAuth 2.1 authentication with PKCE (includes core functionality)

#### **Monitoring (Granular)**
- `monitoring` - Basic monitoring capabilities (includes core functionality)
- `monitoring-http` - HTTP metrics endpoints
- `monitoring-jaeger` - Jaeger tracing support
- `monitoring-otlp` - OTLP tracing support
- `monitoring-console` - Console tracing output

#### **Convenience Combinations**
- `http-with-auth` - HTTP transport + OAuth authentication (includes stdio fallback + core)
- `monitoring-full` - All monitoring features
- `minimal` - Core + STDIO (minimal working setup)
- `full` - Everything enabled

### Recommended Usage Patterns

```bash
# Minimal setup (STDIO only)
cargo add ultrafast-mcp --features="minimal"

# HTTP server with OAuth
cargo add ultrafast-mcp --features="http-with-auth"

# Production setup with monitoring
cargo add ultrafast-mcp --features="http-with-auth,monitoring-full"

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

## 📚 Examples

UltraFast MCP includes comprehensive examples demonstrating various use cases:

### 1. [Basic Echo](./examples/01-basic-echo/) - Getting Started
**Difficulty**: Beginner  
**Focus**: Basic server and client setup with high-performance HTTP transport

**Key Features**:
- Ergonomic API with one-line server startup
- Streamable HTTP transport for high performance
- Type-safe tool calling with serde serialization
- Comprehensive error handling and logging

### 2. [File Operations](./examples/02-file-operations/) - File System Integration
**Difficulty**: Intermediate  
**Focus**: File system operations and complex tool handling

**Key Features**:
- Multiple file operations (read, write, list, delete, search, move)
- Complex tool handler implementation with 12+ tools
- Error handling and path validation
- File metadata handling and directory tree generation

### 3. [Everything Server](./examples/03-everything-server/) - Complete MCP Implementation
**Difficulty**: Advanced  
**Focus**: Complete MCP feature set with all capabilities

**Key Features**:
- Multiple trait implementations (ToolHandler, ResourceHandler, PromptHandler)
- Advanced data generation and processing
- Text analysis capabilities
- Dynamic resource management
- Complete MCP protocol implementation

### 4. [Authentication Example](./examples/04-authentication-example/) - Authentication Methods
**Difficulty**: Intermediate  
**Focus**: Comprehensive authentication support

**Key Features**:
- Multiple authentication methods (Bearer, API Key, Basic, OAuth)
- Server-side validation with JWT
- Client-side authentication middleware
- HTTP transport authentication integration
- Security best practices

## 🔧 API Reference

### Server API

#### Core Server Types
```rust
// Main server implementation
pub struct UltraFastServer { ... }

// Server information
pub struct ServerInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    // ... other fields
}

// Server capabilities
pub struct ServerCapabilities {
    pub tools: Option<ToolsCapability>,
    pub resources: Option<ResourcesCapability>,
    pub prompts: Option<PromptsCapability>,
    // ... other capabilities
}
```

#### Handler Traits
```rust
// Tool execution handler
#[async_trait]
pub trait ToolHandler: Send + Sync {
    async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult>;
    async fn list_tools(&self, request: ListToolsRequest) -> MCPResult<ListToolsResponse>;
}

// Resource management handler
#[async_trait]
pub trait ResourceHandler: Send + Sync {
    async fn read_resource(&self, request: ReadResourceRequest) -> MCPResult<ReadResourceResponse>;
    async fn list_resources(&self, request: ListResourcesRequest) -> MCPResult<ListResourcesResponse>;
}

// Prompt generation handler
#[async_trait]
pub trait PromptHandler: Send + Sync {
    async fn get_prompt(&self, request: GetPromptRequest) -> MCPResult<GetPromptResponse>;
    async fn list_prompts(&self, request: ListPromptsRequest) -> MCPResult<ListPromptsResponse>;
}
```

### Client API

#### Core Client Types
```rust
// Main client implementation
pub struct UltraFastClient { ... }

// Client information
pub struct ClientInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    // ... other fields
}

// Client capabilities
pub struct ClientCapabilities {
    // ... capability fields
}
```

#### Client Methods
```rust
impl UltraFastClient {
    // Connection methods
    pub async fn connect_stdio(&self) -> MCPResult<()>;
    pub async fn connect_streamable_http(&self, url: &str) -> MCPResult<()>;
    
    // Tool operations
    pub async fn call_tool(&self, tool_call: ToolCall) -> MCPResult<ToolResult>;
    pub async fn list_tools(&self, request: ListToolsRequest) -> MCPResult<ListToolsResponse>;
    
    // Resource operations
    pub async fn read_resource(&self, request: ReadResourceRequest) -> MCPResult<ReadResourceResponse>;
    pub async fn list_resources(&self, request: ListResourcesRequest) -> MCPResult<ListResourcesResponse>;
    
    // Lifecycle methods
    pub async fn initialize(&self) -> MCPResult<()>;
    pub async fn shutdown(&self, reason: Option<String>) -> MCPResult<()>;
    pub async fn disconnect(&self) -> MCPResult<()>;
}
```

### Transport API

#### Transport Types
```rust
// Abstract transport trait
#[async_trait]
pub trait Transport: Send + Sync {
    async fn send_message(&mut self, message: JsonRpcMessage) -> Result<()>;
    async fn receive_message(&mut self) -> Result<JsonRpcMessage>;
    async fn close(&mut self) -> Result<()>;
    fn get_state(&self) -> ConnectionState;
}

// Connection state
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    ShuttingDown,
    Failed(String),
}
```

### Authentication API

#### OAuth Types
```rust
// OAuth configuration
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}

// OAuth client
pub struct OAuthClient {
    // ... implementation
}

// Token response
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
    pub token_type: String,
}
```

### Monitoring API

#### Monitoring Types
```rust
// Monitoring system
pub struct MonitoringSystem {
    pub metrics_collector: Arc<MetricsCollector>,
    pub health_checker: Arc<HealthChecker>,
    pub config: MonitoringConfig,
}

// Health checker
pub struct HealthChecker {
    // ... implementation
}

// Metrics collector
pub struct MetricsCollector {
    // ... implementation
}
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](./CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/techgopal/ultrafast-mcp.git
cd ultrafast-mcp

# Install dependencies
cargo build

# Run tests
cargo test

# Run examples
cargo run --example basic-echo
```

### Code Style

- Follow Rust formatting guidelines (`cargo fmt`)
- Run clippy for linting (`cargo clippy`)
- Ensure all tests pass (`cargo test`)
- Add tests for new functionality

## 📄 License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](./LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](./LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

## 🙏 Acknowledgments

- [Model Context Protocol](https://modelcontextprotocol.io/) for the specification
- [Rust Community](https://www.rust-lang.org/community) for the amazing ecosystem
- [Tokio](https://tokio.rs/) for the async runtime
- [Serde](https://serde.rs/) for serialization
- [OpenTelemetry](https://opentelemetry.io/) for observability

---

**UltraFast MCP** - Building the future of AI communication, one protocol at a time! 🚀