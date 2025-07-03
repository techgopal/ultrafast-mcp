//! MCP server implementation for ULTRAFAST MCP
//!
//! This crate provides a high-level server implementation for the Model Context Protocol.

pub mod handlers;
pub mod server;
pub mod context;

// Re-export main types
pub use server::{ServerState, UltraFastServer};
pub use context::Context;

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
