use clap::{Args, Subcommand};
use anyhow::Result;
use colored::*;
use crate::config::Config;

/// Manage server configurations
#[derive(Debug, Args)]
pub struct ServerArgs {
    #[command(subcommand)]
    pub command: ServerCommand,
}

#[derive(Debug, Subcommand)]
pub enum ServerCommand {
    /// List configured servers
    List,
    /// Add a new server configuration
    Add(AddServerArgs),
    /// Remove a server configuration
    Remove(RemoveServerArgs),
    /// Show server details
    Show(ShowServerArgs),
    /// Start a server
    Start(StartServerArgs),
    /// Stop a server
    Stop(StopServerArgs),
}

#[derive(Debug, Args)]
pub struct AddServerArgs {
    /// Server name
    #[arg(value_name = "NAME")]
    pub name: String,

    /// Server command
    #[arg(short, long)]
    pub command: String,

    /// Server arguments
    #[arg(short, long)]
    pub args: Vec<String>,

    /// Environment variables
    #[arg(short, long)]
    pub env: Vec<String>,
}

#[derive(Debug, Args)]
pub struct RemoveServerArgs {
    /// Server name
    #[arg(value_name = "NAME")]
    pub name: String,
}

#[derive(Debug, Args)]
pub struct ShowServerArgs {
    /// Server name
    #[arg(value_name = "NAME")]
    pub name: String,
}

#[derive(Debug, Args)]
pub struct StartServerArgs {
    /// Server name
    #[arg(value_name = "NAME")]
    pub name: String,
}

#[derive(Debug, Args)]
pub struct StopServerArgs {
    /// Server name
    #[arg(value_name = "NAME")]
    pub name: String,
}

pub async fn execute(args: ServerArgs, config: Option<Config>) -> Result<()> {
    match args.command {
        ServerCommand::List => list_servers(config).await,
        ServerCommand::Add(add_args) => add_server(add_args, config).await,
        ServerCommand::Remove(remove_args) => remove_server(remove_args, config).await,
        ServerCommand::Show(show_args) => show_server(show_args, config).await,
        ServerCommand::Start(start_args) => start_server(start_args, config).await,
        ServerCommand::Stop(stop_args) => stop_server(stop_args, config).await,
    }
}

async fn list_servers(config: Option<Config>) -> Result<()> {
    println!("{}", "Configured Servers".green().bold());
    
    if let Some(config) = config {
        if config.servers.is_empty() {
            println!("No servers configured.");
        } else {
            for (name, server) in config.servers {
                println!("📡 {}", name);
                println!("   Version: {}", server.version);
                if let Some(transport) = server.transport.config.get("command") {
                    println!("   Transport: {}", transport);
                }
            }
        }
    } else {
        println!("No configuration found.");
    }
    
    Ok(())
}

async fn add_server(args: AddServerArgs, config: Option<Config>) -> Result<()> {
    println!("➕ Adding server: {}", args.name.green());
    
    let mut config = config.unwrap_or_default();
    
    // Create new server configuration
    let server_config = crate::config::ServerConfig {
        name: args.name.clone(),
        version: "1.0.0".to_string(),
        transport: crate::config::TransportConfig {
            transport_type: "stdio".to_string(),
            config: {
                let mut map = std::collections::HashMap::new();
                map.insert("command".to_string(), serde_json::Value::String(args.command));
                map.insert("args".to_string(), serde_json::Value::String(args.args.join(" ")));
                map
            },
        },
        capabilities: crate::config::ServerCapabilities {
            experimental: std::collections::HashMap::new(),
            logging: None,
            prompts: None,
            resources: None,
            tools: Some(crate::config::ToolsCapability {
                list_changed: true,
            }),
        },
        tools: Vec::new(),
        resources: Vec::new(),
    };
    
    // Add to configuration
    config.servers.insert(args.name.clone(), server_config);
    
    // Save configuration - pass the config file path
    let config_path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("ultrafast-mcp")
        .join("config.toml");
    
    config.save(&config_path)?;
    
    println!("✅ Server '{}' added successfully", args.name);
    Ok(())
}

async fn remove_server(args: RemoveServerArgs, config: Option<Config>) -> Result<()> {
    println!("➖ Removing server: {}", args.name.red());
    
    let mut config = config.unwrap_or_default();
    
    if config.servers.remove(&args.name).is_some() {
        let config_path = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("ultrafast-mcp")
            .join("config.toml");
        
        config.save(&config_path)?;
        println!("✅ Server '{}' removed successfully", args.name);
    } else {
        println!("❌ Server '{}' not found", args.name);
    }
    
    Ok(())
}

async fn show_server(args: ShowServerArgs, config: Option<Config>) -> Result<()> {
    println!("📡 Server: {}", args.name.green().bold());
    
    if let Some(config) = config {
        if let Some(server) = config.servers.get(&args.name) {
            println!("   Name: {}", server.name);
            println!("   Version: {}", server.version);
            println!("   Transport: {}", server.transport.transport_type);
            // TODO: Show more details
        } else {
            println!("Server '{}' not found.", args.name);
        }
    } else {
        println!("No configuration found.");
    }
    
    Ok(())
}

async fn start_server(args: StartServerArgs, config: Option<Config>) -> Result<()> {
    println!("🚀 Starting server: {}", args.name.green());
    
    if let Some(config) = config {
        if let Some(server) = config.servers.get(&args.name) {
            // Extract command and args from transport config
            let command = server.transport.config.get("command")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("No command configured for server '{}'", args.name))?;
            
            let args_str = server.transport.config.get("args")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            
            println!("📡 Command: {} {}", command, args_str);
            
            // For now, just show what would be executed
            // In a real implementation, this would start the process and track it
            println!("✅ Server '{}' started (simulation)", args.name);
            println!("💡 In production, this would:");
            println!("   - Start the process: {} {}", command, args_str);
            println!("   - Track the process ID");
            println!("   - Monitor health");
            println!("   - Store runtime state");
        } else {
            anyhow::bail!("Server '{}' not found in configuration", args.name);
        }
    } else {
        anyhow::bail!("No configuration found");
    }
    
    Ok(())
}

async fn stop_server(args: StopServerArgs, config: Option<Config>) -> Result<()> {
    println!("🛑 Stopping server: {}", args.name.red());
    
    if let Some(_config) = config {
        // For now, just show what would be executed
        // In a real implementation, this would stop the tracked process
        println!("✅ Server '{}' stopped (simulation)", args.name);
        println!("💡 In production, this would:");
        println!("   - Send SIGTERM to the process");
        println!("   - Wait for graceful shutdown");
        println!("   - Send SIGKILL if needed");
        println!("   - Clean up runtime state");
    } else {
        anyhow::bail!("No configuration found");
    }
    
    Ok(())
}
