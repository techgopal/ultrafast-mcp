//! Configuration management for UltraFast MCP
//!
//! This module provides configuration structures and utilities for managing
//! timeouts, connections, and other operational parameters.

// Include the base config module from the config subdirectory
#[path = "config/base.rs"]
pub mod base;

use serde::{Deserialize, Serialize};
use std::time::Duration;

// Re-export common config types
pub use base::{
    BaseConfig, ConfigBuilder, ConfigDefaults, NetworkConfig as BaseNetworkConfig,
    RetryConfig as BaseRetryConfig, SecurityConfig as BaseSecurityConfig,
    TimeoutConfig as BaseTimeoutConfig,
};

/// Timeout configuration for MCP operations
///
/// This is the original timeout config, now with base config support
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimeoutConfig {
    /// Default timeout for all operations
    pub default_timeout: Duration,

    /// Timeout for connection establishment
    pub connect_timeout: Duration,

    /// Timeout for individual requests
    pub request_timeout: Duration,

    /// Timeout for receiving responses
    pub response_timeout: Duration,

    /// Timeout for tool execution
    pub tool_execution_timeout: Duration,

    /// Timeout for resource reading
    pub resource_read_timeout: Duration,

    /// Timeout for prompt generation
    pub prompt_generation_timeout: Duration,

    /// Timeout for sampling operations
    pub sampling_timeout: Duration,

    /// Timeout for completion operations
    pub completion_timeout: Duration,

    /// Timeout for shutdown operations
    pub shutdown_timeout: Duration,

    /// Heartbeat interval for connection health
    pub heartbeat_interval: Duration,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            default_timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            response_timeout: Duration::from_secs(30),
            tool_execution_timeout: Duration::from_secs(300), // 5 minutes
            resource_read_timeout: Duration::from_secs(60),
            prompt_generation_timeout: Duration::from_secs(30),
            sampling_timeout: Duration::from_secs(120), // 2 minutes for LLM calls
            completion_timeout: Duration::from_secs(30),
            shutdown_timeout: Duration::from_secs(5),
            heartbeat_interval: Duration::from_secs(30),
        }
    }
}

impl base::ConfigDefaults for TimeoutConfig {}

impl base::BaseConfig for TimeoutConfig {
    fn validate(&self) -> crate::MCPResult<()> {
        use crate::validation::timeout::validate_timeout;

        validate_timeout(self.default_timeout)?;
        validate_timeout(self.connect_timeout)?;
        validate_timeout(self.request_timeout)?;
        validate_timeout(self.response_timeout)?;
        validate_timeout(self.tool_execution_timeout)?;
        validate_timeout(self.resource_read_timeout)?;
        validate_timeout(self.prompt_generation_timeout)?;
        validate_timeout(self.sampling_timeout)?;
        validate_timeout(self.completion_timeout)?;
        validate_timeout(self.shutdown_timeout)?;
        validate_timeout(self.heartbeat_interval)?;

        Ok(())
    }

    fn config_name(&self) -> &'static str {
        "MCPTimeoutConfig"
    }
}

impl TimeoutConfig {
    /// Create a high-performance configuration with shorter timeouts
    pub fn high_performance() -> Self {
        Self {
            default_timeout: Duration::from_secs(10),
            connect_timeout: Duration::from_secs(3),
            request_timeout: Duration::from_secs(10),
            response_timeout: Duration::from_secs(10),
            tool_execution_timeout: Duration::from_secs(60),
            resource_read_timeout: Duration::from_secs(30),
            prompt_generation_timeout: Duration::from_secs(15),
            sampling_timeout: Duration::from_secs(60),
            completion_timeout: Duration::from_secs(15),
            shutdown_timeout: Duration::from_secs(3),
            heartbeat_interval: Duration::from_secs(15),
        }
    }

