//! # Version Management Module
//!
//! Centralized version management for the Model Context Protocol (MCP).
//!
//! This module provides a single source of truth for MCP protocol versions,
//! version negotiation, and compatibility checking. It ensures consistency
//! across the entire codebase and makes version updates easier to manage.

use serde::{Deserialize, Serialize};

/// MCP Protocol Version
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ProtocolVersion {
    /// Year component (e.g., 2025)
    pub year: u16,
    /// Month component (e.g., 6)
    pub month: u8,
    /// Day component (e.g., 18)
    pub day: u8,
}

impl ProtocolVersion {
    /// Create a new protocol version
    pub fn new(year: u16, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }

    /// Parse a version string in the format "YYYY-MM-DD"
    pub fn parse(version_str: &str) -> Result<Self, VersionParseError> {
        let parts: Vec<&str> = version_str.split('-').collect();
        if parts.len() != 3 {
            return Err(VersionParseError::InvalidFormat);
        }

        let year = parts[0]
            .parse::<u16>()
            .map_err(|_| VersionParseError::InvalidYear)?;
        let month = parts[1]
            .parse::<u8>()
            .map_err(|_| VersionParseError::InvalidMonth)?;
        let day = parts[2]
            .parse::<u8>()
            .map_err(|_| VersionParseError::InvalidDay)?;

        // Validate date components
        if month == 0 || month > 12 {
            return Err(VersionParseError::InvalidMonth);
        }
        if day == 0 || day > 31 {
            return Err(VersionParseError::InvalidDay);
        }

        Ok(Self { year, month, day })
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    /// Check if this version is compatible with another version
    pub fn is_compatible_with(&self, other: &ProtocolVersion) -> bool {
        // For now, we only support exact matches
        // In the future, we can implement semantic version compatibility
        self == other
    }

    /// Get the latest supported version
    pub fn latest() -> Self {
        Self::new(2025, 6, 18)
    }

    /// Get all supported versions (ordered from oldest to newest)
    pub fn supported_versions() -> Vec<Self> {
        vec![
            Self::new(2024, 11, 5), // Previous version for backward compatibility
            Self::new(2025, 6, 18), // Current version (latest)
        ]
    }
}

impl std::fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl From<&str> for ProtocolVersion {
    fn from(version_str: &str) -> Self {
        Self::parse(version_str).unwrap_or_else(|_| Self::latest())
    }
}

impl From<String> for ProtocolVersion {
    fn from(version_str: String) -> Self {
        Self::from(version_str.as_str())
    }
}

/// Version parse error
#[derive(Debug, thiserror::Error)]
pub enum VersionParseError {
    #[error("Invalid version format, expected YYYY-MM-DD")]
    InvalidFormat,
    #[error("Invalid year component")]
    InvalidYear,
    #[error("Invalid month component")]
    InvalidMonth,
    #[error("Invalid day component")]
    InvalidDay,
}

/// Version negotiator for protocol compatibility
pub struct VersionNegotiator {
    supported_versions: Vec<ProtocolVersion>,
}

impl VersionNegotiator {
    /// Create a new version negotiator with supported versions
    pub fn new(supported_versions: Vec<ProtocolVersion>) -> Self {
        Self { supported_versions }
    }

    /// Create a version negotiator with default supported versions
    pub fn default() -> Self {
        Self::new(ProtocolVersion::supported_versions())
    }

    /// Negotiate protocol version with client
    pub fn negotiate(
        &self,
        requested_version: &str,
    ) -> Result<ProtocolVersion, VersionNegotiationError> {
        let requested = ProtocolVersion::parse(requested_version)
            .map_err(VersionNegotiationError::ParseError)?;

        // Try exact match first
        if self.supported_versions.contains(&requested) {
            return Ok(requested);
        }

        // Try backward compatibility
        for supported in &self.supported_versions {
            if requested.is_compatible_with(supported) {
                return Ok(supported.clone());
            }
        }

        // Fallback to latest supported version
        self.supported_versions
            .last()
            .cloned()
            .ok_or(VersionNegotiationError::NoSupportedVersions)
    }

    /// Get all supported versions
    pub fn supported_versions(&self) -> &[ProtocolVersion] {
        &self.supported_versions
    }

