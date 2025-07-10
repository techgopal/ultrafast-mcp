//! Common configuration trait patterns
//!
//! This module re-exports configuration traits from the config module
//! for easier access and to maintain backward compatibility.

// Re-export config traits from the config module
pub use crate::config::base::{BaseConfig, ConfigDefaults, ConfigBuilder}; 