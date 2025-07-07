# Everything MCP Server Example (STDIO)

This example demonstrates a complete MCP server implementation using STDIO transport that implements all handler traits and provides comprehensive MCP 2025-06-18 features including progress notifications, cancellation support, and resource subscriptions.

## Features

### Core MCP Implementation
- **Complete Handler Implementation**: Implements all MCP handler traits
- **Tool Handler**: Provides 11 different tools demonstrating various MCP capabilities
- **Resource Handler**: Provides 100 test resources with pagination support
- **Prompt Handler**: Provides 3 different prompts including resource-embedded prompts
- **STDIO Transport**: Uses standard input/output for communication
- **Monitoring**: Includes full monitoring capabilities

### New MCP 2025-06-18 Features
- **Progress Notifications**: Long-running operations with progress tracking
- **Cancellation Support**: Cancellable operations that can be interrupted
- **Resource Subscriptions**: Subscribe to resource changes and receive notifications
- **Enhanced Completion**: Advanced completion and elicitation handlers
- **Comprehensive Notifications**: Support for all MCP notification types
- **Resource Templates**: Dynamic resource discovery with templates
- **Logging Integration**: Full logging support with level controls

## Available Tools

1. **echo** - Echoes back the input message
2. **add** - Adds two numbers together
3. **longRunningOperation** - Demonstrates progress tracking over multiple steps
4. **cancellableOperation** - Shows cancellation support with periodic checks
5. **notificationDemo** - Demonstrates various MCP notification types
6. **printEnv** - Prints environment variables for debugging
7. **sampleLLM** - Simulates LLM sampling functionality
8. **getTinyImage** - Returns a tiny test image with multiple content types
9. **annotatedMessage** - Demonstrates message annotations and metadata
10. **getResourceReference** - Returns resource references for client usage
11. **getResourceLinks** - Returns multiple resource links with descriptions

## Available Resources

- **100 Test Resources**: `test://static/resource/1` through `test://static/resource/100`
- **Pagination Support**: Resources are returned with cursor-based pagination
- **Resource Templates**: Dynamic resource discovery with `test://static/resource/{id}`
- **Content Variety**: Mix of text and binary content types

## Available Prompts

1. **simple_prompt** - A basic prompt without arguments
2. **complex_prompt** - A prompt with temperature and style arguments
3. **resource_prompt** - A prompt that embeds resource references

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
- **Tools**: Call various tools like `echo`, `add`, `longRunningOperation`, `cancellableOperation`
- **Resources**: List and read available resources with pagination
- **Prompts**: Retrieve and test available prompts with arguments
- **Progress Tracking**: Test `longRunningOperation` to see progress updates
- **Cancellation**: Test `cancellableOperation` and cancel it mid-execution
- **Notifications**: Use `notificationDemo` to trigger various notification types
- **Resource Subscriptions**: Subscribe to resource changes and receive notifications

### Testing Progress and Cancellation

1. **Progress Tracking**:
   ```json
   {
     "jsonrpc": "2.0",
     "id": 1,
     "method": "tools/call",
     "params": {
       "name": "longRunningOperation",
       "arguments": {
         "duration": 20,
         "steps": 10
       }
     }
   }
   ```

2. **Cancellation Support**:
   ```json
   {
     "jsonrpc": "2.0",
     "id": 2,
     "method": "tools/call",
     "params": {
       "name": "cancellableOperation",
       "arguments": {
         "duration": 60,
         "checkInterval": 5
       }
     }
   }
   ```

3. **Notification Demo**:
   ```json
   {
     "jsonrpc": "2.0",
     "id": 3,
     "method": "tools/call",
     "params": {
       "name": "notificationDemo",
       "arguments": {
         "type": "resource_list_changed"
       }
     }
   }
   ```

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