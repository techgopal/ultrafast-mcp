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
//! use ultrafast_mcp_server::{
//!     UltraFastServer, ToolHandler, ToolCall, ToolResult, ToolContent,
//!     ListToolsRequest, ListToolsResponse, ServerInfo, ServerCapabilities,
//!     ToolsCapability, MCPError, MCPResult
//! };
//! use std::sync::Arc;
//!
//! // Define your tool handler
//! struct MyToolHandler;
//!
//! #[async_trait::async_trait]
//! impl ToolHandler for MyToolHandler {
//!     async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult> {
//!         match call.name.as_str() {
//!             "echo" => {
//!                 let message = call.arguments
//!                     .and_then(|args| args.get("message"))
//!                     .and_then(|v| v.as_str())
//!                     .unwrap_or("Hello, World!");
//!
//!                 Ok(ToolResult {
//!                     content: vec![ToolContent::text(message.to_string())],
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
//!                 name: "echo".to_string(),
//!                 description: Some("Echo a message back".to_string()),
//!                 input_schema: serde_json::json!({
//!                     "type": "object",
//!                     "properties": {
//!                         "message": {"type": "string", "default": "Hello, World!"}
//!                     }
//!                 }),
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
//!         name: "example-server".to_string(),
//!         version: "1.0.0".to_string(),
//!         description: Some("An example MCP server".to_string()),
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
//!         .with_tool_handler(Arc::new(MyToolHandler));
//!
//!     // Start the server with STDIO transport
//!     server.run_stdio().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Advanced Server with Multiple Handlers
//!
//! ```rust
//! use ultrafast_mcp_server::{
//!     UltraFastServer, ToolHandler, ResourceHandler, PromptHandler,
//!     ToolCall, ToolResult, ReadResourceRequest, ReadResourceResponse,
//!     GetPromptRequest, GetPromptResponse, MCPError, MCPResult
//! };
//! use std::sync::Arc;
//!
//! // Tool handler implementation
//! struct AdvancedToolHandler;
//!
//! #[async_trait::async_trait]
//! impl ToolHandler for AdvancedToolHandler {
//!     async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult> {
//!         // Implementation details...
//!         todo!()
//!     }
//!
//!     async fn list_tools(&self, _request: ListToolsRequest) -> MCPResult<ListToolsResponse> {
//!         // Implementation details...
//!         todo!()
//!     }
//! }
//!
//! // Resource handler implementation
//! struct FileResourceHandler;
//!
//! #[async_trait::async_trait]
//! impl ResourceHandler for FileResourceHandler {
//!     async fn read_resource(&self, request: ReadResourceRequest) -> MCPResult<ReadResourceResponse> {
//!         // Implementation details...
//!         todo!()
//!     }
//!
//!     // Other resource methods...
//! }
//!
//! // Prompt handler implementation
//! struct TemplatePromptHandler;
//!
//! #[async_trait::async_trait]
//! impl PromptHandler for TemplatePromptHandler {
//!     async fn get_prompt(&self, request: GetPromptRequest) -> MCPResult<GetPromptResponse> {
//!         // Implementation details...
//!         todo!()
//!     }
//!
//!     // Other prompt methods...
//! }
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let server = UltraFastServer::new(server_info, capabilities)
//!         .with_tool_handler(Arc::new(AdvancedToolHandler))
//!         .with_resource_handler(Arc::new(FileResourceHandler))
//!         .with_prompt_handler(Arc::new(TemplatePromptHandler));
//!
//!     // Start with HTTP transport
//!     server.run_streamable_http("127.0.0.1", 8080).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Context and Progress Tracking
//!
//! ```rust
//! use ultrafast_mcp_server::{Context, ProgressTracker};
//!
//! async fn long_running_operation(ctx: &Context) -> MCPResult<()> {
//!     let mut progress = ProgressTracker::new("Processing data", 100);
//!
//!     for i in 0..100 {
//!         // Update progress
//!         progress.update(i, &format!("Processing item {}", i));
//!
//!         // Check for cancellation
//!         if ctx.is_cancelled().await {
//!             return Err(MCPError::request_timeout());
//!         }
//!
//!         // Do work...
//!         tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
//!     }
//!
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

// Re-export main types
pub use context::Context;
pub use server::{ServerState, UltraFastServer};

// Re-export handler traits
pub use handlers::{
    CompletionHandler, ElicitationHandler, PromptHandler, ResourceHandler,
    ResourceSubscriptionHandler, RootsHandler, SamplingHandler, ToolHandler,
};

// Re-export core types for convenience
pub use ultrafast_mcp_core::{
    error::{MCPError, MCPResult},
    protocol::{
        capabilities::{CapabilityNegotiator, ServerCapabilities},
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
