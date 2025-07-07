//! # UltraFast MCP
//!
//! High-performance, ergonomic Model Context Protocol (MCP) implementation for Rust.
//!
//! This crate provides the primary APIs for building MCP servers and clients with
//! exceptional performance, type safety, and developer experience. It implements
//! the MCP 2025-06-18 specification with modern Rust patterns and async/await support.
//!
//! ## Overview
//!
//! UltraFast MCP is designed to be the definitive Rust implementation of the Model
//! Context Protocol, providing:
//!
//! - **Ergonomic APIs**: Simple, intuitive interfaces for server and client development
//! - **Type Safety**: Compile-time guarantees for protocol compliance
//! - **High Performance**: Optimized for throughput and low latency
//! - **Full Feature Support**: Complete MCP specification implementation
//! - **Modern Rust**: Async/await, traits, and zero-cost abstractions
//! - **Production Ready**: Comprehensive error handling, logging, and monitoring
//!
//! ## Primary APIs
//!
//! ### Server Development
//! - **[`UltraFastServer`]**: Create MCP servers with ergonomic, type-safe APIs
//! - **[`ToolHandler`]**: Implement tool functionality with trait-based interfaces
//! - **[`ResourceHandler`]**: Manage resources and content delivery
//! - **[`PromptHandler`]**: Generate dynamic prompts and content
//!
//! ### Client Development
//! - **[`UltraFastClient`]**: Connect to MCP servers with async/await APIs
//! - **[`Transport`]**: Flexible transport layer with HTTP, STDIO, and custom options
//! - **[`ResourceChangeHandler`]**: Handle resource updates and notifications
//!
//! ## Quick Start
//!
//! ### Creating an MCP Server
//!
//! ```rust
//! use ultrafast_mcp::{
//!     UltraFastServer, ToolHandler, ToolCall, ToolResult, ToolContent,
//!     ListToolsRequest, ListToolsResponse, ServerInfo, ServerCapabilities,
//!     ToolsCapability, MCPError, MCPResult
//! };
//! use ultrafast_mcp_core::Tool;
//! use serde::{Deserialize, Serialize};
//! use std::sync::Arc;
//!
//! // Define your tool input/output types
//! #[derive(Deserialize)]
//! struct GreetRequest {
//!     name: String,
//!     greeting: Option<String>,
//! }
//!
//! #[derive(Serialize)]
//! struct GreetResponse {
//!     message: String,
//!     timestamp: String,
//! }
//!
//! // Implement the tool handler
//! struct GreetToolHandler;
//!
//! #[async_trait::async_trait]
//! impl ToolHandler for GreetToolHandler {
//!     async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult> {
//!         match call.name.as_str() {
//!             "greet" => {
//!                 // Parse the arguments
//!                 let args: GreetRequest = serde_json::from_value(
//!                     call.arguments.unwrap_or_default()
//!                 )?;
//!
//!                 // Generate the response
//!                 let greeting = args.greeting.unwrap_or_else(|| "Hello".to_string());
//!                 let message = format!("{}, {}!", greeting, args.name);
//!
//!                 Ok(ToolResult {
//!                     content: vec![ToolContent::text(message)],
//!                     is_error: Some(false),
//!                 })
//!             }
//!             _ => Err(MCPError::method_not_found(
//!                 format!("Unknown tool: {}", call.name)
//!             )),
//!         }
//!     }
//!
//!     async fn list_tools(&self, _request: ListToolsRequest) -> MCPResult<ListToolsResponse> {
//!         Ok(ListToolsResponse {
//!             tools: vec![Tool {
//!                 name: "greet".to_string(),
//!                 description: "Greet a person by name".to_string(),
//!                 input_schema: serde_json::json!({
//!                     "type": "object",
//!                     "properties": {
//!                         "name": {"type": "string"},
//!                         "greeting": {"type": "string", "default": "Hello"}
//!                     },
//!                     "required": ["name"]
//!                 }),
//!                 output_schema: None,
//!             }],
//!             next_cursor: None,
//!         })
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create server configuration
//!     let server_info = ServerInfo {
//!         name: "greeting-server".to_string(),
//!         version: "1.0.0".to_string(),
//!         description: Some("A simple greeting server".to_string()),
//!         authors: None,
//!         homepage: None,
//!         license: None,
//!         repository: None,
//!     };
//!
//!     let capabilities = ServerCapabilities {
//!         tools: Some(ToolsCapability { list_changed: Some(true) }),
//!         ..Default::default()
//!     };
//!
//!     // Create and configure the server
//!     let server = UltraFastServer::new(server_info, capabilities)
//!         .with_tool_handler(Arc::new(GreetToolHandler));
//!
//!     // Start the server with STDIO transport
//!     server.run_stdio().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Creating an MCP Client
//!
//! ```rust,no_run
//! use ultrafast_mcp::{
//!     UltraFastClient, ClientInfo, ClientCapabilities, ToolCall, ToolResult
//! };
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create client configuration
//!     let client_info = ClientInfo {
//!         name: "greeting-client".to_string(),
//!         version: "1.0.0".to_string(),
//!         authors: None,
//!         description: Some("A simple greeting client".to_string()),
//!         homepage: None,
//!         repository: None,
//!         license: None,
//!     };
//!
//!     let capabilities = ClientCapabilities::default();
//!
//!     // Create the client
//!     let client = UltraFastClient::new(client_info, capabilities);
//!
//!     // Connect to the server using STDIO
//!     client.connect_stdio().await?;
//!
//!     // Call a tool
//!     let tool_call = ToolCall {
//!         name: "greet".to_string(),
//!         arguments: Some(json!({
//!             "name": "Alice",
//!             "greeting": "Hello there"
//!         })),
//!     };
//!
//!     let result = client.call_tool(tool_call).await?;
//!     println!("Server response: {:?}", result);
//!
//!     // Disconnect
//!     client.disconnect().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Advanced Features
//!
//! ### Resource Management
//!
//! ```rust
//! use ultrafast_mcp::{ResourceHandler, ReadResourceRequest, ReadResourceResponse, MCPResult};
//! use ultrafast_mcp_core::ResourceContent;
//!
//! struct FileResourceHandler;
//!
//! #[async_trait::async_trait]
//! impl ResourceHandler for FileResourceHandler {
//!     async fn read_resource(&self, request: ReadResourceRequest) -> MCPResult<ReadResourceResponse> {
//!         // Implement file reading logic
//!         let content = std::fs::read_to_string(&request.uri)?;
//!         
//!         Ok(ReadResourceResponse {
//!             contents: vec![ResourceContent::text(request.uri.clone(), content)],
//!         })
//!     }
//!
//!     async fn list_resources(&self, _request: ultrafast_mcp_core::types::resources::ListResourcesRequest) -> MCPResult<ultrafast_mcp_core::types::resources::ListResourcesResponse> {
//!         Ok(ultrafast_mcp_core::types::resources::ListResourcesResponse {
//!             resources: vec![],
//!             next_cursor: None,
//!         })
//!     }
//!
//!     async fn list_resource_templates(&self, _request: ultrafast_mcp_core::types::resources::ListResourceTemplatesRequest) -> MCPResult<ultrafast_mcp_core::types::resources::ListResourceTemplatesResponse> {
//!         Ok(ultrafast_mcp_core::types::resources::ListResourceTemplatesResponse {
//!             resource_templates: vec![],
//!             next_cursor: None,
//!         })
//!     }
//! }
//! ```
//!
//! ### Progress Tracking
//!
//! ```no_run
//! use ultrafast_mcp::{Context, MCPResult, MCPError};
//!
//! async fn long_running_operation(ctx: &Context) -> MCPResult<()> {
//!     for i in 0..100 {
//!         // Check for cancellation
//!         if ctx.is_cancelled().await {
//!             return Err(MCPError::request_timeout());
//!         }
//!
//!         // Do work...
//!         tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Transport Configuration
//!
//! ```no_run
//! use ultrafast_mcp::{TransportConfig, UltraFastClient, ClientInfo, ClientCapabilities};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Configure custom transport
//!     let transport_config = TransportConfig::Stdio;
//!
//!     // Create client
//!     let client_info = ClientInfo {
//!         name: "example-client".to_string(),
//!         version: "1.0.0".to_string(),
//!         authors: None,
//!         description: None,
//!         homepage: None,
//!         repository: None,
//!         license: None,
//!     };
//!     let capabilities = ClientCapabilities::default();
//!     let client = UltraFastClient::new(client_info, capabilities);
//!     
//!     // Connect using STDIO transport
//!     client.connect_stdio().await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! The crate is built on several foundational components:
//!
//! - **[`ultrafast-mcp-core`]**: Core protocol implementation and types
//! - **[`ultrafast-mcp-server`]**: Server implementation and handler traits
//! - **[`ultrafast-mcp-client`]**: Client implementation and connection management
//! - **[`ultrafast-mcp-transport`]**: Transport layer with HTTP, STDIO, and custom options
//! - **[`ultrafast-mcp-auth`]**: Authentication and authorization support
//! - **[`ultrafast-mcp-monitoring`]**: Observability and monitoring capabilities
//!
//! ## Performance Characteristics
//!
//! - **High Throughput**: Optimized for handling thousands of requests per second
//! - **Low Latency**: Sub-millisecond response times for simple operations
//! - **Memory Efficient**: Minimal allocations and efficient data structures
//! - **Scalable**: Designed for concurrent access and horizontal scaling
//! - **Resource Aware**: Efficient resource usage and cleanup
//!
//! ## Transport Options
//!
//! ### Streamable HTTP (Recommended)
//! - **Performance**: 10x faster than HTTP+SSE under load
//! - **Compatibility**: Works with all HTTP proxies and load balancers
//! - **Features**: Session management, authentication, compression
//! - **Use Case**: Production deployments and high-performance scenarios
//!
//! ### HTTP+SSE (Legacy)
//! - **Compatibility**: Backward compatibility with existing MCP implementations
//! - **Features**: Server-sent events for real-time updates
//! - **Use Case**: Legacy systems and gradual migration
//!
//! ### STDIO
//! - **Performance**: Minimal overhead for local communication
//! - **Security**: Process isolation and simple deployment
//! - **Use Case**: Local development and simple integrations
//!
//! ## Error Handling
//!
//! The crate provides comprehensive error handling:
//!
//! ```rust
//! use ultrafast_mcp::{MCPError, MCPResult};
//!
//! async fn robust_operation() -> MCPResult<String> {
//!     // Simulate an operation that might fail
//!     let result = "success".to_string();
//!     
//!     match result.as_str() {
//!         "success" => Ok(result),
//!         _ => Err(MCPError::internal_error("Operation failed".to_string())),
//!     }
//! }
//! ```
//!
//! ## Monitoring and Observability
//!
//! ```rust
//! #[cfg(feature = "monitoring")]
//! use ultrafast_mcp::{MonitoringSystem, MonitoringConfig};
//!
//! #[cfg(feature = "monitoring")]
//! async fn setup_monitoring() -> anyhow::Result<()> {
//!     let config = MonitoringConfig::default();
//!
//!     let monitoring = MonitoringSystem::new(config);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Best Practices
//!
//! ### Server Development
//! - Use strongly-typed request/response structures
//! - Implement proper error handling and recovery
//! - Provide meaningful progress updates for long operations
//! - Use appropriate transport options for your use case
//! - Implement comprehensive logging and monitoring
//!
//! ### Client Development
//! - Handle connection errors gracefully
//! - Implement retry logic for transient failures
//! - Use appropriate timeouts for operations
//! - Validate responses before processing
//! - Clean up resources properly
//!
//! ### Performance Optimization
//! - Use Streamable HTTP for high-performance scenarios
//! - Implement efficient resource management
//! - Minimize allocations in hot paths
//! - Use appropriate concurrency levels
//! - Monitor and profile your applications
//!
//! ## Examples
//!
//! See the `examples/` directory for complete working examples:
//! - Basic echo server and client
//! - File operations with resource management
//! - HTTP server with network operations
//! - Advanced features with comprehensive MCP capabilities
//!
//! ## Contributing
//!
//! When contributing to this crate:
//! - Follow the established patterns and conventions
//! - Ensure comprehensive test coverage
//! - Consider performance implications
//! - Maintain backward compatibility
//! - Update documentation for new features

