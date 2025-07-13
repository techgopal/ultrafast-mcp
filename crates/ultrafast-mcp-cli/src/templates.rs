use crate::config::Config;
use anyhow::{Context, Result};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Template for project generation
pub struct Template {
    pub name: String,
    pub description: String,
    pub files: Vec<TemplateFile>,
}

/// A file in a template
pub struct TemplateFile {
    pub path: String,
    pub content: String,
    pub is_binary: bool,
}

/// Template configuration (for filesystem templates)
#[derive(Debug, Serialize, Deserialize)]
struct TemplateConfig {
    name: String,
    description: String,
    files: Vec<TemplateFileConfig>,
}

/// File configuration in template
#[derive(Debug, Serialize, Deserialize)]
struct TemplateFileConfig {
    path: String,
    is_binary: Option<bool>,
}

impl Template {
    /// Load a template by name
    pub fn load(name: &str, config: Option<&Config>) -> Result<Self> {
        match name {
            "basic" => Ok(Self::basic_template()),
            "server" => Ok(Self::server_template()),
            "client" => Ok(Self::client_template()),
            "full" => Ok(Self::full_template()),
            _ => {
                // Try to load from template directory if configured
                if let Some(config) = config {
                    if let Some(template_info) = config.templates.templates.get(name) {
                        Self::load_from_path(&template_info.path)
                    } else {
                        anyhow::bail!("Unknown template: {}", name);
                    }
                } else {
                    anyhow::bail!("Unknown template: {}", name);
                }
            }
        }
    }

    /// Load template from filesystem path
    pub fn load_from_path(path: &str) -> Result<Self> {
        use std::fs;
        use std::path::Path;

        let template_path = Path::new(path);

        if !template_path.exists() {
            anyhow::bail!("Template path does not exist: {}", path);
        }

        if !template_path.is_dir() {
            anyhow::bail!("Template path is not a directory: {}", path);
        }

        // Look for template.toml or template.json
        let config_path = template_path.join("template.toml");
        let config_path = if config_path.exists() {
            config_path
        } else {
            let json_path = template_path.join("template.json");
            if json_path.exists() {
                json_path
            } else {
                anyhow::bail!("No template.toml or template.json found in: {}", path);
            }
        };

        // Read template configuration
        let config_content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read template config: {config_path:?}"))?;

        let template_config: TemplateConfig =
            if config_path.extension().unwrap_or_default() == "toml" {
                toml::from_str(&config_content).with_context(|| "Failed to parse template.toml")?
            } else {
                serde_json::from_str(&config_content)
                    .with_context(|| "Failed to parse template.json")?
            };

        // Load template files
        let mut files = Vec::new();
        for file_config in &template_config.files {
            let file_path = template_path.join(&file_config.path);
            if file_path.exists() {
                let content = if file_config.is_binary.unwrap_or(false) {
                    // For binary files, read and base64 encode
                    let bytes = fs::read(&file_path)
                        .with_context(|| format!("Failed to read binary file: {file_path:?}"))?;
                    base64::engine::general_purpose::STANDARD.encode(&bytes)
                } else {
                    // For text files, read as string
                    fs::read_to_string(&file_path)
                        .with_context(|| format!("Failed to read text file: {file_path:?}"))?
                };

                files.push(TemplateFile {
                    path: file_config.path.clone(),
                    content,
                    is_binary: file_config.is_binary.unwrap_or(false),
                });
            } else {
                anyhow::bail!("Template file not found: {:?}", file_path);
            }
        }

        Ok(Template {
            name: template_config.name,
            description: template_config.description,
            files,
        })
    }

    /// Generate project from template
    pub fn generate(&self, output_dir: &Path, context: &HashMap<String, String>) -> Result<()> {
        for file in &self.files {
            let file_path = output_dir.join(&file.path);

            // Create parent directories
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            }

            // Process template content
            let content = if file.is_binary {
                file.content.clone()
            } else {
                self.process_template(&file.content, context)?
            };

            // Write file
            std::fs::write(&file_path, content)
                .with_context(|| format!("Failed to write file: {}", file_path.display()))?;
        }

