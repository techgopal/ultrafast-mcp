//! # UltraFast MCP Core
//!
//! Core protocol implementation for the Model Context Protocol (MCP).
//!
//! This crate provides the foundational types, protocol implementations, and utilities
//! for building MCP-compliant servers and clients. It implements the MCP 2025-06-18
//! specification with full type safety and comprehensive error handling.
//!
//! ## Features
//!
//! - **Protocol Implementation**: Complete JSON-RPC 2.0 protocol with MCP extensions
//! - **Type Safety**: Strongly typed request/response structures
//! - **Schema Validation**: JSON Schema generation and validation
//! - **Error Handling**: Comprehensive error types and handling
//! - **Utilities**: URI handling, pagination, progress tracking, and more
//!
//! ## Example
//!
//! ```rust
//! use ultrafast_mcp_core::{
//!     JsonRpcMessage, JsonRpcRequest, RequestId,
//!     InitializeRequest, InitializeResponse,
//!     Tool, ToolCallRequest, ToolCallResponse
//! };
//!
//! // Create an initialization request
//! let init_request = InitializeRequest {
//!     protocol_version: "2025-06-18".to_string(),
//!     capabilities: Default::default(),
//!     client_info: Default::default(),
//! };
//!
//! // Create a tool call request
//! let tool_call = ToolCallRequest {
//!     name: "greet".to_string(),
//!     arguments: Some(serde_json::json!({"name": "Alice"})),
//! };
//! ```
//!
//! ## Modules
//!
//! - [`protocol`]: JSON-RPC protocol implementation and lifecycle management
//! - [`types`]: Core MCP types for tools, resources, prompts, and more
//! - [`schema`]: JSON Schema generation and validation utilities
//! - [`utils`]: Helper utilities for URIs, pagination, progress tracking
//! - [`error`]: Error types and handling

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
    RequestMetadata, ResponseMetadata, ShutdownRequest, VersionNegotiator,
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
