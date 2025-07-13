use crate::config::Config;
use anyhow::{Context, Result};
use clap::Args;
use colored::*;

/// Generate project scaffolding
#[derive(Debug, Args)]
pub struct GenerateArgs {
    /// What to generate
    #[arg(value_name = "TYPE")]
    pub generate_type: String,

    /// Name for the generated item
    #[arg(short, long)]
    pub name: Option<String>,

    /// Template to use
    #[arg(short, long)]
    pub template: Option<String>,

    /// Output directory
    #[arg(short, long)]
    pub output: Option<std::path::PathBuf>,
}

pub async fn execute(args: GenerateArgs, _config: Option<Config>) -> Result<()> {
    println!("{}", "Generating project scaffolding...".green().bold());

    match args.generate_type.as_str() {
        "tool" => generate_tool(&args).await,
        "resource" => generate_resource(&args).await,
        "client" => generate_client(&args).await,
        "server" => generate_server(&args).await,
        _ => {
            anyhow::bail!("Unknown generation type: {}", args.generate_type);
        }
    }
}

async fn generate_tool(args: &GenerateArgs) -> Result<()> {
    let tool_name = args
        .name
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("Tool name is required for tool generation"))?;

    println!("🔧 Generating tool: {tool_name}");

    // Create tools directory if it doesn't exist
    std::fs::create_dir_all("src/tools").context("Failed to create src/tools directory")?;

    let snake_case_name = tool_name.to_lowercase().replace('-', "_");
    let pascal_case_name = snake_case_name
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                }
            }
        })
        .collect::<String>();

    let tool_template = format!(
        r#"//! {tool_name} tool implementation

use serde::{{Deserialize, Serialize}};
use ultrafast_mcp::prelude::*;

/// Request structure for the {tool_name} tool
#[derive(Debug, Deserialize)]
pub struct {pascal_case_name}Request {{
    /// Input message for the tool
    pub message: String,
    /// Optional configuration parameters
    pub options: Option<{pascal_case_name}Options>,
}}

/// Optional configuration for the {tool_name} tool
#[derive(Debug, Deserialize)]
pub struct {pascal_case_name}Options {{
    /// Enable verbose output
    pub verbose: Option<bool>,
    /// Custom formatting options
    pub format: Option<String>,
}}

/// Response structure for the {tool_name} tool
#[derive(Debug, Serialize)]
pub struct {pascal_case_name}Response {{
    /// The result of the tool execution
    pub result: String,
    /// Execution metadata
    pub metadata: {pascal_case_name}Metadata,
}}

/// Metadata for tool execution
#[derive(Debug, Serialize)]
pub struct {pascal_case_name}Metadata {{
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Number of operations performed
    pub operations_count: usize,
}}

/// Main handler function for the {tool_name} tool
pub async fn {snake_case_name}(
    req: {pascal_case_name}Request,
    ctx: Context,
) -> Result<{pascal_case_name}Response, Box<dyn std::error::Error + Send + Sync>> {{
    let start_time = std::time::Instant::now();
    
    // Log the incoming request
    ctx.log_info(&format!("Processing {tool_name} request: {{}}", req.message)).await?;
    
    // Process the request based on options
    let verbose = req.options.as_ref()
        .and_then(|opts| opts.verbose)
        .unwrap_or(false);
    
    if verbose {{
        ctx.log_info("Verbose mode enabled").await?;
    }}
    
    // Implement your actual tool logic here
    let result = match req.message.as_str() {{
        "" => return Err(anyhow::anyhow!("Message cannot be empty")),
        msg if msg.len() > 1000 => return Err(anyhow::anyhow!("Message too long (max 1000 characters)")),
        msg => {{
            // Add your custom processing logic here
            let processed = if let Some(options) = &req.options {{
                if options.verbose.unwrap_or(false) {{
                    format!("[VERBOSE] Processed: {{}}", msg)
                }} else {{
                    format!("Processed: {{}}", msg)
                }}
            }} else {{
                format!("Processed: {{}}", msg)
            }};
            
            // Apply format if specified
            if let Some(options) = &req.options {{
                if let Some(format) = &options.format {{
                    match format.as_str() {{
                        "json" => serde_json::to_string_pretty(&serde_json::json!({{
                            "result": processed,
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        }}))?,
                        "xml" => format!("<result>{{}}</result>", processed),
                        _ => processed
                    }}
                }} else {{
                    processed
                }}
            }} else {{
                processed
            }}
        }}
    }};
    
    let processing_time = start_time.elapsed().as_millis() as u64;
    
    // Report progress
    ctx.progress("Tool execution completed", 1.0, Some(1.0)).await?;
    
    Ok({pascal_case_name}Response {{
        result,
        metadata: {pascal_case_name}Metadata {{
            processing_time_ms: processing_time,
            operations_count: 1,
        }},
    }})
}}

#[cfg(test)]
mod tests {{
    use super::*;
    
    #[tokio::test]
    async fn test_{snake_case_name}_basic_functionality() {{
        let ctx = Context::new();
        let request = {pascal_case_name}Request {{
            message: "test input".to_string(),
            options: None,
        }};
        
        let response = {snake_case_name}(request, ctx).await.unwrap();
        assert_eq!(response.result, "Processed: test input");
        assert!(response.metadata.processing_time_ms >= 0);
    }}
    
    #[tokio::test]
    async fn test_{snake_case_name}_with_options() {{
        let ctx = Context::new();
        let request = {pascal_case_name}Request {{
            message: "test input".to_string(),
            options: Some({pascal_case_name}Options {{
                verbose: Some(true),
                format: Some("json".to_string()),
            }}),
        }};
        
        let response = {snake_case_name}(request, ctx).await.unwrap();
        assert!(response.result.contains("test input"));
    }}
}}
"#
    );

    let file_path = format!("src/tools/{snake_case_name}.rs");
    std::fs::write(&file_path, tool_template).context("Failed to write tool file")?;

    // Update mod.rs to include the new tool
    let mod_file_path = "src/tools/mod.rs";
    let mod_content = if std::path::Path::new(mod_file_path).exists() {
        std::fs::read_to_string(mod_file_path)?
    } else {
        "//! Tools module\n\n".to_string()
    };

    if !mod_content.contains(&format!("pub mod {snake_case_name};")) {
        let updated_mod_content = format!("{mod_content}pub mod {snake_case_name};\n");
        std::fs::write(mod_file_path, updated_mod_content)
            .context("Failed to update tools/mod.rs")?;
    }

    println!("✅ Generated tool template at {file_path}");
    println!("📝 Updated {mod_file_path}");
    println!("\n🔧 To register this tool in your server, add:");
    println!(
        "   .tool(\"{tool_name}\", tools::{snake_case_name}::{snake_case_name});"
    );

    Ok(())
}

