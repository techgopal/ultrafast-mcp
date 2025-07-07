//! Everything MCP Server Example (Streamable HTTP)
//! Comprehensive implementation matching the official MCP everything example

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use ultrafast_mcp::{
    types::{completion, elicitation, prompts, resources, roots, sampling},
    CompletionHandler,
    ElicitationHandler,
    // Monitoring imports
    HttpTransportConfig,
    ListResourcesRequest,
    ListResourcesResponse,
    ListToolsRequest,
    ListToolsResponse,
    MCPResult,
    PromptHandler,
    ReadResourceRequest,
    ReadResourceResponse,
    Resource,
    ResourceContent,
    ResourceHandler,
    ResourceSubscriptionHandler,
    RootsHandler,
    SamplingHandler,
    ServerCapabilities,
    ServerInfo,
    Tool,
    ToolCall,
    ToolContent,
    ToolHandler,
    ToolResult,
    UltraFastServer,
};

// Tiny test image (base64 encoded PNG)
const MCP_TINY_IMAGE: &str = "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";

struct EverythingToolHandler;
#[async_trait::async_trait]
impl ToolHandler for EverythingToolHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult> {
        match call.name.as_str() {
            "echo" => {
                let message = call
                    .arguments
                    .and_then(|args| args.get("message").cloned())
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                    .unwrap_or_else(|| "Hello, World!".to_string());
                Ok(ToolResult {
                    content: vec![ToolContent::text(format!("Echo: {}", message))],
                    is_error: Some(false),
                })
            }
            "add" => {
                let args = call.arguments.unwrap_or_default();
                let a = args.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let b = args.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let sum = a + b;
                Ok(ToolResult {
                    content: vec![ToolContent::text(format!(
                        "The sum of {} and {} is {}.",
                        a, b, sum
                    ))],
                    is_error: Some(false),
                })
            }
            "longRunningOperation" => {
                let args = call.arguments.unwrap_or_default();
                let duration = args
                    .get("duration")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(10.0);
                let steps = args.get("steps").and_then(|v| v.as_u64()).unwrap_or(5);
                let step_duration = duration / steps as f64;

                // Simulate long running operation with progress notifications
                // In a real implementation, you would send actual progress notifications via the server
                for i in 1..=steps {
                    tokio::time::sleep(tokio::time::Duration::from_secs_f64(step_duration)).await;
                    println!("Progress: Step {}/{} completed", i, steps);
                }

                Ok(ToolResult {
                    content: vec![ToolContent::text(format!(
                        "Long running operation completed. Duration: {} seconds, Steps: {}. Progress was tracked through {} steps.",
                        duration, steps, steps
                    ))],
                    is_error: Some(false),
                })
            }
            "printEnv" => {
                let env_vars: HashMap<String, String> = std::env::vars().collect();
                Ok(ToolResult {
                    content: vec![ToolContent::text(
                        serde_json::to_string_pretty(&env_vars).unwrap(),
                    )],
                    is_error: Some(false),
                })
            }
            "sampleLLM" => {
                let args = call.arguments.unwrap_or_default();
                let prompt = args
                    .get("prompt")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Hello");
                let max_tokens = args
                    .get("maxTokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(100);

                // Simulate LLM sampling
                let response = format!(
                    "LLM sampling result for '{}' (max tokens: {}): This is a simulated response.",
                    prompt, max_tokens
                );

                Ok(ToolResult {
                    content: vec![ToolContent::text(response)],
                    is_error: Some(false),
                })
            }
            "getTinyImage" => Ok(ToolResult {
                content: vec![
                    ToolContent::text("This is a tiny image:".to_string()),
                    ToolContent::image(MCP_TINY_IMAGE.to_string(), "image/png".to_string()),
                    ToolContent::text("The image above is the MCP tiny image.".to_string()),
                ],
                is_error: Some(false),
            }),
            "annotatedMessage" => {
                let args = call.arguments.unwrap_or_default();
                let message_type = args
                    .get("messageType")
                    .and_then(|v| v.as_str())
                    .unwrap_or("success");
                let include_image = args
                    .get("includeImage")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let mut content = vec![];

                match message_type {
                    "error" => {
                        content.push(ToolContent::text("Error: Operation failed".to_string()));
                    }
                    "success" => {
                        content.push(ToolContent::text(
                            "Operation completed successfully".to_string(),
                        ));
                    }
                    "debug" => {
                        content.push(ToolContent::text(
                            "Debug: Cache hit ratio 0.95, latency 150ms".to_string(),
                        ));
                    }
                    _ => {
                        content.push(ToolContent::text("Unknown message type".to_string()));
                    }
                }

                if include_image {
                    content.push(ToolContent::image(
                        MCP_TINY_IMAGE.to_string(),
                        "image/png".to_string(),
                    ));
                }

                Ok(ToolResult {
                    content,
                    is_error: Some(false),
                })
            }
            "getResourceReference" => {
                let args = call.arguments.unwrap_or_default();
                let resource_id = args.get("resourceId").and_then(|v| v.as_u64()).unwrap_or(1);

                let resource_uri = format!("test://static/resource/{}", resource_id);

                Ok(ToolResult {
                    content: vec![
                        ToolContent::text(format!(
                            "Returning resource reference for Resource {}:",
                            resource_id
                        )),
                        ToolContent::resource(resource_uri),
                        ToolContent::text(format!(
                            "You can access this resource using the URI: test://static/resource/{}",
                            resource_id
                        )),
                    ],
                    is_error: Some(false),
                })
            }
            "cancellableOperation" => {
                let args = call.arguments.unwrap_or_default();
                let duration = args
                    .get("duration")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(30.0);
                let check_interval = args
                    .get("checkInterval")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(2.0);

                let mut elapsed = 0.0;
                let mut check_count = 0;

                while elapsed < duration {
                    tokio::time::sleep(tokio::time::Duration::from_secs_f64(check_interval)).await;
                    elapsed += check_interval;
                    check_count += 1;

                    // In a real implementation, you would check for cancellation requests here
                    // For now, we'll just simulate periodic checking
                    println!(
                        "Cancellable operation check #{}: {:.1}/{:.1} seconds",
                        check_count, elapsed, duration
                    );
                }

                Ok(ToolResult {
                    content: vec![ToolContent::text(format!(
                        "Cancellable operation completed after {:.1} seconds with {} checks. This operation could be cancelled by sending a cancellation notification.",
                        elapsed, check_count
                    ))],
                    is_error: Some(false),
                })
            }
            "notificationDemo" => {
                let args = call.arguments.unwrap_or_default();
                let notification_type = args.get("type").and_then(|v| v.as_str()).unwrap_or("info");

                let message = match notification_type {
                    "resource_list_changed" => {
                        "This would trigger a resource list changed notification"
                    }
                    "resource_updated" => "This would trigger a resource updated notification",
                    "tool_list_changed" => "This would trigger a tool list changed notification",
                    "prompt_list_changed" => {
                        "This would trigger a prompt list changed notification"
                    }
                    "log_message" => "This would trigger a log message notification",
                    _ => "This would trigger a general notification",
                };

                Ok(ToolResult {
                    content: vec![ToolContent::text(format!(
                        "Notification demo: {}. In a real implementation, this would send a '{}' notification to connected clients.",
                        message, notification_type
                    ))],
                    is_error: Some(false),
                })
            }
            "getResourceLinks" => {
                let args = call.arguments.unwrap_or_default();
                let count = args.get("count").and_then(|v| v.as_u64()).unwrap_or(3);

                let mut content = vec![ToolContent::text(format!(
                    "Here are {} resource links to resources available in this server:",
                    count
                ))];

                for i in 1..=count.min(100) {
                    content.push(ToolContent::resource_with_description(
                        format!("test://static/resource/{}", i),
                        format!("Resource {}: test resource", i),
                    ));
                }

                Ok(ToolResult {
                    content,
                    is_error: Some(false),
                })
            }
            _ => Ok(ToolResult {
                content: vec![ToolContent::text(format!("Unknown tool: {}", call.name))],
                is_error: Some(true),
            }),
        }
    }

