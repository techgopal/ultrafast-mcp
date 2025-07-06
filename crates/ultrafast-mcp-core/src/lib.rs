//! # UltraFast MCP Core
//!
//! Core protocol implementation for the Model Context Protocol (MCP).
//!
//! This crate provides the foundational types, protocol implementations, and utilities
//! for building high-performance MCP-compliant servers and clients. It implements the
//! MCP 2025-06-18 specification with full type safety, comprehensive error handling,
//! and optimized performance characteristics.
//!
//! ## Overview
//!
//! The UltraFast MCP Core crate is designed to be the foundation for all MCP-related
//! functionality. It provides:
//!
//! - **Complete Protocol Implementation**: Full JSON-RPC 2.0 protocol with MCP extensions
//! - **Type Safety**: Strongly typed request/response structures with compile-time guarantees
//! - **Schema Validation**: JSON Schema generation and validation for tool inputs/outputs
//! - **Comprehensive Error Handling**: Detailed error types with context and recovery information
//! - **Performance Optimized**: Zero-copy deserialization and efficient memory usage
//! - **Extensible Architecture**: Modular design for easy extension and customization
//!
//! ## Key Features
//!
//! ### Protocol Implementation
//! - Complete MCP 2025-06-18 specification compliance
//! - JSON-RPC 2.0 protocol with MCP extensions
//! - Lifecycle management (initialize, shutdown, etc.)
//! - Capability negotiation and version management
//!
//! ### Type System
//! - Strongly typed request/response structures
//! - Tool, resource, and prompt type definitions
//! - Comprehensive metadata and context types
//! - Serialization/deserialization with serde
//!
//! ### Schema System
//! - Automatic JSON Schema generation from Rust types
//! - Runtime validation of tool inputs and outputs
//! - Support for complex nested structures and enums
//! - Custom validation rules and constraints
//!
//! ### Utilities
//! - URI handling and validation
//! - Pagination support with cursors
//! - Progress tracking and notifications
//! - Cancellation management
//! - Request/response correlation
//!
//! ## Quick Start
//!
//! ```rust
//! use ultrafast_mcp_core::{
//!     // Protocol types
//!     JsonRpcMessage, JsonRpcRequest, RequestId,
//!     InitializeRequest, InitializeResponse,
//!     // Core types
//!     Tool, ToolCallRequest, ToolCallResponse, ToolContent,
//!     Resource, ReadResourceRequest, ReadResourceResponse,
//!     Prompt, GetPromptRequest, GetPromptResponse,
//!     // Error handling
//!     MCPError, MCPResult,
//!     // Utilities
//!     Uri, ProgressTracker, PaginationParams,
//! };
//! use ultrafast_mcp_core::types::client::ClientInfo;
//! use ultrafast_mcp_core::types::server::ServerInfo;
//!
//! // Create an initialization request
//! let init_request = InitializeRequest {
//!     protocol_version: "2025-06-18".to_string(),
//!     capabilities: Default::default(),
//!     client_info: ClientInfo {
//!         name: "example-client".to_string(),
//!         version: "1.0.0".to_string(),
//!         description: Some("Example MCP client".to_string()),
//!         authors: None,
//!         homepage: None,
//!         license: None,
//!         repository: None,
//!     },
//! };
//!
//! // Create a tool call request
//! let tool_call = ToolCallRequest {
//!     name: "greet".to_string(),
//!     arguments: Some(serde_json::json!({
//!         "name": "Alice",
//!         "greeting": "Hello"
//!     })),
//! };
//!
//! // Handle errors with context
//! fn handle_tool_call(call: ToolCallRequest) -> MCPResult<ToolCallResponse> {
//!     match call.name.as_str() {
//!         "greet" => {
//!             // Process greeting tool
//!             Ok(ToolCallResponse {
//!                 content: vec![ToolContent::text("Hello, Alice!".to_string())],
//!                 is_error: Some(false),
//!             })
//!         }
//!         _ => Err(MCPError::method_not_found(
//!             format!("Unknown tool: {}", call.name)
//!         )),
//!     }
//! }
//! ```
//!
//! ## Architecture
//!
//! The crate is organized into several key modules:
//!
//! - **[`protocol`]**: JSON-RPC protocol implementation, lifecycle management, and message handling
//! - **[`types`]**: Core MCP types for tools, resources, prompts, and client/server information
//! - **[`schema`]**: JSON Schema generation and validation utilities for type-safe tool development
//! - **[`utils`]**: Helper utilities for URIs, pagination, progress tracking, and request management
//! - **[`error`]**: Comprehensive error types with detailed context and recovery information
//!
//! ## Error Handling
//!
//! ```rust
//! use ultrafast_mcp_core::{MCPError, MCPResult};
//!
//! fn process_request(
//!     invalid_protocol: bool,
//!     method_not_found: bool,
//!     internal_failure: bool,
//! ) -> MCPResult<String> {
//!     // Protocol errors
//!     if invalid_protocol {
//!         return Err(MCPError::invalid_request("Invalid protocol version".to_string()));
//!     }
//!
//!     // Method errors
//!     if method_not_found {
//!         return Err(MCPError::method_not_found("Unknown method".to_string()));
//!     }
//!
//!     // Internal errors
//!     if internal_failure {
//!         return Err(MCPError::internal_error("Database connection failed".to_string()));
//!     }
//!
//!     Ok("Success".to_string())
//! }
//! ```
//!
//! ## Performance Considerations
//!
//! - **Zero-copy deserialization** where possible
//! - **Efficient memory usage** with smart pointer usage
//! - **Async/await support** for non-blocking operations
//! - **Minimal allocations** in hot paths
//! - **Optimized serialization** with serde
//!
//! ## Thread Safety
//!
//! All types in this crate are designed to be thread-safe:
//! - Types implement `Send + Sync` where appropriate
//! - Concurrent access is supported through interior mutability
//! - No global state or mutable statics
//!
//! ## Examples
//!
//! See the `examples/` directory for complete working examples:
//! - Basic server and client implementations
//! - Tool development patterns
//! - Error handling best practices
//! - Performance optimization techniques
//!
//! ## Contributing
//!
//! When contributing to this crate:
//! - Follow the established patterns for error handling
//! - Ensure all public APIs are well-documented
//! - Add tests for new functionality
//! - Consider performance implications
//! - Maintain backward compatibility

