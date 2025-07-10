//! Base configuration traits and common implementations
//!
//! This module provides common configuration patterns that can be reused
//! across all crates to reduce duplication.

use std::time::Duration;
use serde::{Serialize, Deserialize};
use crate::validation::timeout::{validate_timeout, validate_retry_count, validate_backoff_multiplier};
use crate::error::{MCPResult, ValidationError};

/// Common configuration defaults
pub trait ConfigDefaults {
    fn default_timeout() -> Duration {
        Duration::from_secs(30)
    }
    
    fn default_retries() -> u32 {
        3
    }
    
    fn default_backoff_multiplier() -> f64 {
        2.0
    }
    
    fn default_host() -> String {
        "127.0.0.1".to_string()
    }
    
    fn default_port() -> u16 {
        8080
    }
}

/// Base configuration trait that all config structs should implement
pub trait BaseConfig: Default + Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync {
    /// Validate the configuration
    fn validate(&self) -> MCPResult<()>;
    
    /// Get configuration name for logging/debugging
    fn config_name(&self) -> &'static str;
}

/// Common timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimeoutConfig {
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
    pub shutdown_timeout: Duration,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(10),
            request_timeout: Self::default_timeout(),
            shutdown_timeout: Duration::from_secs(5),
        }
    }
}

impl ConfigDefaults for TimeoutConfig {}

impl BaseConfig for TimeoutConfig {
    fn validate(&self) -> MCPResult<()> {
        validate_timeout(self.connect_timeout)?;
        validate_timeout(self.request_timeout)?;
        validate_timeout(self.shutdown_timeout)?;
        Ok(())
    }
    
    fn config_name(&self) -> &'static str {
        "TimeoutConfig"
    }
}

/// Common retry configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
    pub enable_jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: Self::default_retries(),
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: Self::default_backoff_multiplier(),
            enable_jitter: true,
        }
    }
}

impl ConfigDefaults for RetryConfig {}

impl BaseConfig for RetryConfig {
    fn validate(&self) -> MCPResult<()> {
        validate_retry_count(self.max_retries)?;
        validate_timeout(self.initial_delay)?;
        validate_timeout(self.max_delay)?;
        validate_backoff_multiplier(self.backoff_multiplier)?;
        
        if self.initial_delay > self.max_delay {
            return Err(ValidationError::InvalidFormat {
                field: "initial_delay".to_string(),
                expected: "less than or equal to max_delay".to_string(),
            }.into());
        }
        
        Ok(())
    }
    
    fn config_name(&self) -> &'static str {
        "RetryConfig"
    }
}

/// Common network configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkConfig {
    pub host: String,
    pub port: u16,
    pub bind_address: Option<String>,
    pub enable_keepalive: bool,
    pub keepalive_timeout: Duration,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            host: Self::default_host(),
            port: Self::default_port(),
            bind_address: None,
            enable_keepalive: true,
            keepalive_timeout: Duration::from_secs(60),
        }
    }
}

impl ConfigDefaults for NetworkConfig {}

impl BaseConfig for NetworkConfig {
    fn validate(&self) -> MCPResult<()> {
        if self.host.is_empty() {
            return Err(ValidationError::RequiredField {
                field: "host".to_string(),
            }.into());
        }
        
        if self.port == 0 {
            return Err(ValidationError::ValueOutOfRange {
                field: "port".to_string(),
                min: "1".to_string(),
                max: "65535".to_string(),
                actual: "0".to_string(),
            }.into());
        }
        
        validate_timeout(self.keepalive_timeout)?;
        
        Ok(())
    }
    
    fn config_name(&self) -> &'static str {
        "NetworkConfig"
    }
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SecurityConfig {
    pub enable_tls: bool,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
    pub ca_cert_path: Option<String>,
    pub allow_insecure: bool,
    pub allowed_origins: Vec<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_tls: false,
            cert_path: None,
            key_path: None,
            ca_cert_path: None,
            allow_insecure: true, // Default to true for development
            allowed_origins: vec!["*".to_string()], // Default to allow all for development
        }
    }
}

impl BaseConfig for SecurityConfig {
    fn validate(&self) -> MCPResult<()> {
        if self.enable_tls {
            if self.cert_path.is_none() {
                return Err(ValidationError::RequiredField {
                    field: "cert_path".to_string(),
                }.into());
            }
            
            if self.key_path.is_none() {
                return Err(ValidationError::RequiredField {
                    field: "key_path".to_string(),
                }.into());
            }
        }
        
        Ok(())
    }
    
    fn config_name(&self) -> &'static str {
        "SecurityConfig"
    }
}

/// Comprehensive configuration builder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigBuilder<T> {
    config: T,
}

impl<T: BaseConfig> ConfigBuilder<T> {
    pub fn new(config: T) -> Self {
        Self { config }
    }
    
    pub fn build(self) -> MCPResult<T> {
        self.config.validate()?;
        Ok(self.config)
    }
    
    pub fn validate(&self) -> MCPResult<()> {
        self.config.validate()
    }
}

/// Macro to implement common configuration patterns
#[macro_export]
macro_rules! impl_config_defaults {
    ($config_type:ty, $name:expr) => {
        impl $crate::config::base::ConfigDefaults for $config_type {}
        
        impl $crate::config::base::BaseConfig for $config_type {
            fn validate(&self) -> $crate::MCPResult<()> {
                // Default implementation - override if needed
                Ok(())
            }
            
            fn config_name(&self) -> &'static str {
                $name
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeout_config_defaults() {
        let config = TimeoutConfig::default();
        assert_eq!(config.connect_timeout, Duration::from_secs(10));
        assert_eq!(config.request_timeout, Duration::from_secs(30));
        assert_eq!(config.shutdown_timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_timeout_config_validation() {
        let config = TimeoutConfig::default();
        assert!(config.validate().is_ok());
        
        let invalid_config = TimeoutConfig {
            connect_timeout: Duration::from_millis(50), // Too short
            request_timeout: Duration::from_secs(30),
            shutdown_timeout: Duration::from_secs(5),
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_retry_config_defaults() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay, Duration::from_millis(100));
        assert_eq!(config.max_delay, Duration::from_secs(30));
        assert_eq!(config.backoff_multiplier, 2.0);
        assert!(config.enable_jitter);
    }

    #[test]
    fn test_retry_config_validation() {
        let config = RetryConfig::default();
        assert!(config.validate().is_ok());
        
        let invalid_config = RetryConfig {
            max_retries: 15, // Too many
            ..Default::default()
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_network_config_defaults() {
        let config = NetworkConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert!(config.enable_keepalive);
    }

    #[test]
    fn test_security_config_defaults() {
        let config = SecurityConfig::default();
        assert!(!config.enable_tls);
        assert!(config.allow_insecure);
        assert_eq!(config.allowed_origins, vec!["*"]);
    }

    #[test]
    fn test_config_builder() {
        let config = TimeoutConfig::default();
        let builder = ConfigBuilder::new(config);
        let built_config = builder.build().unwrap();
        assert_eq!(built_config.config_name(), "TimeoutConfig");
    }
} 