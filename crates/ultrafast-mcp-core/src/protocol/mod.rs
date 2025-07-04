//! # Protocol Module
//!
//! Core protocol implementation for the Model Context Protocol (MCP).
//!
//! This module provides the complete protocol stack for MCP communication, including
//! JSON-RPC 2.0 implementation, lifecycle management, capability negotiation, and
//! message handling. It serves as the foundation for all MCP client and server
//! implementations.
//!
//! ## Overview
//!
//! The protocol module implements the MCP 2025-06-18 specification with the following
//! key components:
//!
//! - **JSON-RPC 2.0**: Complete implementation of the JSON-RPC protocol
//! - **Lifecycle Management**: Connection initialization, shutdown, and state management
//! - **Capability Negotiation**: Feature discovery and negotiation between clients and servers
//! - **Message Handling**: Request/response/notification message processing
//! - **Metadata Management**: Protocol metadata and implementation details
//!
//! ## Architecture
//!
//! The protocol is built around several core concepts:
//!
//! ### Message Flow
//! ```
//! Client                    Server
//!   |                         |
//!   |-- Initialize Request -->|
//!   |<-- Initialize Response--|
//!   |-- Tool Call Request --->|
//!   |<-- Tool Call Response--|
//!   |-- Shutdown Request ---->|
//!   |<-- Shutdown Response----|
//! ```
//!
//! ### Lifecycle Phases
//! 1. **Uninitialized**: Connection established but not yet initialized
//! 2. **Initializing**: Protocol negotiation in progress
//! 3. **Initialized**: Ready for normal operation
//! 4. **Shutdown**: Connection termination in progress
//!
//! ## Modules
//!
//! - **[`jsonrpc`]**: JSON-RPC 2.0 protocol implementation with MCP extensions
//! - **[`lifecycle`]**: Connection lifecycle management and state transitions
//! - **[`capabilities`]**: Feature negotiation and capability discovery
//! - **[`messages`]**: Message type definitions and handling
//! - **[`metadata`]**: Protocol metadata and implementation details
//!
//! ## Usage Examples
//!
//! ### Basic Protocol Usage
//!
//! ```rust
//! use ultrafast_mcp_core::protocol::{
//!     InitializeRequest, InitializeResponse, JsonRpcRequest, JsonRpcResponse,
//!     ServerCapabilities, ClientCapabilities, ServerInfo, ClientInfo
//! };
//!
//! // Create initialization request
//! let init_request = InitializeRequest {
//!     protocol_version: "2025-06-18".to_string(),
//!     capabilities: ClientCapabilities::default(),
//!     client_info: ClientInfo {
//!         name: "example-client".to_string(),
//!         version: "1.0.0".to_string(),
//!         description: Some("Example MCP client".to_string()),
//!     },
//! };
//!
//! // Create initialization response
//! let init_response = InitializeResponse {
//!     protocol_version: "2025-06-18".to_string(),
//!     capabilities: ServerCapabilities::default(),
//!     server_info: ServerInfo {
//!         name: "example-server".to_string(),
//!         version: "1.0.0".to_string(),
//!         description: Some("Example MCP server".to_string()),
//!     },
//! };
//! ```
//!
//! ### JSON-RPC Message Handling
//!
//! ```rust
//! use ultrafast_mcp_core::protocol::{
//!     JsonRpcMessage, JsonRpcRequest, JsonRpcResponse, JsonRpcError
//! };
//!
//! fn handle_message(message: JsonRpcMessage) -> Option<JsonRpcResponse> {
//!     match message {
//!         JsonRpcMessage::Request(request) => {
//!             // Process request and return response
//!             Some(JsonRpcResponse::success(
//!                 serde_json::json!({"result": "success"}),
//!                 request.id
//!             ))
//!         }
//!         JsonRpcMessage::Notification(notification) => {
//!             // Process notification (no response needed)
//!             None
//!         }
//!         JsonRpcMessage::Response(_) => {
//!             // Handle response (typically on client side)
//!             None
//!         }
//!     }
//! }
//! ```
//!
//! ### Capability Negotiation
//!
//! ```rust
//! use ultrafast_mcp_core::protocol::{
//!     ServerCapabilities, ClientCapabilities, ToolsCapability
//! };
//!
//! fn negotiate_capabilities(
//!     client_caps: &ClientCapabilities,
//!     server_caps: &ServerCapabilities,
//! ) -> bool {
//!     // Check if server supports tools if client requests them
//!     if client_caps.tools.is_some() && server_caps.tools.is_none() {
//!         return false; // Incompatible capabilities
//!     }
//!
//!     // Additional capability checks...
//!     true
//! }
//! ```
//!
//! ## Protocol Compliance
//!
//! This implementation is designed to be fully compliant with:
//!
//! - **MCP 2025-06-18 Specification**: Complete protocol compliance
//! - **JSON-RPC 2.0**: Standard JSON-RPC protocol implementation
//! - **RFC 2119**: MUST, SHOULD, MAY compliance for protocol requirements
//!
//! ## Error Handling
//!
//! The protocol module provides comprehensive error handling:
//!
//! - **Protocol Errors**: Invalid JSON-RPC format, unsupported methods
//! - **Lifecycle Errors**: Initialization failures, invalid state transitions
//! - **Capability Errors**: Unsupported features, negotiation failures
//! - **Message Errors**: Malformed messages, serialization issues
//!
//! ## Performance Considerations
//!
//! - **Zero-copy parsing**: Where possible, avoid unnecessary allocations
//! - **Efficient serialization**: Optimized JSON serialization/deserialization
//! - **Minimal validation**: Only validate what's necessary for correctness
//! - **Async support**: Non-blocking operations throughout
//!
//! ## Thread Safety
//!
//! All protocol types are designed to be thread-safe:
//! - Types implement `Send + Sync` where appropriate
//! - Message handling is designed for concurrent access
//! - No mutable global state is used
//!
//! ## Extensibility
//!
//! The protocol is designed to be extensible:
//! - New message types can be added without breaking changes
//! - Capability negotiation allows for feature discovery
//! - Metadata system supports implementation-specific extensions
//! - Version negotiation ensures backward compatibility

pub mod capabilities;
pub mod jsonrpc;
pub mod lifecycle;
pub mod messages;
pub mod metadata;
pub mod version;

pub use capabilities::*;
pub use jsonrpc::*;
pub use lifecycle::*;
pub use messages::*;
pub use metadata::*;
pub use version::{constants, ProtocolVersion, VersionNegotiator as NewVersionNegotiator};