// =========================
// Core Protocol and Types
// =========================
// Re-export core protocol types, errors, schema, and utilities
pub use ultrafast_mcp_core::{
    // Errors (re-export as McpCoreError to avoid conflicts)
    error as McpCoreError,
    // Protocol types
    protocol,
    // Schema
    schema,
    // Types
    types,
    // Utils
    utils,
    // Re-export specific error types
    MCPError,
    MCPResult,
};

// Prelude module for convenient imports
pub mod prelude;

// Re-export commonly used types directly for convenience
pub use ultrafast_mcp_core::types::{
    // Client types
    client::{ClientCapabilities, ClientInfo},
    // Completion types
    completion::{CompleteRequest, CompleteResponse, Completion, CompletionValue},
    // Elicitation types
    elicitation::{ElicitationRequest, ElicitationResponse},
    // Notification types
    notifications::{LogLevel, PingResponse},
    // Prompt types
    prompts::{
        GetPromptRequest, GetPromptResponse, ListPromptsRequest, ListPromptsResponse, Prompt,
        PromptArgument, PromptContent, PromptMessages, PromptRole,
    },
    // Resource types
    resources::{
        ListResourcesRequest, ListResourcesResponse, ReadResourceRequest, ReadResourceResponse,
        Resource, ResourceContent, ResourceTemplate,
    },
    // Roots types
    roots::Root,
    // Sampling types
    sampling::{
        CreateMessageRequest, CreateMessageResponse, ModelPreferences, SamplingContent,
        SamplingRequest, SamplingResponse,
    },
    // Server types
    server::{ServerCapabilities, ServerInfo},
    // Tool types
    tools::{ListToolsRequest, ListToolsResponse, Tool, ToolCall, ToolContent, ToolResult},
};

