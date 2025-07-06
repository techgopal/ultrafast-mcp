//! File Operations Client Example
//!
//! This example demonstrates the new UltraFastClient API by connecting to the file operations server.

use serde::{Deserialize, Serialize};
use tracing::info;
use ultrafast_mcp::{
    ClientCapabilities, ClientInfo, ToolCall, ToolContent, UltraFastClient,
    ListToolsRequest, ListToolsResponse,
};

#[derive(Debug, Serialize, Deserialize)]
struct ReadFileRequest {
    path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReadFileResponse {
    content: String,
    size: u64,
    modified: String,
    path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WriteFileRequest {
    path: String,
    content: String,
    append: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WriteFileResponse {
    path: String,
    size: u64,
    written: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ListFilesRequest {
    path: String,
    recursive: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileInfo {
    name: String,
    path: String,
    is_dir: bool,
    size: Option<u64>,
    modified: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ListFilesResponse {
    files: Vec<FileInfo>,
    total_count: usize,
    path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeleteFileRequest {
    path: String,
    recursive: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeleteFileResponse {
    path: String,
    deleted: bool,
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting File Operations MCP Client");

    // Create client info and capabilities
    let client_info = ClientInfo {
        name: "file-operations-client".to_string(),
        version: "1.0.0".to_string(),
        description: Some("A file operations client demonstrating UltraFastClient".to_string()),
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
    let tools = client.list_tools_default().await?;
    info!("Available tools: {:?}", tools);

    // Test file operations
    let test_dir = "/tmp/mcp_test";
    let test_file = format!("{}/test.txt", test_dir);

    // Create a test file
    let write_request = WriteFileRequest {
        path: test_file.clone(),
        content: "Hello, UltraFast MCP!".to_string(),
        append: Some(false),
    };

    let tool_call = ToolCall {
        name: "write_file".to_string(),
        arguments: Some(serde_json::to_value(write_request)?),
    };

    info!("Creating test file: {}", test_file);
    let result = client.call_tool(tool_call).await?;

    for content in result.content {
        match content {
            ToolContent::Text { text } => {
                info!("Write result: {}", text);
                let response: WriteFileResponse = serde_json::from_str(&text)?;
                println!("File written: {} (size: {})", response.path, response.size);
            }
            _ => {
                info!("Received non-text content: {:?}", content);
            }
        }
    }

    // Read the file back
    let read_request = ReadFileRequest {
        path: test_file.clone(),
    };

    let tool_call = ToolCall {
        name: "read_file".to_string(),
        arguments: Some(serde_json::to_value(read_request)?),
    };

    info!("Reading test file: {}", test_file);
    let result = client.call_tool(tool_call).await?;

    for content in result.content {
        match content {
            ToolContent::Text { text } => {
                info!("Read result: {}", text);
                let response: ReadFileResponse = serde_json::from_str(&text)?;
                println!("File content: {}", response.content);
                println!("File size: {}", response.size);
                println!("Modified: {}", response.modified);
            }
            _ => {
                info!("Received non-text content: {:?}", content);
            }
        }
    }

    // List files in the test directory
    let list_request = ListFilesRequest {
        path: test_dir.to_string(),
        recursive: Some(false),
    };

    let tool_call = ToolCall {
        name: "list_files".to_string(),
        arguments: Some(serde_json::to_value(list_request)?),
    };

    info!("Listing files in: {}", test_dir);
    let result = client.call_tool(tool_call).await?;

    for content in result.content {
        match content {
            ToolContent::Text { text } => {
                info!("List result: {}", text);
                let response: ListFilesResponse = serde_json::from_str(&text)?;
                println!("Found {} files in {}", response.total_count, response.path);
                for file in response.files {
                    println!(
                        "  {} ({})",
                        file.name,
                        if file.is_dir { "dir" } else { "file" }
                    );
                }
            }
            _ => {
                info!("Received non-text content: {:?}", content);
            }
        }
    }

    // Clean up - delete the test file
    let delete_request = DeleteFileRequest {
        path: test_file,
        recursive: Some(false),
    };

    let tool_call = ToolCall {
        name: "delete_file".to_string(),
        arguments: Some(serde_json::to_value(delete_request)?),
    };

    info!("Deleting test file");
    let result = client.call_tool(tool_call).await?;

    for content in result.content {
        match content {
            ToolContent::Text { text } => {
                info!("Delete result: {}", text);
                let response: DeleteFileResponse = serde_json::from_str(&text)?;
                println!("File deleted: {} ({})", response.path, response.message);
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
