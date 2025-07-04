# Quick Start Guide

Get up and running with **ULTRAFAST_MCP** in 5 minutes! This guide will walk you through creating your first MCP server and client.

## 🚀 Prerequisites

- **Rust 1.70+**: [Install Rust](https://rustup.rs/)
- **Cargo**: Comes with Rust installation
- **Basic Rust knowledge**: Familiarity with async/await and serde

## 📦 Installation

### 1. Create a New Project

```bash
# Create a new Rust project
cargo new my-mcp-server
cd my-mcp-server
```

### 2. Add ULTRAFAST_MCP

```bash
# Add the main crate with stdio transport (default)
cargo add ultrafast-mcp

# Or add with HTTP transport for web services
cargo add ultrafast-mcp --features="http"
```

### 3. Update Cargo.toml

Your `Cargo.toml` should look like this:

```toml
[package]
name = "my-mcp-server"
version = "0.1.0"
edition = "2021"

[dependencies]
ultrafast-mcp = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"
```

## 🖥️ Create Your First Server

### 1. Create `src/main.rs`

```rust
use ultrafast_mcp::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct GreetRequest {
    name: String,
}

#[derive(Serialize)]
struct GreetResponse {
    message: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Create a new MCP server
    let server = UltraFastServer::new("My First MCP Server")
        .with_protocol_version("2025-06-18")
        .with_capabilities(ServerCapabilities {
            tools: Some(ToolsCapability { list_changed: true }),
            ..Default::default()
        });
    
    // Add a greeting tool
    server.tool("greet", |request: GreetRequest, ctx: Context| async move {
        // Log the request
        ctx.log_info(&format!("Greeting requested for: {}", request.name)).await?;
        
        // Return the greeting
        Ok(GreetResponse {
            message: format!("Hello, {}! Welcome to ULTRAFAST_MCP!", request.name),
        })
    })
    .description("Greet a user by name")
    .output_schema::<GreetResponse>();
    
    // Run the server using stdio transport
    server.run_stdio().await?;
    Ok(())
}
```

### 2. Build and Run

```bash
# Build the server
cargo build

# Run the server (it will wait for client connections)
cargo run
```

## 🖥️ Create Your First Client

### 1. Create a New Client Project

```bash
# In a new terminal, create a client project
cargo new my-mcp-client
cd my-mcp-client
cargo add ultrafast-mcp
```

### 2. Create `src/main.rs`

```rust
use ultrafast_mcp::prelude::*;

#[derive(Deserialize)]
struct GreetResponse {
    message: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Connect to the server using stdio transport
    let client = UltraFastClient::connect(Transport::Stdio {
        command: "cargo".into(),
        args: vec!["run", "--manifest-path", "../my-mcp-server/Cargo.toml"].into(),
    }).await?;
    
    // Initialize the client
    client.initialize(ClientCapabilities {
        tools: Some(ToolsCapability { list_changed: true }),
        ..Default::default()
    }).await?;
    
    // Call the greeting tool
    let response: GreetResponse = client.call_tool("greet")
        .arg("name", "Alice")
        .await?;
    
    println!("Server says: {}", response.message);
    Ok(())
}
```

### 3. Run the Client

```bash
# Make sure the server is running in another terminal
# Then run the client
cargo run
```

You should see: `Server says: Hello, Alice! Welcome to ULTRAFAST_MCP!`

## 🌐 HTTP Server Example

### 1. Create HTTP Server

```rust
use ultrafast_mcp::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct WeatherRequest {
    city: String,
}

#[derive(Serialize)]
struct WeatherResponse {
    temperature: f64,
    conditions: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let server = UltraFastServer::new("Weather MCP Server")
        .with_protocol_version("2025-06-18")
        .with_capabilities(ServerCapabilities {
            tools: Some(ToolsCapability { list_changed: true }),
            ..Default::default()
        });
    
    // Add a weather tool
    server.tool("get_weather", |request: WeatherRequest, ctx: Context| async move {
        ctx.progress("Fetching weather data...", 0.5, Some(1.0)).await?;
        
        // Simulate weather API call
        let weather = get_weather_data(&request.city).await?;
        
        ctx.progress("Weather data retrieved", 1.0, Some(1.0)).await?;
        ctx.log_info(&format!("Weather requested for: {}", request.city)).await?;
        
        Ok(WeatherResponse {
            temperature: weather.temperature,
            conditions: weather.conditions,
        })
    })
    .description("Get current weather for a city")
    .output_schema::<WeatherResponse>();
    
    // Run HTTP server
    server.run_http("127.0.0.1", 8080).await?;
    Ok(())
}

async fn get_weather_data(city: &str) -> Result<WeatherResponse> {
    // Simulate API call
    Ok(WeatherResponse {
        temperature: 22.5,
        conditions: "Sunny".to_string(),
    })
}
```

### 2. Create HTTP Client

```rust
use ultrafast_mcp::prelude::*;

#[derive(Deserialize)]
struct WeatherResponse {
    temperature: f64,
    conditions: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = UltraFastClient::connect(Transport::Streamable {
        url: "http://127.0.0.1:8080/mcp".into(),
        auth: None,
    }).await?;
    
    client.initialize(ClientCapabilities {
        tools: Some(ToolsCapability { list_changed: true }),
        ..Default::default()
    }).await?;
    
    let weather: WeatherResponse = client.call_tool("get_weather")
        .arg("city", "San Francisco")
        .with_progress(|progress, total, message| {
            println!("Progress: {}/{:?} - {}", progress, total, message.unwrap_or_default());
        })
        .await?;
    
    println!("Weather in San Francisco: {}°C, {}", weather.temperature, weather.conditions);
    Ok(())
}
```

## 🔧 Next Steps

### 1. Explore Examples

```bash
# Clone the repository to see more examples
git clone https://github.com/ultrafast-mcp/ultrafast-mcp.git
cd ultrafast-mcp/examples

# Run the basic echo example
cd 01-basic-echo
cargo run --bin server  # Terminal 1
cargo run --bin client  # Terminal 2
```

### 2. Learn More Features

- **[Tools](./features/tools.md)** - Function execution framework
- **[Resources](./features/resources.md)** - URI-based resource management
- **[Prompts](./features/prompts.md)** - Template-based prompt system
- **[Authentication](./compliance/oauth.md)** - OAuth 2.1 integration

### 3. Production Deployment

- **[Security Best Practices](./advanced/security.md)** - Secure your server
- **[Monitoring](./advanced/monitoring.md)** - Add observability
- **[Deployment](./advanced/deployment.md)** - Deploy to production

## 🐛 Troubleshooting

### Common Issues

#### 1. "Connection refused" Error
- Make sure the server is running before starting the client
- Check that the transport configuration matches between server and client

#### 2. "Tool not found" Error
- Verify the tool name matches exactly (case-sensitive)
- Check that the tool is properly registered in the server

#### 3. Serialization Errors
- Ensure your request/response types derive `Serialize` and `Deserialize`
- Check that the JSON structure matches your Rust types

#### 4. Build Errors
- Make sure you have Rust 1.70+ installed
- Run `cargo clean && cargo build` to clear any cached artifacts

### Getting Help

- **Documentation**: [Complete API Reference](./api-reference/server-api.md)
- **Examples**: [Working Examples](./examples/basic-echo.md)
- **Issues**: [GitHub Issues](https://github.com/ultrafast-mcp/ultrafast-mcp/issues)
- **Discussions**: [GitHub Discussions](https://github.com/ultrafast-mcp/ultrafast-mcp/discussions)

## 🎉 Congratulations!

You've successfully created your first MCP server and client with **ULTRAFAST_MCP**! 

- ✅ Created a working MCP server
- ✅ Implemented a tool with proper error handling
- ✅ Built a client that connects to the server
- ✅ Used both stdio and HTTP transports
- ✅ Added progress tracking and logging

You're now ready to build production-grade MCP applications! 🚀 