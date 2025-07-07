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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_constants() {
        assert!(!CURRENT_VERSION.is_empty());
        assert!(!SUPPORTED_VERSIONS.is_empty());
        assert_eq!(JSONRPC_VERSION, "2.0");
        assert!(MAX_REQUEST_ID_LENGTH > 0);
        assert!(DEFAULT_REQUEST_TIMEOUT > 0);
    }
} 