    async fn list_tools(&self, _request: ListToolsRequest) -> MCPResult<ListToolsResponse> {
        Ok(ListToolsResponse {
            tools: vec![
                Tool::new(
                    "echo".to_string(),
                    "Echoes back the input".to_string(),
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "message": {
                                "type": "string",
                                "description": "Message to echo"
                            }
                        },
                        "required": ["message"]
                    })
                ),
                Tool::new(
                    "add".to_string(),
                    "Adds two numbers".to_string(),
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "a": {
                                "type": "number",
                                "description": "First number"
                            },
                            "b": {
                                "type": "number",
                                "description": "Second number"
                            }
                        },
                        "required": ["a", "b"]
                    })
                ),
                Tool::new(
                    "longRunningOperation".to_string(),
                    "Demonstrates a long running operation with progress updates".to_string(),
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "duration": {
                                "type": "number",
                                "default": 10,
                                "description": "Duration of the operation in seconds"
                            },
                            "steps": {
                                "type": "number",
                                "default": 5,
                                "description": "Number of steps in the operation"
                            }
                        }
                    })
                ),
                Tool::new(
                    "printEnv".to_string(),
                    "Prints all environment variables, helpful for debugging MCP server configuration".to_string(),
                    serde_json::json!({
                        "type": "object",
                        "properties": {}
                    })
                ),
                Tool::new(
                    "sampleLLM".to_string(),
                    "Samples from an LLM using MCP's sampling feature".to_string(),
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "prompt": {
                                "type": "string",
                                "description": "The prompt to send to the LLM"
                            },
                            "maxTokens": {
                                "type": "number",
                                "default": 100,
                                "description": "Maximum number of tokens to generate"
                            }
                        },
                        "required": ["prompt"]
                    })
                ),
                Tool::new(
                    "getTinyImage".to_string(),
                    "Returns the MCP_TINY_IMAGE".to_string(),
                    serde_json::json!({
                        "type": "object",
                        "properties": {}
                    })
                ),
                Tool::new(
                    "annotatedMessage".to_string(),
                    "Demonstrates how annotations can be used to provide metadata about content".to_string(),
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "messageType": {
                                "type": "string",
                                "enum": ["error", "success", "debug"],
                                "description": "Type of message to demonstrate different annotation patterns"
                            },
                            "includeImage": {
                                "type": "boolean",
                                "default": false,
                                "description": "Whether to include an example image"
                            }
                        },
                        "required": ["messageType"]
                    })
                ),
                Tool::new(
                    "getResourceReference".to_string(),
                    "Returns a resource reference that can be used by MCP clients".to_string(),
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "resourceId": {
                                "type": "number",
                                "minimum": 1,
                                "maximum": 100,
                                "description": "ID of the resource to reference (1-100)"
                            }
                        },
                        "required": ["resourceId"]
                    })
                ),
                Tool::new(
                    "getResourceLinks".to_string(),
                    "Returns multiple resource links that reference different types of resources".to_string(),
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "count": {
                                "type": "number",
                                "minimum": 1,
                                "maximum": 10,
                                "default": 3,
                                "description": "Number of resource links to return (1-10)"
                            }
                        }
                    })
                ),
                Tool::new(
                    "cancellableOperation".to_string(),
                    "Demonstrates a cancellable long-running operation that can be interrupted".to_string(),
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "duration": {
                                "type": "number",
                                "default": 30,
                                "description": "Duration of the operation in seconds"
                            },
                            "checkInterval": {
                                "type": "number",
                                "default": 2,
                                "description": "Interval between cancellation checks in seconds"
                            }
                        }
                    })
                ),
                Tool::new(
                    "notificationDemo".to_string(),
                    "Demonstrates various MCP notification types that can be sent to clients".to_string(),
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "type": {
                                "type": "string",
                                "enum": ["resource_list_changed", "resource_updated", "tool_list_changed", "prompt_list_changed", "log_message"],
                                "default": "info",
                                "description": "Type of notification to demonstrate"
                            }
                        }
                    })
                ),
            ],
            next_cursor: None,
        })
    }
}

