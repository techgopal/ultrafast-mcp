//! Handler traits for UltraFastServer
//!
//! This module defines the trait interfaces that server implementations must implement
//! to handle different types of MCP requests.

use async_trait::async_trait;
use ultrafast_mcp_core::{
    error::{MCPError, MCPResult},
    types::{
        completion::{CompleteRequest, CompleteResponse},
        elicitation::{ElicitationRequest, ElicitationResponse},
        prompts::{GetPromptRequest, GetPromptResponse, ListPromptsRequest, ListPromptsResponse},
        resources::{
            ListResourceTemplatesRequest, ListResourceTemplatesResponse, ListResourcesRequest,
            ListResourcesResponse, ReadResourceRequest, ReadResourceResponse,
        },
        sampling::{
            CreateMessageRequest, CreateMessageResponse, SamplingContext, SamplingRequest,
            SamplingResponse, ServerContextInfo, ToolContextInfo, ResourceContextInfo,
            ApprovalStatus, HumanFeedback, CostInfo, IncludeContext, SamplingRole, SamplingContent, StopReason,
        },
        tools::{ListToolsRequest, ListToolsResponse, ToolCall, ToolResult},
        ServerInfo,
    },
};

/// Tool handler trait for processing tool calls
#[async_trait]
pub trait ToolHandler: Send + Sync {
    /// Handle a tool call request
    async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult>;

    /// List available tools
    async fn list_tools(&self, request: ListToolsRequest) -> MCPResult<ListToolsResponse>;
}

/// Resource handler trait for managing resources
#[async_trait]
pub trait ResourceHandler: Send + Sync {
    /// Read a resource
    async fn read_resource(&self, request: ReadResourceRequest) -> MCPResult<ReadResourceResponse>;

    /// List available resources
    async fn list_resources(
        &self,
        request: ListResourcesRequest,
    ) -> MCPResult<ListResourcesResponse>;

    /// List resource templates
    async fn list_resource_templates(
        &self,
        request: ListResourceTemplatesRequest,
    ) -> MCPResult<ListResourceTemplatesResponse>;

    /// Validate resource access against roots (optional implementation)
    /// According to MCP specification, roots are informational and not strictly enforcing.
    /// This method provides advisory validation but does not block access if no root matches.
    async fn validate_resource_access(
        &self,
        uri: &str,
        operation: ultrafast_mcp_core::types::roots::RootOperation,
        roots: &[ultrafast_mcp_core::types::roots::Root],
    ) -> MCPResult<()>;
}

/// Prompt handler trait for managing prompts
#[async_trait]
pub trait PromptHandler: Send + Sync {
    /// Get a specific prompt
    async fn get_prompt(&self, request: GetPromptRequest) -> MCPResult<GetPromptResponse>;

    /// List available prompts
    async fn list_prompts(&self, request: ListPromptsRequest) -> MCPResult<ListPromptsResponse>;
}

/// Sampling handler trait for LLM completions
#[async_trait]
pub trait SamplingHandler: Send + Sync {
    /// Create a message using sampling
    async fn create_message(
        &self,
        request: CreateMessageRequest,
    ) -> MCPResult<CreateMessageResponse>;
}

/// Completion handler trait for autocompletion
#[async_trait]
pub trait CompletionHandler: Send + Sync {
    /// Complete a request
    async fn complete(&self, request: CompleteRequest) -> MCPResult<CompleteResponse>;
}

/// Roots handler trait for filesystem boundary management
#[async_trait]
pub trait RootsHandler: Send + Sync {
    /// List available roots
    async fn list_roots(&self) -> MCPResult<Vec<ultrafast_mcp_core::types::roots::Root>>;
    /// Set/update the list of roots
    async fn set_roots(&self, roots: Vec<ultrafast_mcp_core::types::roots::Root>) -> MCPResult<()> {
        let _ = roots;
        Err(MCPError::method_not_found("Dynamic roots update not implemented".to_string()))
    }
}

