//! JSON Schema generation and validation for the Model Context Protocol (MCP).

pub mod generation;
pub mod validation;

pub use generation::{array_schema, basic_schema, enum_schema, generate_schema_for, object_schema};

pub use validation::{validate_against_schema, validate_tool_input, validate_tool_output};
