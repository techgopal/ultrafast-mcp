//! The ultrafast-mcp prelude.
//!
//! This module re-exports the most commonly used types and traits for ergonomic imports.
//! Use `use ultrafast_mcp::prelude::*;` to bring them all into scope.

#[cfg(feature = "core")]
pub use crate::{
    ClientCapabilities,
    ClientInfo,
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
    ToolAnnotations,
    ToolCall,
    ToolResult,
    UltraFastClient,
};

// Server types are conditionally compiled to avoid doctest issues
#[cfg(all(feature = "core", not(doc)))]
pub use crate::{Context, ToolHandler, UltraFastServer};

// Capabilities that are almost always needed
#[cfg(feature = "core")]
pub use ultrafast_mcp_core::types::{ElicitationCapability, RootsCapability, SamplingCapability};

// Transport types (available with stdio or http features)
#[cfg(feature = "stdio")]
pub use crate::{Transport, TransportConfig};

// HTTP-specific types (available with http feature)
#[cfg(feature = "http")]
pub use crate::{HttpTransportConfig, StreamableHttpClient, StreamableHttpClientConfig};

// AuthConfig only if oauth is enabled
#[cfg(feature = "oauth")]
pub use crate::AuthConfig;

// Monitoring essentials if enabled
#[cfg(feature = "monitoring")]
pub use crate::MonitoringConfig;
