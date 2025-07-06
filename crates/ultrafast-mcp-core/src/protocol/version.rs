//! MCP Protocol Version Management

use std::cmp::Ordering;
use std::fmt;

/// Current protocol version
pub const PROTOCOL_VERSION: &str = "2025-06-18";

/// All supported protocol versions (latest first)
pub const SUPPORTED_VERSIONS: &[&str] = &[
    "2025-06-18",
    "2024-11-05",
];

/// Protocol version representation for comparison
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolVersion {
    year: u32,
    month: u32,
    day: u32,
}

impl ProtocolVersion {
    /// Parse a protocol version string
    pub fn parse(version: &str) -> Result<Self, String> {
        let parts: Vec<&str> = version.split('-').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid version format: {}", version));
        }

        let year = parts[0].parse::<u32>()
            .map_err(|_| format!("Invalid year: {}", parts[0]))?;
        let month = parts[1].parse::<u32>()
            .map_err(|_| format!("Invalid month: {}", parts[1]))?;
        let day = parts[2].parse::<u32>()
            .map_err(|_| format!("Invalid day: {}", parts[2]))?;

        // Basic validation
        if year < 2020 || year > 2099 {
            return Err(format!("Invalid year: {}", year));
        }
        if month < 1 || month > 12 {
            return Err(format!("Invalid month: {}", month));
        }
        if day < 1 || day > 31 {
            return Err(format!("Invalid day: {}", day));
        }

        Ok(ProtocolVersion { year, month, day })
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        format!("{}-{:02}-{:02}", self.year, self.month, self.day)
    }

    /// Check if this version supports a specific feature
    pub fn supports_feature(&self, feature: &str) -> bool {
        match feature {
            "resource_subscriptions" => {
                // Resource subscriptions introduced in 2025-06-18
                *self >= ProtocolVersion::parse("2025-06-18").unwrap()
            }
            "progress_tracking" => {
                // Progress tracking introduced in 2025-06-18
                *self >= ProtocolVersion::parse("2025-06-18").unwrap()
            }
            "enhanced_error_codes" => {
                // Enhanced error codes introduced in 2025-06-18
                *self >= ProtocolVersion::parse("2025-06-18").unwrap()
            }
            "sampling" => {
                // Sampling available in all supported versions
                true
            }
            "elicitation" => {
                // Elicitation available in all supported versions
                true
            }
            "completion" => {
                // Completion available in all supported versions
                true
            }
            "logging" => {
                // Logging available in all supported versions
                true
            }
            "tools" => {
                // Tools available in all supported versions
                true
            }
            "resources" => {
                // Resources available in all supported versions
                true
            }
            "prompts" => {
                // Prompts available in all supported versions
                true
            }
            "roots" => {
                // Roots available in all supported versions
                true
            }
            "list_changed_notifications" => {
                // List changed notifications introduced in 2025-06-18
                *self >= ProtocolVersion::parse("2025-06-18").unwrap()
            }
            "cancellation" => {
                // Cancellation introduced in 2025-06-18
                *self >= ProtocolVersion::parse("2025-06-18").unwrap()
            }
            _ => {
                // Unknown features are assumed to be supported for forward compatibility
                true
            }
        }
    }

    /// Get the latest supported version
    pub fn latest() -> Self {
        Self::parse(PROTOCOL_VERSION).unwrap()
    }

    /// Check if this version is supported
    pub fn is_supported(&self) -> bool {
        SUPPORTED_VERSIONS.iter().any(|v| {
            Self::parse(v).map_or(false, |version| version == *self)
        })
    }
}

