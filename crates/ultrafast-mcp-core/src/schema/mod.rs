//! # Schema Module
//!
//! JSON Schema generation and validation for the Model Context Protocol (MCP).
//!
//! This module provides comprehensive schema functionality for MCP tools, resources,
//! and other data structures. It enables type-safe development by automatically
//! generating JSON schemas from Rust types and providing runtime validation
//! capabilities.
//!
//! ## Overview
//!
//! The schema module provides two main capabilities:
//!
//! - **Schema Generation**: Automatic JSON Schema generation from Rust types
//! - **Schema Validation**: Runtime validation of data against schemas
//!
//! This enables developers to:
//! - Define strongly-typed data structures in Rust
//! - Automatically generate JSON schemas for tool inputs/outputs
//! - Validate incoming data at runtime
//! - Ensure type safety across the MCP ecosystem
//!
//! ## Key Features
//!
//! ### Automatic Schema Generation
//! - Generate JSON schemas from Rust structs and enums
//! - Support for complex nested structures
//! - Custom validation rules and constraints
//! - Documentation preservation in schemas
//!
//! ### Runtime Validation
//! - Validate JSON data against schemas
//! - Detailed error reporting with context
//! - Performance-optimized validation
//! - Support for custom validation rules
//!
//! ### Tool Integration
//! - Automatic schema generation for tool inputs
//! - Runtime validation of tool parameters
//! - Schema-based documentation generation
//! - Type-safe tool development
//!
//! ## Modules
//!
//! - **[`generation`]**: JSON Schema generation from Rust types
//! - **[`validation`]**: Runtime validation of data against schemas
//!
//! ## Usage Examples
//!
//! ### Basic Schema Generation
//!
//! ```rust
//! use ultrafast_mcp_core::schema::{generate_schema_for, SchemaGeneration};
//! use serde::{Deserialize, Serialize};
//! use schemars::JsonSchema;
//!
//! #[derive(Serialize, Deserialize, JsonSchema)]
//! struct UserInput {
//!     name: String,
//!     age: u32,
//!     email: Option<String>,
//! }
//!
//! // Generate schema for the type
//! let schema = generate_schema_for::<UserInput>();
//! println!("Generated schema: {}", serde_json::to_string_pretty(&schema).unwrap());
//! ```
//!
//! ### Tool Schema Definition
//!
//! ```rust
//! use ultrafast_mcp_core::schema::{generate_schema_for, validate_tool_input};
//! use serde::{Deserialize, Serialize};
//! use schemars::JsonSchema;
//!
//! #[derive(Serialize, Deserialize, JsonSchema)]
//! struct GreetToolInput {
//!     name: String,
//!     greeting: Option<String>,
//!     formal: Option<bool>,
//! }
//!
//! #[derive(Serialize, Deserialize, JsonSchema)]
//! struct GreetToolOutput {
//!     message: String,
//!     timestamp: String,
//! }
//!
//! // Generate schemas for tool
//! let input_schema = generate_schema_for::<GreetToolInput>();
//! let output_schema = generate_schema_for::<GreetToolOutput>();
//!
//! // Validate input at runtime
//! let input_data = serde_json::json!({
//!     "name": "Alice",
//!     "greeting": "Hello",
//!     "formal": true
//! });
//!
//! match validate_tool_input(&input_schema, &input_data) {
//!     Ok(()) => println!("Input is valid"),
//!     Err(errors) => println!("Validation errors: {:?}", errors),
//! }
//! ```
//!
//! ### Complex Schema Generation
//!
//! ```rust
//! use ultrafast_mcp_core::schema::{generate_schema_for, object_schema, array_schema, basic_schema, enum_schema};
//! use serde::{Deserialize, Serialize};
//! use schemars::JsonSchema;
//!
//! #[derive(Serialize, Deserialize, JsonSchema)]
//! enum UserRole {
//!     Admin,
//!     User,
//!     Guest,
//! }
//!
//! #[derive(Serialize, Deserialize, JsonSchema)]
//! struct User {
//!     id: u64,
//!     name: String,
//!     role: UserRole,
//!     permissions: Vec<String>,
//!     metadata: std::collections::HashMap<String, serde_json::Value>,
//! }
//!
//! // Generate schema for complex type
//! let schema = generate_schema_for::<User>();
//!
//! // Or build schema manually
//! let manual_schema = object_schema(vec![
//!     ("id", basic_schema("integer"), true),
//!     ("name", basic_schema("string"), true),
//!     ("role", enum_schema(vec!["Admin", "User", "Guest"]), true),
//!     ("permissions", array_schema(basic_schema("string")), false),
//! ]);
//! ```
//!
//! ### Custom Validation
//!
//! ```rust
//! use ultrafast_mcp_core::schema::{validate_against_schema, object_schema, basic_schema};
//! use serde_json::json;
//!
//! // Create a custom schema with validation rules
//! let schema = object_schema(vec![
//!     ("age", serde_json::json!({
//!         "type": "integer",
//!         "minimum": 0,
//!         "maximum": 150
//!     }), true),
//!     ("email", serde_json::json!({
//!         "type": "string",
//!         "format": "email"
//!     }), true),
//!     ("password", serde_json::json!({
//!         "type": "string",
//!         "minLength": 8
//!     }), true),
//! ]);
//!
//! // Validate data
//! let data = json!({
//!     "age": 25,
//!     "email": "user@example.com",
//!     "password": "securepass123"
//! });
//!
//! match validate_against_schema(&schema, &data) {
//!     Ok(()) => println!("Data is valid"),
//!     Err(errors) => println!("Validation errors: {:?}", errors),
//! }
//! ```
//!
//! ## Schema Features
//!
//! ### Supported Types
//! - **Primitive Types**: String, number, boolean, null
//! - **Complex Types**: Objects, arrays, enums
//! - **Optional Fields**: Nullable and optional properties
//! - **Nested Structures**: Recursive schema generation
//! - **Generic Types**: Support for generic Rust types
//!
//! ### Validation Rules
//! - **Type Validation**: Ensure correct data types
//! - **Range Validation**: Min/max values for numbers
//! - **Length Validation**: String and array length constraints
//! - **Pattern Validation**: Regular expression matching
//! - **Format Validation**: Email, URI, date-time formats
//! - **Required Fields**: Mandatory property validation
//!
//! ### Custom Extensions
//! - **Custom Keywords**: Extend schemas with custom validation
//! - **Documentation**: Preserve Rust documentation in schemas
//! - **Examples**: Include example values in schemas
//! - **Metadata**: Add custom metadata to schemas
//!
//! ## Performance Considerations
//!
//! - **Lazy Generation**: Schemas are generated on-demand
//! - **Caching**: Generated schemas can be cached for reuse
//! - **Optimized Validation**: Fast validation algorithms
//! - **Minimal Allocations**: Efficient memory usage
//!
//! ## Thread Safety
//!
//! All schema operations are thread-safe:
//! - Schema generation is stateless and thread-safe
//! - Validation operations are concurrent-safe
//! - No mutable global state is used
//!
//! ## Error Handling
//!
//! The schema module provides comprehensive error handling:
//!
//! - **Validation Errors**: Detailed error messages with context
//! - **Generation Errors**: Clear error messages for schema generation
//! - **Type Errors**: Helpful messages for type conversion issues
//! - **Recovery**: Suggestions for fixing validation errors
//!
//! ## Best Practices
//!
//! ### Schema Design
//! - Use descriptive property names
//! - Provide clear documentation
//! - Include example values
//! - Use appropriate validation rules
//!
//! ### Validation Strategy
//! - Validate early in the request pipeline
//! - Provide clear error messages
//! - Consider performance impact
//! - Cache validation results when appropriate
//!
//! ### Type Safety
//! - Use strongly-typed Rust structs
//! - Leverage compile-time type checking
//! - Validate at runtime for external data
//! - Maintain consistency between types and schemas

/// Trait for types that can provide a JSON schema and schema name
pub trait McpSchema {
    fn schema() -> serde_json::Value;
    fn schema_name() -> String;
}

pub mod generation;
pub mod validation;

pub use generation::*;
pub use validation::*;
