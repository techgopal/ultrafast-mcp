//! Test utilities for UltraFast MCP
//!
//! This crate provides common test fixtures and utilities to reduce duplication
//! across test files in the UltraFast MCP ecosystem.

pub mod fixtures;
pub mod assertions;
pub mod mocks;

pub use fixtures::*;
pub use assertions::*;
pub use mocks::*; 