async fn generate_resource(args: &GenerateArgs) -> Result<()> {
    let resource_name = args
        .name
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("Resource name is required for resource generation"))?;

    println!("📄 Generating resource: {resource_name}");

    // Create resources directory if it doesn't exist
    std::fs::create_dir_all("src/resources").context("Failed to create src/resources directory")?;

    let snake_case_name = resource_name.to_lowercase().replace('-', "_");
    let pascal_case_name = snake_case_name
        .split('_')
        .map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<String>();

    // Write the resource file
    let resource_content = format!(
        r#"use anyhow::Result;
use ultrafast_mcp_core::{{
    protocol::{{Resource, ResourceContents, ResourceTemplate}},
    types::Uri,
}};

/// {resource_name} resource handler
pub struct {pascal_case_name}Resource {{
    base_uri: Uri,
}}

impl {pascal_case_name}Resource {{
    /// Create a new {resource_name} resource
    pub fn new(base_uri: Uri) -> Self {{
        Self {{ base_uri }}
    }}

    /// List all available {resource_name} resources
    pub async fn list_resources(&self) -> Result<Vec<Resource>> {{
        // Implement your resource listing logic here
        // This could scan directories, query databases, or fetch from APIs
        
        let mut resources = Vec::new();
        
        // Example: List files in a directory
        if let Ok(entries) = std::fs::read_dir(self.base_uri.path()) {{
            for entry in entries {{
                if let Ok(entry) = entry {{
                    let path = entry.path();
                    if let Some(file_name) = path.file_name() {{
                        if let Some(name_str) = file_name.to_str() {{
                            let uri = format!("{{}}/{{}}", self.base_uri, name_str).parse()?;
                            let mime_type = if path.extension().and_then(|s| s.to_str()) == Some("json") {{
                                Some("application/json".to_string())
                            }} else if path.extension().and_then(|s| s.to_str()) == Some("txt") {{
                                Some("text/plain".to_string())
                            }} else {{
                                Some("application/octet-stream".to_string())
                            }};
                            
                            resources.push(Resource {{
                                uri,
                                name: Some(name_str.to_string()),
                                description: Some(format!("{resource_name} resource file", name_str)),
                                mime_type,
                            }});
                        }}
                    }}
                }}
            }}
        }}
        
        // Add default example resource if no files found
        if resources.is_empty() {{
            resources.push(Resource {{
                uri: format!("{{}}/{{}}", self.base_uri, "example").parse()?,
                name: Some("Example {resource_name}".to_string()),
                description: Some("An example {resource_name} resource".to_string()),
                mime_type: Some("text/plain".to_string()),
            }});
        }}
        
        Ok(resources)
    }}

    /// Read a specific {resource_name} resource
    pub async fn read_resource(&self, uri: &Uri) -> Result<ResourceContents> {{
        // Implement your resource reading logic here
        // Parse the URI to determine what resource is being requested
        
        let path = uri.path();
        let content = match path {{
            "/example" => {{
                ResourceContents::Text {{
                    uri: uri.clone(),
                    mime_type: Some("text/plain".to_string()),
                    text: "This is an example {resource_name} resource.".to_string(),
                }}
            }}
            _ => {{
                // Try to read from file system
                let file_path = std::path::Path::new(path);
                if file_path.exists() && file_path.is_file() {{
                    let content = std::fs::read_to_string(file_path)?;
                    let mime_type = if let Some(ext) = file_path.extension() {{
                        match ext.to_str() {{
                            Some("json") => Some("application/json".to_string()),
                            Some("txt") => Some("text/plain".to_string()),
                            Some("md") => Some("text/markdown".to_string()),
                            Some("html") => Some("text/html".to_string()),
                            _ => Some("application/octet-stream".to_string()),
                        }}
                    }} else {{
                        Some("text/plain".to_string())
                    }};
                    
                    ResourceContents::Text {{
                        uri: uri.clone(),
                        mime_type,
                        text: content,
                    }}
                }} else {{
                    anyhow::bail!("Resource not found: {{}}", uri);
                }}
            }}
        }};
        
        Ok(content)
    }}

    /// Subscribe to resource changes (if supported)
    pub async fn subscribe(&self, uri: &Uri) -> Result<()> {{
        // Implement resource change subscription
        // This could set up file watchers, database triggers, or API polling
        
        let path = uri.path();
        if let Ok(metadata) = std::fs::metadata(path) {{
            // Store subscription info (in a real implementation, this would be persistent)
            println!("Subscribed to changes for resource: {{}} (last modified: {{:?}})", uri, metadata.modified()?);
        }} else {{
            println!("Subscribed to changes for resource: {{}} (file not found)", uri);
        }}
        Ok(())
    }}

    /// Unsubscribe from resource changes
    pub async fn unsubscribe(&self, uri: &Uri) -> Result<()> {{
        // Implement resource change unsubscription
        // This would clean up watchers, triggers, or polling
        
        println!("Unsubscribed from changes for resource: {{}}", uri);
        Ok(())
    }}

    /// List available resource templates
    pub async fn list_templates(&self) -> Result<Vec<ResourceTemplate>> {{
        // Implement resource template listing
        // This could provide parameterized resource patterns
        
        let templates = vec![
            ResourceTemplate {{
                uri_template: format!("{{}}/{{}}", self.base_uri, "{{name}}"),
                name: Some("Dynamic {resource_name} Resource".to_string()),
                description: Some("A parameterized {resource_name} resource".to_string()),
                mime_type: Some("text/plain".to_string()),
            }},
            ResourceTemplate {{
                uri_template: format!("{{}}/{{}}", self.base_uri, "{{type}}/{{id}}"),
                name: Some("Typed {snake_case_name} Resource".to_string()),
                description: Some("A typed {resource_name} resource with ID".to_string()),
                mime_type: Some("application/json".to_string()),
            }},
        ];
        
        Ok(templates)
    }}
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[tokio::test]
    async fn test_{pascal_case_name}_resource() {{
        let base_uri = "resource://{resource_name}".parse().unwrap();
        let resource = {pascal_case_name}Resource::new(base_uri);
        
        let resources = resource.list_resources().await.unwrap();
        assert!(!resources.is_empty());
    }}
}}
"#
    );

    let resource_file = format!("src/resources/{snake_case_name}.rs");
    std::fs::write(&resource_file, resource_content).context("Failed to write resource file")?;

    // Update mod.rs file to include the new resource
    let mod_file = "src/resources/mod.rs";
    let mod_content = if std::path::Path::new(mod_file).exists() {
        std::fs::read_to_string(mod_file)?
    } else {
        String::new()
    };

    if !mod_content.contains(&format!("pub mod {snake_case_name};")) {
        let new_mod_content = if mod_content.is_empty() {
            format!("pub mod {snake_case_name};\n")
        } else {
            format!("{}\npub mod {};\n", mod_content.trim(), snake_case_name)
        };
        std::fs::write(mod_file, new_mod_content).context("Failed to update resources/mod.rs")?;
    }

    println!("✅ Generated resource: {resource_file}");
    println!("✅ Updated: {mod_file}");
    println!("\n💡 Next steps:");
    println!("   1. Implement the TODO sections in {resource_file}");
    println!("   2. Register the resource in your server");
    println!("   3. Add any required dependencies to Cargo.toml");

    Ok(())
}

