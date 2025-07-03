//! Handler traits for UltraFastServer
//!
//! This module defines the trait interfaces that server implementations must implement
//! to handle different types of MCP requests.

use async_trait::async_trait;
use ultrafast_mcp_core::{
    error::MCPResult,
    types::{
        completion::{CompleteRequest, CompleteResponse},
        elicitation::{ElicitationRequest, ElicitationResponse},
        prompts::{GetPromptRequest, GetPromptResponse, ListPromptsRequest, ListPromptsResponse},
        resources::{
            ListResourceTemplatesRequest, ListResourceTemplatesResponse, ListResourcesRequest,
            ListResourcesResponse, ReadResourceRequest, ReadResourceResponse,
        },
        sampling::{CreateMessageRequest, CreateMessageResponse},
        tools::{ListToolsRequest, ListToolsResponse, ToolCall, ToolResult},
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
}
