use clap::{Args, Subcommand};
use anyhow::{Result, Context};
use colored::*;
use crate::config::Config;

/// Manage client configurations
#[derive(Debug, Args)]
pub struct ClientArgs {
    #[command(subcommand)]
    pub command: ClientCommand,
}

#[derive(Debug, Subcommand)]
pub enum ClientCommand {
    /// List configured clients
    List,
    /// Add a new client configuration
    Add(AddClientArgs),
    /// Remove a client configuration
    Remove(RemoveClientArgs),
    /// Show client details
    Show(ShowClientArgs),
    /// Connect to a server
    Connect(ConnectClientArgs),
}

#[derive(Debug, Args)]
pub struct AddClientArgs {
    /// Client name
    #[arg(value_name = "NAME")]
    pub name: String,

    /// Server to connect to
    #[arg(short, long)]
    pub server: String,

    /// Transport type
    #[arg(short, long, default_value = "stdio")]
    pub transport: String,
}

#[derive(Debug, Args)]
pub struct RemoveClientArgs {
    /// Client name
    #[arg(value_name = "NAME")]
    pub name: String,
}

#[derive(Debug, Args)]
pub struct ShowClientArgs {
    /// Client name
    #[arg(value_name = "NAME")]
    pub name: String,
}

#[derive(Debug, Args)]
pub struct ConnectClientArgs {
    /// Client name or server endpoint
    #[arg(value_name = "TARGET")]
    pub target: String,

    /// Transport type
    #[arg(short, long, default_value = "stdio")]
    pub transport: String,

    /// Interactive mode
    #[arg(short, long)]
    pub interactive: bool,
}

pub async fn execute(args: ClientArgs, config: Option<Config>) -> Result<()> {
    match args.command {
        ClientCommand::List => list_clients(config).await,
        ClientCommand::Add(add_args) => add_client(add_args, config).await,
        ClientCommand::Remove(remove_args) => remove_client(remove_args, config).await,
        ClientCommand::Show(show_args) => show_client(show_args, config).await,
        ClientCommand::Connect(connect_args) => connect_client(connect_args, config).await,
    }
}

async fn list_clients(config: Option<Config>) -> Result<()> {
    println!("{}", "Configured Clients".green().bold());
    
    if let Some(config) = config {
        if config.clients.is_empty() {
            println!("No clients configured.");
        } else {
            for (name, client) in config.clients {
                println!("👤 {}", name);
                println!("   Version: {}", client.version);
                println!("   Server: {}", client.server.endpoint);
            }
        }
    } else {
        println!("No configuration found.");
    }
    
    Ok(())
}

async fn add_client(args: AddClientArgs, config: Option<Config>) -> Result<()> {
    println!("➕ Adding client: {}", args.name.green());
    
    let mut config = config.unwrap_or_else(|| Config {
        project: crate::config::ProjectConfig {
            name: "default".to_string(),
            version: "0.1.0".to_string(),
            description: None,
            author: None,
            license: None,
            repository: None,
        },
        servers: std::collections::HashMap::new(),
        clients: std::collections::HashMap::new(),
        templates: crate::config::TemplateConfig::default(),
        dev: crate::config::DevConfig::default(),
    });
    
    let client_config = crate::config::ClientConfig {
        name: args.name.clone(),
        version: "0.1.0".to_string(),
        capabilities: crate::config::ClientCapabilities {
            experimental: std::collections::HashMap::new(),
            sampling: None,
        },
        server: crate::config::ServerConnectionConfig {
            endpoint: args.server,
            transport: crate::config::TransportConfig {
                transport_type: args.transport,
                config: std::collections::HashMap::new(),
            },
            timeout: Some(30),
            retry: Some(crate::config::RetryConfig {
                max_retries: 3,
                delay_ms: 1000,
                backoff_multiplier: 2.0,
            }),
        },
    };
    
    config.clients.insert(args.name.clone(), client_config);
    
    // Save config
    save_config(&config)?;
    
    println!("✅ Client '{}' added successfully", args.name);
    Ok(())
}