pub mod config;
pub mod error;
pub mod protocol;
pub mod schema;
pub mod types;
pub mod utils;

pub use error::{MCPError, MCPResult};

// Re-export protocol items
pub use protocol::{
    ImplementationMetadata, InitializeRequest, InitializeResponse, InitializedNotification,
    JsonRpcError, JsonRpcMessage, JsonRpcRequest, JsonRpcResponse, LifecyclePhase, LogLevel,
    LogMessage, Message, Notification, ProgressNotification, ProtocolMetadata, RequestId,
    RequestMetadata, ResponseMetadata, ShutdownRequest,
};

// Re-export types items
pub use types::{
    ClientInfo, GetPromptRequest, GetPromptResponse, ListPromptsRequest, ListPromptsResponse,
    ListResourceTemplatesRequest, ListResourceTemplatesResponse, ListResourcesRequest,
    ListResourcesResponse, ListToolsRequest, ListToolsResponse, ModelHint, ModelPreferences,
    Prompt, PromptArgument, PromptContent, PromptMessage, PromptMessages, PromptRole,
    ReadResourceRequest, ReadResourceResponse, Resource, ResourceContent, ResourceReference,
    ResourceTemplate, ResourceUpdatedNotification, SamplingContent, SamplingMessage,
    SamplingRequest, SamplingResponse, SamplingRole, ServerInfo, SubscribeRequest, Tool,
    ToolCallRequest, ToolCallResponse, ToolContent, UnsubscribeRequest,
};

// Re-export schema items explicitly (using what's actually available)
pub use schema::{
    array_schema, basic_schema, enum_schema, generate_schema_for, object_schema,
    validate_against_schema, validate_tool_input, validate_tool_output, SchemaGeneration,
};

// Re-export utils items explicitly (using what's actually available)
pub use utils::{
    Cursor, PaginationInfo, PaginationParams, Progress, ProgressStatus, ProgressTracker, Uri,
};