        Ok(())
    }

    /// Process template content with variables
    fn process_template(&self, content: &str, context: &HashMap<String, String>) -> Result<String> {
        let mut result = content.to_string();

        // Simple variable substitution - in a real implementation you'd use a proper template engine
        for (key, value) in context {
            let placeholder = format!("{{{{{key}}}}}");
            result = result.replace(&placeholder, value);
        }

        Ok(result)
    }

    /// Basic MCP server template
    fn basic_template() -> Self {
        Self {
            name: "basic".to_string(),
            description: "Basic MCP server template".to_string(),
            files: vec![
                TemplateFile {
                    path: "Cargo.toml".to_string(),
                    content: r#"[package]
name = "{{project_name}}"
version = "{{version}}"
edition = "2021"
description = "{{description}}"
authors = ["{{author}}"]
license = "{{license}}"

[dependencies]
ultrafast-mcp = { version = "0.1.0" }
ultrafast-mcp-server = { version = "0.1.0" }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
chrono = { version = "0.4", features = ["serde"] }

[[bin]]
name = "{{project_name}}"
path = "src/main.rs"
"#
                    .to_string(),
                    is_binary: false,
                },
                TemplateFile {
                    path: "src/main.rs".to_string(),
                    content: r#"use ultrafast_mcp::prelude::*;
use anyhow::Result;
use tracing::info;

mod tools;
mod resources;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    info!("Starting {{project_name}} MCP server");
    
    let mut server = UltraFastServer::new(
        ServerInfo {
            name: "{{project_name}}".to_string(),
            version: "{{version}}".to_string(),
            description: Some("{{description}}".to_string()),
            authors: None,
            homepage: None,
            license: None,
            repository: None,
        },
        ServerCapabilities::default(),
    );
    
    // Register tools and resources
    tools::register_tools(&mut server)?;
    resources::register_resources(&mut server)?;
    
    server.run_stdio().await?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_server_creation() {
        let server = UltraFastServer::new(
            ServerInfo {
                name: "test-server".to_string(),
                version: "1.0.0".to_string(),
                description: Some("Test server".to_string()),
                authors: None,
                homepage: None,
                license: None,
                repository: None,
            },
            ServerCapabilities::default(),
        );
        
        assert_eq!(server.info().name, "test-server");
    }
}
"#
                    .to_string(),
                    is_binary: false,
                },
                TemplateFile {
                    path: "README.md".to_string(),
                    content: r#"# {{project_name}}

{{description}}

## Quick Start

Build and run the server:

```bash
cargo run
```

## Testing

Run tests:

```bash
cargo test
```

## Development

Use the MCP CLI for development:

```bash
mcp dev
```

## License

{{license}}
"#
                    .to_string(),
                    is_binary: false,
                },
                TemplateFile {
                    path: ".gitignore".to_string(),
                    content: r#"/target
/Cargo.lock
.env
*.log
.DS_Store
"#
                    .to_string(),
                    is_binary: false,
                },
            ],
        }
    }

    /// Server-focused template
    fn server_template() -> Self {
        let mut template = Self::basic_template();
        template.name = "server".to_string();
        template.description =
            "Advanced MCP server template with multiple tools and resources".to_string();

        // Add server-specific files
        template.files.extend(vec![
                            TemplateFile {
                    path: "src/tools/mod.rs".to_string(),
                    content: r#"//! Tools module

pub mod echo;
pub mod info;
pub mod calculator;

use ultrafast_mcp::prelude::*;
use serde_json::Value;

/// Register all tools with the server
pub fn register_tools(server: &mut UltraFastServer) -> Result<()> {
    server
        .with_tool_handler(Arc::new(echo::EchoHandler))
        .with_tool_handler(Arc::new(info::InfoHandler))
        .with_tool_handler(Arc::new(calculator::CalculatorHandler));
    
    Ok(())
}
"#
                    .to_string(),
                    is_binary: false,
                },
                TemplateFile {
                    path: "src/tools/echo.rs".to_string(),
                    content: r#"//! Echo tool implementation

use ultrafast_mcp::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize)]
pub struct EchoRequest {
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct EchoResponse {
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub echo_count: usize,
}

pub struct EchoHandler;

#[async_trait::async_trait]
impl ToolHandler for EchoHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult> {
        if call.name != "echo" {
            return Err(MCPError::method_not_found(format!("Unknown tool: {}", call.name)));
        }

        let request: EchoRequest = serde_json::from_value(call.arguments.unwrap_or_default())
            .map_err(|e| MCPError::invalid_params(format!("Invalid request: {}", e)))?;

        let response = EchoResponse {
            message: request.message,
            timestamp: Utc::now(),
            echo_count: 1,
        };

        let response_text = serde_json::to_string_pretty(&response)
            .map_err(|e| MCPError::serialization_error(e.to_string()))?;

        Ok(ToolResult {
            content: vec![ToolContent::text(response_text)],
            is_error: None,
        })
    }

    async fn list_tools(&self, _request: ListToolsRequest) -> MCPResult<ListToolsResponse> {
        Ok(ListToolsResponse {
            tools: vec![Tool {
                name: "echo".to_string(),
                description: "Echo back input with timestamp".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "message": {
                            "type": "string",
                            "description": "Message to echo back"
                        }
                    },
                    "required": ["message"]
                }),
                output_schema: None,
            }],
            next_cursor: None,
        })
    }
}
"#
                    .to_string(),
                    is_binary: false,
                },
                TemplateFile {
                    path: "src/tools/info.rs".to_string(),
                    content: r#"//! Info tool implementation

use ultrafast_mcp::prelude::*;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub uptime: String,
    pub tools: Vec<String>,
    pub resources: Vec<String>,
}

