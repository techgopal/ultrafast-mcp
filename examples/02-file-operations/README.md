# File Operations MCP Example

This example demonstrates file system operations using the UltraFast MCP framework. It includes both a server that provides file operations as MCP tools and a client that connects to the server to perform file operations. **This implementation is aligned with the official [MCP filesystem server](https://github.com/modelcontextprotocol/servers/tree/main/src/filesystem) from the Model Context Protocol repository.**

## Features

- **Complete MCP Filesystem Compliance**: Full alignment with the official MCP filesystem server implementation
- **File Reading**: Read file contents with head/tail support
- **File Writing**: Write content to files with overwrite protection
- **File Editing**: Make line-based edits with diff output
- **Directory Operations**: Create directories and list contents
- **File Management**: Move, search, and get detailed file information
- **Security**: Path validation and allowed directories
- **Multiple Transports**: Support for both STDIO and HTTP transports
- **Streamable HTTP**: High-performance HTTP transport with Server-Sent Events

## Tools Available (Official MCP Filesystem Server)

1. **`read_file`**: Read a file from disk
   - Parameters: `path` (string), `head` (optional number), `tail` (optional number)
   - Returns: File content and path

2. **`read_multiple_files`**: Read multiple files simultaneously
   - Parameters: `paths` (array of strings)
   - Returns: Array of file contents with paths

3. **`write_file`**: Write content to a file (overwrites existing)
   - Parameters: `path` (string), `content` (string)
   - Returns: Success status and message

4. **`edit_file`**: Make line-based edits to a text file
   - Parameters: `path` (string), `edits` (array of edit operations), `dry_run` (optional boolean)
   - Returns: Success status, diff, and message

5. **`create_directory`**: Create a new directory
   - Parameters: `path` (string)
   - Returns: Success status and message

6. **`list_directory`**: List files and directories
   - Parameters: `path` (string)
   - Returns: Array of directory entries with metadata

7. **`list_directory_with_sizes`**: List files with sizes and sorting
   - Parameters: `path` (string), `sort_by` (optional: "name" or "size")
   - Returns: Array of directory entries with sizes

8. **`directory_tree`**: Get recursive tree view
   - Parameters: `path` (string)
   - Returns: JSON tree structure

9. **`move_file`**: Move or rename files
   - Parameters: `source` (string), `destination` (string)
   - Returns: Success status and message

10. **`search_files`**: Search for files by pattern
    - Parameters: `path` (string), `pattern` (string), `exclude_patterns` (optional array)
    - Returns: Array of matching file paths

11. **`get_file_info`**: Get detailed file metadata
    - Parameters: `path` (string)
    - Returns: File size, timestamps, permissions, and type

12. **`list_allowed_directories`**: List accessible directories
    - Parameters: None
    - Returns: Array of allowed directory paths

## Security Features

- **Path Validation**: All file operations validate paths against allowed directories
- **Allowed Directories**: Server only operates within specified directory boundaries
- **Symlink Protection**: Handles symlinks safely to prevent directory traversal attacks
- **Atomic Operations**: File writes use atomic operations to prevent race conditions

## Transport Options

### STDIO Transport (Subprocess Mode)
- Local communication between client and server
- Minimal overhead
- Suitable for local development and testing

### HTTP Transport (Network Mode)
- Network-based communication
- Support for remote connections
- Streamable HTTP with Server-Sent Events
- CORS enabled for web applications

## Usage

### Building

```bash
cargo build --release
```

### Running the Server

#### STDIO Transport
```bash
# Use current directory as allowed directory
./target/release/file-ops-server stdio

# Specify allowed directories
./target/release/file-ops-server stdio --allowed-directories /tmp /home/user/documents
```

#### HTTP Transport
```bash
# Default host and port (127.0.0.1:8080)
./target/release/file-ops-server http

# Custom host and port
./target/release/file-ops-server http --host 0.0.0.0 --port 9000

# With allowed directories
./target/release/file-ops-server http --allowed-directories /tmp /var/log
```

### Running the Client

#### STDIO Transport
```bash
./target/release/file-ops-client stdio
```

#### HTTP Transport
```bash
# Default server URL
./target/release/file-ops-client http

# Custom server URL
./target/release/file-ops-client http --server-url http://localhost:9000
```

### Testing with MCP Inspector

1. Start the server in HTTP mode:
   ```bash
   ./target/release/file-ops-server http
   ```

2. Open [MCP Inspector](https://www.npmjs.com/package/@modelcontextprotocol/inspector)

3. Load the `mcp-inspector-config.json` file

4. Test all the available tools interactively

### Automated Testing

Run the automated test script:

```bash
./test_http.sh
```

This script will:
- Build the project
- Start the server in HTTP mode
- Run the client to test all operations
- Clean up test files

## Example Operations

### Basic File Operations
```bash
# Create a directory
./target/release/file-ops-client http --server-url http://localhost:8080

# The client will automatically test:
# - Creating directories
# - Writing files
# - Reading files (full content, head, tail)
# - Listing directories
# - Getting file information
# - Searching files
# - Editing files
# - Moving files
```

### Using MCP Inspector
1. Start the server: `./target/release/file-ops-server http`
2. Open MCP Inspector and load the config
3. Try these operations:
   - **Read a file**: Use `read_file` with a file path
   - **Create a directory**: Use `create_directory` with a path
   - **List contents**: Use `list_directory` with a directory path
   - **Search files**: Use `search_files` with a pattern
   - **Get file info**: Use `get_file_info` with a file path

## Configuration

### Allowed Directories
The server restricts all file operations to specified directories for security:

```bash
# Allow access to multiple directories
./target/release/file-ops-server http --allowed-directories /tmp /home/user /var/log

# Default: current directory (.)
```

### HTTP Configuration
```bash
# Custom host and port
./target/release/file-ops-server http --host 0.0.0.0 --port 9000

# Default: 127.0.0.1:8080
```

## Differences from Official MCP Filesystem Server

This implementation provides the same API as the official MCP filesystem server but with these UltraFast MCP-specific features:

- **Rust Implementation**: Native Rust performance and safety
- **UltraFast MCP Framework**: Built on the UltraFast MCP framework
- **HTTP Transport**: Additional HTTP transport support
- **Enhanced Error Handling**: Comprehensive error handling and logging
- **Type Safety**: Strong typing with Rust's type system

## Security Considerations

- **Path Validation**: All paths are validated against allowed directories
- **No Recursive Operations**: Directory operations are non-recursive by default
- **Atomic Writes**: File writes use atomic operations
- **Symlink Protection**: Handles symlinks safely
- **Input Validation**: All inputs are validated before processing

## Troubleshooting

### Common Issues

1. **Permission Denied**: Check that the server has access to the allowed directories
2. **File Not Found**: Verify the file path is within allowed directories
3. **Connection Refused**: Ensure the server is running and the port is correct
4. **CORS Errors**: The HTTP server includes CORS headers for web applications

### Debug Mode

Enable debug logging:

```bash
RUST_LOG=debug ./target/release/file-ops-server http
RUST_LOG=debug ./target/release/file-ops-client http
```

## Contributing

This example demonstrates how to implement a production-ready MCP server using the UltraFast MCP framework. The implementation follows the official MCP filesystem server specification while leveraging UltraFast MCP's performance and developer experience features. 