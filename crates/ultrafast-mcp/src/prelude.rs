//! The ultrafast-mcp prelude.
//!
//! This module re-exports the most commonly used types and traits for ergonomic imports.
//! Use `use ultrafast_mcp::prelude::*;` to bring them all into scope.

pub use crate::{
    ClientCapabilities,
    ClientInfo,
    Context,
    // Error handling
    MCPError,
    MCPResult,
    // Prompt essentials
    Prompt,
    PromptMessages,
    PromptRole,
    ReadResourceRequest,
    ReadResourceResponse,
    // Resource essentials
    Resource,
    // Sampling essentials
    SamplingRequest,
    SamplingResponse,
    ServerCapabilities,
    // Server and client info/capabilities
    ServerInfo,
    // Tool essentials
    Tool,
    ToolCall,
    ToolHandler,
    ToolResult,
    UltraFastClient,
    UltraFastServer,
};

// Capabilities that are almost always needed
pub use ultrafast_mcp_core::types::{ElicitationCapability, RootsCapability, SamplingCapability};

// AuthConfig only if oauth is enabled
#[cfg(feature = "oauth")]
pub use crate::AuthConfig;

// Monitoring essentials if enabled
#[cfg(feature = "monitoring")]
pub use crate::MonitoringConfig;