// Re-export capability types from protocol
pub use ultrafast_mcp_core::protocol::capabilities::{
    CompletionCapability, LoggingCapability, PromptsCapability, ResourcesCapability,
    ToolsCapability,
};

// =========================
// Server API
// =========================
pub use ultrafast_mcp_server::{
    CompletionHandler, Context, ContextLogger, ElicitationHandler, LoggerConfig, PromptHandler,
    ResourceHandler, ResourceSubscriptionHandler, RootsHandler, SamplingHandler,
    ServerLoggingConfig, ServerState, ToolHandler, ToolRegistrationError, UltraFastServer,
};

// =========================
// Client API
// =========================
pub use ultrafast_mcp_client::{
    UltraFastClient,
    // If client handler traits are added in future, re-export here
};

// =========================
// Transport Layer
// =========================
pub use ultrafast_mcp_transport::{
    create_recovering_transport,
    create_transport,
    // Middleware
    middleware::{
        LoggingMiddleware, MiddlewareTransport, ProgressMiddleware, RateLimitMiddleware,
        TransportMiddleware, ValidationMiddleware,
    },
    // STDIO
    stdio::StdioTransport,
    Transport,
    TransportConfig,
};

// Streamable HTTP (feature = "http")
#[cfg(feature = "http")]
pub use ultrafast_mcp_transport::streamable_http::{
    create_streamable_http_client_default, create_streamable_http_client_with_middleware,
    create_streamable_http_server_default, create_streamable_http_server_with_middleware,
    HttpTransportConfig, HttpTransportServer, HttpTransportState, StreamableHttpClient,
    StreamableHttpClientConfig,
};

