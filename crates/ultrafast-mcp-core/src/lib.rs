//! # UltraFast MCP Core
//!
//! Core protocol implementation for the Model Context Protocol (MCP).
//!
//! This crate provides the foundational types, protocol implementations, and utilities
//! for building high-performance MCP-compliant servers and clients.

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
    LogMessage, Message, Notification, RequestId,
    RequestMetadata, ResponseMetadata, ShutdownRequest,
};

// Re-export types items
pub use types::{
    ClientInfo, GetPromptRequest, GetPromptResponse, ListPromptsRequest, ListPromptsResponse,
    ListResourceTemplatesRequest, ListResourceTemplatesResponse, ListResourcesRequest,
    ListResourcesResponse, ListToolsRequest, ListToolsResponse, ModelHint, ModelPreferences,
    Prompt, PromptArgument, PromptContent, PromptMessage, PromptMessages, PromptRole,
    ReadResourceRequest, ReadResourceResponse, Resource, ResourceContent, ResourceReference,
    ResourceTemplate, SamplingContent, SamplingMessage,
    SamplingRequest, SamplingResponse, SamplingRole, ServerInfo, SubscribeRequest, Tool,
    ToolCallRequest, ToolCallResponse, ToolContent, UnsubscribeRequest,
    // Notification types
    ToolsListChangedNotification, ResourcesListChangedNotification, PromptsListChangedNotification,
    RootsListChangedNotification, LoggingMessageNotification, 
    LogLevelSetRequest, LogLevelSetResponse, CancelledNotification, ProgressNotification, 
    PingRequest, PingResponse, ClientCapabilityNotification, ServerCapabilityNotification, 
    ConnectionStatusNotification, RequestTimeoutNotification, RateLimitNotification, 
    ConnectionStatus, RateLimitType,
};

// Re-export schema items
pub use schema::{
    array_schema, basic_schema, enum_schema, generate_schema_for, object_schema,
    validate_against_schema, validate_tool_input, validate_tool_output,
};

// Re-export utils items
pub use utils::{
    Cursor, PaginationInfo, PaginationParams, Progress, ProgressStatus, ProgressTracker, Uri,
};
