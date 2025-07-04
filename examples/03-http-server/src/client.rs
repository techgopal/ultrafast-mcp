//! HTTP Client Example
//!
//! This example demonstrates the new UltraFastClient API by connecting to the HTTP server.

use serde::{Deserialize, Serialize};
use tracing::info;
use ultrafast_mcp::{ClientCapabilities, ClientInfo, ToolCall, ToolContent, UltraFastClient};

#[derive(Debug, Serialize, Deserialize)]
struct HttpGetRequest {
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct HttpGetResponse {
    url: String,
    status: u16,
    content: String,
    headers: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct HttpPostRequest {
    url: String,
    data: Option<String>,
    headers: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct HttpPostResponse {
    url: String,
    status: u16,
    content: String,
    headers: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct HttpStatusRequest {
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct HttpStatusResponse {
    url: String,
    status: u16,
    is_online: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct HttpInfoRequest {
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct HttpInfoResponse {
    url: String,
    method: String,
    headers: std::collections::HashMap<String, String>,
    content_length: Option<u64>,
}

#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct CreateApiKeyOutput {
    key_id: String,
    api_key: String,
    name: String,
    permissions: Vec<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    success: bool,
    error: Option<String>,
}

#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct ListApiKeysOutput {
    keys: Vec<ApiKeyInfo>,
    total_count: usize,
    error: Option<String>,
}

#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct ApiKeyInfo {
    key_id: String,
    name: String,
    permissions: Vec<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    is_expired: bool,
}

#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct StartTaskOutput {
    task_id: String,
    task_type: String,
    status: String,
    created_at: chrono::DateTime<chrono::Utc>,
    estimated_duration: Option<u64>,
    success: bool,
    error: Option<String>,
}

#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct GetTaskStatusOutput {
    task_id: String,
    task_type: String,
    status: String,
    progress: f64,
    result: Option<serde_json::Value>,
    error: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct WebFetchOutput {
    url: String,
    status_code: u16,
    headers: std::collections::HashMap<String, String>,
    body: String,
    response_time_ms: u64,
    success: bool,
    error: Option<String>,
}

#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct ApiKeyResource {
    key_id: String,
    name: String,
    permissions: Vec<String>,
    created_at: String,
    expires_at: Option<String>,
    is_expired: bool,
    last_used: Option<String>,
}

#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct TaskResource {
    task_id: String,
    task_type: String,
    status: String,
    progress: f64,
    result: Option<serde_json::Value>,
    error: Option<String>,
    created_at: String,
    updated_at: String,
    completed_at: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting HTTP Server MCP Client");

    // Create client info and capabilities
    let client_info = ClientInfo {
        name: "http-server-client".to_string(),
        version: "1.0.0".to_string(),
        description: Some("An HTTP operations client demonstrating UltraFastClient".to_string()),
        authors: None,
        homepage: None,
        license: None,
        repository: None,
    };

    let client_capabilities = ClientCapabilities {
        ..Default::default()
    };

    // Create client
    let client = UltraFastClient::new(client_info, client_capabilities);

    info!("Connecting to server via stdio");

    // Connect to server
    client.connect_stdio().await?;

    info!("Connected! Listing available tools");

    // List available tools
    let tools = client.list_tools().await?;
    info!("Available tools: {:?}", tools);

    // Test HTTP GET
    let get_request = HttpGetRequest {
        url: "https://httpbin.org/get".to_string(),
    };

    let tool_call = ToolCall {
        name: "http_get".to_string(),
        arguments: Some(serde_json::to_value(get_request)?),
    };

    info!("Testing HTTP GET request");
    let result = client.call_tool(tool_call).await?;

    for content in result.content {
        match content {
            ToolContent::Text { text } => {
                info!("GET response: {}", text);
                let response: HttpGetResponse = serde_json::from_str(&text)?;
                println!("GET {} - Status: {}", response.url, response.status);
            }
            _ => {
                info!("Received non-text content: {:?}", content);
            }
        }
    }

    // Test HTTP POST
    let post_request = HttpPostRequest {
        url: "https://httpbin.org/post".to_string(),
        data: Some("Hello, UltraFast MCP!".to_string()),
        headers: None,
    };

    let tool_call = ToolCall {
        name: "http_post".to_string(),
        arguments: Some(serde_json::to_value(post_request)?),
    };

    info!("Testing HTTP POST request");
    let result = client.call_tool(tool_call).await?;

    for content in result.content {
        match content {
            ToolContent::Text { text } => {
                info!("POST response: {}", text);
                let response: HttpPostResponse = serde_json::from_str(&text)?;
                println!("POST {} - Status: {}", response.url, response.status);
            }
            _ => {
                info!("Received non-text content: {:?}", content);
            }
        }
    }

    // Test HTTP Status
    let status_request = HttpStatusRequest {
        url: "https://httpbin.org/status/200".to_string(),
    };

    let tool_call = ToolCall {
        name: "http_status".to_string(),
        arguments: Some(serde_json::to_value(status_request)?),
    };

    info!("Testing HTTP status check");
    let result = client.call_tool(tool_call).await?;

    for content in result.content {
        match content {
            ToolContent::Text { text } => {
                info!("Status response: {}", text);
                let response: HttpStatusResponse = serde_json::from_str(&text)?;
                println!("Status {} - Online: {}", response.url, response.is_online);
            }
            _ => {
                info!("Received non-text content: {:?}", content);
            }
        }
    }

    // Test HTTP Info
    let info_request = HttpInfoRequest {
        url: "https://httpbin.org/headers".to_string(),
    };

    let tool_call = ToolCall {
        name: "http_info".to_string(),
        arguments: Some(serde_json::to_value(info_request)?),
    };

    info!("Testing HTTP info");
    let result = client.call_tool(tool_call).await?;

    for content in result.content {
        match content {
            ToolContent::Text { text } => {
                info!("Info response: {}", text);
                let response: HttpInfoResponse = serde_json::from_str(&text)?;
                println!("Info {} - Method: {}", response.url, response.method);
            }
            _ => {
                info!("Received non-text content: {:?}", content);
            }
        }
    }

    info!("Disconnecting from server");
    client.disconnect().await?;

    Ok(())
}
