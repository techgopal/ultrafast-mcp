# File Operations Example

This example demonstrates the new `UltraFastServer` and `UltraFastClient` APIs with comprehensive file system operations.

## Overview

The example consists of:
- **Server**: An MCP server that provides file system operations (read, write, list, delete)
- **Client**: A client that connects to the server and tests all file operations

## Features Demonstrated

- Creating an MCP server using `UltraFastServer` with multiple tools
- Implementing complex tool handlers with the `ToolHandler` trait
- File system operations: read, write, append, list, delete
- Error handling for file operations
- Creating an MCP client using `UltraFastClient`
- Testing file operations end-to-end
- Cleanup and verification

## Running the Example

### 1. Build the Example

```bash
cd examples/02-file-operations
cargo build
```

### 2. Run the Server

In one terminal:

```bash
cargo run --bin server
```

The server will start and wait for connections on stdio.

### 3. Run the Client

In another terminal:

```bash
cargo run --bin client
```

The client will connect to the server and perform a series of file operations.

## Expected Output

### Server Output
```
🚀 Starting File Operations Server
✅ Server created successfully
📡 Starting server on stdio transport
📨 Received tool call: write_file
📨 Received tool call: read_file
📨 Received tool call: write_file
📨 Received tool call: list_files
📨 Received tool call: delete_file
```

### Client Output
```
🚀 Starting File Operations Client
✅ Client created successfully
🔗 Connected to file operations server
📋 Listing available tools...
🔧 Tool: read_file - Read the contents of a file
🔧 Tool: write_file - Write content to a file
🔧 Tool: list_files - List files in a directory
🔧 Tool: delete_file - Delete a file or directory
📝 Testing write_file tool...
📥 Write response received:
📄 Response: {"path":"test_files/test.txt","size":45,"written":true}
📖 Testing read_file tool...
📥 Read response received:
📄 Response: {"content":"Hello from UltraFastClient!\nThis is a test file.\n","size":45,"modified":"2024-01-01T12:00:00Z","path":"test_files/test.txt"}
📝 Testing append to file...
📥 Append response received:
📄 Response: {"path":"test_files/test.txt","size":67,"written":true}
📁 Testing list_files tool...
📥 List files response received:
📄 Response: {"files":[{"name":"test.txt","path":"test_files/test.txt","is_dir":false,"size":67,"modified":"2024-01-01T12:00:01Z"}],"total_count":1,"path":"test_files"}
🗑️ Testing delete_file tool...
📥 Delete response received:
📄 Response: {"path":"test_files/test.txt","deleted":true,"message":"Successfully deleted"}
🔍 Verifying file was deleted...
✅ File successfully deleted (expected error): File not found: test_files/test.txt
🧹 Cleaning up test directory...
📥 Cleanup response received:
📄 Response: {"path":"test_files","deleted":true,"message":"Successfully deleted"}
🔍 Testing error handling with non-existent file...
✅ Expected error for non-existent file: File not found: non_existent_file.txt
✅ All file operations tests completed successfully!
```

## Code Structure

### Server (`src/server.rs`)

The server demonstrates:
- Creating an `UltraFastServer` with file operation capabilities
- Implementing a comprehensive `ToolHandler` with multiple tools
- File system operations with proper error handling
- Structured request/response handling
- File metadata handling

### Client (`src/client.rs`)

The client demonstrates:
- Creating an `UltraFastClient` with stdio transport
- Testing all file operations systematically
- Error handling and verification
- Cleanup operations
- End-to-end testing workflow

## Available Tools

### 1. `read_file`
Reads the contents of a file.

**Parameters:**
- `path` (string, required): Path to the file to read

**Response:**
```json
{
  "content": "file contents",
  "size": 1234,
  "modified": "2024-01-01T12:00:00Z",
  "path": "/path/to/file"
}
```

### 2. `write_file`
Writes content to a file.

**Parameters:**
- `path` (string, required): Path to the file to write
- `content` (string, required): Content to write to the file
- `append` (boolean, optional): Whether to append to existing file (default: false)

**Response:**
```json
{
  "path": "/path/to/file",
  "size": 1234,
  "written": true
}
```

### 3. `list_files`
Lists files in a directory.

**Parameters:**
- `path` (string, required): Path to the directory to list
- `recursive` (boolean, optional): Whether to list recursively (default: false)

**Response:**
```json
{
  "files": [
    {
      "name": "file.txt",
      "path": "/path/to/file.txt",
      "is_dir": false,
      "size": 1234,
      "modified": "2024-01-01T12:00:00Z"
    }
  ],
  "total_count": 1,
  "path": "/path/to/directory"
}
```

### 4. `delete_file`
Deletes a file or directory.

**Parameters:**
- `path` (string, required): Path to the file or directory to delete
- `recursive` (boolean, optional): Whether to delete directories recursively (default: false)

**Response:**
```json
{
  "path": "/path/to/file",
  "deleted": true,
  "message": "Successfully deleted"
}
```

## Key API Usage

### Server Creation with Multiple Tools
```rust
let server = UltraFastServer::new(
    ServerInfo { /* ... */ },
    ServerCapabilities {
        tools: Some(ToolsCapability { list_changed: Some(true) }),
        resources: Some(ResourcesCapability { /* ... */ }),
        ..Default::default()
    }
)
.with_tool_handler(Arc::new(FileOperationsHandler))
.build()?;
```

### Tool Handler Implementation
```rust
#[async_trait::async_trait]
impl ultrafast_mcp::ToolHandler for FileOperationsHandler {
    async fn handle_tool_call(&self, call: ultrafast_mcp::ToolCall) -> ultrafast_mcp::McpResult<ultrafast_mcp::ToolResult> {
        match call.name.as_str() {
            "read_file" => self.handle_read_file(request).await,
            "write_file" => self.handle_write_file(request).await,
            "list_files" => self.handle_list_files(request).await,
            "delete_file" => self.handle_delete_file(request).await,
            _ => Err(ultrafast_mcp::McpError::method_not_found(format!("Unknown tool: {}", call.name))),
        }
    }
}
```

### Client File Operations
```rust
// Write a file
let write_result = client.call_tool("write_file", json!({
    "path": "test.txt",
    "content": "Hello World!",
    "append": false
})).await?;

// Read a file
let read_result = client.call_tool("read_file", json!({
    "path": "test.txt"
})).await?;

// List files
let list_result = client.call_tool("list_files", json!({
    "path": ".",
    "recursive": false
})).await?;

// Delete a file
let delete_result = client.call_tool("delete_file", json!({
    "path": "test.txt",
    "recursive": false
})).await?;
```

## Error Handling

The example demonstrates comprehensive error handling:
- File not found errors
- Permission errors
- Invalid path errors
- Directory not empty errors
- Network/connection errors

## Security Considerations

The file operations server includes basic safety measures:
- Path validation
- Directory traversal protection
- Recursive deletion confirmation
- Error message sanitization

This example provides a comprehensive demonstration of building a production-ready MCP server with file system operations using the new ergonomic APIs. 