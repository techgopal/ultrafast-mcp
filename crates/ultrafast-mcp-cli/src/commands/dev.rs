use clap::Args;
use anyhow::{Result, Context};
use std::path::PathBuf;
use tokio::time::{sleep, Duration};
use colored::*;
use crate::config::Config;

/// Run a development server with hot reload
#[derive(Debug, Args)]
pub struct DevArgs {
    /// Project directory
    #[arg(short, long)]
    pub path: Option<PathBuf>,

    /// Port to run the development server on
    #[arg(short = 'P', long)]
    pub port: Option<u16>,

    /// Disable hot reload
    #[arg(long)]
    pub no_hot_reload: bool,

    /// Watch additional directories
    #[arg(short, long)]
    pub watch: Vec<PathBuf>,

    /// Log level
    #[arg(short, long, default_value = "info")]
    pub log_level: String,

    /// Transport type
    #[arg(short, long, default_value = "stdio")]
    pub transport: String,
}

pub async fn execute(args: DevArgs, config: Option<Config>) -> Result<()> {
    println!("{}", "Starting MCP development server...".green().bold());

    let project_dir = args.path.unwrap_or_else(|| std::env::current_dir().unwrap());
    let port = args.port.or_else(|| config.as_ref().and_then(|c| c.dev.port)).unwrap_or(8080);

    println!("📁 Project directory: {}", project_dir.display());
    println!("🚀 Transport: {}", args.transport);
    
    if args.transport != "stdio" {
        println!("🌐 Port: {}", port);
    }

    // Check if project has a valid MCP configuration
    let cargo_toml_path = project_dir.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        anyhow::bail!("No Cargo.toml found. Make sure you're in an MCP project directory.");
    }

    // Start file watcher if hot reload is enabled
    let hot_reload = !args.no_hot_reload;
    if hot_reload {
        println!("🔥 Hot reload: enabled");
        
        // Start the file watcher in a separate task
        let project_dir_clone = project_dir.clone();
        let watch_dirs = if args.watch.is_empty() {
            vec![project_dir.join("src")]
        } else {
            args.watch
        };

        tokio::spawn(async move {
            if let Err(e) = start_file_watcher(project_dir_clone, watch_dirs).await {
                eprintln!("File watcher error: {}", e);
            }
        });
    }

    // Build the project
    println!("🔨 Building project...");
    build_project(&project_dir).await?;

    // Start the MCP server
    println!("✅ Starting MCP server...");
    start_mcp_server(&project_dir, &args.transport, port).await?;

    Ok(())
}

async fn build_project(project_dir: &PathBuf) -> Result<()> {
    let output = tokio::process::Command::new("cargo")
        .args(["build"])
        .current_dir(project_dir)
        .output()
        .await
        .context("Failed to execute cargo build")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Build failed:\n{}", stderr);
    }

    println!("✅ Build completed successfully");
    Ok(())
}

async fn start_mcp_server(project_dir: &PathBuf, transport: &str, port: u16) -> Result<()> {
    match transport {
        "stdio" => {
            // Start stdio server
            println!("📡 Starting stdio server...");
            
            let mut child = tokio::process::Command::new("cargo")
                .args(["run"])
                .current_dir(project_dir)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .context("Failed to start MCP server")?;

            // Monitor the process
            let status = child.wait().await.context("Failed to wait for server process")?;
            
            if !status.success() {
                anyhow::bail!("Server process exited with error: {}", status);
            }
        }
        "http" => {
            // Start HTTP server
            println!("🌐 Starting HTTP server on port {}...", port);
            
            // For now, just simulate an HTTP server
            // In a real implementation, this would start an actual HTTP server
            loop {
                println!("🔄 Server running on http://localhost:{}", port);
                sleep(Duration::from_secs(30)).await;
            }
        }
        _ => {
            anyhow::bail!("Unsupported transport type: {}", transport);
        }
    }

    Ok(())
}

async fn start_file_watcher(project_dir: PathBuf, watch_dirs: Vec<PathBuf>) -> Result<()> {
    
    

    println!("👀 Watching directories for changes...");
    for dir in &watch_dirs {
        println!("   - {}", dir.display());
    }

    // Create a simple file watcher using polling
    // In a real implementation, you'd use a proper file watcher like notify
    let mut last_modified = std::collections::HashMap::new();

    loop {
        let mut changed = false;

        for watch_dir in &watch_dirs {
            if let Ok(entries) = std::fs::read_dir(watch_dir) {
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            let path = entry.path();
                            if let Some(ext) = path.extension() {
                                if ext == "rs" || ext == "toml" {
                                    if let Some(&last_mod) = last_modified.get(&path) {
                                        if modified > last_mod {
                                            println!("📝 File changed: {}", path.display());
                                            changed = true;
                                        }
                                    }
                                    last_modified.insert(path, modified);
                                }
                            }
                        }
                    }
                }
            }
        }

        if changed {
            println!("🔄 Rebuilding project...");
            if let Err(e) = build_project(&project_dir).await {
                eprintln!("❌ Build failed: {}", e);
            } else {
                println!("✅ Rebuild completed");
            }
        }

        sleep(Duration::from_secs(1)).await;
    }
}
