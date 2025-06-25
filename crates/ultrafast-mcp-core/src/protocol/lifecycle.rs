use serde::{Deserialize, Serialize};
use crate::types::{ClientCapabilities, ServerCapabilities};
use crate::types::*;

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

/// Version negotiation utility
pub struct VersionNegotiator {
    supported_versions: Vec<String>,
}

impl VersionNegotiator {
    pub fn new(supported_versions: Vec<String>) -> Self {
        Self { supported_versions }
    }
    
    /// Negotiate protocol version with client
    pub fn negotiate(&self, requested_version: &str) -> Result<String, crate::error::ProtocolError> {
        // For now, we only support exact matches
        // In the future, we can implement semantic version compatibility
        if self.supported_versions.contains(&requested_version.to_string()) {
            Ok(requested_version.to_string())
        } else {
            // Return the highest supported version as fallback
            self.supported_versions
                .last()
                .cloned()
                .ok_or_else(|| crate::error::ProtocolError::InitializationFailed(
                    "No supported protocol versions".to_string()
                ))
        }
    }
    
    /// Get all supported versions
    pub fn supported_versions(&self) -> &[String] {
        &self.supported_versions
    }
}

impl Default for VersionNegotiator {
    fn default() -> Self {
        Self::new(vec!["2025-06-18".to_string()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_negotiation() {
        let negotiator = VersionNegotiator::new(vec![
            "2025-06-18".to_string(),
            "2025-06-18".to_string(),
        ]);
        
        // Exact match should work
        assert_eq!(negotiator.negotiate("2025-06-18").unwrap(), "2025-06-18");
        
        // Unsupported version should fallback to latest
        assert_eq!(negotiator.negotiate("2026-01-01").unwrap(), "2025-06-18");
    }
    
    #[test]
    fn test_lifecycle_phases() {
        assert_ne!(LifecyclePhase::Uninitialized, LifecyclePhase::Initialized);
        assert_eq!(LifecyclePhase::Operating, LifecyclePhase::Operating);
    }
}