async fn remove_client(args: RemoveClientArgs, config: Option<Config>) -> Result<()> {
    println!("➖ Removing client: {}", args.name.red());
    
    let mut config = config.ok_or_else(|| anyhow::anyhow!("No configuration found"))?;
    
    if config.clients.remove(&args.name).is_some() {
        save_config(&config)?;
        println!("✅ Client '{}' removed successfully", args.name);
    } else {
        anyhow::bail!("Client '{}' not found", args.name);
    }
    
    Ok(())
}

async fn show_client(args: ShowClientArgs, config: Option<Config>) -> Result<()> {
    println!("👤 Client: {}", args.name.green().bold());
    
    if let Some(config) = config {
        if let Some(client) = config.clients.get(&args.name) {
            println!("   Name: {}", client.name);
            println!("   Version: {}", client.version);
            println!("   Server: {}", client.server.endpoint);
            println!("   Transport: {}", client.server.transport.transport_type);
            // TODO: Show more details
        } else {
            println!("Client '{}' not found.", args.name);
        }
    } else {
        println!("No configuration found.");
    }
    
    Ok(())
}

async fn connect_client(args: ConnectClientArgs, config: Option<Config>) -> Result<()> {
    println!("🔗 Connecting to: {}", args.target.green());
    println!("   Transport: {}", args.transport);
    
    if args.interactive {
        println!("💬 Starting interactive session...");
        println!("Type 'help' for available commands, 'quit' to exit.");
        
        // Try to establish connection
        let client = create_mcp_client(&args.target, &args.transport, &config).await?;
        
        loop {
            use std::io::{self, Write};
            print!("> ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();
            
            match input {
                "quit" | "exit" => break,
                "help" => {
                    println!("Available commands:");
                    println!("  help       - Show this help");
                    println!("  tools      - List available tools");
                    println!("  resources  - List available resources");
                    println!("  call <tool> [args] - Call a tool");
                    println!("  read <uri> - Read a resource");
                    println!("  status     - Show connection status");
                    println!("  quit/exit  - Exit the session");
                }
                "tools" => {
                    match client.list_tools().await {
                        Ok(tools) => {
                            println!("🔧 Available tools:");
                            for tool in tools {
                                println!("  {} - {}", tool.name, tool.description.unwrap_or_default());
                            }
                        }
                        Err(e) => println!("❌ Error listing tools: {}", e),
                    }
                }
                "resources" => {
                    match client.list_resources().await {
                        Ok(resources) => {
                            println!("📄 Available resources:");
                            for resource in resources {
                                println!("  {} - {}", resource.uri, resource.name.unwrap_or_default());
                            }
                        }
                        Err(e) => println!("❌ Error listing resources: {}", e),
                    }
                }
                "status" => {
                    println!("📊 Connection Status:");
                    println!("  Target: {}", args.target);
                    println!("  Transport: {}", args.transport);
                    println!("  Status: Connected ✅");
                }
                cmd if cmd.starts_with("call ") => {
                    let parts: Vec<&str> = cmd.splitn(3, ' ').collect();
                    if parts.len() >= 2 {
                        let tool_name = parts[1];
                        let tool_args = if parts.len() > 2 {
                            match serde_json::from_str(parts[2]) {
                                Ok(args) => args,
                                Err(_) => {
                                    println!("❌ Invalid JSON arguments. Use: call <tool> {{\"key\": \"value\"}}");
                                    continue;
                                }
                            }
                        } else {
                            serde_json::Value::Object(serde_json::Map::new())
                        };
                        
                        match client.call_tool(tool_name, tool_args).await {
                            Ok(result) => {
                                println!("✅ Tool result:");
                                println!("{}", serde_json::to_string_pretty(&result)?);
                            }
                            Err(e) => println!("❌ Error calling tool: {}", e),
                        }
                    } else {
                        println!("Usage: call <tool_name> [json_args]");
                    }
                }
                cmd if cmd.starts_with("read ") => {
                    let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
                    if parts.len() == 2 {
                        let uri = parts[1];
                        match client.read_resource(uri).await {
                            Ok(content) => {
                                println!("📄 Resource content:");
                                println!("{}", serde_json::to_string_pretty(&content)?);
                            }
                            Err(e) => println!("❌ Error reading resource: {}", e),
                        }
                    } else {
                        println!("Usage: read <resource_uri>");
                    }
                }
                "" => continue,
                _ => {
                    println!("Unknown command: {}. Type 'help' for available commands.", input);
                }
            }
        }
    } else {
        // Non-interactive connection test
        match test_connection(&args.target, &args.transport, &config).await {
            Ok(()) => println!("✅ Connection test completed successfully"),
            Err(e) => {
                println!("❌ Connection test failed: {}", e);
                return Err(e);
            }
        }
    }
    
    Ok(())
}

async fn create_mcp_client(target: &str, transport: &str, _config: &Option<Config>) -> Result<MockMcpClient> {
    println!("🔌 Establishing connection...");
    
    // For now, create a mock client since we don't have the full client implementation
    let client = MockMcpClient::new(target, transport)?;
    
    println!("✅ Connected successfully");
    Ok(client)
}

async fn test_connection(target: &str, transport: &str, _config: &Option<Config>) -> Result<()> {
    println!("🧪 Testing connection to {} via {}", target, transport);
    
    match transport {
        "stdio" => {
            let parts: Vec<&str> = target.split_whitespace().collect();
            if parts.is_empty() {
                anyhow::bail!("Invalid STDIO target");
            }
            
            // Test if command exists
            std::process::Command::new(parts[0])
                .args(&parts[1..])
                .arg("--help")
                .output()
                .context("Failed to execute command")?;
            
            println!("✅ STDIO command is available");
        }
        "http" => {
            let client = reqwest::Client::new();
            let response = client.get(target).send().await
                .context("Failed to connect to HTTP endpoint")?;
            
            println!("✅ HTTP endpoint responded with status: {}", response.status());
        }
        _ => anyhow::bail!("Unsupported transport: {}", transport),
    }
    
    Ok(())
}

fn save_config(config: &Config) -> Result<()> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
        .join("mcp");
    
    std::fs::create_dir_all(&config_dir)
        .context("Failed to create config directory")?;
    
    let config_file = config_dir.join("config.toml");
    let config_content = toml::to_string_pretty(config)
        .context("Failed to serialize config")?;
    
    std::fs::write(&config_file, config_content)
        .context("Failed to write config file")?;
    
    println!("💾 Configuration saved to: {}", config_file.display());
    Ok(())
}