impl PartialOrd for ProtocolVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ProtocolVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.year.cmp(&other.year)
            .then_with(|| self.month.cmp(&other.month))
            .then_with(|| self.day.cmp(&other.day))
    }
}

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// Version negotiation utilities
pub fn negotiate_version(requested: &str) -> Result<String, String> {
    // Check if the requested version is supported
    if is_supported_version(requested) {
        return Ok(requested.to_string());
    }

    // If not supported, try to negotiate a compatible version
    let requested_version = ProtocolVersion::parse(requested)?;
    
    // Find the highest supported version that's less than or equal to requested
    let mut best_version: Option<ProtocolVersion> = None;
    
    for supported in SUPPORTED_VERSIONS {
        if let Ok(version) = ProtocolVersion::parse(supported) {
            if version <= requested_version {
                match best_version {
                    None => best_version = Some(version),
                    Some(ref current) => {
                        if version > *current {
                            best_version = Some(version);
                        }
                    }
                }
            }
        }
    }
    
    if let Some(version) = best_version {
        Ok(version.to_string())
    } else {
        // No compatible version found, return the latest supported
        Ok(get_latest_version().to_string())
    }
}

/// Check if a version is supported
pub fn is_supported_version(version: &str) -> bool {
    SUPPORTED_VERSIONS.contains(&version)
}

/// Get the latest supported version
pub fn get_latest_version() -> &'static str {
    PROTOCOL_VERSION
}

/// Get all supported versions
pub fn get_supported_versions() -> &'static [&'static str] {
    SUPPORTED_VERSIONS
}

/// Check if a version supports a specific feature
pub fn version_supports_feature(version: &str, feature: &str) -> bool {
    match ProtocolVersion::parse(version) {
        Ok(v) => v.supports_feature(feature),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let v1 = ProtocolVersion::parse("2025-06-18").unwrap();
        assert_eq!(v1.year, 2025);
        assert_eq!(v1.month, 6);
        assert_eq!(v1.day, 18);

        let v2 = ProtocolVersion::parse("2024-11-05").unwrap();
        assert_eq!(v2.year, 2024);
        assert_eq!(v2.month, 11);
        assert_eq!(v2.day, 5);

        // Test invalid formats
        assert!(ProtocolVersion::parse("2025-06").is_err());
        assert!(ProtocolVersion::parse("invalid").is_err());
        assert!(ProtocolVersion::parse("2025-13-01").is_err());
    }

    #[test]
    fn test_version_comparison() {
        let v1 = ProtocolVersion::parse("2024-11-05").unwrap();
        let v2 = ProtocolVersion::parse("2025-06-18").unwrap();

        assert!(v1 < v2);
        assert!(v2 > v1);
        assert_eq!(v1, v1);
    }

    #[test]
    fn test_version_negotiation() {
        // Test exact match
        assert_eq!(negotiate_version("2025-06-18").unwrap(), "2025-06-18");
        
        // Test fallback
        assert_eq!(negotiate_version("2024-11-05").unwrap(), "2024-11-05");
        
        // Test unknown version
        let result = negotiate_version("2023-01-01");
        assert!(result.is_ok());
    }

    #[test]
    fn test_feature_support() {
        let v1 = ProtocolVersion::parse("2024-11-05").unwrap();
        let v2 = ProtocolVersion::parse("2025-06-18").unwrap();

        // Features available in all versions
        assert!(v1.supports_feature("tools"));
        assert!(v2.supports_feature("tools"));
        assert!(v1.supports_feature("resources"));
        assert!(v2.supports_feature("resources"));

        // Features introduced in 2025-06-18
        assert!(!v1.supports_feature("resource_subscriptions"));
        assert!(v2.supports_feature("resource_subscriptions"));
        assert!(!v1.supports_feature("progress_tracking"));
        assert!(v2.supports_feature("progress_tracking"));
    }

    #[test]
    fn test_version_support_check() {
        assert!(is_supported_version("2025-06-18"));
        assert!(is_supported_version("2024-11-05"));
        assert!(!is_supported_version("2023-01-01"));
    }

    #[test]
    fn test_version_supports_feature_function() {
        assert!(version_supports_feature("2025-06-18", "resource_subscriptions"));
        assert!(!version_supports_feature("2024-11-05", "resource_subscriptions"));
        assert!(version_supports_feature("2024-11-05", "tools"));
        assert!(version_supports_feature("2025-06-18", "tools"));
    }
}