struct EverythingResourceHandler;
#[async_trait::async_trait]
impl ResourceHandler for EverythingResourceHandler {
    async fn read_resource(&self, request: ReadResourceRequest) -> MCPResult<ReadResourceResponse> {
        let uri = request.uri;

        if uri.starts_with("test://static/resource/") {
            let id = uri.split("/").last().unwrap_or("1");
            let resource_id = id.parse::<u64>().unwrap_or(1);

            let resource = if resource_id % 2 == 0 {
                ResourceContent::text(
                    uri.clone(),
                    format!("Resource {}: This is a plaintext resource", resource_id),
                )
            } else {
                let data = format!("Resource {}: This is a base64 blob", resource_id);
                ResourceContent::blob(
                    uri.clone(),
                    BASE64.encode(data.as_bytes()),
                    "application/octet-stream".to_string(),
                )
            };

            Ok(ReadResourceResponse {
                contents: vec![resource],
            })
        } else {
            Err(anyhow::anyhow!("Unknown resource: {}", uri).into())
        }
    }

    async fn list_resources(
        &self,
        request: ListResourcesRequest,
    ) -> MCPResult<ListResourcesResponse> {
        let cursor = request.cursor;
        let page_size = 10;
        let mut start_index = 0;

        if let Some(cursor_str) = cursor {
            if let Ok(decoded) = BASE64.decode(cursor_str) {
                if let Ok(decoded_str) = String::from_utf8(decoded) {
                    if let Ok(index) = decoded_str.parse::<usize>() {
                        start_index = index;
                    }
                }
            }
        }

        let mut resources = vec![];
        for i in start_index..(start_index + page_size).min(100) {
            let resource_id = i + 1;
            let uri = format!("test://static/resource/{}", resource_id);
            let name = format!("Resource {}", resource_id);

            let resource = Resource::new(uri, name);
            resources.push(resource);
        }

        let next_cursor = if start_index + page_size < 100 {
            Some(BASE64.encode((start_index + page_size).to_string()))
        } else {
            None
        };

        Ok(ListResourcesResponse {
            resources,
            next_cursor,
        })
    }

