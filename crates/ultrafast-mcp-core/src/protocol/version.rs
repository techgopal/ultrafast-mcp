//! MCP Protocol Version Management

use std::cmp::Ordering;

/// Current MCP protocol version
pub const PROTOCOL_VERSION: &str = "2025-06-18";

/// Supported protocol versions (in chronological order, oldest first)
pub const SUPPORTED_VERSIONS: &[&str] = &[
    "2024-11-05",
    "2025-06-18",
];

/// Minimum supported protocol version
pub const MIN_SUPPORTED_VERSION: &str = "2024-11-05";

/// Maximum supported protocol version
pub const MAX_SUPPORTED_VERSION: &str = "2025-06-18";

/// Protocol version information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolVersion {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub raw: String,
}

impl ProtocolVersion {
    /// Parse a protocol version string
    pub fn parse(version: &str) -> Result<Self, String> {
        let parts: Vec<&str> = version.split('-').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid version format: {}", version));
        }

        let year = parts[0].parse::<u16>()
            .map_err(|_| format!("Invalid year in version: {}", version))?;
        let month = parts[1].parse::<u8>()
            .map_err(|_| format!("Invalid month in version: {}", version))?;
        let day = parts[2].parse::<u8>()
            .map_err(|_| format!("Invalid day in version: {}", version))?;

        // Basic validation
        if month < 1 || month > 12 {
            return Err(format!("Invalid month in version: {}", version));
        }
        if day < 1 || day > 31 {
            return Err(format!("Invalid day in version: {}", version));
        }

        Ok(ProtocolVersion {
            year,
            month,
            day,
            raw: version.to_string(),
        })
    }

    /// Check if this version is compatible with another version
    pub fn is_compatible_with(&self, other: &ProtocolVersion) -> bool {
        // For now, we require exact match or within supported range
        SUPPORTED_VERSIONS.contains(&self.raw.as_str()) && 
        SUPPORTED_VERSIONS.contains(&other.raw.as_str())
    }
}

impl PartialOrd for ProtocolVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ProtocolVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.year.cmp(&other.year) {
            Ordering::Equal => match self.month.cmp(&other.month) {
                Ordering::Equal => self.day.cmp(&other.day),
                other => other,
            },
            other => other,
        }
    }
}

/// Check if a protocol version is supported
pub fn is_supported_version(version: &str) -> bool {
    SUPPORTED_VERSIONS.contains(&version)
}

/// Get the latest supported protocol version
pub fn get_latest_version() -> &'static str {
    PROTOCOL_VERSION
}

/// Get the minimum supported protocol version
pub fn get_min_version() -> &'static str {
    MIN_SUPPORTED_VERSION
}

/// Negotiate the best protocol version between client and server
pub fn negotiate_version(client_version: &str) -> Result<String, String> {
    let client_parsed = ProtocolVersion::parse(client_version)?;
    
    // If client version is supported, use it
    if is_supported_version(client_version) {
        return Ok(client_version.to_string());
    }
    
    // Parse min and max supported versions for comparison
    let min_supported = ProtocolVersion::parse(MIN_SUPPORTED_VERSION)
        .map_err(|e| format!("Invalid min supported version: {}", e))?;
    let max_supported = ProtocolVersion::parse(MAX_SUPPORTED_VERSION)
        .map_err(|e| format!("Invalid max supported version: {}", e))?;
    
    // Reject versions that are too old or too new
    if client_parsed < min_supported {
        return Err(format!(
            "Client version {} is too old. Minimum supported version: {}",
            client_version, MIN_SUPPORTED_VERSION
        ));
    }
    
    if client_parsed > max_supported {
        return Err(format!(
            "Client version {} is not yet supported. Maximum supported version: {}",
            client_version, MAX_SUPPORTED_VERSION
        ));
    }
    
    // If we reach here, the client version is within our supported range
    // but not exactly supported. Find the highest supported version <= client version
    let mut best_version: Option<&str> = None;
    for &supported in SUPPORTED_VERSIONS {
        if let Ok(supported_parsed) = ProtocolVersion::parse(supported) {
            if supported_parsed <= client_parsed {
                best_version = Some(supported);
            }
        }
    }
    
    match best_version {
        Some(version) => Ok(version.to_string()),
        None => Err(format!(
            "No compatible version found for client version {}. Supported versions: {:?}",
            client_version, SUPPORTED_VERSIONS
        )),
    }
}

/// Validate protocol version and provide helpful error messages
pub fn validate_version(version: &str) -> Result<(), String> {
    if version.is_empty() {
        return Err("Protocol version cannot be empty".to_string());
    }
    
    // Try to parse the version
    let parsed = ProtocolVersion::parse(version)?;
    
    // Check if it's supported
    if !is_supported_version(version) {
        let min_version = ProtocolVersion::parse(MIN_SUPPORTED_VERSION).unwrap();
        let max_version = ProtocolVersion::parse(MAX_SUPPORTED_VERSION).unwrap();
        
        if parsed < min_version {
            return Err(format!(
                "Protocol version {} is too old. Minimum supported version: {}",
                version, MIN_SUPPORTED_VERSION
            ));
        } else if parsed > max_version {
            return Err(format!(
                "Protocol version {} is not yet supported. Maximum supported version: {}",
                version, MAX_SUPPORTED_VERSION
            ));
        } else {
            return Err(format!(
                "Protocol version {} is not supported. Supported versions: {:?}",
                version, SUPPORTED_VERSIONS
            ));
        }
    }
    
    Ok(())
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
        assert_eq!(version.raw, "2025-06-18");
    }

    #[test]
    fn test_version_comparison() {
        let v1 = ProtocolVersion::parse("2024-11-05").unwrap();
        let v2 = ProtocolVersion::parse("2025-06-18").unwrap();
        assert!(v1 < v2);
        assert!(v2 > v1);
    }

    #[test]
    fn test_version_negotiation() {
        // Exact match
        assert_eq!(negotiate_version("2025-06-18").unwrap(), "2025-06-18");
        
        // Client has older version
        assert_eq!(negotiate_version("2024-11-05").unwrap(), "2024-11-05");
        
        // Client has newer unsupported version - should get latest supported
        assert!(negotiate_version("2026-01-01").is_err());
        
        // Client has very old version
        assert!(negotiate_version("2023-01-01").is_err());
    }

    #[test]
    fn test_validation() {
        assert!(validate_version("2025-06-18").is_ok());
        assert!(validate_version("2024-11-05").is_ok());
        assert!(validate_version("").is_err());
        assert!(validate_version("invalid").is_err());
        assert!(validate_version("2026-01-01").is_err());
        assert!(validate_version("2023-01-01").is_err());
    }
}
