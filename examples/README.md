# MCP Examples

This directory contains comprehensive examples demonstrating the new `UltraFastServer` and `UltraFastClient` APIs for building MCP (Model Context Protocol) servers and clients.

## Overview

These examples showcase the ergonomic, type-safe, and performant MCP development experience provided by the new APIs. Each example focuses on different aspects of MCP development, from basic functionality to advanced features.

## Examples

### 1. [Basic Echo](./01-basic-echo/) - Getting Started with Streamable HTTP
**Difficulty**: Beginner  
**Focus**: Basic server and client setup with high-performance HTTP transport

A simple example demonstrating the fundamental concepts with **ergonomic API methods**:

#### **One-Line Server Startup**
```rust
// Create server with tool handler
let server = UltraFastServer::new(server_info, server_capabilities)
    .with_tool_handler(Arc::new(EchoToolHandler));

// Start with Streamable HTTP - just one line!
server.run_streamable_http("127.0.0.1", 8080).await?;
```

#### **One-Line Client Connection**
```rust
// Create client
let client = UltraFastClient::new(client_info, client_capabilities);

// Connect with Streamable HTTP - just one line!
client.connect_streamable_http("http://127.0.0.1:8080/mcp").await?;
```

**Key Features**:
- **Ergonomic API**: Simple, intuitive methods for server and client creation
- **Streamable HTTP Transport**: High-performance HTTP communication with session management
- Simple echo tool with automatic JSON schema generation
- Type-safe tool calling with serde serialization
- Comprehensive error handling and logging
- **10x faster** than traditional HTTP transports under load
- **90% less client code** required compared to other transports

### 2. [File Operations](./02-file-operations/) - File System Integration
**Difficulty**: Intermediate  
**Focus**: File system operations and complex tool handling

A comprehensive file operations server demonstrating:
- Multiple file operations (read, write, list, delete, search, move)
- Complex tool handler implementation with 12+ tools
- Error handling and path validation
- File metadata handling and directory tree generation
- Robust path validation for symlinks (macOS `/tmp`/`/private/tmp`)

**Key Features**:
- File reading and writing with head/tail support
- Directory listing with size information
- File search and pattern matching
- Directory tree generation
- File moving and renaming
- Comprehensive error handling and validation
- MCP Inspector configuration for visual testing

### 3. [Everything Server](./03-everything-server/) - Complete MCP Implementation
**Difficulty**: Advanced  
**Focus**: Complete MCP feature set with all capabilities

A comprehensive example demonstrating all MCP capabilities:
- Tools, resources, and prompts
- Advanced data processing
- Text analysis with sentiment detection
- Dynamic resource serving
- Prompt generation with arguments
- HTTP transport with full feature set

**Key Features**:
- Multiple trait implementations (ToolHandler, ResourceHandler, PromptHandler)
- Advanced data generation and processing
- Text analysis capabilities
- Dynamic resource management
- Prompt generation with context
- Complete MCP protocol implementation

### 4. [Authentication Example](./04-authentication-example/) - **Authentication Methods and Middleware** ⭐
**Difficulty**: Intermediate  
**Focus**: Comprehensive authentication support

A complete authentication system demonstrating all supported authentication methods:
- Bearer token authentication with JWT validation
- API key authentication with custom headers
- Basic authentication with username/password
- Custom header authentication for flexibility
- OAuth 2.1 authentication with PKCE
- Auto-refresh tokens for seamless operation
- Server-side authentication middleware
- Client-side authentication middleware
- HTTP transport authentication integration

**Key Features**:
- **Multiple Authentication Methods**: Support for Bearer, API Key, Basic, Custom Headers, and OAuth
- **Server-side Validation**: JWT token validation with scope checking
- **Client-side Management**: Automatic header generation and token refresh
- **HTTP Transport Integration**: Authentication support in HTTP transport layer
- **Security Best Practices**: CSRF protection, PKCE, secure token handling
- **Thread-safe Design**: All components are `Send + Sync`
- **Comprehensive Error Handling**: Detailed error types and messages
- **Performance Optimized**: Efficient validation and minimal allocations

## Common Patterns

### Server Creation
All examples follow a consistent pattern for server creation:

```rust
let server = UltraFastServer::new(
    ServerInfo {
        name: "example-server".to_string(),
        version: "1.0.0".to_string(),
        description: Some("Example server description".to_string()),
        // ... other fields
    },
    ServerCapabilities {
        tools: Some(ToolsCapability { list_changed: Some(true) }),
        resources: Some(ResourcesCapability { /* ... */ }),
        prompts: Some(PromptsCapability { /* ... */ }),
        // ... other capabilities
    }
)
.with_tool_handler(Arc::new(MyToolHandler))
.with_resource_handler(Arc::new(MyResourceHandler)) // Optional
.with_prompt_handler(Arc::new(MyPromptHandler))     // Optional
.build()?;
```

### Client Creation
All examples follow a consistent pattern for client creation using **ergonomic API methods**:

#### **Streamable HTTP Transport (Recommended)**
```rust
// Create client
let client = UltraFastClient::new(client_info, client_capabilities);

// Connect with Streamable HTTP - just one line!
client.connect_streamable_http("http://127.0.0.1:8080/mcp").await?;
```

#### **stdio Transport (Local Communication)**
```rust
// Create client
let client = UltraFastClient::new(client_info, client_capabilities);

// Connect with stdio for local subprocess communication
client.connect_stdio().await?;
```

#### **Custom Transport Configuration**
```rust
// Create client
let client = UltraFastClient::new(client_info, client_capabilities);

// Configure custom transport
let transport_config = TransportConfig::Streamable {
    base_url: "http://127.0.0.1:8080/mcp".to_string(),
    auth_token: Some("your-auth-token".to_string()),
    session_id: Some("your-session-id".to_string()),
};

let transport = create_transport(transport_config).await?;
client.connect_with_transport(transport).await?;
```