/// Elicitation handler trait for user input collection
#[async_trait]
pub trait ElicitationHandler: Send + Sync {
    /// Handle an elicitation request
    async fn handle_elicitation(
        &self,
        request: ElicitationRequest,
    ) -> MCPResult<ElicitationResponse>;
}

/// Resource subscription handler trait
#[async_trait]
pub trait ResourceSubscriptionHandler: Send + Sync {
    /// Subscribe to a resource
    async fn subscribe(&self, uri: String) -> MCPResult<()>;

    /// Unsubscribe from a resource
    async fn unsubscribe(&self, uri: String) -> MCPResult<()>;

    /// Notify about a resource change
    async fn notify_change(&self, uri: String, content: serde_json::Value) -> MCPResult<()>;
}

/// Handler for advanced sampling features including context collection and human-in-the-loop
#[async_trait]
pub trait AdvancedSamplingHandler: Send + Sync {
    /// Collect context information for sampling requests
    async fn collect_context(
        &self,
        include_context: &IncludeContext,
        request: &SamplingRequest,
    ) -> MCPResult<Option<SamplingContext>>;

    /// Handle human-in-the-loop approval workflow
    async fn handle_human_approval(
        &self,
        request: &SamplingRequest,
        response: &SamplingResponse,
    ) -> MCPResult<ApprovalStatus>;

    /// Process human feedback and modifications
    async fn process_human_feedback(
        &self,
        request: &SamplingRequest,
        feedback: &HumanFeedback,
    ) -> MCPResult<SamplingResponse>;

    /// Estimate cost for sampling request
    async fn estimate_cost(&self, request: &SamplingRequest) -> MCPResult<CostInfo>;

    /// Validate sampling request with advanced checks
    async fn validate_sampling_request(&self, request: &SamplingRequest) -> MCPResult<Vec<String>>;
}

/// Default implementation of advanced sampling features
pub struct DefaultAdvancedSamplingHandler {
    server_info: ServerInfo,
    tools: Vec<ToolContextInfo>,
    resources: Vec<ResourceContextInfo>,
}

impl DefaultAdvancedSamplingHandler {
    pub fn new(server_info: ServerInfo) -> Self {
        Self {
            server_info,
            tools: Vec::new(),
            resources: Vec::new(),
        }
    }

    pub fn with_tools(mut self, tools: Vec<ToolContextInfo>) -> Self {
        self.tools = tools;
        self
    }

    pub fn with_resources(mut self, resources: Vec<ResourceContextInfo>) -> Self {
        self.resources = resources;
        self
    }
}

#[async_trait]
impl AdvancedSamplingHandler for DefaultAdvancedSamplingHandler {
    async fn collect_context(
        &self,
        include_context: &IncludeContext,
        _request: &SamplingRequest,
    ) -> MCPResult<Option<SamplingContext>> {
        match include_context {
            IncludeContext::None => Ok(None),
            IncludeContext::ThisServer => {
                let server_info = ServerContextInfo {
                    name: self.server_info.name.clone(),
                    version: self.server_info.version.clone(),
                    description: self.server_info.description.clone(),
                    capabilities: vec!["tools".to_string(), "resources".to_string(), "prompts".to_string()],
                };

                Ok(Some(SamplingContext {
                    server_info: Some(server_info),
                    available_tools: Some(self.tools.clone()),
                    available_resources: Some(self.resources.clone()),
                    conversation_history: None,
                    user_preferences: None,
                }))
            }
            IncludeContext::AllServers => {
                // In a real implementation, this would collect context from all connected servers
                let server_info = ServerContextInfo {
                    name: self.server_info.name.clone(),
                    version: self.server_info.version.clone(),
                    description: self.server_info.description.clone(),
                    capabilities: vec!["tools".to_string(), "resources".to_string(), "prompts".to_string()],
                };

                Ok(Some(SamplingContext {
                    server_info: Some(server_info),
                    available_tools: Some(self.tools.clone()),
                    available_resources: Some(self.resources.clone()),
                    conversation_history: None,
                    user_preferences: None,
                }))
            }
        }
    }

