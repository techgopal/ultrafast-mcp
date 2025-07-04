//! # Utilities Module
//!
//! Common utilities and helper functions for the Model Context Protocol (MCP).
//!
//! This module provides essential utilities that are commonly used across the MCP
//! ecosystem, including URI handling, pagination support, progress tracking,
//! and cancellation management. These utilities are designed to be efficient,
//! thread-safe, and easy to use.
//!
//! ## Overview
//!
//! The utilities module provides several key areas of functionality:
//!
//! - **URI Handling**: Parsing, validation, and manipulation of URIs
//! - **Pagination**: Cursor-based pagination for large result sets
//! - **Progress Tracking**: Progress reporting and status updates
//! - **Cancellation**: Request cancellation and timeout management
//!
//! These utilities are designed to work seamlessly with the MCP protocol and
//! provide consistent behavior across different implementations.
//!
//! ## Key Features
//!
//! ### URI Management
//! - Parse and validate URIs according to RFC 3986
//! - Extract components (scheme, authority, path, query, fragment)
//! - Join and resolve relative URIs
//! - Support for custom URI schemes
//!
//! ### Pagination Support
//! - Cursor-based pagination for efficient data retrieval
//! - Pagination parameter handling and validation
//! - Support for different pagination strategies
//! - Automatic cursor generation and management
//!
//! ### Progress Tracking
//! - Progress reporting with percentage and status
//! - Progress notification support
//! - Progress cleanup and management
//! - Integration with MCP notification system
//!
//! ### Cancellation Management
//! - Request cancellation support
//! - Timeout handling and management
//! - Cancellation notification system
//! - Integration with async/await patterns
//!
//! ## Modules
//!
//! - **[`uri`]**: URI parsing, validation, and manipulation utilities
//! - **[`pagination`]**: Cursor-based pagination support and management
//! - **[`progress`]**: Progress tracking and status reporting utilities
//! - **[`cancellation`]**: Request cancellation and timeout management
//!
//! ## Usage Examples
//!
//! ### URI Handling
//!
//! ```rust
//! use ultrafast_mcp_core::utils::{Uri, UriBuilder};
//!
//! // Parse a URI
//! let uri = Uri::parse("file:///path/to/document.txt").unwrap();
//! assert_eq!(uri.scheme(), "file");
//! assert_eq!(uri.path(), "/path/to/document.txt");
//!
//! // Build a URI
//! let uri = UriBuilder::new()
//!     .scheme("https")
//!     .authority("api.example.com")
//!     .path("/v1/resources")
//!     .query_param("limit", "10")
//!     .query_param("offset", "20")
//!     .build()
//!     .unwrap();
//!
//! assert_eq!(uri.to_string(), "https://api.example.com/v1/resources?limit=10&offset=20");
//!
//! // Join URIs
//! let base = Uri::parse("https://api.example.com/v1/").unwrap();
//! let relative = Uri::parse("resources/123").unwrap();
//! let joined = base.join(&relative).unwrap();
//! assert_eq!(joined.to_string(), "https://api.example.com/v1/resources/123");
//! ```
//!
//! ### Pagination
//!
//! ```rust
//! use ultrafast_mcp_core::utils::{PaginationParams, PaginationInfo, Cursor};
//!
//! // Create pagination parameters
//! let params = PaginationParams::new()
//!     .limit(10)
//!     .cursor(Some("cursor_123".to_string()))
//!     .build();
//!
//! // Process paginated results
//! let items = vec!["item1", "item2", "item3"];
//! let next_cursor = Some("cursor_456".to_string());
//!
//! let pagination_info = PaginationInfo::new()
//!     .items(items.len())
//!     .has_more(next_cursor.is_some())
//!     .next_cursor(next_cursor)
//!     .build();
//!
//! // Generate a cursor
//! let cursor = Cursor::new("prefix", &["key1", "key2"]).to_string();
//! ```
//!
//! ### Progress Tracking
//!
//! ```rust
//! use ultrafast_mcp_core::utils::{ProgressTracker, ProgressStatus};
//!
//! // Create a progress tracker
//! let mut tracker = ProgressTracker::new("Processing items", 100);
//!
//! // Update progress
//! tracker.update(25, "Processing batch 1");
//! tracker.update(50, "Processing batch 2");
//! tracker.update(75, "Processing batch 3");
//! tracker.complete("All items processed");
//!
//! // Check status
//! match tracker.status() {
//!     ProgressStatus::InProgress { current, total, message } => {
//!         println!("Progress: {}/{} - {}", current, total, message);
//!     }
//!     ProgressStatus::Completed { message } => {
//!         println!("Completed: {}", message);
//!     }
//!     ProgressStatus::Failed { error } => {
//!         println!("Failed: {}", error);
//!     }
//! }
//! ```
//!
//! ### Cancellation Management
//!
//! ```rust
//! use ultrafast_mcp_core::utils::{CancellationManager, CancellationToken};
//! use std::time::Duration;
//!
//! // Create a cancellation manager
//! let mut manager = CancellationManager::new();
//!
//! // Register a request
//! let token = manager.register_request("request_123", "tools/call");
//!
//! // Check for cancellation
//! if token.is_cancelled() {
//!     return Err(MCPError::request_timeout());
//! }
//!
//! // Set a timeout
//! let timeout_token = manager.set_timeout("request_123", Duration::from_secs(30));
//!
//! // Cancel a request
//! manager.cancel_request("request_123", "User cancelled");
//!
//! // Clean up expired requests
//! manager.cleanup_expired();
//! ```
//!
//! ## Performance Considerations
//!
//! ### URI Operations
//! - Efficient parsing with minimal allocations
//! - Lazy evaluation of URI components
//! - Caching of parsed URI components
//! - Zero-copy operations where possible
//!
//! ### Pagination
//! - Efficient cursor generation and parsing
//! - Minimal memory overhead for pagination state
//! - Optimized cursor encoding/decoding
//! - Support for large result sets
//!
//! ### Progress Tracking
//! - Lightweight progress state management
//! - Efficient progress notification delivery
//! - Minimal overhead for progress updates
//! - Automatic cleanup of completed progress
//!
//! ### Cancellation
//! - Fast cancellation checks
//! - Efficient timeout management
//! - Minimal memory usage for cancellation state
//! - Automatic cleanup of expired requests
//!
//! ## Thread Safety
//!
//! All utilities are designed to be thread-safe:
//! - URI operations are immutable and thread-safe
//! - Pagination utilities are stateless and thread-safe
//! - Progress tracking supports concurrent access
//! - Cancellation management is thread-safe
//!
//! ## Error Handling
//!
//! The utilities provide comprehensive error handling:
//!
//! - **URI Errors**: Invalid URI format, unsupported schemes
//! - **Pagination Errors**: Invalid cursors, malformed parameters
//! - **Progress Errors**: Invalid progress states, notification failures
//! - **Cancellation Errors**: Timeout failures, cancellation conflicts
//!
//! ## Best Practices
//!
//! ### URI Usage
//! - Always validate URIs before use
//! - Use appropriate URI schemes for different resource types
//! - Handle URI parsing errors gracefully
//! - Consider URI encoding/decoding requirements
//!
//! ### Pagination Strategy
//! - Use cursor-based pagination for large datasets
//! - Provide meaningful cursor values
//! - Handle pagination errors appropriately
//! - Consider performance implications of pagination
//!
//! ### Progress Reporting
//! - Provide meaningful progress messages
//! - Update progress at appropriate intervals
//! - Handle progress notification failures
//! - Clean up progress state when done
//!
//! ### Cancellation Handling
//! - Check for cancellation regularly in long-running operations
//! - Provide appropriate timeout values
//! - Handle cancellation gracefully
//! - Clean up cancellation state when done
//!
//! ## Integration
//!
//! These utilities integrate seamlessly with the MCP ecosystem:
//!
//! - **Protocol Integration**: Work with MCP request/response types
//! - **Transport Integration**: Support for different transport mechanisms
//! - **Error Integration**: Consistent error handling across utilities
//! - **Notification Integration**: Progress and cancellation notifications

pub mod cancellation;
pub mod pagination;
pub mod progress;
pub mod uri;

pub use cancellation::*;
pub use pagination::*;
pub use progress::*;
pub use uri::*;
