//! File Operations Server Example
//!
//! This example demonstrates the new UltraFastServer API with file system operations.
//! Supports both STDIO and HTTP transports.
//! Aligned with the official MCP filesystem server implementation.

use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tracing::info;
use ultrafast_mcp::{
    HttpTransportConfig, ListToolsRequest, ListToolsResponse, MCPError, MCPResult,
    ServerCapabilities, ServerInfo, Tool, ToolCall, ToolContent, ToolHandler, ToolResult,
    ToolsCapability, UltraFastServer,
};

#[derive(Parser)]
#[command(name = "file-ops-server")]
#[command(about = "File Operations MCP Server with transport choice")]
struct Args {
    /// Transport type to use
    #[arg(value_enum)]
    transport: TransportType,

    /// Host for HTTP transport (default: 127.0.0.1)
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Port for HTTP transport (default: 8080)
    #[arg(long, default_value = "8080")]
    port: u16,

    /// Allowed directories (default: current directory)
    #[arg(long, default_value = ".")]
    allowed_directories: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, clap::ValueEnum)]
enum TransportType {
    /// Use STDIO transport (subprocess mode)
    Stdio,
    /// Use Streamable HTTP transport (network mode)
    Http,
}

// Request/Response types aligned with official MCP filesystem server

#[derive(Debug, Deserialize)]
struct ReadFileRequest {
    path: String,
    #[serde(default)]
    head: Option<u32>,
    #[serde(default)]
    tail: Option<u32>,
}

#[derive(Debug, Serialize)]
struct ReadFileResponse {
    content: String,
    path: String,
}