    /// Create a long-running configuration with extended timeouts
    pub fn long_running() -> Self {
        Self {
            default_timeout: Duration::from_secs(300), // 5 minutes
            connect_timeout: Duration::from_secs(30),
            request_timeout: Duration::from_secs(300),
            response_timeout: Duration::from_secs(300),
            tool_execution_timeout: Duration::from_secs(1800), // 30 minutes
            resource_read_timeout: Duration::from_secs(600),   // 10 minutes
            prompt_generation_timeout: Duration::from_secs(120),
            sampling_timeout: Duration::from_secs(600), // 10 minutes for complex LLM calls
            completion_timeout: Duration::from_secs(120),
            shutdown_timeout: Duration::from_secs(30),
            heartbeat_interval: Duration::from_secs(60),
        }
    }

    /// Get timeout for a specific operation type
    pub fn get_timeout_for_operation(&self, operation: &str) -> Duration {
        match operation {
            "connect" => self.connect_timeout,
            "request" => self.request_timeout,
            "response" => self.response_timeout,
            "tool_execution" | "tools/call" => self.tool_execution_timeout,
            "resource_read" | "resources/read" => self.resource_read_timeout,
            "prompt_generation" | "prompts/get" => self.prompt_generation_timeout,
            "sampling" | "sampling/createMessage" => self.sampling_timeout,
            "completion" | "completion/complete" => self.completion_timeout,
            "shutdown" => self.shutdown_timeout,
            "heartbeat" | "ping" => self.heartbeat_interval,
            _ => self.default_timeout,
        }
    }

    /// Check if a timeout value is valid
    pub fn validate_timeout(&self, timeout: Duration) -> bool {
        timeout >= Duration::from_millis(100) && timeout <= Duration::from_secs(3600)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_timeout_config() {
        let config = TimeoutConfig::default();
        assert_eq!(config.default_timeout, Duration::from_secs(30));
        assert_eq!(config.connect_timeout, Duration::from_secs(10));
        assert_eq!(config.request_timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_high_performance_timeout_config() {
        let config = TimeoutConfig::high_performance();
        assert_eq!(config.default_timeout, Duration::from_secs(10));
        assert_eq!(config.connect_timeout, Duration::from_secs(3));
        assert!(config.connect_timeout < TimeoutConfig::default().connect_timeout);
    }

    #[test]
    fn test_long_running_timeout_config() {
        let config = TimeoutConfig::long_running();
        assert_eq!(config.default_timeout, Duration::from_secs(300));
        assert_eq!(config.tool_execution_timeout, Duration::from_secs(1800));
        assert!(config.tool_execution_timeout > TimeoutConfig::default().tool_execution_timeout);
    }

    #[test]
    fn test_get_timeout_for_operation() {
        let config = TimeoutConfig::default();
        assert_eq!(
            config.get_timeout_for_operation("connect"),
            config.connect_timeout
        );
        assert_eq!(
            config.get_timeout_for_operation("tools/call"),
            config.tool_execution_timeout
        );
        assert_eq!(
            config.get_timeout_for_operation("unknown"),
            config.default_timeout
        );
    }

    #[test]
    fn test_timeout_validation() {
        let config = TimeoutConfig::default();
        assert!(config.validate_timeout(Duration::from_secs(30)));
        assert!(!config.validate_timeout(Duration::from_millis(50))); // Too short
        assert!(!config.validate_timeout(Duration::from_secs(3700))); // Too long
    }

    #[test]
    fn test_config_validation() {
        let config = TimeoutConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_base_configs() {
        let timeout_config = BaseTimeoutConfig::default();
        assert!(timeout_config.validate().is_ok());
        assert_eq!(timeout_config.config_name(), "TimeoutConfig");

        let retry_config = BaseRetryConfig::default();
        assert!(retry_config.validate().is_ok());
        assert_eq!(retry_config.config_name(), "RetryConfig");

        let network_config = BaseNetworkConfig::default();
        assert!(network_config.validate().is_ok());
        assert_eq!(network_config.config_name(), "NetworkConfig");

        let security_config = BaseSecurityConfig::default();
        assert!(security_config.validate().is_ok());
        assert_eq!(security_config.config_name(), "SecurityConfig");
    }
}
