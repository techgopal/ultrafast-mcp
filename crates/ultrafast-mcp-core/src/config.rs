//! Configuration constants for UltraFast MCP Core

/// Protocol configuration constants
pub const CURRENT_VERSION: &str = "2025-06-18";
pub const SUPPORTED_VERSIONS: &[&str] = &["2025-06-18", "2025-03-26", "2024-11-05"];
pub const JSONRPC_VERSION: &str = "2.0";

/// Request ID configuration constants
pub const MAX_REQUEST_ID_LENGTH: usize = 1000;
pub const MIN_REQUEST_ID_NUMBER: i64 = -999_999_999;
pub const MAX_REQUEST_ID_NUMBER: i64 = 999_999_999;

/// Validation configuration constants
pub const MAX_TOOL_NAME_LENGTH: usize = 100;
pub const MAX_METHOD_NAME_LENGTH: usize = 100;
pub const MAX_URI_LENGTH: usize = 2048;
pub const MAX_ERROR_MESSAGE_LENGTH: usize = 1000;

/// Performance configuration constants
pub const DEFAULT_REQUEST_TIMEOUT: u64 = 30;
pub const DEFAULT_CONNECTION_TIMEOUT: u64 = 10;
pub const MAX_CONCURRENT_REQUESTS: usize = 100;
pub const DEFAULT_BUFFER_SIZE: usize = 8192;

/// Security configuration constants
pub const MAX_AUTH_ATTEMPTS: u32 = 3;
pub const TOKEN_EXPIRATION_TIME: u64 = 3600; // 1 hour
pub const MAX_TOKEN_LENGTH: usize = 4096;

/// Logging configuration constants
pub const DEFAULT_LOG_LEVEL: &str = "info";
pub const MAX_LOG_MESSAGE_LENGTH: usize = 10000;
pub const DEFAULT_LOG_FORMAT: &str = "json";

/// Timeout configuration constants (MCP 2025-06-18 compliance)
pub const DEFAULT_INITIALIZE_TIMEOUT: u64 = 30;      // 30 seconds for initialization
pub const DEFAULT_OPERATION_TIMEOUT: u64 = 300;      // 5 minutes for normal operations
pub const DEFAULT_TOOL_CALL_TIMEOUT: u64 = 60;       // 1 minute for tool calls
pub const DEFAULT_RESOURCE_TIMEOUT: u64 = 30;        // 30 seconds for resource operations
pub const DEFAULT_SAMPLING_TIMEOUT: u64 = 600;       // 10 minutes for sampling operations
pub const DEFAULT_ELICITATION_TIMEOUT: u64 = 300;    // 5 minutes for elicitation
pub const DEFAULT_COMPLETION_TIMEOUT: u64 = 30;      // 30 seconds for completion
pub const DEFAULT_PING_TIMEOUT: u64 = 10;            // 10 seconds for ping/pong
pub const DEFAULT_SHUTDOWN_TIMEOUT: u64 = 30;        // 30 seconds for graceful shutdown
pub const MAX_OPERATION_TIMEOUT: u64 = 3600;         // 1 hour maximum timeout
pub const MIN_OPERATION_TIMEOUT: u64 = 1;            // 1 second minimum timeout

/// Progress notification configuration
pub const DEFAULT_PROGRESS_INTERVAL: u64 = 5;        // 5 seconds between progress updates
pub const MAX_PROGRESS_INTERVAL: u64 = 60;           // 1 minute maximum progress interval
pub const MIN_PROGRESS_INTERVAL: u64 = 1;            // 1 second minimum progress interval

/// Cancellation configuration
pub const DEFAULT_CANCELLATION_TIMEOUT: u64 = 10;    // 10 seconds for cancellation to take effect
pub const MAX_CANCELLATION_TIMEOUT: u64 = 60;        // 1 minute maximum cancellation timeout

use std::time::Duration;