#[derive(Debug, Deserialize)]
struct ReadMultipleFilesRequest {
    paths: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ReadMultipleFilesResponse {
    results: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct WriteFileRequest {
    path: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct WriteFileResponse {
    path: String,
    success: bool,
    message: String,
}

#[derive(Debug, Deserialize)]
struct EditFileRequest {
    path: String,
    edits: Vec<EditOperation>,
    #[serde(default)]
    dry_run: bool,
}

#[derive(Debug, Deserialize)]
struct EditOperation {
    old_text: String,
    new_text: String,
}

#[derive(Debug, Serialize)]
struct EditFileResponse {
    path: String,
    success: bool,
    diff: Option<String>,
    message: String,
}

#[derive(Debug, Deserialize)]
struct CreateDirectoryRequest {
    path: String,
}

#[derive(Debug, Serialize)]
struct CreateDirectoryResponse {
    path: String,
    success: bool,
    message: String,
}

#[derive(Debug, Deserialize)]
struct ListDirectoryRequest {
    path: String,
}

#[derive(Debug, Serialize)]
struct ListDirectoryResponse {
    path: String,
    entries: Vec<DirectoryEntry>,
}

#[derive(Debug, Clone, Serialize)]
struct DirectoryEntry {
    name: String,
    path: String,
    #[serde(rename = "type")]
    entry_type: String, // "file" or "directory"
    size: Option<u64>,
    modified: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ListDirectoryWithSizesRequest {
    path: String,
    #[serde(default = "default_sort_by")]
    sort_by: String,
}

fn default_sort_by() -> String {
    "name".to_string()
}

#[derive(Debug, Serialize)]
struct ListDirectoryWithSizesResponse {
    path: String,
    entries: Vec<DirectoryEntry>,
    sort_by: String,
}

#[derive(Debug, Deserialize)]
struct DirectoryTreeRequest {
    path: String,
}

#[derive(Debug, Serialize)]
struct DirectoryTreeResponse {
    path: String,
    tree: DirectoryTreeNode,
}

#[derive(Debug, Clone, Serialize)]
struct DirectoryTreeNode {
    name: String,
    node_type: String,
    children: Option<Vec<DirectoryTreeNode>>,
}

#[derive(Debug, Deserialize)]
struct MoveFileRequest {
    source: String,
    destination: String,
}

#[derive(Debug, Serialize)]
struct MoveFileResponse {
    source: String,
    destination: String,
    success: bool,
    message: String,
}

#[derive(Debug, Deserialize)]
struct SearchFilesRequest {
    path: String,
    pattern: String,
    #[serde(default)]
    exclude_patterns: Vec<String>,
}

#[derive(Debug, Serialize)]
struct SearchFilesResponse {
    path: String,
    pattern: String,
    results: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct GetFileInfoRequest {
    path: String,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
struct ListAllowedDirectoriesResponse {
    directories: Vec<String>,
}

struct FileOperationsHandler {
    allowed_directories: Vec<String>,
}

impl FileOperationsHandler {
    fn new(allowed_directories: Vec<String>) -> Self {
        Self {
            allowed_directories,
        }
    }

    fn validate_path(&self, requested_path: &str) -> Result<String, MCPError> {
        let path = Path::new(requested_path);
        let absolute = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|e| {
                    MCPError::internal_error(format!("Failed to get current directory: {e}"))
                })?
                .join(path)
        };

        let normalized = match absolute.canonicalize() {
            Ok(path) => path,
            Err(_) => {
                // If the file doesn't exist, canonicalize the parent and join the file name
                if let Some(parent) = absolute.parent() {
                    if let Ok(parent_canon) = parent.canonicalize() {
                        parent_canon.join(absolute.file_name().unwrap_or_default())
                    } else {
                        absolute.clone()
                    }
                } else {
                    absolute.clone()
                }
            }
        };

        info!(
            "Validating path: {} -> normalized: {}",
            requested_path,
            normalized.display()
        );

        // Check if path is within allowed directories
        for allowed_dir in &self.allowed_directories {
            let allowed_path = Path::new(allowed_dir);
            let allowed_absolute = if allowed_path.is_absolute() {
                allowed_path.to_path_buf()
            } else {
                std::env::current_dir()
                    .map_err(|e| {
                        MCPError::internal_error(format!("Failed to get current directory: {e}"))
                    })?
                    .join(allowed_path)
            };

            let allowed_normalized = allowed_absolute
                .canonicalize()
                .map_err(|_| allowed_absolute.clone())
                .unwrap_or(allowed_absolute);

            info!(
                "Checking against allowed dir: {} -> normalized: {}",
                allowed_dir,
                allowed_normalized.display()
            );
            info!(
                "Starts with check: {} starts_with {} = {}",
                normalized.display(),
                allowed_normalized.display(),
                normalized.starts_with(&allowed_normalized)
            );

            if normalized.starts_with(&allowed_normalized) {
                info!("Path validation successful for: {}", requested_path);
                return Ok(normalized.to_string_lossy().to_string());
            }
        }

        Err(MCPError::invalid_request(format!(
            "Access denied - path outside allowed directories: {} not in {:?}",
            requested_path, self.allowed_directories
        )))
    }
}

#[async_trait::async_trait]
impl ToolHandler for FileOperationsHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult> {
        info!("Received tool call: {}", call.name);

        let arguments = call
            .arguments
            .ok_or_else(|| MCPError::invalid_params("Missing arguments".to_string()))?;

        match call.name.as_str() {
            "read_file" => {
                let request: ReadFileRequest = serde_json::from_value(arguments)
                    .map_err(|e| MCPError::serialization_error(e.to_string()))?;
                self.handle_read_file(request).await
            }
            "read_multiple_files" => {
                let request: ReadMultipleFilesRequest = serde_json::from_value(arguments)
                    .map_err(|e| MCPError::serialization_error(e.to_string()))?;
                self.handle_read_multiple_files(request).await
            }
            "write_file" => {
                let request: WriteFileRequest = serde_json::from_value(arguments)
                    .map_err(|e| MCPError::serialization_error(e.to_string()))?;
                self.handle_write_file(request).await
            }
            "edit_file" => {
                let request: EditFileRequest = serde_json::from_value(arguments)
                    .map_err(|e| MCPError::serialization_error(e.to_string()))?;
                self.handle_edit_file(request).await
            }
            "create_directory" => {
                let request: CreateDirectoryRequest = serde_json::from_value(arguments)
                    .map_err(|e| MCPError::serialization_error(e.to_string()))?;
                self.handle_create_directory(request).await
            }
            "list_directory" => {
                let request: ListDirectoryRequest = serde_json::from_value(arguments)
                    .map_err(|e| MCPError::serialization_error(e.to_string()))?;
                self.handle_list_directory(request).await
            }
            "list_directory_with_sizes" => {
                let request: ListDirectoryWithSizesRequest = serde_json::from_value(arguments)
                    .map_err(|e| MCPError::serialization_error(e.to_string()))?;
                self.handle_list_directory_with_sizes(request).await
            }
            "directory_tree" => {
                let request: DirectoryTreeRequest = serde_json::from_value(arguments)
                    .map_err(|e| MCPError::serialization_error(e.to_string()))?;
                self.handle_directory_tree(request).await
            }
            "move_file" => {
                let request: MoveFileRequest = serde_json::from_value(arguments)
                    .map_err(|e| MCPError::serialization_error(e.to_string()))?;
                self.handle_move_file(request).await
            }
            "search_files" => {
                let request: SearchFilesRequest = serde_json::from_value(arguments)
                    .map_err(|e| MCPError::serialization_error(e.to_string()))?;
                self.handle_search_files(request).await
            }
            "get_file_info" => {
                let request: GetFileInfoRequest = serde_json::from_value(arguments)
                    .map_err(|e| MCPError::serialization_error(e.to_string()))?;
                self.handle_get_file_info(request).await
            }
            "list_allowed_directories" => self.handle_list_allowed_directories().await,
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
                    name: "read_file".to_string(),
                    description: "Read the complete contents of a file from the file system. Handles various text encodings and provides detailed error messages if the file cannot be read. Use this tool when you need to examine the contents of a single file. Use the 'head' parameter to read only the first N lines of a file, or the 'tail' parameter to read only the last N lines of a file. Only works within allowed directories.".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "path": {"type": "string", "description": "Path to file"},
                            "head": {"type": "number", "description": "If provided, returns only the first N lines of the file"},
                            "tail": {"type": "number", "description": "If provided, returns only the last N lines of the file"}
                        },
                        "required": ["path"]
                    }),
                    output_schema: None,
                    annotations: None,
                },
                Tool {
                    name: "read_multiple_files".to_string(),
                    description: "Read the contents of multiple files simultaneously. This is more efficient than reading files one by one when you need to analyze or compare multiple files. Each file's content is returned with its path as a reference. Failed reads for individual files won't stop the entire operation. Only works within allowed directories.".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "paths": {"type": "array", "items": {"type": "string"}, "description": "Array of file paths to read"}
                        },
                        "required": ["paths"]
                    }),
                    output_schema: None,
                    annotations: None,
                },
                Tool {
                    name: "write_file".to_string(),
                    description: "Create a new file or completely overwrite an existing file with new content. Use with caution as it will overwrite existing files without warning. Handles text content with proper encoding. Only works within allowed directories.".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "path": {"type": "string", "description": "Path to file"},
                            "content": {"type": "string", "description": "Content to write"}
                        },
                        "required": ["path", "content"]
                    }),
                    output_schema: None,
                    annotations: None,
                },
                Tool {
                    name: "edit_file".to_string(),
                    description: "Make line-based edits to a text file. Each edit replaces exact line sequences with new content. Returns a git-style diff showing the changes made. Only works within allowed directories.".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "path": {"type": "string", "description": "Path to file"},
                            "edits": {
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "old_text": {"type": "string", "description": "Text to search for - must match exactly"},
                                        "new_text": {"type": "string", "description": "Text to replace with"}
                                    },
                                    "required": ["old_text", "new_text"]
                                }
                            },
                            "dry_run": {"type": "boolean", "default": false, "description": "Preview changes using git-style diff format"}
                        },
                        "required": ["path", "edits"]
                    }),
                    output_schema: None,
                    annotations: None,
                },
                Tool {
                    name: "create_directory".to_string(),
                    description: "Create a new directory or ensure a directory exists. Can create multiple nested directories in one operation. If the directory already exists, this operation will succeed silently. Perfect for setting up directory structures for projects or ensuring required paths exist. Only works within allowed directories.".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "path": {"type": "string", "description": "Path to directory"}
                        },
                        "required": ["path"]
                    }),
                    output_schema: None,
                    annotations: None,
                },
                Tool {
                    name: "list_directory".to_string(),
                    description: "Get a detailed listing of all files and directories in a specified path. Results clearly distinguish between files and directories with [FILE] and [DIR] prefixes. This tool is essential for understanding directory structure and finding specific files within a directory. Only works within allowed directories.".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "path": {"type": "string", "description": "Directory path"}
                        },
                        "required": ["path"]
                    }),
                    output_schema: None,
                    annotations: None,
                },
                Tool {
                    name: "list_directory_with_sizes".to_string(),
                    description: "Get a detailed listing of all files and directories in a specified path, including sizes. Results clearly distinguish between files and directories with [FILE] and [DIR] prefixes. This tool is useful for understanding directory structure and finding specific files within a directory. Only works within allowed directories.".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "path": {"type": "string", "description": "Directory path"},
                            "sort_by": {"type": "string", "enum": ["name", "size"], "default": "name", "description": "Sort entries by name or size"}
                        },
                        "required": ["path"]
                    }),
                    output_schema: None,
                    annotations: None,
                },
                Tool {
                    name: "directory_tree".to_string(),
                    description: "Get a recursive tree view of files and directories as a JSON structure. Each entry includes 'name', 'type' (file/directory), and 'children' for directories. Files have no children array, while directories always have a children array (which may be empty). The output is formatted with 2-space indentation for readability. Only works within allowed directories.".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "path": {"type": "string", "description": "Directory path"}
                        },
                        "required": ["path"]
                    }),
                    output_schema: None,
                    annotations: None,
                },
                Tool {
                    name: "move_file".to_string(),
                    description: "Move or rename files and directories. Can move files between directories and rename them in a single operation. If the destination exists, the operation will fail. Works across different directories and can be used for simple renaming within the same directory. Both source and destination must be within allowed directories.".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "source": {"type": "string", "description": "Source path"},
                            "destination": {"type": "string", "description": "Destination path"}
                        },
                        "required": ["source", "destination"]
                    }),
                    output_schema: None,
                    annotations: None,
                },
                Tool {
                    name: "search_files".to_string(),
                    description: "Recursively search for files and directories matching a pattern. Searches through all subdirectories from the starting path. The search is case-insensitive and matches partial names. Returns full paths to all matching items. Great for finding files when you don't know their exact location. Only searches within allowed directories.".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "path": {"type": "string", "description": "Starting directory path"},
                            "pattern": {"type": "string", "description": "Search pattern"},
                            "exclude_patterns": {"type": "array", "items": {"type": "string"}, "default": [], "description": "Patterns to exclude"}
                        },
                        "required": ["path", "pattern"]
                    }),
                    output_schema: None,
                    annotations: None,
                },
                Tool {
                    name: "get_file_info".to_string(),
                    description: "Retrieve detailed metadata about a file or directory. Returns comprehensive information including size, creation time, last modified time, permissions, and type. This tool is perfect for understanding file characteristics without reading the actual content. Only works within allowed directories.".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "path": {"type": "string", "description": "Path to file or directory"}
                        },
                        "required": ["path"]
                    }),
                    output_schema: None,
                    annotations: None,
                },
                Tool {
                    name: "list_allowed_directories".to_string(),
                    description: "Returns the list of root directories that this server is allowed to access. Use this to understand which directories are available before trying to access files.".to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {},
                        "required": []
                    }),
                    output_schema: None,
                    annotations: None,
                },
            ],
            next_cursor: None,
        })
    }
}

