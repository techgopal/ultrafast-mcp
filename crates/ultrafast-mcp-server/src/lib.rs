//! # UltraFast MCP Server
//!
//! High-performance server implementation for the Model Context Protocol (MCP).
//!
//! This crate provides a complete, production-ready server implementation for the MCP
//! 2025-06-18 specification. It offers ergonomic APIs, comprehensive error handling,
//! and high-performance characteristics suitable for both development and production use.
//!
//! ## Overview
//!
//! The UltraFast MCP Server is designed to be the definitive server implementation
//! for the Model Context Protocol, providing:
//!
//! - **Ergonomic APIs**: Simple, intuitive interfaces for server development
//! - **Type Safety**: Compile-time guarantees for protocol compliance
//! - **High Performance**: Optimized for throughput and low latency
//! - **Full Feature Support**: Complete MCP specification implementation
//! - **Production Ready**: Comprehensive error handling, logging, and monitoring
//! - **Extensible Architecture**: Modular design for easy customization
//!
//! ## Key Features
//!
//! ### Core Server Functionality
//! - **Lifecycle Management**: Connection initialization, shutdown, and state management
//! - **Capability Negotiation**: Feature discovery and negotiation with clients
//! - **Message Handling**: Request/response/notification processing
//! - **Error Handling**: Comprehensive error types and recovery mechanisms
//! - **State Management**: Thread-safe server state and context management
//!
//! ### Handler System
//! - **Tool Handler**: Execute tools and provide results
//! - **Resource Handler**: Manage resources and content delivery
//! - **Prompt Handler**: Generate dynamic prompts and content
//! - **Sampling Handler**: LLM sampling and message generation
//! - **Completion Handler**: Autocompletion and suggestion support
//! - **Elicitation Handler**: User input collection and validation
//! - **Roots Handler**: Filesystem boundary management
//!
//! ### Transport Support
//! - **STDIO Transport**: Local communication with minimal overhead
//! - **HTTP Transport**: Web-based communication with OAuth support
//! - **Streamable HTTP**: High-performance HTTP transport (recommended)
//! - **Custom Transport**: Extensible transport layer architecture
//!
//! ## Architecture
//!
//! The server is built around several core components:
//!
//! ```text
//! ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
//! │   Transport     │    │   Protocol      │    │   Handlers      │
//! │   Layer         │◄──►│   Protocol      │◄──►│   Layer         │
//! └─────────────────┘    └─────────────────┘    └─────────────────┘
//!         │                       │                       │
//!         │                       │                       │
//!         ▼                       ▼                       ▼
//! ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
//! │   Context       │    │   State         │    │   Utilities     │
//! │   Management    │    │   Management    │    │   & Helpers     │
//! └─────────────────┘    └─────────────────┘    └─────────────────┘
//! ```
//!
//! ## Modules
//!
//! - **[`server`]**: Core server implementation and state management
//! - **[`handlers`]**: Trait definitions for all handler types
//! - **[`context`]**: Context management for request processing
//!
//! ## Usage Examples
//!
//! ### Basic Server Setup
//!
//! ```rust
//! use ultrafast_mcp_server::{UltraFastServer, ToolHandler};
//! use ultrafast_mcp_core::types::tools::{ToolCall, ToolResult, ToolContent, Tool, ListToolsRequest, ListToolsResponse};
//! use ultrafast_mcp_core::types::server::ServerInfo;
//! use ultrafast_mcp_core::protocol::capabilities::{ServerCapabilities, ToolsCapability};
//! use ultrafast_mcp_core::error::{MCPError, MCPResult};
//! use std::sync::Arc;
//! // Add anyhow as a dev-dependency for doctests
//! // [dev-dependencies]
//! // anyhow = "1"
//!
//! struct MyToolHandler;
//!
//! #[async_trait::async_trait]
//! impl ToolHandler for MyToolHandler {
//!     async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult> {
//!         match call.name.as_str() {
//!             "echo" => {
//!                 let message = call.arguments
//!                     .and_then(|args| args.get("message").cloned())
//!                     .and_then(|v| v.as_str().map(|s| s.to_string()))
//!                     .unwrap_or_else(|| "Hello, World!".to_string());
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
//!     async fn list_tools(&self, _request: ListToolsRequest) -> MCPResult<ListToolsResponse> {
//!         Ok(ListToolsResponse {
//!             tools: vec![Tool {
//!                 name: "echo".to_string(),
//!                 description: "Echo a message back".to_string(),
//!                 input_schema: serde_json::json!({
//!                     "type": "object",
//!                     "properties": {
//!                         "message": {"type": "string", "default": "Hello, World!"}
//!                     },
//!                     "required": ["message"]
//!                 }),
//!                 output_schema: Some(serde_json::json!({
//!                     "type": "object",
//!                     "properties": {
//!                         "output": {"type": "string"}
//!                     }
//!                 })),
//!             }],
//!             next_cursor: None,
//!         })
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let server_info = ServerInfo {
//!         name: "example-server".to_string(),
//!         version: "1.0.0".to_string(),
//!         description: Some("An example MCP server".to_string()),
//!         authors: None,
//!         homepage: None,
//!         license: None,
//!         repository: None,
//!     };
//!     let capabilities = ServerCapabilities {
//!         tools: Some(ToolsCapability { list_changed: Some(true) }),
//!         ..Default::default()
//!     };
//!     let server = UltraFastServer::new(server_info, capabilities)
//!         .with_tool_handler(Arc::new(MyToolHandler));
//!     // Start the server with STDIO transport
//!     server.run_stdio().await?;
//!     Ok(())
//! }
//! ```
//!
//! ### Advanced Server with Multiple Handlers
//!
//! ```rust
//! use ultrafast_mcp_server::{UltraFastServer, ToolHandler, ResourceHandler, PromptHandler};
//! use ultrafast_mcp_core::types::tools::{ToolCall, ToolResult, ListToolsRequest, ListToolsResponse, ToolContent};
//! use ultrafast_mcp_core::types::resources::{ReadResourceRequest, ReadResourceResponse};
//! use ultrafast_mcp_core::types::prompts::{GetPromptRequest, GetPromptResponse};
//! use ultrafast_mcp_core::error::{MCPError, MCPResult};
//! use std::sync::Arc;
//!
//! struct AdvancedToolHandler;
//!
//! #[async_trait::async_trait]
//! impl ToolHandler for AdvancedToolHandler {
//!     async fn handle_tool_call(&self, _call: ToolCall) -> MCPResult<ToolResult> {
//!         Ok(ToolResult {
//!             content: vec![ToolContent::text("Tool executed successfully".to_string())],
//!             is_error: Some(false),
//!         })
//!     }
//!     async fn list_tools(&self, _request: ListToolsRequest) -> MCPResult<ListToolsResponse> {
//!         Ok(ListToolsResponse {
//!             tools: vec![],
//!             next_cursor: None,
//!         })
//!     }
//! }
//!
//! struct FileResourceHandler;
//!
//! #[async_trait::async_trait]
//! impl ResourceHandler for FileResourceHandler {
//!     async fn read_resource(&self, _request: ReadResourceRequest) -> MCPResult<ReadResourceResponse> {
//!         Ok(ReadResourceResponse {
//!             contents: vec![],
//!         })
//!     }
//!     async fn list_resources(&self, _request: ultrafast_mcp_core::types::resources::ListResourcesRequest) -> MCPResult<ultrafast_mcp_core::types::resources::ListResourcesResponse> {
//!         Ok(ultrafast_mcp_core::types::resources::ListResourcesResponse {
//!             resources: vec![],
//!             next_cursor: None,
//!         })
//!     }
//!     async fn list_resource_templates(&self, _request: ultrafast_mcp_core::types::resources::ListResourceTemplatesRequest) -> MCPResult<ultrafast_mcp_core::types::resources::ListResourceTemplatesResponse> {
//!         Ok(ultrafast_mcp_core::types::resources::ListResourceTemplatesResponse {
//!             resource_templates: vec![],
//!             next_cursor: None,
//!         })
//!     }
//! }
//!
//! struct TemplatePromptHandler;
//!
//! #[async_trait::async_trait]
//! impl PromptHandler for TemplatePromptHandler {
//!     async fn get_prompt(&self, _request: GetPromptRequest) -> MCPResult<GetPromptResponse> {
//!         Ok(GetPromptResponse {
//!             description: None,
//!             messages: vec![],
//!         })
//!     }
//!     async fn list_prompts(&self, _request: ultrafast_mcp_core::types::prompts::ListPromptsRequest) -> MCPResult<ultrafast_mcp_core::types::prompts::ListPromptsResponse> {
//!         Ok(ultrafast_mcp_core::types::prompts::ListPromptsResponse {
//!             prompts: vec![],
//!             next_cursor: None,
//!         })
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let server_info = ultrafast_mcp_core::types::server::ServerInfo {
//!         name: "example-server".to_string(),
//!         version: "1.0.0".to_string(),
//!         description: Some("An example MCP server".to_string()),
//!         authors: None,
//!         homepage: None,
//!         license: None,
//!         repository: None,
//!     };
//!     let capabilities = ultrafast_mcp_core::protocol::capabilities::ServerCapabilities::default();
//!     let server = UltraFastServer::new(server_info, capabilities)
//!         .with_tool_handler(Arc::new(AdvancedToolHandler))
//!         .with_resource_handler(Arc::new(FileResourceHandler))
//!         .with_prompt_handler(Arc::new(TemplatePromptHandler));
//!     // Start with STDIO transport (or replace with HTTP if available)
//!     server.run_stdio().await?;
//!     Ok(())
//! }
//! ```
//!
//! ### Context and Progress Tracking
//!
//! ```rust
//! use ultrafast_mcp_server::{Context};
//! use ultrafast_mcp_core::utils::ProgressTracker;
//! use ultrafast_mcp_core::error::{MCPError, MCPResult};
//!
//! async fn long_running_operation(ctx: &Context) -> MCPResult<()> {
//!     let mut progress = ProgressTracker::new();
//!     for i in 0..100 {
//!         progress.update(&format!("Processing item {}", i), i);
//!         if ctx.is_cancelled().await {
//!             return Err(MCPError::request_timeout());
//!         }
//!         tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
//!     }
//!     progress.complete("All items processed");
//!     Ok(())
//! }
//! ```
//!
//! ## Server States
//!
//! The server operates in several distinct states:
//!
//! - **Uninitialized**: Server created but not yet connected
//! - **Initializing**: Protocol negotiation in progress
//! - **Initialized**: Ready for normal operation
//! - **Shutdown**: Connection termination in progress
//!
//! ## Handler System
//!
//! The server uses a trait-based handler system for extensibility:
//!
//! ### Tool Handler
//! Handles tool execution requests and provides results:
//! - `handle_tool_call`: Execute a specific tool with parameters
//! - `list_tools`: Provide available tools and their schemas
//!
//! ### Resource Handler
//! Manages resource access and content delivery:
//! - `read_resource`: Read resource content by URI
//! - `list_resources`: List available resources
//! - `list_resource_templates`: Provide resource templates
//!
//! ### Prompt Handler
//! Generates dynamic prompts and content:
//! - `get_prompt`: Generate a prompt with arguments
//! - `list_prompts`: List available prompts
//!
//! ### Additional Handlers
//! - **Sampling Handler**: LLM sampling and message generation
//! - **Completion Handler**: Autocompletion and suggestions
//! - **Elicitation Handler**: User input collection
//! - **Roots Handler**: Filesystem boundary management
//!
//! ## Error Handling
//!
//! The server provides comprehensive error handling:
//!
//! - **Protocol Errors**: Invalid requests, unsupported methods
//! - **Handler Errors**: Tool execution failures, resource access issues
//! - **Transport Errors**: Connection failures, timeout issues
//! - **Internal Errors**: Server implementation issues
//!
//! ## Performance Considerations
//!
//! - **Concurrent Processing**: Multiple requests processed simultaneously
//! - **Efficient Memory Usage**: Minimal allocations in hot paths
//! - **Optimized Serialization**: Fast JSON serialization/deserialization
//! - **Resource Management**: Efficient cleanup and resource reuse
//! - **Caching**: Intelligent caching of frequently accessed data
//!
//! ## Thread Safety
//!
//! All server components are designed to be thread-safe:
//! - Handler implementations must be `Send + Sync`
//! - Server state is protected by appropriate synchronization
//! - Concurrent access to shared resources is safe
//! - No mutable global state is used
//!
//! ## Monitoring and Observability
//!
//! The server supports comprehensive monitoring:
//!
//! - **Metrics**: Request counts, response times, error rates
//! - **Logging**: Structured logging with different levels
//! - **Tracing**: Distributed tracing for request flows
//! - **Health Checks**: Server health and readiness endpoints
//!
//! ## Best Practices
//!
//! ### Handler Implementation
//! - Implement proper error handling and recovery
//! - Provide meaningful error messages
//! - Use appropriate timeouts for operations
//! - Implement progress tracking for long operations
//! - Handle cancellation requests gracefully
//!
//! ### Performance Optimization
//! - Use efficient data structures and algorithms
//! - Minimize allocations in hot paths
//! - Implement appropriate caching strategies
//! - Use async/await for I/O operations
//! - Profile and optimize critical paths
//!
//! ### Security Considerations
//! - Validate all input parameters
//! - Implement proper access controls
//! - Use secure transport options
//! - Handle sensitive data appropriately
//! - Implement rate limiting where appropriate
//!
//! ## Examples
//!
//! See the `examples/` directory for complete working examples:
//! - Basic echo server
//! - File operations server
//! - HTTP operations server
//! - Advanced features server

pub mod context;
pub mod handlers;
pub mod server;

pub use context::{Context, ContextLogger, LoggerConfig};
pub use handlers::*;
/// All re-exports for convenience
pub use server::{ServerLoggingConfig, ServerState, ToolRegistrationError, UltraFastServer};

// Re-export transport types for convenience
pub use ultrafast_mcp_transport::{create_transport, Transport, TransportConfig};

#[cfg(feature = "http")]
pub use ultrafast_mcp_transport::streamable_http::server::HttpTransportConfig;

#[cfg(feature = "monitoring")]
pub use ultrafast_mcp_monitoring::metrics::RequestTimer;
#[cfg(feature = "monitoring")]
pub use ultrafast_mcp_monitoring::{HealthStatus, MonitoringConfig, MonitoringSystem};

pub use ultrafast_mcp_core::{
    error::{MCPError, MCPResult},
    protocol::{
        capabilities::ServerCapabilities,
        jsonrpc::{JsonRpcError, JsonRpcMessage, JsonRpcRequest, JsonRpcResponse},
    },
    types::{
        completion::{CompleteRequest, CompleteResponse},
        elicitation::{ElicitationRequest, ElicitationResponse},
        notifications::{
            CancelledNotification, LogLevel, LogLevelSetRequest, LogLevelSetResponse,
            LoggingMessageNotification, PingRequest, ProgressNotification,
        },
        prompts::{
            GetPromptRequest, GetPromptResponse, ListPromptsRequest, ListPromptsResponse, Prompt,
        },
        resources::{
            ListResourceTemplatesRequest, ListResourceTemplatesResponse, ListResourcesRequest,
            ListResourcesResponse, ReadResourceRequest, ReadResourceResponse, Resource,
            ResourceTemplate,
        },
        roots::Root,
        sampling::{CreateMessageRequest, CreateMessageResponse},
        server::ServerInfo,
        tools::{ListToolsRequest, ListToolsResponse, Tool, ToolCall, ToolResult},
    },
    utils::{CancellationManager, PingManager},
};
