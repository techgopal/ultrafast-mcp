use crate::error::{MCPError, MCPResult, ResourceError};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A simple URI wrapper for MCP resources
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Uri(String);

impl Uri {
    /// Create a new URI from a string
    pub fn new(uri: impl Into<String>) -> Self {
        Self(uri.into())
    }

    /// Get the URI as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Check if this is a file URI
    pub fn is_file(&self) -> bool {
        self.0.starts_with("file://")
    }

    /// Check if this is an HTTP/HTTPS URI
    pub fn is_http(&self) -> bool {
        self.0.starts_with("http://") || self.0.starts_with("https://")
    }

    /// Get the scheme component of the URI
    pub fn scheme(&self) -> Option<&str> {
        self.0.split("://").next()
    }

    /// Validate the URI format
    pub fn validate(&self) -> MCPResult<()> {
        if self.0.is_empty() {
            return Err(MCPError::Resource(ResourceError::InvalidUri(
                "URI cannot be empty".to_string(),
            )));
        }

        // Basic validation - must have a scheme or be a relative path
        if !self.0.contains("://") && (self.0.starts_with('/') || !self.0.starts_with('/')) {
            return Ok(());
        }

        // Must have valid scheme
        if let Some(scheme) = self.scheme() {
            if scheme.is_empty()
                || !scheme
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.')
            {
                return Err(MCPError::Resource(ResourceError::InvalidUri(format!(
                    "Invalid URI scheme: {scheme}"
                ))));
            }
        }

        Ok(())
    }
}

impl fmt::Display for Uri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Uri {
    fn from(s: String) -> Self {
        Uri::new(s)
    }
}

impl From<&str> for Uri {
    fn from(s: &str) -> Self {
        Uri::new(s)
    }
}

impl AsRef<str> for Uri {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uri_creation() {
        let uri = Uri::new("file:///path/to/file.txt");
        assert_eq!(uri.as_str(), "file:///path/to/file.txt");
    }

    #[test]
    fn test_uri_scheme() {
        let file_uri = Uri::new("file:///path/to/file.txt");
        assert_eq!(file_uri.scheme(), Some("file"));
        assert!(file_uri.is_file());

        let http_uri = Uri::new("https://example.com/path");
        assert_eq!(http_uri.scheme(), Some("https"));
        assert!(http_uri.is_http());
    }

    #[test]
    fn test_uri_validation() {
        assert!(Uri::new("file:///path").validate().is_ok());
        assert!(Uri::new("https://example.com").validate().is_ok());
        assert!(Uri::new("/absolute/path").validate().is_ok());
        assert!(Uri::new("relative/path").validate().is_ok());
        assert!(Uri::new("").validate().is_err());
    }
}