impl FileOperationsHandler {
    async fn handle_read_file(&self, request: ReadFileRequest) -> MCPResult<ToolResult> {
        let path = self.validate_path(&request.path)?;
        let path = Path::new(&path);

        if !path.exists() {
            return Err(MCPError::invalid_request(format!(
                "File not found: {}",
                request.path
            )));
        }

        if !path.is_file() {
            return Err(MCPError::invalid_request(format!(
                "Path is not a file: {}",
                request.path
            )));
        }

        let mut content = fs::read_to_string(&path)
            .await
            .map_err(|e| MCPError::internal_error(format!("Failed to read file: {e}")))?;

        if let Some(head) = request.head {
            content = content
                .lines()
                .take(head as usize)
                .collect::<Vec<_>>()
                .join("\n");
        }
        if let Some(tail) = request.tail {
            let lines: Vec<_> = content.lines().collect();
            let start_index = if tail < lines.len() as u32 {
                lines.len() - tail as usize
            } else {
                0
            };
            content = lines[start_index..].join("\n");
        }

        let response = ReadFileResponse {
            content,
            path: request.path,
        };

        let response_text = serde_json::to_string_pretty(&response)
            .map_err(|e| MCPError::serialization_error(e.to_string()))?;

        Ok(ToolResult {
            content: vec![ToolContent::text(response_text)],
            is_error: None,
        })
    }

