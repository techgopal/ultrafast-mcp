use crate::types::*;
use crate::types::{ClientCapabilities, ServerCapabilities};
use serde::{Deserialize, Serialize};
use super::version::{ProtocolVersion, VersionNegotiator as NewVersionNegotiator};

/// MCP connection lifecycle phases
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecyclePhase {
    Uninitialized,
    Initializing,
    Initialized,
    Operating,
    ShuttingDown,
    Shutdown,
}

/// Initialize request sent by client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeRequest {
    /// Protocol version (e.g., "2025-06-18")
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,

    /// Client capabilities
    pub capabilities: ClientCapabilities,

    /// Information about the client
    #[serde(rename = "clientInfo")]
    pub client_info: ClientInfo,
}

impl InitializeRequest {
    /// Validate the protocol version format
    pub fn validate_protocol_version(&self) -> Result<(), crate::error::ProtocolError> {
        // Check if version follows YYYY-MM-DD format
        if !self.protocol_version.chars().all(|c| c.is_ascii_digit() || c == '-') {
            return Err(crate::error::ProtocolError::InvalidVersion(
                "Protocol version must contain only digits and hyphens".to_string(),
            ));
        }

        // Check if version has correct format (YYYY-MM-DD)
        let parts: Vec<&str> = self.protocol_version.split('-').collect();
        if parts.len() != 3 {
            return Err(crate::error::ProtocolError::InvalidVersion(
                "Protocol version must be in YYYY-MM-DD format".to_string(),
            ));
        }

        // Validate year, month, day
        if parts[0].len() != 4 || parts[1].len() != 2 || parts[2].len() != 2 {
            return Err(crate::error::ProtocolError::InvalidVersion(
                "Protocol version must be in YYYY-MM-DD format".to_string(),
            ));
        }

        // Validate ranges
        if let (Ok(year), Ok(month), Ok(day)) = (
            parts[0].parse::<u16>(),
            parts[1].parse::<u8>(),
            parts[2].parse::<u8>(),
        ) {
            if year < 2020 || year > 2030 {
                return Err(crate::error::ProtocolError::InvalidVersion(
                    "Year must be between 2020 and 2030".to_string(),
                ));
            }
            if month < 1 || month > 12 {
                return Err(crate::error::ProtocolError::InvalidVersion(
                    "Month must be between 1 and 12".to_string(),
                ));
            }
            if day < 1 || day > 31 {
                return Err(crate::error::ProtocolError::InvalidVersion(
                    "Day must be between 1 and 31".to_string(),
                ));
            }
        } else {
            return Err(crate::error::ProtocolError::InvalidVersion(
                "Protocol version components must be valid numbers".to_string(),
            ));
        }

        Ok(())
    }
}

/// Initialize response sent by server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResponse {
    /// Protocol version that will be used
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,

    /// Server capabilities
    pub capabilities: ServerCapabilities,

    /// Information about the server
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,

    /// Optional instructions for the client
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

/// Initialized notification sent by client after receiving initialize response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializedNotification {
    // This is just a marker notification - no parameters needed
}

/// Shutdown request (can be sent by either client or server)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutdownRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Trait for lifecycle management
#[async_trait::async_trait]
pub trait LifecycleManager {
    /// Get current lifecycle phase
    fn phase(&self) -> LifecyclePhase;

    /// Handle initialize request
    async fn initialize(
        &mut self,
        request: InitializeRequest,
    ) -> Result<InitializeResponse, crate::error::MCPError>;

    /// Handle initialized notification
    async fn initialized(&mut self) -> Result<(), crate::error::MCPError>;

    /// Handle shutdown request
    async fn shutdown(&mut self, request: ShutdownRequest) -> Result<(), crate::error::MCPError>;

    /// Check if operation is allowed in current phase
    fn can_operate(&self) -> bool {
        matches!(self.phase(), LifecyclePhase::Operating)
    }
}

/// Version negotiation utility (deprecated - use version::VersionNegotiator instead)
#[deprecated(since = "1.0.0", note = "Use version::VersionNegotiator instead")]
pub struct VersionNegotiator {
    inner: NewVersionNegotiator,
}

impl VersionNegotiator {
    pub fn new(supported_versions: Vec<String>) -> Self {
        let versions: Vec<ProtocolVersion> = supported_versions
            .into_iter()
            .filter_map(|s| ProtocolVersion::parse(&s).ok())
            .collect();
        Self {
            inner: NewVersionNegotiator::new(versions),
        }
    }

    /// Negotiate protocol version with client
    pub fn negotiate(
        &self,
        requested_version: &str,
    ) -> Result<String, crate::error::ProtocolError> {
        self.inner
            .negotiate(requested_version)
            .map(|v| v.to_string())
            .map_err(|_| {
                crate::error::ProtocolError::InitializationFailed(
                    "Version negotiation failed".to_string(),
                )
            })
    }

    /// Get all supported versions
    pub fn supported_versions(&self) -> Vec<String> {
        self.inner
            .supported_versions()
            .iter()
            .map(|v| v.to_string())
            .collect()
    }
}

impl Default for VersionNegotiator {
    fn default() -> Self {
        Self {
            inner: NewVersionNegotiator::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_negotiation() {
        let negotiator =
            VersionNegotiator::new(vec!["2024-11-05".to_string(), "2025-06-18".to_string()]);

        // Exact match should work
        assert_eq!(negotiator.negotiate("2025-06-18").unwrap(), "2025-06-18");

        // Unsupported version should fallback to latest (2025-06-18)
        assert_eq!(negotiator.negotiate("2026-01-01").unwrap(), "2025-06-18");
    }

    #[test]
    fn test_lifecycle_phases() {
        assert_ne!(LifecyclePhase::Uninitialized, LifecyclePhase::Initialized);
        assert_eq!(LifecyclePhase::Operating, LifecyclePhase::Operating);
    }

    #[test]
    fn test_protocol_version_validation() {
        // Valid versions
        let valid_request = InitializeRequest {
            protocol_version: "2025-06-18".to_string(),
            capabilities: Default::default(),
            client_info: Default::default(),
        };
        assert!(valid_request.validate_protocol_version().is_ok());

        let valid_request2 = InitializeRequest {
            protocol_version: "2024-11-05".to_string(),
            capabilities: Default::default(),
            client_info: Default::default(),
        };
        assert!(valid_request2.validate_protocol_version().is_ok());

        // Invalid versions
        let invalid_request = InitializeRequest {
            protocol_version: "invalid".to_string(),
            capabilities: Default::default(),
            client_info: Default::default(),
        };
        assert!(invalid_request.validate_protocol_version().is_err());

        let invalid_request2 = InitializeRequest {
            protocol_version: "2025-6-18".to_string(), // Missing leading zero
            capabilities: Default::default(),
            client_info: Default::default(),
        };
        assert!(invalid_request2.validate_protocol_version().is_err());

        let invalid_request3 = InitializeRequest {
            protocol_version: "2025-06-18-extra".to_string(), // Too many parts
            capabilities: Default::default(),
            client_info: Default::default(),
        };
        assert!(invalid_request3.validate_protocol_version().is_err());

        let invalid_request4 = InitializeRequest {
            protocol_version: "2019-06-18".to_string(), // Year too early
            capabilities: Default::default(),
            client_info: Default::default(),
        };
        assert!(invalid_request4.validate_protocol_version().is_err());

        let invalid_request5 = InitializeRequest {
            protocol_version: "2031-06-18".to_string(), // Year too late
            capabilities: Default::default(),
            client_info: Default::default(),
        };
        assert!(invalid_request5.validate_protocol_version().is_err());
    }
}
