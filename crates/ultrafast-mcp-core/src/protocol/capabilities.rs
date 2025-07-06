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

/// Result of capability negotiation
#[derive(Debug, Clone)]
pub struct CapabilityNegotiationResult {
    /// Negotiated client capabilities
    pub client_capabilities: ClientCapabilities,
    /// Negotiated server capabilities  
    pub server_capabilities: ServerCapabilities,
    /// List of warnings about unsupported features
    pub warnings: Vec<String>,
}

/// Capability negotiation utilities
pub struct CapabilityNegotiator;

impl CapabilityNegotiator {
    /// Negotiate capabilities between client and server
    /// Returns the intersection of compatible capabilities
    pub fn negotiate(
        client_caps: &ClientCapabilities,
        server_caps: &ServerCapabilities,
        protocol_version: &str,
    ) -> Result<CapabilityNegotiationResult, String> {
        let mut negotiated_client = ClientCapabilities::default();
        let mut negotiated_server = ServerCapabilities::default();
        let mut warnings = Vec::new();

        // Negotiate tools capability
        if server_caps.tools.is_some() {
            negotiated_server.tools = server_caps.tools.clone();
        } else if client_caps.sampling.is_some() {
            warnings.push(
                "Client supports sampling but server does not provide tools capability".to_string(),
            );
        }

        // Negotiate resources capability
        if server_caps.resources.is_some() {
            let mut negotiated_resources = server_caps.resources.clone().unwrap();

            // Check if client can handle subscriptions based on protocol version
            if let Some(ref server_res) = server_caps.resources {
                if server_res.subscribe == Some(true)
                    && !Self::version_supports_feature(protocol_version, "resource_subscriptions")
                {
                    // Disable subscriptions for older protocol versions
                    negotiated_resources.subscribe = Some(false);
                    warnings.push(format!(
                        "Resource subscriptions disabled for protocol version {}",
                        protocol_version
                    ));
                }
            }

            negotiated_server.resources = Some(negotiated_resources);
        }

        // Negotiate prompts capability
        if server_caps.prompts.is_some() {
            negotiated_server.prompts = server_caps.prompts.clone();
        }

        // Negotiate logging capability
        if server_caps.logging.is_some() {
            negotiated_server.logging = server_caps.logging.clone();
        }

        // Negotiate completion capability
        if server_caps.completion.is_some() {
            negotiated_server.completion = server_caps.completion.clone();
        }

        // Negotiate client capabilities
        if client_caps.roots.is_some() {
            negotiated_client.roots = client_caps.roots.clone();
        }

        if client_caps.sampling.is_some() && server_caps.tools.is_some() {
            // Client can only use sampling if server supports tools
            negotiated_client.sampling = client_caps.sampling.clone();
        } else if client_caps.sampling.is_some() {
            warnings.push(
                "Client sampling capability disabled - server does not support tools".to_string(),
            );
        }

        if client_caps.elicitation.is_some() {
            negotiated_client.elicitation = client_caps.elicitation.clone();
        }

        // Validate that we have at least one compatible capability
        if negotiated_server.tools.is_none()
            && negotiated_server.resources.is_none()
            && negotiated_server.prompts.is_none()
        {
            return Err("No compatible capabilities found between client and server".to_string());
        }

        Ok(CapabilityNegotiationResult {
            client_capabilities: negotiated_client,
            server_capabilities: negotiated_server,
            warnings,
        })
    }

    /// Check if a capability is supported by server
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

    /// Check if a specific feature is supported within a capability
    pub fn supports_feature(caps: &ServerCapabilities, capability: &str, feature: &str) -> bool {
        match (capability, feature) {
            ("resources", "subscribe") => caps
                .resources
                .as_ref()
                .and_then(|r| r.subscribe)
                .unwrap_or(false),
            ("tools", "list_changed") => caps
                .tools
                .as_ref()
                .and_then(|t| t.list_changed)
                .unwrap_or(false),
            ("resources", "list_changed") => caps
                .resources
                .as_ref()
                .and_then(|r| r.list_changed)
                .unwrap_or(false),
            ("prompts", "list_changed") => caps
                .prompts
                .as_ref()
                .and_then(|p| p.list_changed)
                .unwrap_or(false),
            _ => false,
        }
    }

    /// Check if a protocol version supports a specific feature
    pub fn version_supports_feature(version: &str, feature: &str) -> bool {
        // Use the version module's implementation
        crate::protocol::version::version_supports_feature(version, feature)
    }

    /// Validate compatibility between client and server capabilities
    pub fn validate_compatibility(
        client_caps: &ClientCapabilities,
        server_caps: &ServerCapabilities,
    ) -> Result<(), String> {
        // Check if client requires sampling but server doesn't support tools
        if client_caps.sampling.is_some() && server_caps.tools.is_none() {
            return Err(
                "Client requires sampling capability but server does not support tools".to_string(),
            );
        }

        // Check if we have at least one overlapping capability
        let has_overlap = server_caps.tools.is_some()
            || server_caps.resources.is_some()
            || server_caps.prompts.is_some()
            || server_caps.logging.is_some()
            || server_caps.completion.is_some();

        if !has_overlap {
            return Err("No overlapping capabilities between client and server".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_negotiation() {
        let client_caps = ClientCapabilities {
            roots: Some(RootsCapability {
                list_changed: Some(true),
            }),
            sampling: Some(SamplingCapability {}),
            elicitation: None,
        };

        let server_caps = ServerCapabilities {
            tools: Some(ToolsCapability {
                list_changed: Some(true),
            }),
            resources: Some(ResourcesCapability {
                subscribe: Some(true),
                list_changed: Some(true),
            }),
            prompts: None,
            logging: Some(LoggingCapability {}),
            completion: None,
        };

        let result =
            CapabilityNegotiator::negotiate(&client_caps, &server_caps, "2025-06-18").unwrap();

        assert!(CapabilityNegotiator::client_supports_capability(
            &result.client_capabilities,
            "roots"
        ));
        assert!(CapabilityNegotiator::supports_capability(
            &result.server_capabilities,
            "tools"
        ));
        assert!(!CapabilityNegotiator::supports_capability(
            &result.server_capabilities,
            "prompts"
        ));
        assert!(CapabilityNegotiator::supports_feature(
            &result.server_capabilities,
            "resources",
            "subscribe"
        ));
    }

    #[test]
    fn test_version_based_feature_support() {
        assert!(CapabilityNegotiator::version_supports_feature(
            "2025-06-18",
            "resource_subscriptions"
        ));
        assert!(!CapabilityNegotiator::version_supports_feature(
            "2024-11-05",
            "resource_subscriptions"
        ));
        assert!(CapabilityNegotiator::version_supports_feature(
            "2025-06-18",
            "progress_tracking"
        ));
        assert!(!CapabilityNegotiator::version_supports_feature(
            "2024-11-05",
            "progress_tracking"
        ));
    }

    #[test]
    fn test_compatibility_validation() {
        let client_with_sampling = ClientCapabilities {
            sampling: Some(SamplingCapability {}),
            ..Default::default()
        };

        let server_without_tools = ServerCapabilities {
            resources: Some(ResourcesCapability::default()),
            ..Default::default()
        };

        let result = CapabilityNegotiator::validate_compatibility(
            &client_with_sampling,
            &server_without_tools,
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("sampling capability but server does not support tools"));
    }

    #[test]
    fn test_no_overlap_error() {
        let client_caps = ClientCapabilities::default();
        let server_caps = ServerCapabilities::default();

        let result = CapabilityNegotiator::negotiate(&client_caps, &server_caps, "2025-06-18");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("No compatible capabilities found"));
    }
}