// =========================
// Authentication (feature = "oauth")
// =========================
#[cfg(feature = "oauth")]
pub use ultrafast_mcp_auth::{
    // Re-export auth types (avoiding conflicts with core types)
    error as McpAuthError,
    extract_bearer_token,
    generate_pkce_params,
    generate_session_id,
    generate_state,
    oauth,
    pkce,
    types as AuthTypes,
    validation,
    AuthError,
    AuthResult,
    AuthorizationServerMetadata,
    ClientRegistrationRequest,
    ClientRegistrationResponse,
    OAuthClient,
    // Re-export specific types
    OAuthConfig,
    // Re-export as AuthConfig for convenience
    OAuthConfig as AuthConfig,
    PkceParams,
    TokenClaims,
    TokenResponse,
    TokenValidator,
};

// =========================
// Monitoring (feature = "monitoring")
// =========================
#[cfg(feature = "monitoring")]
pub use ultrafast_mcp_monitoring::{
    config::MonitoringConfig,
    // Re-export monitoring types explicitly for better discoverability
    health::{HealthCheck, HealthCheckResult, HealthChecker, HealthStatus},
    MetricsCollector,
    MonitoringSystem,
    RequestMetrics,
    RequestTimer,
    SystemMetrics,
    TransportMetrics,
    // Also export everything else
    *,
};

// =========================
// Macros - REMOVED
// =========================
// Macros have been removed as they provided no immediate benefit
// and were only stub implementations. The current API is already
// ergonomic and production-ready.