    /// Check if a version is supported
    pub fn supports(&self, version: &ProtocolVersion) -> bool {
        self.supported_versions.contains(version)
    }

    /// Get the latest supported version
    pub fn latest(&self) -> Option<&ProtocolVersion> {
        self.supported_versions.last()
    }
}

/// Version negotiation error
#[derive(Debug, thiserror::Error)]
pub enum VersionNegotiationError {
    #[error("Failed to parse version: {0}")]
    ParseError(#[from] VersionParseError),
    #[error("No supported protocol versions available")]
    NoSupportedVersions,
    #[error("Incompatible protocol version: {0}")]
    IncompatibleVersion(String),
}

/// Constants for commonly used versions
pub mod constants {
    use super::ProtocolVersion;

    /// Current MCP protocol version (2025-06-18)
    pub const CURRENT_VERSION: &str = "2025-06-18";
    
    /// Previous MCP protocol version (2024-11-05)
    pub const PREVIOUS_VERSION: &str = "2024-11-05";

    /// Current protocol version as ProtocolVersion
    pub fn current() -> ProtocolVersion {
        ProtocolVersion::new(2025, 6, 18)
    }

    /// Previous protocol version as ProtocolVersion
    pub fn previous() -> ProtocolVersion {
        ProtocolVersion::new(2024, 11, 5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let version = ProtocolVersion::parse("2025-06-18").unwrap();
        assert_eq!(version.year, 2025);
        assert_eq!(version.month, 6);
        assert_eq!(version.day, 18);
        assert_eq!(version.to_string(), "2025-06-18");
    }

    #[test]
    fn test_version_parsing_invalid() {
        assert!(ProtocolVersion::parse("invalid").is_err());
        assert!(ProtocolVersion::parse("2025-13-01").is_err()); // Invalid month
        assert!(ProtocolVersion::parse("2025-06-32").is_err()); // Invalid day
    }

    #[test]
    fn test_version_comparison() {
        let v1 = ProtocolVersion::new(2025, 6, 18);
        let v2 = ProtocolVersion::new(2024, 11, 5);
        
        assert!(v1 > v2);
        assert!(v2 < v1);
        assert_eq!(v1, v1);
    }

    #[test]
    fn test_version_negotiation() {
        let negotiator = VersionNegotiator::default();
        
        // Exact match should work
        let negotiated = negotiator.negotiate("2025-06-18").unwrap();
        assert_eq!(negotiated, ProtocolVersion::new(2025, 6, 18));
        
        // Unsupported version should fallback to latest (2025-06-18)
        let negotiated = negotiator.negotiate("2026-01-01").unwrap();
        assert_eq!(negotiated, ProtocolVersion::new(2025, 6, 18));
    }

    #[test]
    fn test_version_negotiation_with_multiple_versions() {
        let negotiator = VersionNegotiator::new(vec![
            ProtocolVersion::new(2024, 11, 5),
            ProtocolVersion::new(2025, 6, 18),
        ]);
        
        // Exact match should work
        let negotiated = negotiator.negotiate("2025-06-18").unwrap();
        assert_eq!(negotiated, ProtocolVersion::new(2025, 6, 18));
        
        // Unsupported version should fallback to latest (2025-06-18)
        let negotiated = negotiator.negotiate("2026-01-01").unwrap();
        assert_eq!(negotiated, ProtocolVersion::new(2025, 6, 18));
    }

    #[test]
    fn test_version_constants() {
        assert_eq!(constants::CURRENT_VERSION, "2025-06-18");
        assert_eq!(constants::PREVIOUS_VERSION, "2024-11-05");
        assert_eq!(constants::current(), ProtocolVersion::new(2025, 6, 18));
        assert_eq!(constants::previous(), ProtocolVersion::new(2024, 11, 5));
    }

    #[test]
    fn test_version_display() {
        let version = ProtocolVersion::new(2025, 6, 18);
        assert_eq!(version.to_string(), "2025-06-18");
        assert_eq!(format!("{}", version), "2025-06-18");
    }

    #[test]
    fn test_version_from_string() {
        let version: ProtocolVersion = "2025-06-18".into();
        assert_eq!(version, ProtocolVersion::new(2025, 6, 18));
        
        let version: ProtocolVersion = "invalid".into();
        assert_eq!(version, ProtocolVersion::latest());
    }
} 