    async fn list_resource_templates(
        &self,
        _request: resources::ListResourceTemplatesRequest,
    ) -> MCPResult<resources::ListResourceTemplatesResponse> {
        Ok(resources::ListResourceTemplatesResponse {
            resource_templates: vec![resources::ResourceTemplate::new(
                "test://static/resource/{id}".to_string(),
                "Static Resource".to_string(),
            )],
            next_cursor: None,
        })
    }
}

struct EverythingPromptHandler;
#[async_trait::async_trait]
impl PromptHandler for EverythingPromptHandler {
    async fn get_prompt(
        &self,
        request: prompts::GetPromptRequest,
    ) -> MCPResult<prompts::GetPromptResponse> {
        let name = request.name;
        let args = request.arguments;

        match name.as_str() {
            "simple_prompt" => Ok(prompts::GetPromptResponse {
                description: Some("A prompt without arguments".to_string()),
                messages: vec![prompts::PromptMessage::user(prompts::PromptContent::text(
                    "This is a simple prompt without arguments.".to_string(),
                ))],
            }),
            "complex_prompt" => {
                let temperature = args
                    .as_ref()
                    .and_then(|a| a.get("temperature").and_then(|v| v.as_str()))
                    .unwrap_or("0.7");
                let style = args
                    .as_ref()
                    .and_then(|a| a.get("style").and_then(|v| v.as_str()))
                    .unwrap_or("default");

                Ok(prompts::GetPromptResponse {
                    description: Some("A prompt with arguments".to_string()),
                    messages: vec![
                        prompts::PromptMessage::user(prompts::PromptContent::text(format!("This is a complex prompt with arguments: temperature={}, style={}", temperature, style))),
                        prompts::PromptMessage::assistant(prompts::PromptContent::text("I understand. You've provided a complex prompt with temperature and style arguments. How would you like me to proceed?".to_string())),
                        prompts::PromptMessage::user(prompts::PromptContent::image(MCP_TINY_IMAGE.to_string(), "image/png".to_string())),
                    ],
                })
            }
            "resource_prompt" => {
                let resource_id = args
                    .as_ref()
                    .and_then(|a| a.get("resourceId").and_then(|v| v.as_str()))
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(1);

                // Allow 0 as a valid resource ID, or use 1 as default if invalid
                let valid_resource_id = if resource_id == 0 {
                    1 // Use 1 as default for 0
                } else if !(1..=100).contains(&resource_id) {
                    return Err(anyhow::anyhow!(
                        "Invalid resourceId: {}. Must be a number between 1 and 100.",
                        resource_id
                    )
                    .into());
                } else {
                    resource_id
                };

                let resource_uri = format!("test://static/resource/{}", valid_resource_id);

                Ok(prompts::GetPromptResponse {
                    description: Some("A prompt that includes an embedded resource reference".to_string()),
                    messages: vec![
                        prompts::PromptMessage::user(prompts::PromptContent::text(format!("This prompt includes Resource {}. Please analyze the following resource:", valid_resource_id))),
                        prompts::PromptMessage::user(prompts::PromptContent::resource_link(
                            format!("Resource {}", valid_resource_id),
                            resource_uri,
                        )),
                    ],
                })
            }
            _ => Err(anyhow::anyhow!("Unknown prompt: {}", name).into()),
        }
    }