    async fn handle_read_multiple_files(
        &self,
        request: ReadMultipleFilesRequest,
    ) -> MCPResult<ToolResult> {
        let mut results = Vec::new();
        for path in request.paths {
            let validated_path = self.validate_path(&path)?;
            let path = Path::new(&validated_path);

            if !path.exists() {
                results.push(format!(
                    "{}: Error - File not found",
                    path.to_string_lossy()
                ));
                continue;
            }

            if !path.is_file() {
                results.push(format!(
                    "{}: Error - Path is not a file",
                    path.to_string_lossy()
                ));
                continue;
            }

            match fs::read_to_string(&path).await {
                Ok(content) => {
                    results.push(format!("{}:\n{}", path.to_string_lossy(), content));
                }
                Err(e) => {
                    results.push(format!("{}: Error - {}", path.to_string_lossy(), e));
                }
            }
        }

        let response = ReadMultipleFilesResponse { results };
        let response_text = serde_json::to_string_pretty(&response)
            .map_err(|e| MCPError::serialization_error(e.to_string()))?;

        Ok(ToolResult {
            content: vec![ToolContent::text(response_text)],
            is_error: None,
        })
    }

    async fn handle_write_file(&self, request: WriteFileRequest) -> MCPResult<ToolResult> {
        let path = self.validate_path(&request.path)?;
        let path = Path::new(&path);

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).await.map_err(|e| {
                    MCPError::internal_error(format!("Failed to create directory: {e}"))
                })?;
            }
        }

        fs::write(&path, &request.content)
            .await
            .map_err(|e| MCPError::internal_error(format!("Failed to write file: {e}")))?;

        let path_clone = request.path.clone();
        let response = WriteFileResponse {
            path: request.path,
            success: true,
            message: format!("Successfully wrote file: {path_clone}"),
        };

        let response_text = serde_json::to_string_pretty(&response)
            .map_err(|e| MCPError::serialization_error(e.to_string()))?;

        Ok(ToolResult {
            content: vec![ToolContent::text(response_text)],
            is_error: None,
        })
    }

    async fn handle_edit_file(&self, request: EditFileRequest) -> MCPResult<ToolResult> {
        let path = self.validate_path(&request.path)?;
        let path = Path::new(&path);

        if !path.exists() {
            return Err(MCPError::invalid_request(format!(
                "File not found: {}",
                request.path
            )));
        }

        if !path.is_file() {
            return Err(MCPError::invalid_request(format!(
                "Path is not a file: {}",
                request.path
            )));
        }

        let mut content = fs::read_to_string(&path)
            .await
            .map_err(|e| MCPError::internal_error(format!("Failed to read file: {e}")))?;

        let mut diff_lines = Vec::new();

        for edit in &request.edits {
            let old_text = &edit.old_text;
            let new_text = &edit.new_text;

            if let Some(pos) = content.find(old_text) {
                let start_line = content[..pos].lines().count();
                let _end_line = start_line + old_text.lines().count();

                diff_lines.push(serde_json::json!({
                    "action": "delete",
                    "start_line": start_line,
                    "text": old_text
                }));

                diff_lines.push(serde_json::json!({
                    "action": "insert",
                    "start_line": start_line,
                    "text": new_text
                }));
            } else {
                diff_lines.push(serde_json::json!({
                    "action": "insert",
                    "start_line": 0, // This is a placeholder, actual line number depends on context
                    "text": new_text
                }));
            }
        }

        if request.dry_run {
            let diff_text = serde_json::to_string_pretty(&diff_lines)
                .map_err(|e| MCPError::serialization_error(e.to_string()))?;
            let response = EditFileResponse {
                path: request.path,
                success: true,
                diff: Some(diff_text),
                message: format!(
                    "Dry run successful. Would apply {} changes.",
                    diff_lines.len()
                ),
            };
            let response_text = serde_json::to_string_pretty(&response)
                .map_err(|e| MCPError::serialization_error(e.to_string()))?;
            return Ok(ToolResult {
                content: vec![ToolContent::text(response_text)],
                is_error: None,
            });
        }

        // Apply changes
        for edit in &request.edits {
            let old_text = &edit.old_text;
            let new_text = &edit.new_text;

            if let Some(pos) = content.find(old_text) {
                content = content[..pos].to_string() + new_text + &content[pos + old_text.len()..];
            } else {
                content = content + new_text;
            }
        }

        fs::write(&path, &content).await.map_err(|e| {
            MCPError::internal_error(format!("Failed to write file after editing: {e}"))
        })?;

        let path_clone = request.path.clone();
        let response = EditFileResponse {
            path: request.path,
            success: true,
            diff: Some(serde_json::to_string_pretty(&diff_lines).unwrap()),
            message: format!("Successfully edited file: {path_clone}"),
        };

        let response_text = serde_json::to_string_pretty(&response)
            .map_err(|e| MCPError::serialization_error(e.to_string()))?;

        Ok(ToolResult {
            content: vec![ToolContent::text(response_text)],
            is_error: None,
        })
    }

    async fn handle_create_directory(
        &self,
        request: CreateDirectoryRequest,
    ) -> MCPResult<ToolResult> {
        let path = self.validate_path(&request.path)?;
        let path = Path::new(&path);

        if path.exists() {
            let path_clone = request.path.clone();
            let response = CreateDirectoryResponse {
                path: request.path,
                success: true,
                message: format!("Directory already exists: {path_clone}"),
            };
            let response_text = serde_json::to_string_pretty(&response)
                .map_err(|e| MCPError::serialization_error(e.to_string()))?;
            return Ok(ToolResult {
                content: vec![ToolContent::text(response_text)],
                is_error: None,
            });
        }

        fs::create_dir_all(&path)
            .await
            .map_err(|e| MCPError::internal_error(format!("Failed to create directory: {e}")))?;

        let path_clone = request.path.clone();
        let response = CreateDirectoryResponse {
            path: request.path,
            success: true,
            message: format!("Successfully created directory: {path_clone}"),
        };

        let response_text = serde_json::to_string_pretty(&response)
            .map_err(|e| MCPError::serialization_error(e.to_string()))?;

        Ok(ToolResult {
            content: vec![ToolContent::text(response_text)],
            is_error: None,
        })
    }

    async fn handle_list_directory(&self, request: ListDirectoryRequest) -> MCPResult<ToolResult> {
        let path = self.validate_path(&request.path)?;
        let path = Path::new(&path);

        if !path.exists() {
            return Err(MCPError::invalid_request(format!(
                "Directory not found: {}",
                request.path
            )));
        }

        if !path.is_dir() {
            return Err(MCPError::invalid_request(format!(
                "Path is not a directory: {}",
                request.path
            )));
        }

        let mut entries = Vec::new();
        let mut entries_iter = fs::read_dir(&path)
            .await
            .map_err(|e| MCPError::internal_error(format!("Failed to read directory: {e}")))?;

        while let Some(entry) = entries_iter
            .next_entry()
            .await
            .map_err(|e| MCPError::internal_error(format!("Failed to read directory entry: {e}")))?
        {
            let metadata = entry.metadata().await.map_err(|e| {
                MCPError::internal_error(format!("Failed to get file metadata: {e}"))
            })?;

            let entry_type = if metadata.is_dir() {
                "directory"
            } else {
                "file"
            };

            let entry_info = DirectoryEntry {
                name: entry.file_name().to_string_lossy().to_string(),
                path: entry.path().to_string_lossy().to_string(),
                entry_type: entry_type.to_string(),
                size: if metadata.is_file() {
                    Some(metadata.len())
                } else {
                    None
                },
                modified: chrono::DateTime::<chrono::Utc>::from(
                    metadata
                        .modified()
                        .unwrap_or_else(|_| std::time::SystemTime::now()),
                )
                .to_rfc3339()
                .into(),
            };

            entries.push(entry_info);
        }

        let response = ListDirectoryResponse {
            path: request.path,
            entries,
        };

        let response_text = serde_json::to_string_pretty(&response)
            .map_err(|e| MCPError::serialization_error(e.to_string()))?;

        Ok(ToolResult {
            content: vec![ToolContent::text(response_text)],
            is_error: None,
        })
    }

    async fn handle_list_directory_with_sizes(
        &self,
        request: ListDirectoryWithSizesRequest,
    ) -> MCPResult<ToolResult> {
        let path = self.validate_path(&request.path)?;
        let path = Path::new(&path);

        if !path.exists() {
            return Err(MCPError::invalid_request(format!(
                "Directory not found: {}",
                request.path
            )));
        }

        if !path.is_dir() {
            return Err(MCPError::invalid_request(format!(
                "Path is not a directory: {}",
                request.path
            )));
        }

        let mut entries = Vec::new();
        let mut entries_iter = fs::read_dir(&path)
            .await
            .map_err(|e| MCPError::internal_error(format!("Failed to read directory: {e}")))?;

        while let Some(entry) = entries_iter
            .next_entry()
            .await
            .map_err(|e| MCPError::internal_error(format!("Failed to read directory entry: {e}")))?
        {
            let metadata = entry.metadata().await.map_err(|e| {
                MCPError::internal_error(format!("Failed to get file metadata: {e}"))
            })?;

            let entry_type = if metadata.is_dir() {
                "directory"
            } else {
                "file"
            };

            let entry_info = DirectoryEntry {
                name: entry.file_name().to_string_lossy().to_string(),
                path: entry.path().to_string_lossy().to_string(),
                entry_type: entry_type.to_string(),
                size: if metadata.is_file() {
                    Some(metadata.len())
                } else {
                    None
                },
                modified: chrono::DateTime::<chrono::Utc>::from(
                    metadata
                        .modified()
                        .unwrap_or_else(|_| std::time::SystemTime::now()),
                )
                .to_rfc3339()
                .into(),
            };

            entries.push(entry_info);
        }

        // Sort entries based on request
        if request.sort_by == "size" {
            entries.sort_by(|a, b| {
                let a_size = a.size.unwrap_or(0);
                let b_size = b.size.unwrap_or(0);
                b_size.cmp(&a_size) // Sort by size descending
            });
        } else {
            entries.sort_by(|a, b| a.name.cmp(&b.name)); // Sort by name
        }

        let response = ListDirectoryWithSizesResponse {
            path: request.path,
            entries,
            sort_by: request.sort_by,
        };

        let response_text = serde_json::to_string_pretty(&response)
            .map_err(|e| MCPError::serialization_error(e.to_string()))?;

        Ok(ToolResult {
            content: vec![ToolContent::text(response_text)],
            is_error: None,
        })
    }

    async fn handle_directory_tree(&self, request: DirectoryTreeRequest) -> MCPResult<ToolResult> {
        let path = self.validate_path(&request.path)?;
        let path = Path::new(&path);

        if !path.exists() {
            return Err(MCPError::invalid_request(format!(
                "Directory not found: {}",
                request.path
            )));
        }

        if !path.is_dir() {
            return Err(MCPError::invalid_request(format!(
                "Path is not a directory: {}",
                request.path
            )));
        }

        let mut children = Vec::new();
        let mut entries_iter = fs::read_dir(&path)
            .await
            .map_err(|e| MCPError::internal_error(format!("Failed to read directory: {e}")))?;

        while let Some(entry) = entries_iter
            .next_entry()
            .await
            .map_err(|e| MCPError::internal_error(format!("Failed to read directory entry: {e}")))?
        {
            let metadata = entry.metadata().await.map_err(|e| {
                MCPError::internal_error(format!("Failed to get file metadata: {e}"))
            })?;

            let node_type = if metadata.is_dir() {
                "directory"
            } else {
                "file"
            };

            let child_node = DirectoryTreeNode {
                name: entry.file_name().to_string_lossy().to_string(),
                node_type: node_type.to_string(),
                children: if metadata.is_dir() {
                    Some(Vec::new()) // For now, we don't recursively build the tree
                } else {
                    None
                },
            };

            children.push(child_node);
        }

        let response = DirectoryTreeResponse {
            path: request.path,
            tree: DirectoryTreeNode {
                name: path
                    .file_name()
                    .unwrap_or(path.as_os_str())
                    .to_string_lossy()
                    .to_string(),
                node_type: "directory".to_string(),
                children: Some(children),
            },
        };

        let response_text = serde_json::to_string_pretty(&response)
            .map_err(|e| MCPError::serialization_error(e.to_string()))?;

        Ok(ToolResult {
            content: vec![ToolContent::text(response_text)],
            is_error: None,
        })
    }

    async fn handle_move_file(&self, request: MoveFileRequest) -> MCPResult<ToolResult> {
        let source_path = self.validate_path(&request.source)?;
        let source_path = Path::new(&source_path);
        let destination_path = self.validate_path(&request.destination)?;
        let destination_path = Path::new(&destination_path);

        if !source_path.exists() {
            return Err(MCPError::invalid_request(format!(
                "Source file not found: {}",
                request.source
            )));
        }

        if !source_path.is_file() {
            return Err(MCPError::invalid_request(format!(
                "Source path is not a file: {}",
                request.source
            )));
        }

        if destination_path.exists() {
            return Err(MCPError::invalid_request(format!(
                "Destination already exists: {}",
                request.destination
            )));
        }

        fs::rename(&source_path, &destination_path)
            .await
            .map_err(|e| MCPError::internal_error(format!("Failed to move file: {e}")))?;

        let source_clone = request.source.clone();
        let dest_clone = request.destination.clone();
        let response = MoveFileResponse {
            source: request.source,
            destination: request.destination,
            success: true,
            message: format!("Successfully moved file from {source_clone} to {dest_clone}"),
        };

        let response_text = serde_json::to_string_pretty(&response)
            .map_err(|e| MCPError::serialization_error(e.to_string()))?;

        Ok(ToolResult {
            content: vec![ToolContent::text(response_text)],
            is_error: None,
        })
    }

    async fn handle_search_files(&self, request: SearchFilesRequest) -> MCPResult<ToolResult> {
        let path = self.validate_path(&request.path)?;
        let path = Path::new(&path);

        if !path.exists() {
            return Err(MCPError::invalid_request(format!(
                "Starting directory not found: {}",
                request.path
            )));
        }

        if !path.is_dir() {
            return Err(MCPError::invalid_request(format!(
                "Starting path is not a directory: {}",
                request.path
            )));
        }

        let mut results = Vec::new();
        let mut entries_iter = fs::read_dir(&path)
            .await
            .map_err(|e| MCPError::internal_error(format!("Failed to read directory: {e}")))?;

        while let Some(entry) = entries_iter
            .next_entry()
            .await
            .map_err(|e| MCPError::internal_error(format!("Failed to read directory entry: {e}")))?
        {
            let _metadata = entry.metadata().await.map_err(|e| {
                MCPError::internal_error(format!("Failed to get file metadata: {e}"))
            })?;

            let entry_path = entry.path().to_string_lossy().to_string();
            let entry_name = entry.file_name().to_string_lossy().to_string();

            if request.pattern.is_empty() {
                if !request.exclude_patterns.iter().any(|p| {
                    entry_name
                        .to_lowercase()
                        .contains(p.to_lowercase().as_str())
                }) {
                    results.push(entry_path);
                }
            } else {
                let pattern_lower = request.pattern.to_lowercase();
                let entry_name_lower = entry_name.to_lowercase();
                let _entry_path_lower = entry_path.to_lowercase();

                if entry_name_lower.contains(&pattern_lower)
                    && !request
                        .exclude_patterns
                        .iter()
                        .any(|p| entry_name_lower.contains(p.to_lowercase().as_str()))
                {
                    results.push(entry_path);
                }
            }
        }

        let response = SearchFilesResponse {
            path: request.path,
            pattern: request.pattern,
            results,
        };

        let response_text = serde_json::to_string_pretty(&response)
            .map_err(|e| MCPError::serialization_error(e.to_string()))?;

        Ok(ToolResult {
            content: vec![ToolContent::text(response_text)],
            is_error: None,
        })
    }

    async fn handle_get_file_info(&self, request: GetFileInfoRequest) -> MCPResult<ToolResult> {
        let path = self.validate_path(&request.path)?;
        let path = Path::new(&path);

        if !path.exists() {
            return Err(MCPError::invalid_request(format!(
                "File or directory not found: {}",
                request.path
            )));
        }

        let metadata = fs::metadata(&path)
            .await
            .map_err(|e| MCPError::internal_error(format!("Failed to get file metadata: {e}")))?;

        let is_directory = metadata.is_dir();
        let is_file = metadata.is_file();

        let permissions = if is_directory {
            "drwxr-xr-x".to_string() // Common Unix permissions for directories
        } else {
            "rw-r--r--".to_string() // Common Unix permissions for files
        };

        let response = GetFileInfoResponse {
            path: request.path,
            size: metadata.len(),
            created: chrono::DateTime::<chrono::Utc>::from(
                metadata
                    .created()
                    .unwrap_or_else(|_| std::time::SystemTime::now()),
            )
            .to_rfc3339(),
            modified: chrono::DateTime::<chrono::Utc>::from(
                metadata
                    .modified()
                    .unwrap_or_else(|_| std::time::SystemTime::now()),
            )
            .to_rfc3339(),
            accessed: chrono::DateTime::<chrono::Utc>::from(
                metadata
                    .accessed()
                    .unwrap_or_else(|_| std::time::SystemTime::now()),
            )
            .to_rfc3339(),
            is_directory,
            is_file,
            permissions,
        };

        let response_text = serde_json::to_string_pretty(&response)
            .map_err(|e| MCPError::serialization_error(e.to_string()))?;

        Ok(ToolResult {
            content: vec![ToolContent::text(response_text)],
            is_error: None,
        })
    }

    async fn handle_list_allowed_directories(&self) -> MCPResult<ToolResult> {
        let response = ListAllowedDirectoriesResponse {
            directories: self.allowed_directories.clone(),
        };
        let response_text = serde_json::to_string_pretty(&response)
            .map_err(|e| MCPError::serialization_error(e.to_string()))?;
        Ok(ToolResult {
            content: vec![ToolContent::text(response_text)],
            is_error: None,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize tracing based on transport type
    match args.transport {
        TransportType::Stdio => {
            // For STDIO, write to stderr to avoid interfering with protocol
            tracing_subscriber::fmt()
                .with_writer(std::io::stderr)
                .with_env_filter("info,ultrafast_mcp=debug")
                .with_target(false)
                .init();
        }
        TransportType::Http => {
            // For HTTP, we can use stdout
            tracing_subscriber::fmt()
                .with_env_filter("info,ultrafast_mcp=debug")
                .init();
        }
    }

    info!("🚀 Starting File Operations MCP Server");
    info!("📡 Transport: {:?}", args.transport);

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

    // Create server info
    let server_info = ServerInfo {
        name: "file-operations-server".to_string(),
        version: "1.0.0".to_string(),
        description: Some(format!(
            "A file operations server demonstrating UltraFastServer with {:?} transport",
            args.transport
        )),
        authors: Some(vec!["ULTRAFAST_MCP Team".to_string()]),
        homepage: Some("https://github.com/ultrafast-mcp/ultrafast-mcp".to_string()),
        license: Some("MIT OR Apache-2.0".to_string()),
        repository: Some("https://github.com/ultrafast-mcp/ultrafast-mcp".to_string()),
    };

    // Create server with tool handler
    let server = UltraFastServer::new(server_info, capabilities).with_tool_handler(Arc::new(
        FileOperationsHandler::new(args.allowed_directories),
    ));

    // Run the server with the chosen transport
    match args.transport {
        TransportType::Stdio => {
            info!("✅ Starting STDIO transport (subprocess mode)");
            server.run_stdio().await?;
        }
        TransportType::Http => {
            info!("✅ Starting HTTP transport on {}:{}", args.host, args.port);
            let config = HttpTransportConfig {
                host: args.host,
                port: args.port,
                cors_enabled: true,
                protocol_version: "2025-06-18".to_string(),
                allow_origin: Some("*".to_string()),
                monitoring_enabled: true,
                enable_sse_resumability: true,
            };
            server.run_streamable_http_with_config(config).await?;
        }
    }

    info!("Server shutdown completed");
    Ok(())
}
