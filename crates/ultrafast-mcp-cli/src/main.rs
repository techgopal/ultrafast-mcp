use clap::{Parser, Subcommand};
use anyhow::Result;
use tracing::{info, debug};

mod commands;
mod config;
mod templates;
mod utils;

use commands::*;

/// ULTRAFAST MCP CLI - A fast, efficient Model Context Protocol implementation
#[derive(Parser)]
#[command(name = "mcp")]
#[command(about = "ULTRAFAST MCP CLI - A fast, efficient Model Context Protocol implementation")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(author = "ULTRAFAST MCP Team")]
pub struct Cli {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Enable debug logging
    #[arg(short, long, global = true)]
    debug: bool,

    /// Configuration file path
    #[arg(short, long, global = true)]
    config: Option<std::path::PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new MCP project
    Init(InitArgs),
    /// Generate project scaffolding
    Generate(GenerateArgs),
    /// Run a development server
    Dev(DevArgs),
    /// Build the project
    Build(BuildArgs),
    /// Test MCP connections
    Test(TestArgs),
    /// Validate MCP schemas and configurations
    Validate(ValidateArgs),
    /// Show project information
    Info(InfoArgs),
    /// Manage server configurations
    Server(ServerArgs),
    /// Manage client configurations
    Client(ClientArgs),
    /// Generate shell completions
    Completions(CompletionsArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let log_level = if cli.debug {
        "debug"
    } else if cli.verbose {
        "info"
    } else {
        "warn"
    };

    tracing_subscriber::fmt()
        .with_env_filter(format!("ultrafast_mcp_cli={}", log_level))
        .init();

    info!("Starting ULTRAFAST MCP CLI v{}", env!("CARGO_PKG_VERSION"));
    debug!("Debug logging enabled");

    // Load configuration if specified
    let config = if let Some(config_path) = cli.config {
        debug!("Loading configuration from: {}", config_path.display());
        Some(config::Config::load(&config_path)?)
    } else {
        None
    };

    // Execute command
    match cli.command {
        Commands::Init(args) => init::execute(args, config).await,
        Commands::Generate(args) => generate::execute(args, config).await,
        Commands::Dev(args) => dev::execute(args, config).await,
        Commands::Build(args) => build::execute(args, config).await,
        Commands::Test(args) => test::execute(args, config).await,
        Commands::Validate(args) => validate::execute(args, config).await,
        Commands::Info(args) => info::execute(args, config).await,
        Commands::Server(args) => server::execute(args, config).await,
        Commands::Client(args) => client::execute(args, config).await,
        Commands::Completions(args) => completions::execute(args, config).await,
    }
}