    async fn list_prompts(
        &self,
        _request: prompts::ListPromptsRequest,
    ) -> MCPResult<prompts::ListPromptsResponse> {
        Ok(prompts::ListPromptsResponse {
            prompts: vec![
                prompts::Prompt::new("simple_prompt".to_string())
                    .with_description("A prompt without arguments".to_string()),
                prompts::Prompt::new("complex_prompt".to_string())
                    .with_description("A prompt with arguments".to_string())
                    .with_arguments(vec![
                        prompts::PromptArgument::new("temperature".to_string()),
                        prompts::PromptArgument::new("style".to_string()),
                    ]),
                prompts::Prompt::new("resource_prompt".to_string())
                    .with_description(
                        "A prompt that includes an embedded resource reference".to_string(),
                    )
                    .with_arguments(vec![prompts::PromptArgument::new("resourceId".to_string())]),
            ],
            next_cursor: None,
        })
    }
}

struct EverythingSamplingHandler;
#[async_trait::async_trait]
impl SamplingHandler for EverythingSamplingHandler {
    async fn create_message(
        &self,
        request: sampling::CreateMessageRequest,
    ) -> MCPResult<sampling::CreateMessageResponse> {
        let messages = request.messages;
        let system_prompt = request
            .system_prompt
            .unwrap_or_else(|| "You are a helpful test server.".to_string());
        let max_tokens = request.max_tokens.unwrap_or(100);
        let temperature = request.temperature.unwrap_or(0.7);

        // Simulate LLM response
        let response_text = format!("Simulated LLM response based on {} messages, system prompt: '{}', max tokens: {}, temperature: {}", 
            messages.len(), system_prompt, max_tokens, temperature);

        Ok(sampling::CreateMessageResponse {
            role: "assistant".to_string(),
            content: sampling::SamplingContent::text(response_text),
            model: Some("simulated-model".to_string()),
            stop_reason: Some("length".to_string()),
        })
    }
}

struct EverythingCompletionHandler;
#[async_trait::async_trait]
impl CompletionHandler for EverythingCompletionHandler {
    async fn complete(
        &self,
        request: completion::CompleteRequest,
    ) -> MCPResult<completion::CompleteResponse> {
        let ref_type = request.ref_type.as_str();
        let argument = request.argument;

        match ref_type {
            "ref/resource" => {
                let uri = request.ref_name;
                let _resource_id = uri.split("/").last().unwrap_or("");

                let values = vec!["1", "2", "3", "4", "5"]
                    .into_iter()
                    .filter(|id| {
                        if let Some(arg) = &argument {
                            id.starts_with(arg)
                        } else {
                            true
                        }
                    })
                    .map(completion::CompletionValue::new)
                    .collect();

                Ok(completion::CompleteResponse {
                    completion: completion::Completion::new(values),
                    metadata: None,
                })
            }
            "ref/prompt" => {
                let arg_name = request.ref_name.as_str();
                let values = match arg_name {
                    "temperature" => vec!["0", "0.5", "0.7", "1.0"],
                    "style" => vec!["casual", "formal", "technical", "friendly"],
                    "resourceId" => vec!["0", "1", "2", "3", "4", "5"],
                    _ => vec![],
                };

                let filtered_values = values
                    .into_iter()
                    .filter(|value| {
                        if let Some(arg) = &argument {
                            value.starts_with(arg)
                        } else {
                            true
                        }
                    })
                    .map(completion::CompletionValue::new)
                    .collect();

                Ok(completion::CompleteResponse {
                    completion: completion::Completion::new(filtered_values),
                    metadata: None,
                })
            }
            _ => Ok(completion::CompleteResponse {
                completion: completion::Completion::new(vec![]),
                metadata: None,
            }),
        }
    }
}

struct EverythingRootsHandler;
#[async_trait::async_trait]
impl RootsHandler for EverythingRootsHandler {
    async fn list_roots(&self) -> MCPResult<Vec<roots::Root>> {
        Ok(vec![roots::Root {
            uri: "test://static/".to_string(),
            name: Some("Static Resources".to_string()),
            security: None,
        }])
    }
}

struct EverythingElicitationHandler;
#[async_trait::async_trait]
impl ElicitationHandler for EverythingElicitationHandler {
    async fn handle_elicitation(
        &self,
        _request: elicitation::ElicitationRequest,
    ) -> MCPResult<elicitation::ElicitationResponse> {
        Ok(elicitation::ElicitationResponse {
            session_id: Some("test-session".to_string()),
            step: Some(1),
            value: serde_json::json!("elicitation response"),
            cancelled: Some(false),
            validation_errors: None,
            timestamp: None,
        })
    }
}