    async fn handle_human_approval(
        &self,
        request: &SamplingRequest,
        _response: &SamplingResponse,
    ) -> MCPResult<ApprovalStatus> {
        // Check if human approval is required
        if let Some(hitl) = &request.human_in_the_loop {
            if hitl.require_prompt_approval.unwrap_or(false) {
                // In a real implementation, this would trigger a UI prompt for approval
                return Ok(ApprovalStatus::Pending);
            }
            if hitl.require_completion_approval.unwrap_or(false) {
                // In a real implementation, this would trigger a UI prompt for approval
                return Ok(ApprovalStatus::Pending);
            }
        }

        Ok(ApprovalStatus::Approved)
    }

    async fn process_human_feedback(
        &self,
        _request: &SamplingRequest,
        feedback: &HumanFeedback,
    ) -> MCPResult<SamplingResponse> {
        // In a real implementation, this would process the human feedback
        // and potentially modify the response based on the feedback
        Ok(SamplingResponse {
            role: SamplingRole::Assistant,
            content: SamplingContent::Text {
                text: format!("Response modified based on feedback: {}", 
                    feedback.reason.as_deref().unwrap_or("No reason provided")),
            },
            model: Some("human-modified".to_string()),
            stop_reason: Some(StopReason::EndTurn),
            approval_status: Some(ApprovalStatus::Modified),
            request_id: None,
            processing_time_ms: None,
            cost_info: None,
            included_context: None,
            human_feedback: Some(feedback.clone()),
            warnings: None,
        })
    }

    async fn estimate_cost(&self, request: &SamplingRequest) -> MCPResult<CostInfo> {
        let input_tokens = request.estimate_input_tokens()
            .map_err(|e| MCPError::invalid_request(e))?;
        let output_tokens = request.max_tokens.unwrap_or(1000);

        // Simple cost estimation: $0.002 per 1K input tokens, $0.012 per 1K output tokens
        let input_cost_cents = (input_tokens as f64 / 1000.0) * 0.2; // $0.002 * 100 = 0.2 cents
        let output_cost_cents = (output_tokens as f64 / 1000.0) * 1.2; // $0.012 * 100 = 1.2 cents
        let total_cost_cents = input_cost_cents + output_cost_cents;

        Ok(CostInfo {
            total_cost_cents,
            input_cost_cents,
            output_cost_cents,
            input_tokens,
            output_tokens,
            model: "gpt-4".to_string(),
        })
    }

