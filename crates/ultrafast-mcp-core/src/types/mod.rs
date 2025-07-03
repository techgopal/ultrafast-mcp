//! # Types Module
//!
//! Core type definitions for the Model Context Protocol (MCP).
//!
//! This module provides all the fundamental types used throughout the MCP ecosystem,
//! including tools, resources, prompts, and various request/response structures.
//! These types are designed to be type-safe, serializable, and fully compliant
//! with the MCP 2025-06-18 specification.
//!
//! ## Overview
//!
//! The types module serves as the foundation for all MCP data structures and provides:
//!
//! - **Tool Types**: Tool definitions, calls, and results
//! - **Resource Types**: Resource management and content handling
//! - **Prompt Types**: Prompt generation and management
//! - **Client/Server Types**: Information and capability structures
//! - **Notification Types**: Progress, logging, and status updates
//! - **Utility Types**: Common patterns like pagination and cursors
//!
//! ## Core Concepts
//!
//! ### Tools
//! Tools are the primary mechanism for extending MCP functionality:
//! - **Tool Definition**: Schema, description, and metadata
//! - **Tool Call**: Request to execute a tool with parameters
//! - **Tool Result**: Response from tool execution
//!
//! ### Resources
//! Resources provide access to external data and content:
//! - **Resource Definition**: URI, content type, and metadata
//! - **Resource Content**: Actual data with MIME type information
//! - **Resource Templates**: Reusable resource definitions
//!
//! ### Prompts
//! Prompts enable dynamic content generation:
//! - **Prompt Definition**: Template with arguments and metadata
//! - **Prompt Content**: Generated content with role information
//! - **Prompt Arguments**: Parameters for prompt generation
//!
//! ## Modules
//!
//! - **[`tools`]**: Tool-related types including definitions, calls, and results
//! - **[`resources`]**: Resource management types and content handling
//! - **[`prompts`]**: Prompt generation and template management
//! - **[`client`]**: Client information and capability structures
//! - **[`server`]**: Server information and capability structures
//! - **[`notifications`]**: Progress, logging, and status notification types
//! - **[`completion`]**: Autocompletion and suggestion types
//! - **[`elicitation`]**: User input collection and validation types
//! - **[`sampling`]**: LLM sampling and message generation types
//! - **[`roots`]**: Filesystem root and boundary management types
//!
//! ## Usage Examples
//!
//! ### Tool Definition and Usage
//!
//! ```rust
//! use ultrafast_mcp_core::types::{
//!     Tool, ToolCall, ToolResult, ToolContent,
//!     ListToolsRequest, ListToolsResponse
//! };
//!
//! // Define a tool
//! let tool = Tool {
//!     name: "greet".to_string(),
//!     description: Some("Greet a person by name".to_string()),
//!     input_schema: serde_json::json!({
//!         "type": "object",
//!         "properties": {
//!             "name": {"type": "string"},
//!             "greeting": {"type": "string", "default": "Hello"}
//!         },
//!         "required": ["name"]
//!     }),
//! };
//!
//! // Create a tool call
//! let tool_call = ToolCall {
//!     name: "greet".to_string(),
//!     arguments: Some(serde_json::json!({
//!         "name": "Alice",
//!         "greeting": "Hello there"
//!     })),
//! };
//!
//! // Create a tool result
//! let tool_result = ToolResult {
//!     content: vec![ToolContent::text("Hello there, Alice!".to_string())],
//!     is_error: Some(false),
//! };
//! ```
//!
//! ### Resource Management
//!
//! ```rust
//! use ultrafast_mcp_core::types::{
//!     Resource, ResourceContent, ReadResourceRequest, ReadResourceResponse
//! };
//!
//! // Define a resource
//! let resource = Resource {
//!     uri: "file:///path/to/document.txt".to_string(),
//!     name: "Document".to_string(),
//!     description: Some("A text document".to_string()),
//!     mime_type: "text/plain".to_string(),
//! };
//!
//! // Create a read request
//! let read_request = ReadResourceRequest {
//!     uri: "file:///path/to/document.txt".to_string(),
//! };
//!
//! // Create a read response
//! let read_response = ReadResourceResponse {
//!     contents: vec![ResourceContent::text("Document content".to_string())],
//! };
//! ```
//!
//! ### Prompt Generation
//!
//! ```rust
//! use ultrafast_mcp_core::types::{
//!     Prompt, PromptContent, PromptRole, GetPromptRequest, GetPromptResponse
//! };
//!
//! // Define a prompt
//! let prompt = Prompt {
//!     name: "summarize".to_string(),
//!     description: Some("Summarize text content".to_string()),
//!     arguments: serde_json::json!({
//!         "type": "object",
//!         "properties": {
//!             "text": {"type": "string"},
//!             "max_length": {"type": "integer", "default": 100}
//!         },
//!         "required": ["text"]
//!     }),
//! };
//!
//! // Create a prompt request
//! let prompt_request = GetPromptRequest {
//!     name: "summarize".to_string(),
//!     arguments: Some(serde_json::json!({
//!         "text": "Long text to summarize...",
//!         "max_length": 50
//!     })),
//! };
//!
//! // Create a prompt response
//! let prompt_response = GetPromptResponse {
//!     content: vec![PromptContent {
//!         role: PromptRole::User,
//!         content: vec![PromptContent::text("Summarize this text...".to_string())],
//!     }],
//! };
//! ```
//!
//! ### Client/Server Information
//!
//! ```rust
//! use ultrafast_mcp_core::types::{
//!     ClientInfo, ServerInfo, ClientCapabilities, ServerCapabilities
//! };
//!
//! // Client information
//! let client_info = ClientInfo {
//!     name: "example-client".to_string(),
//!     version: "1.0.0".to_string(),
//!     description: Some("An example MCP client".to_string()),
//! };
//!
//! // Server information
//! let server_info = ServerInfo {
//!     name: "example-server".to_string(),
//!     version: "1.0.0".to_string(),
//!     description: Some("An example MCP server".to_string()),
//! };
//!
//! // Capabilities
//! let client_capabilities = ClientCapabilities {
//!     tools: Some(ToolsCapability { list_changed: Some(true) }),
//!     resources: Some(ResourcesCapability { list_changed: Some(true) }),
//!     prompts: Some(PromptsCapability { list_changed: Some(true) }),
//! };
//! ```
//!
//! ## Serialization
//!
//! All types in this module support serialization and deserialization:
//!
//! - **JSON Serialization**: All types implement `Serialize` and `Deserialize`
//! - **Schema Generation**: Automatic JSON Schema generation for validation
//! - **Type Safety**: Compile-time guarantees for data structure correctness
//! - **Validation**: Runtime validation of data against schemas
//!
//! ## Type Safety
//!
//! The type system provides several safety guarantees:
//!
//! - **Compile-time Validation**: Type checking prevents many runtime errors
//! - **Schema Validation**: Runtime validation ensures data correctness
//! - **Error Handling**: Comprehensive error types for all failure modes
//! - **Null Safety**: Optional fields are properly handled
//!
//! ## Performance Considerations
//!
//! - **Efficient Serialization**: Optimized JSON serialization/deserialization
//! - **Minimal Allocations**: Smart use of references and owned data
//! - **Lazy Validation**: Validation only when necessary
//! - **Memory Efficiency**: Compact data structures where possible
//!
//! ## Thread Safety
//!
//! All types are designed to be thread-safe:
//! - Types implement `Send + Sync` where appropriate
//! - Immutable by default for safety
//! - Interior mutability where needed
//!
//! ## Extensibility
//!
//! The type system is designed to be extensible:
//! - New types can be added without breaking changes
//! - Optional fields allow for backward compatibility
//! - Schema evolution is supported
//! - Custom validation rules can be added

pub mod client;
pub mod completion;
pub mod elicitation;
pub mod notifications;
pub mod prompts;
pub mod resources;
pub mod roots;
pub mod sampling;
pub mod server;
pub mod tools;

pub use client::*;
pub use completion::*;
pub use elicitation::*;
pub use notifications::*;
pub use prompts::*;
pub use resources::*;
pub use roots::*;
pub use sampling::*;
pub use server::*;
pub use tools::*;