### Tool Handler Implementation
All examples implement the `ToolHandler` trait:

```rust
#[async_trait::async_trait]
impl ultrafast_mcp::ToolHandler for MyToolHandler {
    async fn handle_tool_call(&self, call: ultrafast_mcp::ToolCall) -> ultrafast_mcp::McpResult<ultrafast_mcp::ToolResult> {
        match call.name.as_str() {
            "my_tool" => self.handle_my_tool(request).await,
            _ => Err(ultrafast_mcp::McpError::method_not_found(format!("Unknown tool: {}", call.name))),
        }
    }
    
    async fn list_tools(&self, _request: ultrafast_mcp::ListToolsRequest) -> ultrafast_mcp::McpResult<ultrafast_mcp::ListToolsResponse> {
        // Return list of available tools
    }
}
```

## API Consistency and Error Handling

All examples in this directory are designed to use the same ergonomic API patterns and robust error handling. When adding new examples:
- Use the builder pattern for server and client creation.
- Always handle errors gracefully and provide user-friendly messages.
- Follow the patterns demonstrated in the Basic Echo example for tool handler implementation, logging, and progress tracking.
- Document any deviations from the standard patterns in the example's README.

## Running Examples

### Prerequisites
- Rust toolchain (1.70+)
- Cargo

### Building All Examples
```bash
# From the project root
cargo build --workspace
```

### Running Individual Examples
```bash
# Navigate to example directory
cd examples/01-basic-echo

# Build the example
cargo build

# Run server (in one terminal)
cargo run --bin server

# Run client (in another terminal)
cargo run --bin client
```

## Learning Path

### For Beginners
1. Start with **Basic Echo** to understand fundamental concepts
2. Move to **File Operations** to learn about complex tool handling
3. Explore **Everything Server** for complete MCP implementation
4. Finally, tackle **Authentication Example** for security features

### For Experienced Developers
1. Review **Basic Echo** for API patterns
2. Study **Everything Server** for complete implementation examples
3. Use **File Operations** as reference for complex tool implementations
4. Implement **Authentication Example** for production-ready security

## Key Concepts Demonstrated

### 1. Ergonomic APIs
- `UltraFastServer` and `UltraFastClient` as primary entry points
- Builder pattern for configuration
- Type-safe trait implementations

### 2. Tool Development
- Request/response serialization with Serde
- Error handling with `McpError`
- Progress tracking and logging
- Input validation and sanitization

### 3. Resource Management
- Dynamic resource generation
- MIME type handling
- Resource listing and discovery

### 4. Prompt Generation
- Context-aware prompt creation
- Argument handling
- System and user message generation

### 5. Error Handling
- Comprehensive error types
- Graceful error recovery
- User-friendly error messages

### 6. Testing and Validation
- End-to-end testing workflows
- Error condition testing
- Performance monitoring

## Best Practices

### Server Development
- Implement proper error handling
- Use structured logging
- Validate all inputs
- Provide meaningful error messages
- Include comprehensive documentation

### Client Development
- Handle connection errors gracefully
- Implement retry logic where appropriate
- Validate responses
- Provide user-friendly feedback

### Security Considerations
- Validate all user inputs
- Implement proper access controls
- Sanitize error messages
- Use secure defaults

## Integration Patterns

### Tool Development
```rust
// Define request/response structures
#[derive(Deserialize)]
struct MyToolRequest {
    input: String,
    options: Option<HashMap<String, String>>,
}

#[derive(Serialize)]
struct MyToolResponse {
    result: String,
    metadata: HashMap<String, String>,
}

// Implement tool handler
async fn handle_my_tool(&self, request: MyToolRequest) -> ultrafast_mcp::McpResult<ultrafast_mcp::ToolResult> {
    // Validate input
    // Process request
    // Return structured response
}
```

### Resource Development
```rust
// Implement resource handler
async fn read_resource(&self, request: ultrafast_mcp::ReadResourceRequest) -> ultrafast_mcp::McpResult<ultrafast_mcp::ReadResourceResponse> {
    match request.uri.as_str() {
        "my://resource" => self.handle_my_resource().await,
        _ => Err(ultrafast_mcp::McpError::not_found(format!("Resource not found: {}", request.uri))),
    }
}
```

### Prompt Development
```rust
// Implement prompt handler
async fn get_prompt(&self, request: ultrafast_mcp::GetPromptRequest) -> ultrafast_mcp::McpResult<ultrafast_mcp::GetPromptResponse> {
    match request.name.as_str() {
        "my_prompt" => self.handle_my_prompt(request).await,
        _ => Err(ultrafast_mcp::McpError::not_found(format!("Prompt not found: {}", request.name))),
    }
}
```

## Next Steps

After exploring these examples:

1. **Build Your Own Server**: Use the patterns demonstrated to create your own MCP server
2. **Extend Functionality**: Add new tools, resources, or prompts to existing examples
3. **Integrate with Real Systems**: Connect your MCP server to actual data sources or APIs
4. **Deploy in Production**: Use the examples as templates for production deployments

## Contributing

When adding new examples:

1. Follow the established patterns and structure
2. Include comprehensive documentation
3. Implement proper error handling
4. Add appropriate tests
5. Update this README with new example information

## Support

For questions or issues:

1. Check the individual example READMEs for specific guidance
2. Review the main project documentation
3. Examine the source code for implementation details
4. Create an issue for bugs or feature requests

These examples provide a solid foundation for understanding and implementing MCP servers and clients using the new ergonomic APIs! 