async fn generate_client(args: &GenerateArgs) -> Result<()> {
    let client_name = args
        .name
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("Client name is required for client generation"))?;

    let output_dir = args
        .output
        .as_deref()
        .unwrap_or_else(|| std::path::Path::new("."));

    println!("👤 Generating client: {client_name}");

    let project_dir = output_dir.join(client_name);
    std::fs::create_dir_all(&project_dir).context("Failed to create project directory")?;

    let _snake_case_name = client_name.to_lowercase().replace('-', "_");

    // Generate Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{client_name}"
version = "0.1.0"
edition = "2021"

[dependencies]
ultrafast-mcp-client = {{ path = "../crates/ultrafast-mcp-client" }}
ultrafast-mcp-core = {{ path = "../crates/ultrafast-mcp-core" }}
ultrafast-mcp-transport = {{ path = "../crates/ultrafast-mcp-transport" }}
tokio = {{ version = "1.0", features = ["full"] }}
anyhow = "1.0"
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
clap = {{ version = "4.0", features = ["derive"] }}
colored = "2.0"
"#
    );

    std::fs::write(project_dir.join("Cargo.toml"), cargo_toml)
        .context("Failed to write Cargo.toml")?;

    // Generate main.rs
    let main_rs = format!(
        r#"use anyhow::Result;
use clap::{{Parser, Subcommand}};
use colored::*;
use ultrafast_mcp_client::{{ClientBuilder, Connection}};
use ultrafast_mcp_core::protocol::{{CallToolRequest, ListToolsRequest, Tool}};
use ultrafast_mcp_transport::{{stdio::StdioTransport, streamable_http::server::HttpTransportServer}};

#[derive(Parser)]
#[command(name = "{client_name}", about = "A custom MCP client")]
struct Cli {{
    #[command(subcommand)]
    command: Commands,
}}

#[derive(Subcommand)]
enum Commands {{
    /// Connect to a server via STDIO
    Stdio {{
        /// Server command to execute
        #[arg(long)]
        command: String,
        /// Server arguments
        #[arg(long)]
        args: Vec<String>,
    }},
    /// Connect to a server via HTTP
    Http {{
        /// Server URL
        #[arg(long)]
        url: String,
    }},
    /// List available tools
    Tools {{
        /// Transport type
        #[arg(long, default_value = "stdio")]
        transport: String,
        /// Server endpoint
        #[arg(long)]
        endpoint: String,
    }},
    /// Call a specific tool
    Call {{
        /// Tool name to call
        #[arg(long)]
        tool: String,
        /// Tool arguments as JSON
        #[arg(long)]
        args: Option<String>,
        /// Transport type
        #[arg(long, default_value = "stdio")]
        transport: String,
        /// Server endpoint
        #[arg(long)]
        endpoint: String,
    }},
}}

#[tokio::main]
async fn main() -> Result<()> {{
    let cli = Cli::parse();
    
    match cli.command {{
        Commands::Stdio {{ command, args }} => {{
            connect_stdio(&command, &args).await?;
        }}
        Commands::Http {{ url }} => {{
            connect_http(&url).await?;
        }}
        Commands::Tools {{ transport, endpoint }} => {{
            list_tools(&transport, &endpoint).await?;
        }}
        Commands::Call {{ tool, args, transport, endpoint }} => {{
            call_tool(&tool, args.as_deref(), &transport, &endpoint).await?;
        }}
    }}
    
    Ok(())
}}

async fn connect_stdio(command: &str, args: &[String]) -> Result<()> {{
    println!("{{}}", "Connecting via STDIO...".green());
    
    let transport = StdioTransport::new(command, args)?;
    let mut client = ClientBuilder::new()
        .with_transport(transport)
        .build()
        .await?;
    
    println!("✅ Connected to server");
    
    // Initialize the connection
    client.initialize("{client_name}".to_string(), "0.1.0".to_string()).await?;
    
    println!("🎉 Client initialized successfully");
    
    Ok(())
}}

async fn connect_http(url: &str) -> Result<()> {{
    println!("{{}}", "Connecting via HTTP...".green());
    
    let transport = HttpTransport::new(url.parse()?)?;
    let mut client = ClientBuilder::new()
        .with_transport(transport)
        .build()
        .await?;
    
    println!("✅ Connected to server at {{}}", url);
    
    // Initialize the connection
    client.initialize("{client_name}".to_string(), "0.1.0".to_string()).await?;
    
    println!("🎉 Client initialized successfully");
    
    Ok(())
}}

async fn list_tools(transport: &str, endpoint: &str) -> Result<()> {{
    println!("{{}}", "Listing available tools...".green());
    
    let mut client = create_client(transport, endpoint).await?;
    
    let request = ListToolsRequest {{
        cursor: None,
    }};
    
    let response = client.list_tools(request).await?;
    
    if response.tools.is_empty() {{
        println!("No tools available");
    }} else {{
        println!("📋 Available tools:");
        for tool in response.tools {{
            println!("  🔧 {{}}", tool.name.green());
            if let Some(description) = tool.description {{
                println!("     {{}}", description.dimmed());
            }}
        }}
    }}
    
    Ok(())
}}

async fn call_tool(tool_name: &str, args: Option<&str>, transport: &str, endpoint: &str) -> Result<()> {{
    println!("{{}}", format!("Calling tool: {{}}", tool_name).green());
    
    let mut client = create_client(transport, endpoint).await?;
    
    let arguments = match args {{
        Some(json_str) => serde_json::from_str(json_str)?,
        None => serde_json::Value::Object(serde_json::Map::new()),
    }};
    
    let request = CallToolRequest {{
        name: tool_name.to_string(),
        arguments,
    }};
    
    let response = client.call_tool(request).await?;
    
    println!("✅ Tool executed successfully");
    println!("📄 Result:");
    for content in response.content {{
        match content {{
            ultrafast_mcp_core::protocol::ToolResult::Text {{ text, .. }} => {{
                println!("{{}}", text);
            }}
            ultrafast_mcp_core::protocol::ToolResult::ImageData {{ data, mime_type, .. }} => {{
                println!("🖼️  Image result ({{}}): {{}} bytes", mime_type, data.len());
            }}
            ultrafast_mcp_core::protocol::ToolResult::EmbeddedResource {{ resource, .. }} => {{
                println!("📎 Embedded resource: {{}}", resource.uri);
            }}
        }}
    }}
    
    Ok(())
}}

async fn create_client(transport: &str, endpoint: &str) -> Result<ultrafast_mcp_client::Client> {{
    match transport {{
        "stdio" => {{
            let parts: Vec<&str> = endpoint.split_whitespace().collect();
            let command = parts[0];
            let args = &parts[1..];
            
            let transport = StdioTransport::new(command, args)?;
            let mut client = ClientBuilder::new()
                .with_transport(transport)
                .build()
                .await?;
                
            client.initialize("{client_name}".to_string(), "0.1.0".to_string()).await?;
            Ok(client)
        }}
        "http" => {{
            let transport = HttpTransport::new(endpoint.parse()?)?;
            let mut client = ClientBuilder::new()
                .with_transport(transport)
                .build()
                .await?;
                
            client.initialize("{client_name}".to_string(), "0.1.0".to_string()).await?;
            Ok(client)
        }}
        _ => anyhow::bail!("Unsupported transport type: {{}}", transport),
    }}
}}
"#
    );

    std::fs::create_dir_all(project_dir.join("src"))?;
    std::fs::write(project_dir.join("src/main.rs"), main_rs).context("Failed to write main.rs")?;

    // Generate README.md
    let readme = format!(
        r#"# {client_name}

A custom MCP (Model Context Protocol) client built with Ultrafast MCP.

## Usage

### List available tools from a server

```bash
# Via STDIO
cargo run -- tools --transport stdio --endpoint "your-server-command arg1 arg2"

# Via HTTP
cargo run -- tools --transport http --endpoint "http://localhost:8080"
```

### Call a tool

```bash
# Call a tool with arguments
cargo run -- call --tool "tool_name" --args '{{"param": "value"}}' --transport stdio --endpoint "your-server-command"
```

### Connect to servers

```bash
# Connect via STDIO
cargo run -- stdio --command "your-server-command" --args arg1 --args arg2

# Connect via HTTP
cargo run -- http --url "http://localhost:8080"
```

## Configuration

The client can connect to MCP servers using:
- **STDIO transport**: For local processes
- **HTTP transport**: For remote servers

## Development

```bash
# Build the client
cargo build

# Run with debugging
RUST_LOG=debug cargo run -- [command]
```

## Examples

See the main MCP workspace examples for server implementations to test with.
"#
    );

    std::fs::write(project_dir.join("README.md"), readme).context("Failed to write README.md")?;

    println!("✅ Generated client project: {}", project_dir.display());
    println!("✅ Created:");
    println!("   📄 Cargo.toml");
    println!("   📄 src/main.rs");
    println!("   📄 README.md");
    println!("\n💡 Next steps:");
    println!("   1. cd {client_name}");
    println!("   2. cargo build");
    println!("   3. cargo run -- --help");

    Ok(())
}

