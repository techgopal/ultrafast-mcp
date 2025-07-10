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
//! use ultrafast_mcp_core::utils::Uri;
//!
//! // Parse a URI
//! let uri = Uri::new("file:///path/to/document.txt");
//! assert_eq!(uri.scheme(), Some("file"));
//! assert!(uri.is_file());
//! assert_eq!(uri.as_str(), "file:///path/to/document.txt");
//!
//! let http_uri = Uri::new("https://api.example.com/v1/resources/123");
//! assert_eq!(http_uri.scheme(), Some("https"));
//! assert!(http_uri.is_http());
//! assert_eq!(http_uri.as_str(), "https://api.example.com/v1/resources/123");
//! ```
//!
//! ### Pagination
//!
//! ```rust
//! use ultrafast_mcp_core::utils::{PaginationParams, PaginationInfo, Cursor};
//!
//! // Create pagination parameters
//! let params = PaginationParams::new().with_limit(10);
//!
//! // Process paginated results
//! let items = vec!["item1", "item2", "item3"];
//! let next_cursor = Some(Cursor::new("cursor_456"));
//!
//! let pagination_info = PaginationInfo::with_total(items.len() as u64, next_cursor.clone());
//!
//! // Generate a cursor
//! let cursor = Cursor::new("prefix");
//! ```
//!
//! ### Progress Tracking
//!
//! ```rust
//! use ultrafast_mcp_core::utils::{ProgressTracker, ProgressStatus};
//!
//! // Create a progress tracker
//! let mut tracker = ProgressTracker::new();
//!
//! // Start and update progress
//! let progress1 = tracker.start("batch1");
//! progress1.update(25);
//! progress1.description = Some("Processing batch 1".to_string());
//!
//! let progress2 = tracker.start("batch2");
//! progress2.update(50);
//! progress2.description = Some("Processing batch 2".to_string());
//!
//! let progress3 = tracker.start("batch3");
//! progress3.update(75);
//! progress3.description = Some("Processing batch 3".to_string());
//!
//! // Complete progress
//! tracker.complete("batch1");
//! tracker.complete("batch2");
//! tracker.complete("batch3");
//!
//! // Check status
//! if let Some(progress) = tracker.get("batch1") {
//!     match progress.status {
//!         ProgressStatus::Running => {
//!             println!("Progress: {}/{} - {}",
//!                 progress.current,
//!                 progress.total.unwrap_or(0),
//!                 progress.description.as_deref().unwrap_or(""));
//!         }
//!         ProgressStatus::Completed => {
//!             println!("Completed");
//!         }
//!         ProgressStatus::Failed => {
//!             println!("Failed");
//!         }
//!         _ => {}
//!     }
//! }
//! ```
//!
//! ### Cancellation Management
//!
//! ```rust
//! use ultrafast_mcp_core::utils::CancellationManager;
//! use std::time::Duration;
//! use ultrafast_mcp_core::MCPError;
//!
//! // Create a cancellation manager
//! let manager = CancellationManager::new();
//!
//! // Register a request
//! let _ = manager.register_request("request_123".into(), "tools/call".to_string());
//!
//! // Cancel a request
//! let _ = manager.cancel_request(&"request_123".into(), Some("User cancelled".to_string()));
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

// New identifiers module for consolidating ID generation functions
pub mod identifiers;

pub use cancellation::*;
pub use pagination::*;
pub use progress::*;
pub use uri::*;
pub use identifiers::*;