    async fn validate_sampling_request(&self, request: &SamplingRequest) -> MCPResult<Vec<String>> {
        let mut warnings = Vec::new();

        // Check for potential issues
        if request.messages.is_empty() {
            warnings.push("No messages provided for sampling".to_string());
        }

        if let Some(temp) = request.temperature {
            if temp > 1.0 {
                warnings.push("Temperature is very high, may produce unpredictable results".to_string());
            }
        }

        if let Some(max_tokens) = request.max_tokens {
            if max_tokens > 10000 {
                warnings.push("Very high max_tokens may be expensive".to_string());
            }
        }

        if request.requires_human_approval() {
            warnings.push("Human approval required - response may be delayed".to_string());
        }

        if request.requires_image_modality() {
            warnings.push("Image modality detected - ensure model supports vision".to_string());
        }

        Ok(warnings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Mock implementations for testing
    struct MockToolHandler;

    #[async_trait]
    impl ToolHandler for MockToolHandler {
        async fn handle_tool_call(&self, _call: ToolCall) -> MCPResult<ToolResult> {
            Ok(ToolResult {
                content: vec![ultrafast_mcp_core::types::tools::ToolContent::text(
                    "mock result".to_string(),
                )],
                is_error: None,
            })
        }

        async fn list_tools(&self, _request: ListToolsRequest) -> MCPResult<ListToolsResponse> {
            Ok(ListToolsResponse {
                tools: vec![],
                next_cursor: None,
            })
        }
    }

    struct MockResourceHandler;

    #[async_trait]
    impl ResourceHandler for MockResourceHandler {
        async fn read_resource(
            &self,
            _request: ReadResourceRequest,
        ) -> MCPResult<ReadResourceResponse> {
            Ok(ReadResourceResponse {
                contents: vec![ultrafast_mcp_core::types::resources::ResourceContent::text(
                    "mock://resource".to_string(),
                    "mock resource".to_string(),
                )],
            })
        }

        async fn list_resources(
            &self,
            _request: ListResourcesRequest,
        ) -> MCPResult<ListResourcesResponse> {
            Ok(ListResourcesResponse {
                resources: vec![],
                next_cursor: None,
            })
        }

            async fn list_resource_templates(
        &self,
        _request: ListResourceTemplatesRequest,
    ) -> MCPResult<ListResourceTemplatesResponse> {
        Ok(ListResourceTemplatesResponse {
            resource_templates: vec![],
            next_cursor: None,
        })
    }

    async fn validate_resource_access(
        &self,
        uri: &str,
        operation: ultrafast_mcp_core::types::roots::RootOperation,
        roots: &[ultrafast_mcp_core::types::roots::Root],
    ) -> MCPResult<()> {
        if roots.is_empty() {
            return Ok(());
        }
        for root in roots {
            if uri.starts_with(&root.uri) {
                if root.uri.starts_with("file://") && uri.starts_with("file://") {
                    let validator = ultrafast_mcp_core::types::roots::RootSecurityValidator::default();
                    return validator
                        .validate_access(root, uri, operation)
                        .map_err(|e| MCPError::Resource(ultrafast_mcp_core::error::ResourceError::AccessDenied(format!("Root validation failed: {}", e))));
                } else {
                    return Ok(());
                }
            }
        }
        Ok(())
    }
    }

    #[tokio::test]
    async fn test_tool_handler() {
        let handler = MockToolHandler;
        let call = ToolCall {
            name: "test".to_string(),
            arguments: Some(json!({"test": "data"})),
        };

        let result = handler.handle_tool_call(call).await.unwrap();
        assert_eq!(result.content.len(), 1);
    }

    #[tokio::test]
    async fn test_resource_handler() {
        let handler = MockResourceHandler;
        let request = ReadResourceRequest {
            uri: "test://resource".to_string(),
        };

        let result = handler.read_resource(request).await.unwrap();
        assert_eq!(result.contents.len(), 1);
    }

    #[tokio::test]
    async fn test_root_validation_informational() {
        let handler = MockResourceHandler;
        
        // Test with no roots configured - should allow access
        let result = handler
            .validate_resource_access(
                "test://static/resource/1",
                ultrafast_mcp_core::types::roots::RootOperation::Read,
                &[],
            )
            .await;
        assert!(result.is_ok(), "Should allow access when no roots are configured");

        // Test with roots configured but no matching root - should allow access (informational)
        let roots = vec![
            ultrafast_mcp_core::types::roots::Root {
                uri: "file:///tmp".to_string(),
                name: Some("Test Root".to_string()),
                security: None,
            }
        ];
        
        let result = handler
            .validate_resource_access(
                "test://static/resource/1",
                ultrafast_mcp_core::types::roots::RootOperation::Read,
                &roots,
            )
            .await;
        assert!(result.is_ok(), "Should allow access when no matching root is found (informational nature)");

        // Test with matching root - use file URI so validator logic is exercised
        let roots = vec![
            ultrafast_mcp_core::types::roots::Root {
                uri: "file:///tmp/static/".to_string(),
                name: Some("Test Root".to_string()),
                security: Some(ultrafast_mcp_core::types::roots::RootSecurityConfig {
                    allow_read: true,
                    ..Default::default()
                }),
            }
        ];
        
        let result = handler
            .validate_resource_access(
                "file:///tmp/static/resource/1",
                ultrafast_mcp_core::types::roots::RootOperation::Read,
                &roots,
            )
            .await;
        assert!(result.is_ok(), "Should allow access when matching root allows it");
    }
}
