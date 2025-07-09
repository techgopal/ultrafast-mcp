//! HTTP Server Example
//!
//! This example demonstrates the new UltraFastServer API with HTTP operations.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tracing::info;
use ultrafast_mcp::{
    ListToolsRequest, ListToolsResponse, MCPError, MCPResult, ServerCapabilities, ServerInfo, Tool,
    ToolCall, ToolContent, ToolHandler, ToolResult, ToolsCapability, UltraFastServer,
};

#[derive(Debug, Deserialize)]
struct HttpGetRequest {
    url: String,
}

#[derive(Debug, Serialize)]
struct HttpGetResponse {
    url: String,
    status: u16,
    content: String,
    headers: std::collections::HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct HttpPostRequest {
    url: String,
    data: Option<String>,
    headers: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
struct HttpPostResponse {
    url: String,
    status: u16,
    content: String,
    headers: std::collections::HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct HttpStatusRequest {
    url: String,
}

#[derive(Debug, Serialize)]
struct HttpStatusResponse {
    url: String,
    status: u16,
    is_online: bool,
}

#[derive(Debug, Deserialize)]
struct HttpInfoRequest {
    url: String,
}

#[derive(Debug, Serialize)]
struct HttpInfoResponse {
    url: String,
    method: String,
    headers: std::collections::HashMap<String, String>,
    content_length: Option<u64>,
}

struct HttpOperationsHandler;

#[async_trait::async_trait]
impl ToolHandler for HttpOperationsHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult> {
        info!("Received tool call: {}", call.name);

        match call.name.as_str() {
            "http_get" => {
                let request: HttpGetRequest =
                    serde_json::from_value(call.arguments.unwrap_or_default())
                        .map_err(|e| MCPError::serialization_error(e.to_string()))?;

                self.handle_http_get(request).await
            }
            "http_post" => {
                let request: HttpPostRequest =
                    serde_json::from_value(call.arguments.unwrap_or_default())
                        .map_err(|e| MCPError::serialization_error(e.to_string()))?;

                self.handle_http_post(request).await
            }
            "http_status" => {
                let request: HttpStatusRequest =
                    serde_json::from_value(call.arguments.unwrap_or_default())
                        .map_err(|e| MCPError::serialization_error(e.to_string()))?;

                self.handle_http_status(request).await
            }
            "http_info" => {
                let request: HttpInfoRequest =
                    serde_json::from_value(call.arguments.unwrap_or_default())
                        .map_err(|e| MCPError::serialization_error(e.to_string()))?;

                self.handle_http_info(request).await
            }
            _ => Err(MCPError::method_not_found(format!(
                "Unknown tool: {}",
                call.name
            ))),
        }
    }

    async fn list_tools(&self, _request: ListToolsRequest) -> MCPResult<ListToolsResponse> {
        Ok(ListToolsResponse {
            tools: vec![
                Tool {
                    name: "http_get".to_string(),
                    description: "Perform an HTTP GET request".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "url": {
                                "type": "string",
                                "description": "URL to perform GET request on"
                            }
                        },
                        "required": ["url"]
                    }),
                    output_schema: None,
                    annotations: None,
                },
                Tool {
                    name: "http_post".to_string(),
                    description: "Perform an HTTP POST request".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "url": {
                                "type": "string",
                                "description": "URL to perform POST request on"
                            },
                            "data": {
                                "type": "string",
                                "description": "Data to send in POST request"
                            },
                            "headers": {
                                "type": "object",
                                "description": "Additional headers to include"
                            }
                        },
                        "required": ["url"]
                    }),
                    output_schema: None,
                    annotations: None,
                },
                Tool {
                    name: "http_status".to_string(),
                    description: "Check the status of an HTTP endpoint".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "url": {
                                "type": "string",
                                "description": "URL to check status of"
                            }
                        },
                        "required": ["url"]
                    }),
                    output_schema: None,
                    annotations: None,
                },
                Tool {
                    name: "http_info".to_string(),
                    description: "Get information about an HTTP request".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "url": {
                                "type": "string",
                                "description": "URL to get information about"
                            }
                        },
                        "required": ["url"]
                    }),
                    output_schema: None,
                    annotations: None,
                },
            ],
            next_cursor: None,
        })
    }
}

