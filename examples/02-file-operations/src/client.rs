//! File Operations Client Example
//!
//! This example demonstrates the new UltraFastClient API by connecting to the file operations server.
//! Supports both STDIO and HTTP transports.
//! Aligned with the official MCP filesystem server implementation.

use clap::Parser;
use serde::{Deserialize, Serialize};
use tracing::info;
use ultrafast_mcp::{ClientCapabilities, ClientInfo, ToolCall, ToolContent, UltraFastClient};

#[derive(Parser)]
#[command(name = "file-ops-client")]
#[command(about = "File Operations MCP Client with transport choice")]
struct Args {
    /// Transport type to use
    #[arg(value_enum)]
    transport: TransportType,

    /// Server URL for HTTP transport (default: http://127.0.0.1:8080)
    #[arg(long, default_value = "http://127.0.0.1:8080")]
    server_url: String,
}

#[derive(Clone, Copy, Debug, PartialEq, clap::ValueEnum)]
enum TransportType {
    /// Use STDIO transport (subprocess mode)
    Stdio,
    /// Use HTTP transport (network mode)
    Http,
}

// Request/Response types aligned with official MCP filesystem server

#[derive(Debug, Serialize, Deserialize)]
struct ReadFileRequest {
    path: String,
    #[serde(default)]
    head: Option<u32>,
    #[serde(default)]
    tail: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReadFileResponse {
    content: String,
    path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReadMultipleFilesRequest {
    paths: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReadMultipleFilesResponse {
    results: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WriteFileRequest {
    path: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WriteFileResponse {
    path: String,
    success: bool,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct EditFileRequest {
    path: String,
    edits: Vec<EditOperation>,
    #[serde(default)]
    dry_run: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct EditOperation {
    old_text: String,
    new_text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct EditFileResponse {
    path: String,
    success: bool,
    diff: Option<String>,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateDirectoryRequest {
    path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateDirectoryResponse {
    path: String,
    success: bool,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ListDirectoryRequest {
    path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ListDirectoryResponse {
    path: String,
    entries: Vec<DirectoryEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DirectoryEntry {
    name: String,
    path: String,
    #[serde(rename = "type")]
    entry_type: String,
    size: Option<u64>,
    modified: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ListDirectoryWithSizesRequest {
    path: String,
    #[serde(default = "default_sort_by")]
    sort_by: String,
}

fn default_sort_by() -> String {
    "name".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
struct ListDirectoryWithSizesResponse {
    path: String,
    entries: Vec<DirectoryEntry>,
    sort_by: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DirectoryTreeRequest {
    path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DirectoryTreeResponse {
    path: String,
    tree: DirectoryTreeNode,
}

#[derive(Debug, Serialize, Deserialize)]
struct DirectoryTreeNode {
    name: String,
    #[serde(rename = "type")]
    node_type: String,
    children: Option<Vec<DirectoryTreeNode>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MoveFileRequest {
    source: String,
    destination: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MoveFileResponse {
    source: String,
    destination: String,
    success: bool,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SearchFilesRequest {
    path: String,
    pattern: String,
    #[serde(default)]
    exclude_patterns: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SearchFilesResponse {
    path: String,
    pattern: String,
    results: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GetFileInfoRequest {
    path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GetFileInfoResponse {
    path: String,
    size: u64,
    created: String,
    modified: String,
    accessed: String,
    is_directory: bool,
    is_file: bool,
    permissions: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ListAllowedDirectoriesResponse {
    directories: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info,ultrafast_mcp=debug")
        .init();

    info!("🚀 Starting File Operations MCP Client");
    info!("📡 Transport: {:?}", args.transport);

    // Create client info and capabilities
    let client_info = ClientInfo {
        name: "file-operations-client".to_string(),
        version: "1.0.0".to_string(),
        description: Some(format!(
            "A file operations client demonstrating UltraFastClient with {:?} transport",
            args.transport
        )),
        authors: Some(vec!["ULTRAFAST_MCP Team".to_string()]),
        homepage: Some("https://github.com/ultrafast-mcp/ultrafast-mcp".to_string()),
        license: Some("MIT OR Apache-2.0".to_string()),
        repository: Some("https://github.com/ultrafast-mcp/ultrafast-mcp".to_string()),
    };

    let client_capabilities = ClientCapabilities {
        ..Default::default()
    };

    // Create client
    let client = UltraFastClient::new(client_info, client_capabilities);

    // Connect to server based on transport type
    match args.transport {
        TransportType::Stdio => {
            info!("🔌 Connecting to server via STDIO");
            client.connect_stdio().await?;
        }
        TransportType::Http => {
            info!("🌐 Connecting to server via HTTP: {}", args.server_url);
            client.connect_streamable_http(&args.server_url).await?;
        }
    }

    info!("Connected! Listing available tools");

    // List available tools
    let tools = client.list_tools_default().await?;
    info!("Available tools: {:?}", tools);

    // Test file operations
    let test_dir = "/tmp/mcp_test";
    let test_file = format!("{test_dir}/test.txt");

    // First, check allowed directories
    info!("Checking allowed directories");
    let allowed_dirs_call = ToolCall {
        name: "list_allowed_directories".to_string(),
        arguments: Some(serde_json::json!({})),
    };

    let result = client.call_tool(allowed_dirs_call).await?;
    for content in result.content {
        match content {
            ToolContent::Text { text } => {
                info!("Allowed directories: {}", text);
                let response: ListAllowedDirectoriesResponse = serde_json::from_str(&text)?;
                println!("Allowed directories: {:?}", response.directories);
            }
            _ => {
                info!("Received non-text content: {:?}", content);
            }
        }
    }

    // Create a test directory
    let create_dir_request = CreateDirectoryRequest {
        path: test_dir.to_string(),
    };

    let tool_call = ToolCall {
        name: "create_directory".to_string(),
        arguments: Some(serde_json::to_value(create_dir_request)?),
    };

    info!("Creating test directory: {}", test_dir);
    let result = client.call_tool(tool_call).await?;

    for content in result.content {
        match content {
            ToolContent::Text { text } => {
                info!("Create directory result: {}", text);
                let response: CreateDirectoryResponse = serde_json::from_str(&text)?;
                println!(
                    "Directory created: {} ({})",
                    response.path, response.message
                );
            }
            _ => {
                info!("Received non-text content: {:?}", content);
            }
        }
    }

    // Create a test file
    let write_request = WriteFileRequest {
        path: test_file.clone(),
        content: "Hello, UltraFast MCP!\nThis is a test file.\n".to_string(),
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
                println!("File written: {} ({})", response.path, response.message);
            }
            _ => {
                info!("Received non-text content: {:?}", content);
            }
        }
    }

    // Read the file back
    let read_request = ReadFileRequest {
        path: test_file.clone(),
        head: None,
        tail: None,
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
            }
            _ => {
                info!("Received non-text content: {:?}", content);
            }
        }
    }

    // Read just the first line
    let read_head_request = ReadFileRequest {
        path: test_file.clone(),
        head: Some(1),
        tail: None,
    };

    let tool_call = ToolCall {
        name: "read_file".to_string(),
        arguments: Some(serde_json::to_value(read_head_request)?),
    };

    info!("Reading first line of test file: {}", test_file);
    let result = client.call_tool(tool_call).await?;

    for content in result.content {
        match content {
            ToolContent::Text { text } => {
                info!("Read head result: {}", text);
                let response: ReadFileResponse = serde_json::from_str(&text)?;
                println!("First line: {}", response.content);
            }
            _ => {
                info!("Received non-text content: {:?}", content);
            }
        }
    }

    // List files in the test directory
    let list_request = ListDirectoryRequest {
        path: test_dir.to_string(),
    };

    let tool_call = ToolCall {
        name: "list_directory".to_string(),
        arguments: Some(serde_json::to_value(list_request)?),
    };

    info!("Listing files in: {}", test_dir);
    let result = client.call_tool(tool_call).await?;

    for content in result.content {
        match content {
            ToolContent::Text { text } => {
                info!("List result: {}", text);
                let response: ListDirectoryResponse = serde_json::from_str(&text)?;
                println!(
                    "Found {} entries in {}",
                    response.entries.len(),
                    response.path
                );
                for entry in response.entries {
                    println!("  {} ({})", entry.name, entry.entry_type);
                }
            }
            _ => {
                info!("Received non-text content: {:?}", content);
            }
        }
    }

    // List directory with sizes
    let list_sizes_request = ListDirectoryWithSizesRequest {
        path: test_dir.to_string(),
        sort_by: "size".to_string(),
    };

    let tool_call = ToolCall {
        name: "list_directory_with_sizes".to_string(),
        arguments: Some(serde_json::to_value(list_sizes_request)?),
    };

    info!("Listing files with sizes in: {}", test_dir);
    let result = client.call_tool(tool_call).await?;

    for content in result.content {
        match content {
            ToolContent::Text { text } => {
                info!("List with sizes result: {}", text);
                let response: ListDirectoryWithSizesResponse = serde_json::from_str(&text)?;
                println!(
                    "Found {} entries in {} (sorted by {})",
                    response.entries.len(),
                    response.path,
                    response.sort_by
                );
                for entry in response.entries {
                    let size_str = entry
                        .size
                        .map(|s| format!("{s} bytes"))
                        .unwrap_or_else(|| "N/A".to_string());
                    println!("  {} ({}) - {}", entry.name, entry.entry_type, size_str);
                }
            }
            _ => {
                info!("Received non-text content: {:?}", content);
            }
        }
    }

    // Get file info
    let file_info_request = GetFileInfoRequest {
        path: test_file.clone(),
    };

    let tool_call = ToolCall {
        name: "get_file_info".to_string(),
        arguments: Some(serde_json::to_value(file_info_request)?),
    };

    info!("Getting file info for: {}", test_file);
    let result = client.call_tool(tool_call).await?;

    for content in result.content {
        match content {
            ToolContent::Text { text } => {
                info!("File info result: {}", text);
                let response: GetFileInfoResponse = serde_json::from_str(&text)?;
                println!("File info for {}:", response.path);
                println!("  Size: {} bytes", response.size);
                println!(
                    "  Type: {}",
                    if response.is_file {
                        "file"
                    } else {
                        "directory"
                    }
                );
                println!("  Created: {}", response.created);
                println!("  Modified: {}", response.modified);
                println!("  Permissions: {}", response.permissions);
            }
            _ => {
                info!("Received non-text content: {:?}", content);
            }
        }
    }

    // Search for files
    let search_request = SearchFilesRequest {
        path: test_dir.to_string(),
        pattern: "test".to_string(),
        exclude_patterns: vec![],
    };

    let tool_call = ToolCall {
        name: "search_files".to_string(),
        arguments: Some(serde_json::to_value(search_request)?),
    };

    info!("Searching for files with pattern 'test' in: {}", test_dir);
    let result = client.call_tool(tool_call).await?;

    for content in result.content {
        match content {
            ToolContent::Text { text } => {
                info!("Search result: {}", text);
                let response: SearchFilesResponse = serde_json::from_str(&text)?;
                println!(
                    "Found {} files matching '{}' in {}",
                    response.results.len(),
                    response.pattern,
                    response.path
                );
                for result_path in response.results {
                    println!("  {result_path}");
                }
            }
            _ => {
                info!("Received non-text content: {:?}", content);
            }
        }
    }

    // Edit the file
    let edit_request = EditFileRequest {
        path: test_file.clone(),
        edits: vec![EditOperation {
            old_text: "Hello, UltraFast MCP!".to_string(),
            new_text: "Hello, Updated UltraFast MCP!".to_string(),
        }],
        dry_run: false,
    };

    let tool_call = ToolCall {
        name: "edit_file".to_string(),
        arguments: Some(serde_json::to_value(edit_request)?),
    };

    info!("Editing test file: {}", test_file);
    let result = client.call_tool(tool_call).await?;

    for content in result.content {
        match content {
            ToolContent::Text { text } => {
                info!("Edit result: {}", text);
                let response: EditFileResponse = serde_json::from_str(&text)?;
                println!("File edited: {} ({})", response.path, response.message);
                if let Some(diff) = response.diff {
                    println!("Diff: {diff}");
                }
            }
            _ => {
                info!("Received non-text content: {:?}", content);
            }
        }
    }

    // Read the edited file
    let read_request = ReadFileRequest {
        path: test_file.clone(),
        head: None,
        tail: None,
    };

    let tool_call = ToolCall {
        name: "read_file".to_string(),
        arguments: Some(serde_json::to_value(read_request)?),
    };

    info!("Reading edited test file: {}", test_file);
    let result = client.call_tool(tool_call).await?;

    for content in result.content {
        match content {
            ToolContent::Text { text } => {
                info!("Read edited result: {}", text);
                let response: ReadFileResponse = serde_json::from_str(&text)?;
                println!("Edited file content: {}", response.content);
            }
            _ => {
                info!("Received non-text content: {:?}", content);
            }
        }
    }

    // Move the file
    let moved_file = format!("{test_dir}/moved_test.txt");
    let move_request = MoveFileRequest {
        source: test_file.clone(),
        destination: moved_file.clone(),
    };

    let tool_call = ToolCall {
        name: "move_file".to_string(),
        arguments: Some(serde_json::to_value(move_request)?),
    };

    info!("Moving test file from {} to {}", test_file, moved_file);
    let result = client.call_tool(tool_call).await?;

    for content in result.content {
        match content {
            ToolContent::Text { text } => {
                info!("Move result: {}", text);
                let response: MoveFileResponse = serde_json::from_str(&text)?;
                println!(
                    "File moved: {} ({})",
                    response.destination, response.message
                );
            }
            _ => {
                info!("Received non-text content: {:?}", content);
            }
        }
    }

    // List files again to see the moved file
    let list_request = ListDirectoryRequest {
        path: test_dir.to_string(),
    };

    let tool_call = ToolCall {
        name: "list_directory".to_string(),
        arguments: Some(serde_json::to_value(list_request)?),
    };

    info!("Listing files after move in: {}", test_dir);
    let result = client.call_tool(tool_call).await?;

    for content in result.content {
        match content {
            ToolContent::Text { text } => {
                info!("List after move result: {}", text);
                let response: ListDirectoryResponse = serde_json::from_str(&text)?;
                println!(
                    "Found {} entries in {} after move",
                    response.entries.len(),
                    response.path
                );
                for entry in response.entries {
                    println!("  {} ({})", entry.name, entry.entry_type);
                }
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