/// Comprehensive timeout configuration for MCP operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeoutConfig {
    /// Timeout for initialization phase
    pub initialize_timeout: Duration,
    /// Timeout for normal operations
    pub operation_timeout: Duration,
    /// Timeout for tool calls
    pub tool_call_timeout: Duration,
    /// Timeout for resource operations
    pub resource_timeout: Duration,
    /// Timeout for sampling operations
    pub sampling_timeout: Duration,
    /// Timeout for elicitation operations
    pub elicitation_timeout: Duration,
    /// Timeout for completion operations
    pub completion_timeout: Duration,
    /// Timeout for ping/pong operations
    pub ping_timeout: Duration,
    /// Timeout for shutdown operations
    pub shutdown_timeout: Duration,
    /// Timeout for cancellation to take effect
    pub cancellation_timeout: Duration,
    /// Interval for progress notifications
    pub progress_interval: Duration,
    /// Maximum timeout for any operation
    pub max_timeout: Duration,
    /// Minimum timeout for any operation
    pub min_timeout: Duration,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            initialize_timeout: Duration::from_secs(DEFAULT_INITIALIZE_TIMEOUT),
            operation_timeout: Duration::from_secs(DEFAULT_OPERATION_TIMEOUT),
            tool_call_timeout: Duration::from_secs(DEFAULT_TOOL_CALL_TIMEOUT),
            resource_timeout: Duration::from_secs(DEFAULT_RESOURCE_TIMEOUT),
            sampling_timeout: Duration::from_secs(DEFAULT_SAMPLING_TIMEOUT),
            elicitation_timeout: Duration::from_secs(DEFAULT_ELICITATION_TIMEOUT),
            completion_timeout: Duration::from_secs(DEFAULT_COMPLETION_TIMEOUT),
            ping_timeout: Duration::from_secs(DEFAULT_PING_TIMEOUT),
            shutdown_timeout: Duration::from_secs(DEFAULT_SHUTDOWN_TIMEOUT),
            cancellation_timeout: Duration::from_secs(DEFAULT_CANCELLATION_TIMEOUT),
            progress_interval: Duration::from_secs(DEFAULT_PROGRESS_INTERVAL),
            max_timeout: Duration::from_secs(MAX_OPERATION_TIMEOUT),
            min_timeout: Duration::from_secs(MIN_OPERATION_TIMEOUT),
        }
    }
}

impl TimeoutConfig {
    /// Create a new timeout configuration with custom values
    pub fn new(
        initialize_timeout: Duration,
        operation_timeout: Duration,
        tool_call_timeout: Duration,
        resource_timeout: Duration,
        sampling_timeout: Duration,
        elicitation_timeout: Duration,
        completion_timeout: Duration,
        ping_timeout: Duration,
        shutdown_timeout: Duration,
        cancellation_timeout: Duration,
        progress_interval: Duration,
    ) -> Self {
        Self {
            initialize_timeout: Self::clamp_timeout(initialize_timeout),
            operation_timeout: Self::clamp_timeout(operation_timeout),
            tool_call_timeout: Self::clamp_timeout(tool_call_timeout),
            resource_timeout: Self::clamp_timeout(resource_timeout),
            sampling_timeout: Self::clamp_timeout(sampling_timeout),
            elicitation_timeout: Self::clamp_timeout(elicitation_timeout),
            completion_timeout: Self::clamp_timeout(completion_timeout),
            ping_timeout: Self::clamp_timeout(ping_timeout),
            shutdown_timeout: Self::clamp_timeout(shutdown_timeout),
            cancellation_timeout: Self::clamp_timeout(cancellation_timeout),
            progress_interval: Self::clamp_progress_interval(progress_interval),
            max_timeout: Duration::from_secs(MAX_OPERATION_TIMEOUT),
            min_timeout: Duration::from_secs(MIN_OPERATION_TIMEOUT),
        }
    }

    /// Create a timeout configuration optimized for high-performance scenarios
    pub fn high_performance() -> Self {
        Self {
            initialize_timeout: Duration::from_secs(10),
            operation_timeout: Duration::from_secs(60),
            tool_call_timeout: Duration::from_secs(30),
            resource_timeout: Duration::from_secs(15),
            sampling_timeout: Duration::from_secs(120),
            elicitation_timeout: Duration::from_secs(60),
            completion_timeout: Duration::from_secs(15),
            ping_timeout: Duration::from_secs(5),
            shutdown_timeout: Duration::from_secs(15),
            cancellation_timeout: Duration::from_secs(5),
            progress_interval: Duration::from_secs(2),
            max_timeout: Duration::from_secs(300),
            min_timeout: Duration::from_secs(1),
        }
    }

