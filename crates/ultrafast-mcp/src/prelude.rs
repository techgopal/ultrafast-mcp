//! The ultrafast-mcp prelude.
//!
//! This module re-exports the most commonly used types and traits for ergonomic imports.
//! Use `use ultrafast_mcp::prelude::*;` to bring them all into scope.

pub use crate::{
    UltraFastServer,
    UltraFastClient,
    Context,
    // Server and client info/capabilities
    ServerInfo, ServerCapabilities, ClientInfo, ClientCapabilities,
    // Tool essentials
    Tool, ToolCall, ToolResult, ToolHandler,
    // Resource essentials
    Resource, ReadResourceRequest, ReadResourceResponse,
    // Prompt essentials
    Prompt, PromptMessages, PromptRole,
    // Sampling essentials
    SamplingRequest, SamplingResponse,
    // Error handling
    MCPError, MCPResult,
};

// Capabilities that are almost always needed
pub use ultrafast_mcp_core::types::{
    RootsCapability, SamplingCapability, ElicitationCapability,
};

// AuthConfig only if oauth is enabled
#[cfg(feature = "oauth")]
pub use crate::AuthConfig;

// Monitoring essentials if enabled
#[cfg(feature = "monitoring")]
pub use crate::MonitoringConfig; 