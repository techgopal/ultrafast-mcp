pub mod protocol;
pub mod types;
pub mod schema;
pub mod utils;
pub mod error;

pub use error::{MCPError, MCPResult};

// Re-export protocol items
pub use protocol::{
    JsonRpcRequest, JsonRpcResponse, JsonRpcError, JsonRpcMessage, RequestId,
    InitializeRequest, InitializeResponse, InitializedNotification,
    ShutdownRequest, LifecyclePhase, VersionNegotiator,
    Message, Notification, ProgressNotification, LogMessage, LogLevel,
    ImplementationMetadata, ProtocolMetadata, RequestMetadata, ResponseMetadata,
};

// Re-export types items
pub use types::{
    ServerInfo, ClientInfo, 
    Tool, ToolCallRequest, ToolCallResponse, ToolContent, ResourceReference,
    ListToolsRequest, ListToolsResponse,
    Resource, ResourceTemplate, ResourceContent,
    ReadResourceRequest, ReadResourceResponse,
    ListResourcesRequest, ListResourcesResponse,
    ListResourceTemplatesRequest, ListResourceTemplatesResponse,
    SubscribeRequest, UnsubscribeRequest, ResourceUpdatedNotification,
    Prompt, PromptArgument, PromptMessage, PromptRole, PromptContent,
    GetPromptRequest, GetPromptResponse,
    ListPromptsRequest, ListPromptsResponse, PromptMessages,
    SamplingRequest, SamplingResponse, SamplingMessage, SamplingRole, SamplingContent,
    ModelPreferences, ModelHint,
};

// Re-export schema items explicitly (using what's actually available)
pub use schema::{
    SchemaGeneration,
    validate_against_schema, validate_tool_input, validate_tool_output,
    generate_schema_for, basic_schema, object_schema, array_schema, enum_schema
};

// Re-export utils items explicitly (using what's actually available)  
pub use utils::{
    Uri, Cursor, PaginationParams, PaginationInfo,
    Progress, ProgressStatus, ProgressTracker
};
