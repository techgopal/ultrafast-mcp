//! File Operations Server Example
//!
//! This example demonstrates the new UltraFastServer API with file system operations.

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tracing::info;
use ultrafast_mcp::{
    ListToolsRequest, ListToolsResponse, MCPError, MCPResult, ServerCapabilities, ServerInfo, Tool,
    ToolCall, ToolContent, ToolHandler, ToolResult, ToolsCapability, UltraFastServer,
};

#[derive(Debug, Deserialize)]
struct ReadFileRequest {
    path: String,
}

#[derive(Debug, Serialize)]
struct ReadFileResponse {
    content: String,
    size: u64,
    modified: String,
    path: String,
}

#[derive(Debug, Deserialize)]
struct WriteFileRequest {
    path: String,
    content: String,
    append: Option<bool>,
}

#[derive(Debug, Serialize)]
struct WriteFileResponse {
    path: String,
    size: u64,
    written: bool,
}

#[derive(Debug, Deserialize)]
struct ListFilesRequest {
    path: String,
    #[allow(dead_code)]
    recursive: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
struct FileInfo {
    name: String,
    path: String,
    is_dir: bool,
    size: Option<u64>,
    modified: Option<String>,
}

#[derive(Debug, Serialize)]
struct ListFilesResponse {
    files: Vec<FileInfo>,
    total_count: usize,
    path: String,
}

#[derive(Debug, Deserialize)]
struct DeleteFileRequest {
    path: String,
    recursive: Option<bool>,
}

#[derive(Debug, Serialize)]
struct DeleteFileResponse {
    path: String,
    deleted: bool,
    message: String,
}

struct FileOperationsHandler;

#[async_trait::async_trait]
impl ToolHandler for FileOperationsHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult> {
        info!("Received tool call: {}", call.name);

        match call.name.as_str() {
            "read_file" => {
                let request: ReadFileRequest =
                    serde_json::from_value(call.arguments.unwrap_or_default())
                        .map_err(|e| MCPError::serialization_error(e.to_string()))?;

                self.handle_read_file(request).await
            }
            "write_file" => {
                let request: WriteFileRequest =
                    serde_json::from_value(call.arguments.unwrap_or_default())
                        .map_err(|e| ultrafast_mcp::MCPError::serialization_error(e.to_string()))?;

                self.handle_write_file(request).await
            }
            "list_files" => {
                let request: ListFilesRequest =
                    serde_json::from_value(call.arguments.unwrap_or_default())
                        .map_err(|e| ultrafast_mcp::MCPError::serialization_error(e.to_string()))?;

                self.handle_list_files(request).await
            }
            "delete_file" => {
                let request: DeleteFileRequest =
                    serde_json::from_value(call.arguments.unwrap_or_default())
                        .map_err(|e| ultrafast_mcp::MCPError::serialization_error(e.to_string()))?;

                self.handle_delete_file(request).await
            }
            _ => Err(ultrafast_mcp::MCPError::method_not_found(format!(
                "Unknown tool: {}",
                call.name
            ))),
        }
    }

    async fn list_tools(&self, _request: ListToolsRequest) -> MCPResult<ListToolsResponse> {
        Ok(ListToolsResponse {
            tools: vec![
                Tool {
                    name: "read_file".to_string(),
                    description: "Read the contents of a file".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "Path to the file to read"
                            }
                        },
                        "required": ["path"]
                    }),
                    output_schema: None,
                },
                Tool {
                    name: "write_file".to_string(),
                    description: "Write content to a file".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "Path to the file to write"
                            },
                            "content": {
                                "type": "string",
                                "description": "Content to write to the file"
                            },
                            "append": {
                                "type": "boolean",
                                "description": "Whether to append to existing file (default: false)"
                            }
                        },
                        "required": ["path", "content"]
                    }),
                    output_schema: None,
                },
                Tool {
                    name: "list_files".to_string(),
                    description: "List files in a directory".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "Path to the directory to list"
                            },
                            "recursive": {
                                "type": "boolean",
                                "description": "Whether to list files recursively (default: false)"
                            }
                        },
                        "required": ["path"]
                    }),
                    output_schema: None,
                },
                Tool {
                    name: "delete_file".to_string(),
                    description: "Delete a file or directory".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "Path to the file or directory to delete"
                            },
                            "recursive": {
                                "type": "boolean",
                                "description": "Whether to delete directories recursively (default: false)"
                            }
                        },
                        "required": ["path"]
                    }),
                    output_schema: None,
                },
            ],
            next_cursor: None,
        })
    }
}

impl FileOperationsHandler {
    async fn handle_read_file(&self, request: ReadFileRequest) -> MCPResult<ToolResult> {
        let path = Path::new(&request.path);

        if !path.exists() {
            return Err(ultrafast_mcp::MCPError::invalid_request(format!(
                "File not found: {}",
                request.path
            )));
        }

        if !path.is_file() {
            return Err(ultrafast_mcp::MCPError::invalid_request(format!(
                "Path is not a file: {}",
                request.path
            )));
        }

        let content = fs::read_to_string(&path).await.map_err(|e| {
            ultrafast_mcp::MCPError::internal_error(format!("Failed to read file: {}", e))
        })?;

        let metadata = fs::metadata(&path).await.map_err(|e| {
            ultrafast_mcp::MCPError::internal_error(format!("Failed to get file metadata: {}", e))
        })?;

        let response = ReadFileResponse {
            content,
            size: metadata.len(),
            modified: chrono::DateTime::<chrono::Utc>::from(
                metadata.modified().unwrap_or(std::time::SystemTime::now()),
            )
            .to_rfc3339(),
            path: request.path,
        };

        let response_text = serde_json::to_string_pretty(&response)
            .map_err(|e| ultrafast_mcp::MCPError::serialization_error(e.to_string()))?;

        Ok(ToolResult {
            content: vec![ToolContent::text(response_text)],
            is_error: None,
        })
    }