async fn generate_server(args: &GenerateArgs) -> Result<()> {
    let server_name = args.name.as_deref().unwrap_or("mcp-server");

    println!("🖥️ Generating server: {server_name}");

    // Create project directory
    std::fs::create_dir_all(server_name).context("Failed to create server directory")?;

    let snake_case_name = server_name.to_lowercase().replace('-', "_");

    // Generate Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{server_name}"
version = "0.1.0"
edition = "2021"

[dependencies]
ultrafast-mcp = {{ path = "../ultrafast-mcp" }}
tokio = {{ version = "1.0", features = ["full"] }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"

[lib]
name = "{snake_case_name}"
path = "src/lib.rs"

[[bin]]
name = "{server_name}"
path = "src/main.rs"
"#
    );

    std::fs::write(format!("{server_name}/Cargo.toml"), cargo_toml)
        .context("Failed to write Cargo.toml")?;

    // Create src directory
    std::fs::create_dir_all(format!("{server_name}/src"))
        .context("Failed to create src directory")?;

    // Generate main.rs
    let main_rs = format!(
        r#"//! {server_name} MCP Server
//! 
//! A Model Context Protocol server implementation.

use anyhow::Result;
use tracing::{{info, warn}};
use ultrafast_mcp::prelude::*;

mod tools;

#[tokio::main]
async fn main() -> Result<()> {{
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    info!("Starting {server_name} MCP Server");

    // Create the server
    let server = UltraFastServer::new("{server_name}")
        .tool("echo", tools::echo::echo)
        .tool("info", tools::info::info)
        .resource("status", tools::resources::status)
        .run_stdio()
        .await;

    match server {{
        Ok(_) => {{
            info!("Server stopped gracefully");
            Ok(())
        }}
        Err(e) => {{
            warn!("Server error: {{}}", e);
            Err(e)
        }}
    }}
}}
"#
    );

    std::fs::write(format!("{server_name}/src/main.rs"), main_rs)
        .context("Failed to write main.rs")?;

    // Generate lib.rs
    let lib_rs = format!(
        r#"//! {server_name} Library
//! 
//! Core functionality for the {server_name} MCP server.

pub mod tools;

pub use tools::*;
"#
    );

    std::fs::write(format!("{server_name}/src/lib.rs"), lib_rs)
        .context("Failed to write lib.rs")?;

    // Create tools directory and files
    std::fs::create_dir_all(format!("{server_name}/src/tools"))
        .context("Failed to create tools directory")?;

    // Generate tools/mod.rs
    let tools_mod_rs = r#"//! Tools module

pub mod echo;
pub mod info;
pub mod resources;
"#;

    std::fs::write(format!("{server_name}/src/tools/mod.rs"), tools_mod_rs)
        .context("Failed to write tools/mod.rs")?;

    // Generate echo tool
    let echo_tool = r#"//! Echo tool implementation

use serde::{Deserialize, Serialize};
use ultrafast_mcp::prelude::*;

#[derive(Debug, Deserialize)]
pub struct EchoRequest {
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct EchoResponse {
    pub echo: String,
    pub timestamp: String,
}

pub async fn echo(
    req: EchoRequest,
    ctx: Context,
) -> Result<EchoResponse, Box<dyn std::error::Error + Send + Sync>> {
    ctx.log_info(&format!("Echo request: {}", req.message)).await?;
    
    Ok(EchoResponse {
        echo: req.message,
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}
"#;

    std::fs::write(format!("{server_name}/src/tools/echo.rs"), echo_tool)
        .context("Failed to write echo tool")?;

    // Generate info tool
    let info_tool = r#"//! Server info tool

use serde::{Deserialize, Serialize};
use ultrafast_mcp::prelude::*;

#[derive(Debug, Deserialize)]
pub struct InfoRequest {}

#[derive(Debug, Serialize)]
pub struct InfoResponse {
    pub server_name: String,
    pub version: String,
    pub uptime: String,
    pub capabilities: Vec<String>,
}

pub async fn info(
    _req: InfoRequest,
    ctx: Context,
) -> Result<InfoResponse, Box<dyn std::error::Error + Send + Sync>> {
    ctx.log_info("Server info requested").await?;
    
    Ok(InfoResponse {
        server_name: env!("CARGO_PKG_NAME").to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime: format!("{:?}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default()),
        capabilities: vec![
            "tools".to_string(),
            "resources".to_string(),
            "stdio".to_string(),
        ],
    })
}
"#;

    std::fs::write(format!("{server_name}/src/tools/info.rs"), info_tool)
        .context("Failed to write info tool")?;

    // Generate resources module
    let resources_rs = r#"//! Resource handlers

use serde::Serialize;
use ultrafast_mcp::prelude::*;

#[derive(Debug, Serialize)]
pub struct StatusResource {
    pub status: String,
    pub timestamp: String,
    pub version: String,
}

pub async fn status(
    _uri: String,
    ctx: Context,
) -> Result<StatusResource, Box<dyn std::error::Error + Send + Sync>> {
    ctx.log_info("Status resource requested").await?;
    
    Ok(StatusResource {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
"#;

    std::fs::write(
        format!("{server_name}/src/tools/resources.rs"),
        resources_rs,
    )
    .context("Failed to write resources module")?;

    // Generate README.md
    let readme = format!(
        r#"# {server_name} MCP Server

A Model Context Protocol (MCP) server built with ULTRAFAST MCP.

## Features

- **Echo Tool**: Simple echo functionality for testing
- **Info Tool**: Server information and capabilities
- **Status Resource**: Health check and status information

## Usage

### Running the Server

```bash
cargo run
```

### Testing with MCP Client

```bash
# Test the echo tool
echo '{{"method": "tools/call", "params": {{"name": "echo", "arguments": {{"message": "Hello, World!"}}}}}}' | cargo run

# Get server info
echo '{{"method": "tools/call", "params": {{"name": "info", "arguments": {{}}}}}}' | cargo run
```

### Available Tools

#### Echo Tool
- **Name**: `echo`
- **Description**: Echoes back the input message with a timestamp
- **Parameters**:
  - `message` (string): The message to echo

#### Info Tool  
- **Name**: `info`
- **Description**: Returns server information and capabilities
- **Parameters**: None

### Available Resources

#### Status Resource
- **URI**: `status`
- **Description**: Returns server health status and metadata

## Development

### Adding New Tools

1. Create a new file in `src/tools/`
2. Implement your tool following the pattern in `echo.rs`
3. Add the module to `src/tools/mod.rs`
4. Register the tool in `src/main.rs`

### Adding New Resources

1. Add resource handlers to `src/tools/resources.rs`
2. Register resources in `src/main.rs`

## License

This project is licensed under the MIT License.
"#
    );

    std::fs::write(format!("{server_name}/README.md"), readme)
        .context("Failed to write README.md")?;

    println!("✅ Generated MCP server project at {server_name}/");
    println!("📁 Project structure:");
    println!("   ├── Cargo.toml");
    println!("   ├── README.md");
    println!("   └── src/");
    println!("       ├── main.rs");
    println!("       ├── lib.rs");
    println!("       └── tools/");
    println!("           ├── mod.rs");
    println!("           ├── echo.rs");
    println!("           ├── info.rs");
    println!("           └── resources.rs");
    println!("\n🚀 To get started:");
    println!("   cd {server_name}");
    println!("   cargo run");

    Ok(())
}