pub struct InfoHandler;

#[async_trait::async_trait]
impl ToolHandler for InfoHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult> {
        if call.name != "info" {
            return Err(MCPError::method_not_found(format!("Unknown tool: {}", call.name)));
        }

        let info = ServerInfo {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: Some("{{description}}".to_string()),
            uptime: "0s".to_string(), // TODO: Calculate actual uptime
            tools: vec![
                "echo".to_string(),
                "info".to_string(),
                "calculate".to_string(),
            ],
            resources: vec![
                "status://server".to_string(),
                "config://server".to_string(),
            ],
        };

        let response_text = serde_json::to_string_pretty(&info)
            .map_err(|e| MCPError::serialization_error(e.to_string()))?;

        Ok(ToolResult {
            content: vec![ToolContent::text(response_text)],
            is_error: None,
        })
    }

    async fn list_tools(&self, _request: ListToolsRequest) -> MCPResult<ListToolsResponse> {
        Ok(ListToolsResponse {
            tools: vec![Tool {
                name: "info".to_string(),
                description: "Get server information and status".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
                output_schema: None,
            }],
            next_cursor: None,
        })
    }
}
"#
                    .to_string(),
                    is_binary: false,
                },
            TemplateFile {
                path: "src/tools/calculator.rs".to_string(),
                content: r#"//! Calculator tool implementation

use ultrafast_mcp::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CalculatorRequest {
    pub operation: String,
    pub a: f64,
    pub b: f64,
}

#[derive(Debug, Serialize)]
pub struct CalculatorResponse {
    pub result: f64,
    pub operation: String,
}

pub struct CalculatorHandler;

#[async_trait::async_trait]
impl ToolHandler for CalculatorHandler {
    async fn handle_tool_call(&self, call: ToolCall) -> MCPResult<ToolResult> {
        if call.name != "calculate" {
            return Err(MCPError::method_not_found(format!("Unknown tool: {}", call.name)));
        }

        let request: CalculatorRequest = serde_json::from_value(call.arguments.unwrap_or_default())
            .map_err(|e| MCPError::invalid_params(format!("Invalid request: {}", e)))?;

        let result = match request.operation.as_str() {
            "add" => request.a + request.b,
            "subtract" => request.a - request.b,
            "multiply" => request.a * request.b,
            "divide" => {
                if request.b == 0.0 {
                    return Err(MCPError::invalid_params("Division by zero".to_string()));
                }
                request.a / request.b
            }
            _ => return Err(MCPError::invalid_params(format!("Unknown operation: {}", request.operation))),
        };

        let response = CalculatorResponse {
            result,
            operation: request.operation,
        };

        let response_text = serde_json::to_string_pretty(&response)
            .map_err(|e| MCPError::serialization_error(e.to_string()))?;

        Ok(ToolResult {
            content: vec![ToolContent::text(response_text)],
            is_error: None,
        })
    }

    async fn list_tools(&self, _request: ListToolsRequest) -> MCPResult<ListToolsResponse> {
        Ok(ListToolsResponse {
            tools: vec![Tool {
                name: "calculate".to_string(),
                description: "Perform basic mathematical operations".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "operation": {
                            "type": "string",
                            "enum": ["add", "subtract", "multiply", "divide"],
                            "description": "Mathematical operation to perform"
                        },
                        "a": {
                            "type": "number",
                            "description": "First operand"
                        },
                        "b": {
                            "type": "number",
                            "description": "Second operand"
                        }
                    },
                    "required": ["operation", "a", "b"]
                }),
                output_schema: None,
            }],
            next_cursor: None,
        })
    }
}
"#
                .to_string(),
                is_binary: false,
            },
            TemplateFile {
                path: "src/resources/mod.rs".to_string(),
                content: r#"//! Resources module

pub mod status;
pub mod config;

use ultrafast_mcp::prelude::*;

/// Register all resources with the server
pub fn register_resources(server: &mut UltraFastServer) -> Result<()> {
    server
        .with_resource_handler(Arc::new(status::StatusResourceHandler))
        .with_resource_handler(Arc::new(config::ConfigResourceHandler));
    
    Ok(())
}
"#
                .to_string(),
                is_binary: false,
            },
            TemplateFile {
                path: "src/resources/status.rs".to_string(),
                content: r#"//! Status resource implementation

use ultrafast_mcp::prelude::*;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ServerStatus {
    pub status: String,
    pub uptime: String,
    pub version: String,
    pub tools_count: usize,
    pub resources_count: usize,
}

pub struct StatusResourceHandler;

#[async_trait::async_trait]
impl ResourceHandler for StatusResourceHandler {
    async fn read_resource(&self, request: ReadResourceRequest) -> MCPResult<ReadResourceResponse> {
        if request.uri != "status://server" {
            return Err(MCPError::resource_not_found(format!("Resource not found: {}", request.uri)));
        }

        let status = ServerStatus {
            status: "healthy".to_string(),
            uptime: "0s".to_string(), // TODO: Calculate actual uptime
            version: env!("CARGO_PKG_VERSION").to_string(),
            tools_count: 3, // echo, info, calculator
            resources_count: 2, // status, config
        };

        let content = serde_json::to_string_pretty(&status)
            .map_err(|e| MCPError::serialization_error(e.to_string()))?;

        Ok(ReadResourceResponse {
            contents: vec![ResourceContent::text(content)],
        })
    }

    async fn list_resources(&self, _request: ListResourcesRequest) -> MCPResult<ListResourcesResponse> {
        Ok(ListResourcesResponse {
            resources: vec![Resource {
                uri: "status://server".to_string(),
                name: Some("Server Status".to_string()),
                description: Some("Current server status and metrics".to_string()),
                mime_type: Some("application/json".to_string()),
            }],
            next_cursor: None,
        })
    }

    async fn list_resource_templates(&self, _request: ListResourceTemplatesRequest) -> MCPResult<ListResourceTemplatesResponse> {
        Ok(ListResourceTemplatesResponse {
            resource_templates: vec![],
            next_cursor: None,
        })
    }
}
"#
                .to_string(),
                is_binary: false,
            },
            TemplateFile {
                path: "src/resources/config.rs".to_string(),
                content: r#"//! Config resource implementation

use ultrafast_mcp::prelude::*;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ServerConfig {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub capabilities: Vec<String>,
    pub features: Vec<String>,
}

pub struct ConfigResourceHandler;

#[async_trait::async_trait]
impl ResourceHandler for ConfigResourceHandler {
    async fn read_resource(&self, request: ReadResourceRequest) -> MCPResult<ReadResourceResponse> {
        if request.uri != "config://server" {
            return Err(MCPError::resource_not_found(format!("Resource not found: {}", request.uri)));
        }

        let config = ServerConfig {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: Some("{{description}}".to_string()),
            capabilities: vec![
                "tools".to_string(),
                "resources".to_string(),
            ],
            features: vec![
                "echo".to_string(),
                "info".to_string(),
                "calculator".to_string(),
                "status".to_string(),
                "config".to_string(),
            ],
        };

        let content = serde_json::to_string_pretty(&config)
            .map_err(|e| MCPError::serialization_error(e.to_string()))?;

        Ok(ReadResourceResponse {
            contents: vec![ResourceContent::text(content)],
        })
    }

    async fn list_resources(&self, _request: ListResourcesRequest) -> MCPResult<ListResourcesResponse> {
        Ok(ListResourcesResponse {
            resources: vec![Resource {
                uri: "config://server".to_string(),
                name: Some("Server Configuration".to_string()),
                description: Some("Server configuration and capabilities".to_string()),
                mime_type: Some("application/json".to_string()),
            }],
            next_cursor: None,
        })
    }

    async fn list_resource_templates(&self, _request: ListResourceTemplatesRequest) -> MCPResult<ListResourceTemplatesResponse> {
        Ok(ListResourceTemplatesResponse {
            resource_templates: vec![],
            next_cursor: None,
        })
    }
}
"#
                .to_string(),
                is_binary: false,
            },
        ]);

        template
    }

    /// Client-focused template
    fn client_template() -> Self {
        Self {
            name: "client".to_string(),
            description: "MCP client template".to_string(),
            files: vec![
                TemplateFile {
                    path: "Cargo.toml".to_string(),
                    content: r#"[package]
name = "{{project_name}}"
version = "{{version}}"
edition = "2021"
description = "{{description}}"
authors = ["{{author}}"]
license = "{{license}}"

[dependencies]
ultrafast-mcp = { version = "0.1.0" }
ultrafast-mcp-client = { version = "0.1.0" }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"

[[bin]]
name = "{{project_name}}"
path = "src/main.rs"
"#
                    .to_string(),
                    is_binary: false,
                },
                TemplateFile {
                    path: "src/main.rs".to_string(),
                    content: r#"use ultrafast_mcp::prelude::*;
use ultrafast_mcp_client::{Client, ClientBuilder};
use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    info!("Starting {{project_name}} MCP client");
    
    let client = ClientBuilder::new()
        .with_info(ClientInfo {
            name: "{{project_name}}".to_string(),
            version: "{{version}}".to_string(),
        })
        .build()?;
    
    // Connect to server (stdio by default)
    client.connect_stdio().await?;
    
    // Initialize connection
    client.initialize().await?;
    
    // List available tools
    let tools = client.list_tools().await?;
    info!("Available tools: {:?}", tools);
    
    // List available resources
    let resources = client.list_resources().await?;
    info!("Available resources: {:?}", resources);
    
    Ok(())
}
"#
                    .to_string(),
                    is_binary: false,
                },
            ],
        }
    }

    /// Full-featured template with both server and client
    fn full_template() -> Self {
        let mut template = Self::server_template();
        template.name = "full".to_string();
        template.description =
            "Full-featured MCP project with server, client, and examples".to_string();

        // Add client files and examples
        template.files.extend(vec![
            TemplateFile {
                path: "client/Cargo.toml".to_string(),
                content: r#"[package]
name = "{{project_name}}-client"
version = "{{version}}"
edition = "2021"
description = "{{description}} - Client"
authors = ["{{author}}"]
license = "{{license}}"

[dependencies]
ultrafast-mcp = { version = "0.1.0" }
ultrafast-mcp-client = { version = "0.1.0" }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
clap = { version = "4.0", features = ["derive"] }

[[bin]]
name = "{{project_name}}-client"
path = "src/main.rs"
"#
                .to_string(),
                is_binary: false,
            },
            TemplateFile {
                path: "client/src/main.rs".to_string(),
                content: r#"use ultrafast_mcp::prelude::*;
use ultrafast_mcp_client::UltraFastClient;
use anyhow::Result;
use tracing::info;
use clap::Parser;

#[derive(Parser)]
#[command(name = "{{project_name}}-client")]
#[command(about = "MCP client for {{project_name}}")]
struct Args {
    /// Server command to connect to
    #[arg(short, long, default_value = "cargo run")]
    server: String,
    
    /// Tool to call
    #[arg(short, long)]
    tool: Option<String>,
    
    /// Tool arguments (JSON)
    #[arg(short, long)]
    args: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();
    info!("Starting {{project_name}} MCP client");
    
    let client_info = ClientInfo {
        name: "{{project_name}}-client".to_string(),
        version: "{{version}}".to_string(),
        authors: None,
        description: Some("Client for {{project_name}} MCP server".to_string()),
        homepage: None,
        repository: None,
        license: None,
    };
    
    let capabilities = ClientCapabilities::default();
    let client = UltraFastClient::new(client_info, capabilities);
    
    // Connect to server using STDIO
    client.connect_stdio().await?;
    
    // Initialize connection
    client.initialize().await?;
    
    // List available tools
    let tools = client.list_tools().await?;
    info!("Available tools: {:?}", tools);
    
    // Call specific tool if requested
    if let Some(tool_name) = args.tool {
        let args_value = if let Some(args_str) = args.args {
            serde_json::from_str(&args_str)?
        } else {
            serde_json::json!({})
        };
        
        let tool_call = ToolCall {
            name: tool_name,
            arguments: Some(args_value),
        };
        
        let result = client.call_tool(tool_call).await?;
        println!("Tool result: {:?}", result);
    }
    
    Ok(())
}
"#
                .to_string(),
                is_binary: false,
            },
            TemplateFile {
                path: "examples/basic_usage.rs".to_string(),
                content: r#"//! Basic usage example for {{project_name}}

use ultrafast_mcp::prelude::*;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // This example shows how to use the {{project_name}} MCP server
    // from a client application
    
    println!("{{project_name}} Basic Usage Example");
    println!("===================================");
    println!();
    println!("1. Start the server:");
    println!("   cargo run");
    println!();
    println!("2. In another terminal, run the client:");
    println!("   cd client && cargo run");
    println!();
    println!("3. Or use the MCP CLI to test:");
    println!("   mcp test --server 'cargo run'");
    println!();
    println!("Available tools:");
    println!("  - echo: Echo back input with timestamp");
    println!("  - info: Get server information");
    println!("  - calculate: Perform mathematical operations");
    println!();
    println!("Available resources:");
    println!("  - status://server: Server status and metrics");
    println!("  - config://server: Server configuration");
    
    Ok(())
}
"#
                .to_string(),
                is_binary: false,
            },
            TemplateFile {
                path: "examples/advanced_usage.rs".to_string(),
                content: r#"//! Advanced usage example for {{project_name}}

use ultrafast_mcp::prelude::*;
use ultrafast_mcp_client::UltraFastClient;
use anyhow::Result;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    println!("{{project_name}} Advanced Usage Example");
    println!("======================================");
    
    // Create client
    let client_info = ClientInfo {
        name: "advanced-example".to_string(),
        version: "1.0.0".to_string(),
        authors: None,
        description: Some("Advanced usage example".to_string()),
        homepage: None,
        repository: None,
        license: None,
    };
    
    let client = UltraFastClient::new(client_info, ClientCapabilities::default());
    
    // Connect and initialize
    client.connect_stdio().await?;
    client.initialize().await?;
    
    // Example 1: Echo tool
    println!("\\n1. Testing echo tool:");
    let echo_call = ToolCall {
        name: "echo".to_string(),
        arguments: Some(json!({
            "message": "Hello from advanced example!"
        })),
    };
    
    let echo_result = client.call_tool(echo_call).await?;
    println!("Echo result: {:?}", echo_result);
    
    // Example 2: Calculator tool
    println!("\\n2. Testing calculator tool:");
    let calc_call = ToolCall {
        name: "calculate".to_string(),
        arguments: Some(json!({
            "operation": "multiply",
            "a": 15.5,
            "b": 2.0
        })),
    };
    
    let calc_result = client.call_tool(calc_call).await?;
    println!("Calculator result: {:?}", calc_result);
    
    // Example 3: Server info
    println!("\\n3. Testing info tool:");
    let info_call = ToolCall {
        name: "info".to_string(),
        arguments: Some(json!({})),
    };
    
    let info_result = client.call_tool(info_call).await?;
    println!("Info result: {:?}", info_result);
    
    // Example 4: Read status resource
    println!("\\n4. Reading status resource:");
    let status_request = ReadResourceRequest {
        uri: "status://server".to_string(),
    };
    
    let status_result = client.read_resource(status_request).await?;
    println!("Status result: {:?}", status_result);
    
    println!("\\nAdvanced example completed successfully!");
    
    Ok(())
}
"#
                .to_string(),
                is_binary: false,
            },
        ]);

        template
    }
}