    async fn handle_write_file(&self, request: WriteFileRequest) -> MCPResult<ToolResult> {
        let path = Path::new(&request.path);

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).await.map_err(|e| {
                    ultrafast_mcp::MCPError::internal_error(format!(
                        "Failed to create directory: {}",
                        e
                    ))
                })?;
            }
        }

        let content = if request.append.unwrap_or(false) {
            let existing = if path.exists() {
                fs::read_to_string(&path).await.unwrap_or_default()
            } else {
                String::new()
            };
            existing + &request.content
        } else {
            request.content
        };

        fs::write(&path, &content).await.map_err(|e| {
            ultrafast_mcp::MCPError::internal_error(format!("Failed to write file: {}", e))
        })?;

        let metadata = fs::metadata(&path).await.map_err(|e| {
            ultrafast_mcp::MCPError::internal_error(format!("Failed to get file metadata: {}", e))
        })?;

        let response = WriteFileResponse {
            path: request.path,
            size: metadata.len(),
            written: true,
        };

        let response_text = serde_json::to_string_pretty(&response)
            .map_err(|e| ultrafast_mcp::MCPError::serialization_error(e.to_string()))?;

        Ok(ToolResult {
            content: vec![ToolContent::text(response_text)],
            is_error: None,
        })
    }

    async fn handle_list_files(&self, request: ListFilesRequest) -> MCPResult<ToolResult> {
        let path = Path::new(&request.path);

        if !path.exists() {
            return Err(ultrafast_mcp::MCPError::invalid_request(format!(
                "Directory not found: {}",
                request.path
            )));
        }

        if !path.is_dir() {
            return Err(ultrafast_mcp::MCPError::invalid_request(format!(
                "Path is not a directory: {}",
                request.path
            )));
        }

        let mut files = Vec::new();
        let mut entries = fs::read_dir(&path).await.map_err(|e| {
            ultrafast_mcp::MCPError::internal_error(format!("Failed to read directory: {}", e))
        })?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            ultrafast_mcp::MCPError::internal_error(format!(
                "Failed to read directory entry: {}",
                e
            ))
        })? {
            let metadata = entry.metadata().await.map_err(|e| {
                ultrafast_mcp::MCPError::internal_error(format!(
                    "Failed to get file metadata: {}",
                    e
                ))
            })?;

            let file_info = FileInfo {
                name: entry.file_name().to_string_lossy().to_string(),
                path: entry.path().to_string_lossy().to_string(),
                is_dir: metadata.is_dir(),
                size: if metadata.is_file() {
                    Some(metadata.len())
                } else {
                    None
                },
                modified: chrono::DateTime::<chrono::Utc>::from(
                    metadata.modified().unwrap_or(std::time::SystemTime::now()),
                )
                .to_rfc3339()
                .into(),
            };

            files.push(file_info);
        }

        let response = ListFilesResponse {
            files: files.clone(),
            total_count: files.len(),
            path: request.path,
        };

        let response_text = serde_json::to_string_pretty(&response)
            .map_err(|e| ultrafast_mcp::MCPError::serialization_error(e.to_string()))?;

        Ok(ToolResult {
            content: vec![ToolContent::text(response_text)],
            is_error: None,
        })
    }

    async fn handle_delete_file(&self, request: DeleteFileRequest) -> MCPResult<ToolResult> {
        let path = Path::new(&request.path);

        if !path.exists() {
            return Err(ultrafast_mcp::MCPError::invalid_request(format!(
                "File not found: {}",
                request.path
            )));
        }

        let deleted = if path.is_dir() && request.recursive.unwrap_or(false) {
            fs::remove_dir_all(&path).await.is_ok()
        } else if path.is_file() {
            fs::remove_file(&path).await.is_ok()
        } else if path.is_dir() {
            fs::remove_dir(&path).await.is_ok()
        } else {
            false
        };

        let message = if deleted {
            format!("Successfully deleted: {}", request.path)
        } else {
            format!("Failed to delete: {}", request.path)
        };

        let response = DeleteFileResponse {
            path: request.path,
            deleted,
            message,
        };

        let response_text = serde_json::to_string_pretty(&response)
            .map_err(|e| ultrafast_mcp::MCPError::serialization_error(e.to_string()))?;

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

    info!("Starting File Operations MCP Server");

    // Create server capabilities
    let capabilities = ServerCapabilities {
        tools: Some(ToolsCapability {
            list_changed: Some(true),
        }),
        resources: Some(ultrafast_mcp::ResourcesCapability {
            list_changed: Some(true),
            subscribe: Some(false),
        }),
        ..Default::default()
    };

    // Create server
    let server = UltraFastServer::new(
        ServerInfo {
            name: "file-operations-server".to_string(),
            version: "1.0.0".to_string(),
            description: Some("A file operations server demonstrating UltraFastServer".to_string()),
            authors: None,
            homepage: None,
            license: None,
            repository: None,
        },
        capabilities,
    )
    .with_tool_handler(Arc::new(FileOperationsHandler));

    info!("Server created, starting stdio transport");

    // Run the server
    server.run_stdio().await?;

    Ok(())
}
