# Everything MCP Server Example (STDIO)

This example demonstrates a complete MCP server implementation using STDIO transport that implements all handler traits and provides at least one tool, resource, and prompt.

## Features

- **Complete Handler Implementation**: Implements all MCP handler traits
- **Tool Handler**: Provides an `echo` tool that returns the input message
- **Resource Handler**: Provides a dummy resource for demonstration
- **Prompt Handler**: Provides a hello prompt for demonstration
- **STDIO Transport**: Uses standard input/output for communication
- **Monitoring**: Includes full monitoring capabilities

## Running the Server

```bash
# Build the server
cargo build --release

# Run the server (will read from stdin, write to stdout)
./target/release/server
```

## Running the Client

```bash
# Build the client
cargo build --release

# Run the client (will connect to server via stdio)
./target/release/client
```

## Testing with MCP Inspector

The [MCP Inspector](https://github.com/modelcontextprotocol/inspector) is a web-based tool for testing and debugging MCP servers. You can use it to interactively explore and validate your server implementation.

### Prerequisites

1. **Install Node.js** (v18 or later)
2. **Install MCP Inspector**:
   ```bash
   npm install -g @modelcontextprotocol/inspector
   ```

### Steps to Test

1. **Build the server**
   ```bash
   cargo build --release
   ```
2. **Start the MCP Inspector**
   ```bash
   mcp-inspector
   ```
3. **Connect the Inspector to your server**
   - Open your browser and go to `http://localhost:3000`
   - Click **"Add Server"**
   - Select **"Command"** as the connection type
   - Enter the path to your server binary:
     ```
     /path/to/your/project/target/release/everything-server-stdio
     ```
   - Click **"Connect"**

4. **Test server features**
   - Use the Inspector UI to call tools, list resources, get prompts, and test ping/logging.

### Example Test Scenarios

- **Ping**: Test connection health with the ping method
- **Tools**: Call the `echo` tool
- **Resources**: List and read available resources
- **Prompts**: Retrieve and test available prompts

### Troubleshooting

- If you see "Method not implemented" errors, ensure your server implements all required MCP methods.
- If the Inspector cannot connect, verify the server binary path and permissions.
- For schema validation errors, check your tool/resource schemas.

### Quick Start with npx and Config File

You can also launch the MCP Inspector and connect to your server in one step using a config file:

```bash
npx @modelcontextprotocol/inspector --config mcp-inspector-config.json --server everything-server-stdio
```

- This will start the Inspector and automatically connect to the `everything-server-stdio` binary using the settings in `mcp-inspector-config.json`.
- Make sure your server binary is built and available in your PATH or specify the full path if needed.

---

## Handler Types Implemented

1. **ToolHandler**: `EchoToolHandler` - Provides an echo tool
2. **ResourceHandler**: `DummyResourceHandler` - Provides dummy resources
3. **PromptHandler**: `DummyPromptHandler` - Provides hello prompts
4. **SamplingHandler**: `DummySamplingHandler` - Stub implementation
5. **CompletionHandler**: `DummyCompletionHandler` - Stub implementation
6. **RootsHandler**: `DummyRootsHandler` - Stub implementation
7. **ElicitationHandler**: `DummyElicitationHandler` - Stub implementation
8. **ResourceSubscriptionHandler**: `DummySubscriptionHandler` - Stub implementation

## Available Tools

- **echo**: Returns the input message (takes a `message` parameter)

## Available Resources

- **file:///dummy.txt**: A dummy text resource

## Available Prompts

- **hello-prompt**: A simple hello prompt

## Architecture

This example shows how to:
- Implement all MCP handler traits
- Use the builder pattern for server configuration
- Handle tool calls with proper argument parsing
- Provide resources and prompts
- Use STDIO transport for communication
- Enable monitoring and health checks

The server demonstrates the complete MCP protocol implementation with all handler types, making it a comprehensive example for understanding the full MCP server capabilities. 