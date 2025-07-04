//! MCP Protocol Version Management

/// Current MCP protocol version
pub const PROTOCOL_VERSION: &str = "2025-06-18";

/// Supported protocol versions
pub const SUPPORTED_VERSIONS: &[&str] = &["2025-06-18"];

/// Check if a protocol version is supported
pub fn is_supported_version(version: &str) -> bool {
    SUPPORTED_VERSIONS.contains(&version)
}

/// Get the latest supported protocol version
pub fn get_latest_version() -> &'static str {
    PROTOCOL_VERSION
}