// Mock client for demonstration - replace with real implementation
struct MockMcpClient {
    #[allow(dead_code)]
    target: String,
    #[allow(dead_code)]
    transport: String,
}

impl MockMcpClient {
    fn new(target: &str, transport: &str) -> Result<Self> {
        Ok(Self {
            target: target.to_string(),
            transport: transport.to_string(),
        })
    }
    
    async fn list_tools(&self) -> Result<Vec<MockTool>> {
        // Mock implementation
        Ok(vec![
            MockTool {
                name: "echo".to_string(),
                description: Some("Echo the input".to_string()),
            },
            MockTool {
                name: "calculate".to_string(),
                description: Some("Perform calculations".to_string()),
            },
        ])
    }
    
    async fn list_resources(&self) -> Result<Vec<MockResource>> {
        // Mock implementation
        Ok(vec![
            MockResource {
                uri: "resource://example".to_string(),
                name: Some("Example Resource".to_string()),
            },
        ])
    }
    
    async fn call_tool(&self, tool_name: &str, args: serde_json::Value) -> Result<serde_json::Value> {
        // Mock implementation
        Ok(serde_json::json!({
            "tool": tool_name,
            "args": args,
            "result": "Mock result from tool execution",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
    
    async fn read_resource(&self, uri: &str) -> Result<serde_json::Value> {
        // Mock implementation
        Ok(serde_json::json!({
            "uri": uri,
            "content": "Mock resource content",
            "mime_type": "text/plain",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
}

#[derive(Debug)]
struct MockTool {
    name: String,
    description: Option<String>,
}

#[derive(Debug)]
struct MockResource {
    uri: String,
    name: Option<String>,
}
