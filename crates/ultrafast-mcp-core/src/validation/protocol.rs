//! Protocol validation utilities

use crate::error::{MCPResult, ProtocolError};
use crate::protocol::version::PROTOCOL_VERSION;

/// Validate protocol version against supported versions
///
/// Consolidates implementations from:
/// - core/protocol/lifecycle.rs
/// - transport/streamable_http/server.rs
/// - client/src/lib.rs
pub fn validate_protocol_version(version: &str) -> MCPResult<()> {
    if version != PROTOCOL_VERSION {
        return Err(ProtocolError::InvalidVersion(format!(
            "Expected protocol version {PROTOCOL_VERSION}, got {version}"
        ))
        .into());
    }
    Ok(())
}

/// Check if a protocol version is supported
pub fn is_supported_version(version: &str) -> bool {
    // For now, we only support the exact current version
    // In the future, this could check against a list of supported versions
    version == PROTOCOL_VERSION
}

/// Validate that a method name is valid for MCP
pub fn validate_method_name(method: &str) -> MCPResult<()> {
    if method.is_empty() {
        return Err(
            ProtocolError::InvalidRequest("Method name cannot be empty".to_string()).into(),
        );
    }

    // Method names should only contain alphanumeric characters, forward slashes, and underscores
    if !method
        .chars()
        .all(|c| c.is_alphanumeric() || c == '/' || c == '_')
    {
        return Err(ProtocolError::InvalidRequest(format!(
            "Invalid method name: {method}. Method names can only contain alphanumeric characters, '/', and '_'"
        )).into());
    }

    Ok(())
}

/// Validate Origin header for security
///
/// Consolidates implementation from transport/streamable_http/server.rs
pub fn validate_origin(origin: Option<&str>, allowed_origin: Option<&str>, host: &str) -> bool {
    if let Some(origin_str) = origin {
        // If allow_origin is set to "*", allow all origins
        if let Some(allowed) = allowed_origin {
            if allowed == "*" {
                return true;
            }
            return origin_str == allowed;
        }
        // For localhost, allow any localhost origin
        if host == "127.0.0.1" || host == "localhost" || host == "0.0.0.0" {
            return origin_str.contains("localhost") || origin_str.contains("127.0.0.1");
        }
        return false;
    }
    // Allow requests without Origin header for local development
    host == "127.0.0.1" || host == "localhost" || host == "0.0.0.0"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_protocol_version() {
        // Valid version should pass
        assert!(validate_protocol_version(PROTOCOL_VERSION).is_ok());

        // Invalid version should fail
        assert!(validate_protocol_version("invalid-version").is_err());
    }

    #[test]
    fn test_is_supported_version() {
        assert!(is_supported_version(PROTOCOL_VERSION));
        assert!(!is_supported_version("invalid-version"));
    }

    #[test]
    fn test_validate_method_name() {
        // Valid method names
        assert!(validate_method_name("initialize").is_ok());
        assert!(validate_method_name("tools/list").is_ok());
        assert!(validate_method_name("tools_list").is_ok());
        assert!(validate_method_name("notifications/tools/listChanged").is_ok());

        // Invalid method names
        assert!(validate_method_name("").is_err());
        assert!(validate_method_name("method with spaces").is_err());
        assert!(validate_method_name("method-with-dashes").is_err());
        assert!(validate_method_name("method.with.dots").is_err());
    }

    #[test]
    fn test_validate_origin() {
        // Wildcard allowed origin
        assert!(validate_origin(
            Some("https://example.com"),
            Some("*"),
            "example.com"
        ));

        // Specific allowed origin
        assert!(validate_origin(
            Some("https://example.com"),
            Some("https://example.com"),
            "example.com"
        ));
        assert!(!validate_origin(
            Some("https://evil.com"),
            Some("https://example.com"),
            "example.com"
        ));

        // Localhost scenarios
        assert!(validate_origin(
            Some("http://localhost:3000"),
            None,
            "localhost"
        ));
        assert!(validate_origin(
            Some("http://127.0.0.1:3000"),
            None,
            "127.0.0.1"
        ));
        assert!(validate_origin(None, None, "localhost"));

        // Production scenarios
        assert!(!validate_origin(
            Some("https://evil.com"),
            None,
            "production.com"
        ));
        assert!(!validate_origin(None, None, "production.com"));
    }
}
