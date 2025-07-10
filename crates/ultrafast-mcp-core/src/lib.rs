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
pub mod traits;
pub mod types;
pub mod utils;
pub mod validation;

pub use error::{MCPError, MCPResult};

// Re-export protocol items
pub use protocol::{
    ImplementationMetadata, InitializeRequest, InitializeResponse, InitializedNotification,
    JsonRpcError, JsonRpcMessage, JsonRpcRequest, JsonRpcResponse, LifecyclePhase, LogLevel,
    LogMessage, Message, Notification, RequestId, RequestMetadata, ResponseMetadata,
    ShutdownRequest,
};

// Re-export types items
pub use types::{
    CancelledNotification,
    ClientCapabilityNotification,
    ClientInfo,
    ConnectionStatus,
    ConnectionStatusNotification,
    GetPromptRequest,
    GetPromptResponse,
    ListPromptsRequest,
    ListPromptsResponse,
    ListResourceTemplatesRequest,
    ListResourceTemplatesResponse,
    ListResourcesRequest,
    ListResourcesResponse,
    ListToolsRequest,
    ListToolsResponse,
    LogLevelSetRequest,
    LogLevelSetResponse,
    LoggingMessageNotification,
    ModelHint,
    ModelPreferences,
    PingRequest,
    PingResponse,
    ProgressNotification,
    Prompt,
    PromptArgument,
    PromptContent,
    PromptMessage,
    PromptMessages,
    PromptRole,
    PromptsListChangedNotification,
    RateLimitNotification,
    RateLimitType,
    ReadResourceRequest,
    ReadResourceResponse,
    RequestTimeoutNotification,
    Resource,
    ResourceContent,
    ResourceReference,
    ResourceTemplate,
    ResourcesListChangedNotification,
    RootsListChangedNotification,
    SamplingContent,
    SamplingMessage,
    SamplingRequest,
    SamplingResponse,
    SamplingRole,
    ServerCapabilityNotification,
    ServerInfo,
    SubscribeRequest,
    Tool,
    ToolCallRequest,
    ToolCallResponse,
    ToolContent,
    // Notification types
    ToolsListChangedNotification,
    UnsubscribeRequest,
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