impl HttpOperationsHandler {
    async fn handle_http_get(&self, request: HttpGetRequest) -> MCPResult<ToolResult> {
        let client = reqwest::Client::new();
        let response = client
            .get(&request.url)
            .send()
            .await
            .map_err(|e| MCPError::internal_error(format!("HTTP request failed: {}", e)))?;

        let status = response.status().as_u16();
        let headers: std::collections::HashMap<String, String> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let content = response.text().await.map_err(|e| {
            MCPError::internal_error(format!("Failed to read response body: {}", e))
        })?;

        let response_data = HttpGetResponse {
            url: request.url,
            status,
            content,
            headers,
        };

        let response_text = serde_json::to_string_pretty(&response_data)
            .map_err(|e| MCPError::serialization_error(e.to_string()))?;

        Ok(ToolResult {
            content: vec![ToolContent::text(response_text)],
            is_error: None,
        })
    }

    async fn handle_http_post(&self, request: HttpPostRequest) -> MCPResult<ToolResult> {
        let client = reqwest::Client::new();
        let mut req_builder = client.post(&request.url);

        if let Some(data) = request.data {
            req_builder = req_builder.body(data);
        }

        if let Some(headers) = request.headers {
            for (key, value) in headers {
                req_builder = req_builder.header(key, value);
            }
        }

        let response = req_builder
            .send()
            .await
            .map_err(|e| MCPError::internal_error(format!("HTTP request failed: {}", e)))?;

        let status = response.status().as_u16();
        let response_headers: std::collections::HashMap<String, String> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let content = response.text().await.map_err(|e| {
            MCPError::internal_error(format!("Failed to read response body: {}", e))
        })?;

        let response_data = HttpPostResponse {
            url: request.url,
            status,
            content,
            headers: response_headers,
        };

        let response_text = serde_json::to_string_pretty(&response_data)
            .map_err(|e| MCPError::serialization_error(e.to_string()))?;

        Ok(ToolResult {
            content: vec![ToolContent::text(response_text)],
            is_error: None,
        })
    }

    async fn handle_http_status(&self, request: HttpStatusRequest) -> MCPResult<ToolResult> {
        let client = reqwest::Client::new();
        let response = client.head(&request.url).send().await;

        let (status, is_online) = match response {
            Ok(resp) => (resp.status().as_u16(), true),
            Err(_) => (0, false),
        };

        let response_data = HttpStatusResponse {
            url: request.url,
            status,
            is_online,
        };

        let response_text = serde_json::to_string_pretty(&response_data)
            .map_err(|e| MCPError::serialization_error(e.to_string()))?;

        Ok(ToolResult {
            content: vec![ToolContent::text(response_text)],
            is_error: None,
        })
    }

    async fn handle_http_info(&self, request: HttpInfoRequest) -> MCPResult<ToolResult> {
        let client = reqwest::Client::new();
        let response = client.head(&request.url).send().await;

        let response_data = if let Ok(resp) = response {
            let headers: std::collections::HashMap<String, String> = resp
                .headers()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                .collect();

            let content_length = resp
                .headers()
                .get("content-length")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok());

            HttpInfoResponse {
                url: request.url,
                method: "HEAD".to_string(),
                headers,
                content_length,
            }
        } else {
            HttpInfoResponse {
                url: request.url,
                method: "HEAD".to_string(),
                headers: std::collections::HashMap::new(),
                content_length: None,
            }
        };

        let response_text = serde_json::to_string_pretty(&response_data)
            .map_err(|e| MCPError::serialization_error(e.to_string()))?;

        Ok(ToolResult {
            content: vec![ToolContent::text(response_text)],
            is_error: None,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let start_time = Instant::now();
    info!("Starting HTTP Server MCP Server");

    // Set up graceful shutdown
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

    // Handle shutdown signals
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        info!("Received shutdown signal");
        let _ = shutdown_tx.send(());
    });

    // Create server capabilities
    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability {
            list_changed: Some(true),
        }),
        ..Default::default()
    };

    // Create server
    let server = UltraFastServer::new(
        ServerInfo {
            name: "http-server".to_string(),
            version: "1.0.0".to_string(),
            description: Some(
                "An HTTP operations server demonstrating UltraFastServer".to_string(),
            ),
            authors: None,
            homepage: None,
            license: None,
            repository: None,
        },
        capabilities,
    )
    .with_tool_handler(Arc::new(HttpOperationsHandler));

    info!("Server created, starting stdio transport");

    // Run the server with graceful shutdown
    tokio::select! {
        result = server.run_stdio() => {
            if let Err(e) = result {
                eprintln!("Server error: {}", e);
            }
        }
        _ = shutdown_rx => {
            info!("Shutting down server gracefully");
        }
    }

    let uptime = start_time.elapsed();
    info!("Server stopped after {:?}", uptime);

    Ok(())
}