struct EverythingSubscriptionHandler;
#[async_trait::async_trait]
impl ResourceSubscriptionHandler for EverythingSubscriptionHandler {
    async fn subscribe(&self, uri: String) -> MCPResult<()> {
        println!("Subscribed to resource: {}", uri);
        Ok(())
    }

    async fn unsubscribe(&self, uri: String) -> MCPResult<()> {
        println!("Unsubscribed from resource: {}", uri);
        Ok(())
    }

    async fn notify_change(&self, uri: String, content: Value) -> MCPResult<()> {
        println!("Resource changed: {} -> {:?}", uri, content);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing with monitoring support
    tracing_subscriber::fmt()
        .with_env_filter(
            "info,ultrafast_mcp=debug,ultrafast_mcp_transport=debug,ultrafast_mcp_monitoring=debug",
        )
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    let server_info = ServerInfo {
        name: "example-servers/everything".to_string(),
        version: "1.0.0".to_string(),
        description: Some(
            "Everything MCP Server - Comprehensive example implementing all MCP features"
                .to_string(),
        ),
        authors: None,
        homepage: None,
        license: None,
        repository: None,
    };

    let capabilities = ServerCapabilities {
        tools: Some(ultrafast_mcp::ToolsCapability {
            list_changed: Some(true),
        }),
        resources: Some(ultrafast_mcp::ResourcesCapability {
            subscribe: Some(true),
            list_changed: Some(true),
        }),
        prompts: Some(ultrafast_mcp::PromptsCapability {
            list_changed: Some(true),
        }),
        logging: Some(ultrafast_mcp::LoggingCapability {}),
        completion: Some(ultrafast_mcp::CompletionCapability {}),
    };

    let server = UltraFastServer::new(server_info, capabilities)
        .with_tool_handler(Arc::new(EverythingToolHandler))
        .with_resource_handler(Arc::new(EverythingResourceHandler))
        .with_prompt_handler(Arc::new(EverythingPromptHandler))
        .with_sampling_handler(Arc::new(EverythingSamplingHandler))
        .with_completion_handler(Arc::new(EverythingCompletionHandler))
        .with_roots_handler(Arc::new(EverythingRootsHandler))
        .with_elicitation_handler(Arc::new(EverythingElicitationHandler))
        .with_subscription_handler(Arc::new(EverythingSubscriptionHandler));

    println!("🚀 Starting Everything MCP Server with Streamable HTTP");
    println!("✅ Server created successfully with all handlers");
    println!("🌐 Server starting on 0.0.0.0:8080");
    println!("📊 Monitoring dashboard available at http://127.0.0.1:8081");
    println!("📋 Available tools:");
    println!("  • echo - Echoes back the input");
    println!("  • add - Adds two numbers");
    println!("  • longRunningOperation - Long-running operation with progress tracking");
    println!("  • cancellableOperation - Cancellable long-running operation");
    println!("  • notificationDemo - Demonstrates MCP notification types");
    println!("  • printEnv - Prints environment variables");
    println!("  • sampleLLM - Simulates LLM sampling");
    println!("  • getTinyImage - Returns a tiny test image");
    println!("  • annotatedMessage - Demonstrates message annotations");
    println!("  • getResourceReference - Returns resource references");
    println!("  • getResourceLinks - Returns multiple resource links");
    println!("📁 Available resources: 100 test resources (test://static/resource/1-100)");
    println!("💬 Available prompts: simple_prompt, complex_prompt, resource_prompt");
    println!("🔧 New MCP 2025-06-18 features:");
    println!("  • Progress notifications (demonstrated in longRunningOperation)");
    println!("  • Cancellation support (demonstrated in cancellableOperation)");
    println!("  • Resource subscriptions and notifications");
    println!("  • Enhanced completion and elicitation handlers");
    println!("  • Comprehensive notification system");
    println!("🌐 HTTP-specific features:");
    println!("  • CORS support enabled");
    println!("  • Streamable HTTP transport");
    println!("  • Real-time monitoring dashboard");

    // Create explicit HTTP transport configuration with monitoring enabled
    let transport_config = HttpTransportConfig {
        host: "0.0.0.0".to_string(),
        port: 8080,
        cors_enabled: true,
        protocol_version: "2025-06-18".to_string(),
        allow_origin: Some("*".to_string()), // Allow all origins for development
        monitoring_enabled: true,            // Explicitly enable monitoring
    };

    // Run the server with explicit monitoring configuration
    server.run_http(transport_config).await?;
    Ok(())
}
