use anyhow::{Result, Context};
use std::collections::HashMap;
use std::path::Path;
use crate::config::Config;
use serde::{Deserialize, Serialize};
use base64::Engine;

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
            .with_context(|| format!("Failed to read template config: {:?}", config_path))?;
            
        let template_config: TemplateConfig = if config_path.extension().unwrap_or_default() == "toml" {
            toml::from_str(&config_content)
                .with_context(|| "Failed to parse template.toml")?
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
                        .with_context(|| format!("Failed to read binary file: {:?}", file_path))?;
                    base64::engine::general_purpose::STANDARD.encode(&bytes)
                } else {
                    // For text files, read as string
                    fs::read_to_string(&file_path)
                        .with_context(|| format!("Failed to read text file: {:?}", file_path))?
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
            let placeholder = format!("{{{{{}}}}}", key);
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

[[bin]]
name = "{{project_name}}"
path = "src/main.rs"
"#.to_string(),
                    is_binary: false,
                },
                TemplateFile {
                    path: "src/main.rs".to_string(),
                    content: r#"use ultrafast_mcp::prelude::*;
use ultrafast_mcp_server::{Server, ServerBuilder};
use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    info!("Starting {{project_name}} MCP server");
    
    let server = ServerBuilder::new()
        .with_info(ServerInfo {
            name: "{{project_name}}".to_string(),
            version: "{{version}}".to_string(),
        })
        .with_tool("echo", "Echo back the input", echo_handler)
        .with_resource("info://server", "Server information", info_handler)
        .build()?;
    
    server.run_stdio().await?;
    
    Ok(())
}

async fn echo_handler(args: serde_json::Value) -> Result<serde_json::Value> {
    Ok(args)
}

async fn info_handler(_uri: String) -> Result<Resource> {
    Ok(Resource {
        uri: "info://server".to_string(),
        name: "Server Information".to_string(),
        description: Some("Information about this MCP server".to_string()),
        mime_type: Some("application/json".to_string()),
        text: Some(serde_json::json!({
            "name": "{{project_name}}",
            "version": "{{version}}",
            "description": "{{description}}"
        }).to_string()),
        blob: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_echo_handler() {
        let input = serde_json::json!({"message": "test"});
        let result = echo_handler(input.clone()).await.unwrap();
        assert_eq!(result, input);
    }
}
"#.to_string(),
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
"#.to_string(),
                    is_binary: false,
                },
                TemplateFile {
                    path: ".gitignore".to_string(),
                    content: r#"/target
/Cargo.lock
.env
*.log
.DS_Store
"#.to_string(),
                    is_binary: false,
                },
            ],
        }
    }

    /// Server-focused template
    fn server_template() -> Self {
        let mut template = Self::basic_template();
        template.name = "server".to_string();
        template.description = "Advanced MCP server template with multiple tools and resources".to_string();
        
        // TODO: Add more server-specific files
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
"#.to_string(),
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
"#.to_string(),
                    is_binary: false,
                },
            ],
        }
    }

    /// Full-featured template with both server and client
    fn full_template() -> Self {
        let mut template = Self::server_template();
        template.name = "full".to_string();
        template.description = "Full-featured MCP project with server, client, and examples".to_string();
        
        // TODO: Add client files and examples
        template
    }
}
