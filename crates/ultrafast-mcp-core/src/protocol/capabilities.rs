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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

impl ServerCapabilities {
    /// Check if server supports a specific capability
    pub fn supports_capability(&self, capability: &str) -> bool {
        match capability {
            "tools" => self.tools.is_some(),
            "resources" => self.resources.is_some(),
            "prompts" => self.prompts.is_some(),
            "logging" => self.logging.is_some(),
            "completion" => self.completion.is_some(),
            _ => false,
        }
    }

    /// Check if server supports a specific feature within a capability
    pub fn supports_feature(&self, capability: &str, feature: &str) -> bool {
        match (capability, feature) {
            ("tools", "list_changed") => {
                self.tools.as_ref().and_then(|t| t.list_changed).unwrap_or(false)
            }
            ("resources", "subscribe") => {
                self.resources.as_ref().and_then(|r| r.subscribe).unwrap_or(false)
            }
            ("resources", "list_changed") => {
                self.resources.as_ref().and_then(|r| r.list_changed).unwrap_or(false)
            }
            ("prompts", "list_changed") => {
                self.prompts.as_ref().and_then(|p| p.list_changed).unwrap_or(false)
            }
            _ => false,
        }
    }
}

impl ClientCapabilities {
    /// Check if client supports a specific capability
    pub fn supports_capability(&self, capability: &str) -> bool {
        match capability {
            "roots" => self.roots.is_some(),
            "sampling" => self.sampling.is_some(),
            "elicitation" => self.elicitation.is_some(),
            _ => false,
        }
    }
}

/// Validate compatibility between client and server capabilities
pub fn validate_compatibility(
    client_caps: &ClientCapabilities,
    server_caps: &ServerCapabilities,
) -> Result<(), String> {
    // Check if client wants sampling but server doesn't support tools
    if client_caps.sampling.is_some() && server_caps.tools.is_none() {
        return Err("Client supports sampling but server does not provide tools capability".to_string());
    }

    // Check if we have at least one compatible capability
    if !server_caps.supports_capability("tools")
        && !server_caps.supports_capability("resources")
        && !server_caps.supports_capability("prompts")
    {
        return Err("No compatible capabilities found between client and server".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_negotiation() {
        let server_caps = ServerCapabilities {
            tools: Some(ToolsCapability { list_changed: Some(true) }),
            ..Default::default()
        };
        let client_caps = ClientCapabilities {
            sampling: Some(SamplingCapability {}),
            ..Default::default()
        };

        // Client can use sampling if server supports tools
        assert!(validate_compatibility(&client_caps, &server_caps).is_ok());
    }

    #[test]
    fn test_compatibility_validation() {
        let server_caps = ServerCapabilities {
            tools: Some(ToolsCapability { list_changed: Some(true) }),
            ..Default::default()
        };
        let client_caps = ClientCapabilities::default();

        // Valid compatibility
        assert!(validate_compatibility(&client_caps, &server_caps).is_ok());

        // Invalid: client wants sampling but server has no tools
        let client_with_sampling = ClientCapabilities {
            sampling: Some(SamplingCapability {}),
            ..Default::default()
        };
        let server_without_tools = ServerCapabilities::default();
        assert!(validate_compatibility(&client_with_sampling, &server_without_tools).is_err());
    }

    #[test]
    fn test_no_overlap_error() {
        let server_caps = ServerCapabilities::default();
        let client_caps = ClientCapabilities::default();

        // No compatible capabilities
        assert!(validate_compatibility(&client_caps, &server_caps).is_err());
    }
}
