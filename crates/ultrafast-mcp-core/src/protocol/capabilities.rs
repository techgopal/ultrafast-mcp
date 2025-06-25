use serde::{Deserialize, Serialize};

/// Client capabilities that can be negotiated during initialization
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClientCapabilities {
    /// Filesystem roots capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roots: Option<RootsCapability>,
    
    /// LLM sampling capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling: Option<SamplingCapability>,
    
    /// User input elicitation capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elicitation: Option<ElicitationCapability>,
}

/// Server capabilities that can be advertised during initialization
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerCapabilities {
    /// Tools capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,
    
    /// Resources capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourcesCapability>,
    
    /// Prompts capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptsCapability>,
    
    /// Logging capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<LoggingCapability>,
    
    /// Completion capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion: Option<CompletionCapability>,
}

/// Roots capability for filesystem boundary management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootsCapability {
    /// Whether the client supports list_changed notifications
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Sampling capability for LLM completions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingCapability {
    // Sampling capability has no specific parameters in MCP 2025-06-18
}

/// Elicitation capability for user input collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElicitationCapability {
    // Elicitation capability has no specific parameters in MCP 2025-06-18
}

/// Tools capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCapability {
    /// Whether the server supports list_changed notifications
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Resources capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesCapability {
    /// Whether the server supports resource subscriptions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribe: Option<bool>,
    
    /// Whether the server supports list_changed notifications
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Prompts capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsCapability {
    /// Whether the server supports list_changed notifications
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Logging capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingCapability {
    // Logging capability has no specific parameters in MCP 2025-06-18
}

/// Completion capability for argument autocompletion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionCapability {
    // Completion capability has no specific parameters in MCP 2025-06-18
}

/// Capability negotiation utilities
pub struct CapabilityNegotiator;

impl CapabilityNegotiator {
    /// Negotiate capabilities between client and server
    pub fn negotiate(
        client_caps: &ClientCapabilities,
        server_caps: &ServerCapabilities,
    ) -> (ClientCapabilities, ServerCapabilities) {
        // For Phase 1, we do simple capability passing
        // In future phases, we can implement more sophisticated negotiation
        (client_caps.clone(), server_caps.clone())
    }
    
    /// Check if a capability is supported
    pub fn supports_capability(caps: &ServerCapabilities, capability: &str) -> bool {
        match capability {
            "tools" => caps.tools.is_some(),
            "resources" => caps.resources.is_some(),
            "prompts" => caps.prompts.is_some(),
            "logging" => caps.logging.is_some(),
            "completion" => caps.completion.is_some(),
            _ => false,
        }
    }
    
    /// Check if client supports a capability
    pub fn client_supports_capability(caps: &ClientCapabilities, capability: &str) -> bool {
        match capability {
            "roots" => caps.roots.is_some(),
            "sampling" => caps.sampling.is_some(),
            "elicitation" => caps.elicitation.is_some(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_capability_negotiation() {
        let client_caps = ClientCapabilities {
            roots: Some(RootsCapability { list_changed: Some(true) }),
            sampling: Some(SamplingCapability {}),
            elicitation: None,
        };
        
        let server_caps = ServerCapabilities {
            tools: Some(ToolsCapability { list_changed: Some(true) }),
            resources: Some(ResourcesCapability { 
                subscribe: Some(true), 
                list_changed: Some(true) 
            }),
            prompts: None,
            logging: Some(LoggingCapability {}),
            completion: None,
        };
        
        let (negotiated_client, negotiated_server) = 
            CapabilityNegotiator::negotiate(&client_caps, &server_caps);
        
        assert!(CapabilityNegotiator::client_supports_capability(&negotiated_client, "roots"));
        assert!(CapabilityNegotiator::supports_capability(&negotiated_server, "tools"));
        assert!(!CapabilityNegotiator::supports_capability(&negotiated_server, "prompts"));
    }
}