    /// Create a timeout configuration optimized for long-running operations
    pub fn long_running() -> Self {
        Self {
            initialize_timeout: Duration::from_secs(60),
            operation_timeout: Duration::from_secs(1800), // 30 minutes
            tool_call_timeout: Duration::from_secs(300),  // 5 minutes
            resource_timeout: Duration::from_secs(60),
            sampling_timeout: Duration::from_secs(3600),  // 1 hour
            elicitation_timeout: Duration::from_secs(600), // 10 minutes
            completion_timeout: Duration::from_secs(60),
            ping_timeout: Duration::from_secs(30),
            shutdown_timeout: Duration::from_secs(60),
            cancellation_timeout: Duration::from_secs(30),
            progress_interval: Duration::from_secs(10),
            max_timeout: Duration::from_secs(7200), // 2 hours
            min_timeout: Duration::from_secs(5),
        }
    }

    /// Get timeout for a specific operation type
    pub fn get_timeout_for_operation(&self, operation: &str) -> Duration {
        match operation {
            "initialize" => self.initialize_timeout,
            "shutdown" => self.shutdown_timeout,
            "ping" => self.ping_timeout,
            "tools/call" => self.tool_call_timeout,
            "resources/read" | "resources/list" => self.resource_timeout,
            "sampling/createMessage" => self.sampling_timeout,
            "elicitation/request" => self.elicitation_timeout,
            "completion/complete" => self.completion_timeout,
            _ => self.operation_timeout,
        }
    }

    /// Validate that a timeout is within acceptable bounds
    pub fn validate_timeout(&self, timeout: Duration) -> bool {
        timeout >= self.min_timeout && timeout <= self.max_timeout
    }

    /// Clamp a timeout to valid bounds
    fn clamp_timeout(timeout: Duration) -> Duration {
        let min = Duration::from_secs(MIN_OPERATION_TIMEOUT);
        let max = Duration::from_secs(MAX_OPERATION_TIMEOUT);
        timeout.clamp(min, max)
    }

    /// Clamp a progress interval to valid bounds
    fn clamp_progress_interval(interval: Duration) -> Duration {
        let min = Duration::from_secs(MIN_PROGRESS_INTERVAL);
        let max = Duration::from_secs(MAX_PROGRESS_INTERVAL);
        interval.clamp(min, max)
    }

    /// Check if progress notifications should be sent based on interval
    pub fn should_send_progress(&self, last_progress: std::time::Instant) -> bool {
        last_progress.elapsed() >= self.progress_interval
    }
}

/// Validation error types
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid timeout value: {0}")]
    InvalidTimeout(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeout_config_default() {
        let config = TimeoutConfig::default();
        assert_eq!(config.initialize_timeout, Duration::from_secs(DEFAULT_INITIALIZE_TIMEOUT));
        assert_eq!(config.operation_timeout, Duration::from_secs(DEFAULT_OPERATION_TIMEOUT));
        assert_eq!(config.tool_call_timeout, Duration::from_secs(DEFAULT_TOOL_CALL_TIMEOUT));
    }

    #[test]
    fn test_timeout_config_high_performance() {
        let config = TimeoutConfig::high_performance();
        assert_eq!(config.initialize_timeout, Duration::from_secs(10));
        assert_eq!(config.operation_timeout, Duration::from_secs(60));
        assert_eq!(config.progress_interval, Duration::from_secs(2));
    }

    #[test]
    fn test_timeout_config_long_running() {
        let config = TimeoutConfig::long_running();
        assert_eq!(config.operation_timeout, Duration::from_secs(1800));
        assert_eq!(config.sampling_timeout, Duration::from_secs(3600));
        assert_eq!(config.progress_interval, Duration::from_secs(10));
    }

    #[test]
    fn test_get_timeout_for_operation() {
        let config = TimeoutConfig::default();
        assert_eq!(config.get_timeout_for_operation("initialize"), config.initialize_timeout);
        assert_eq!(config.get_timeout_for_operation("tools/call"), config.tool_call_timeout);
        assert_eq!(config.get_timeout_for_operation("unknown"), config.operation_timeout);
    }

    #[test]
    fn test_validate_timeout() {
        let config = TimeoutConfig::default();
        assert!(config.validate_timeout(Duration::from_secs(30)));
        assert!(!config.validate_timeout(Duration::from_secs(0)));
        assert!(!config.validate_timeout(Duration::from_secs(4000)));
    }

    #[test]
    fn test_should_send_progress() {
        let config = TimeoutConfig::default();
        let now = std::time::Instant::now();
        
        // Should not send progress immediately
        assert!(!config.should_send_progress(now));
        
        // Should send progress after interval
        let past = now - Duration::from_secs(10);
        assert!(config.should_send_progress(past));
    }
}
