//! Common trait definitions for UltraFast MCP
//!
//! This module contains common trait patterns that are reused across
//! different crates to ensure consistency and reduce duplication.

pub mod handler;
pub mod config;
pub mod validator;

pub use handler::*;
pub use config::*;
pub use validator::*; 