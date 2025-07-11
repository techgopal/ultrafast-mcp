//! Common trait definitions for UltraFast MCP
//!
//! This module contains common trait patterns that are reused across
//! different crates to ensure consistency and reduce duplication.

pub mod config;
pub mod handler;
pub mod validator;

pub use config::*;
pub use handler::*;
pub use validator::*;
