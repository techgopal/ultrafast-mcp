use crate::config::Config;
use anyhow::Result;
use clap::Args;
use colored::*;

/// Show project information
#[derive(Debug, Args)]
pub struct InfoArgs {
    /// Show detailed information
    #[arg(short, long)]
    pub detailed: bool,

    /// Output format
    #[arg(long, default_value = "human")]
    pub format: String,
}

pub async fn execute(args: InfoArgs, config: Option<Config>) -> Result<()> {
    println!("{}", "MCP Project Information".green().bold());
    println!();

    // Show CLI version
    println!("🚀 ULTRAFAST MCP CLI");
    println!("   Version: {}", env!("CARGO_PKG_VERSION"));
    println!();

    // Show project info if available
    let current_dir = std::env::current_dir()?;
    let cargo_toml = current_dir.join("Cargo.toml");

    if cargo_toml.exists() {
        println!("📦 Current Project");
        println!("   Directory: {}", current_dir.display());

        // Parse Cargo.toml for project info
        if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
            if let Ok(toml_value) = toml::from_str::<toml::Value>(&content) {
                if let Some(package) = toml_value.get("package") {
                    if let Some(name) = package.get("name") {
                        println!("   Name: {}", name.as_str().unwrap_or("unknown"));
                    }
                    if let Some(version) = package.get("version") {
                        println!("   Version: {}", version.as_str().unwrap_or("unknown"));
                    }
                    if let Some(description) = package.get("description") {
                        println!("   Description: {}", description.as_str().unwrap_or("none"));
                    }
                }
            }
        }
        println!();
    }

    // Show configuration info
    if let Some(config) = config {
        println!("⚙️ Configuration");
        println!("   Project: {}", config.project.name);
        println!("   Version: {}", config.project.version);
        if let Some(desc) = config.project.description {
            println!("   Description: {desc}");
        }
        println!("   Servers: {}", config.servers.len());
        println!("   Clients: {}", config.clients.len());
        println!();
    }

    if args.detailed {
        println!("📋 Available Commands:");
        println!("   mcp init      - Initialize a new MCP project");
        println!("   mcp generate  - Generate project scaffolding");
        println!("   mcp dev       - Run development server");
        println!("   mcp build     - Build the project");
        println!("   mcp test      - Test MCP connections");
        println!("   mcp validate  - Validate schemas and configurations");
        println!("   mcp server    - Manage server configurations");
        println!("   mcp client    - Manage client configurations");
        println!();
    }

    Ok(())
}
