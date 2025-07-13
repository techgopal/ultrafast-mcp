//! # UltraFast MCP CLI
//!
//! Command-line interface for the UltraFast Model Context Protocol (MCP) implementation.
//!
//! This crate provides a comprehensive command-line tool for developing, testing, and
//! managing MCP servers and clients. It offers project scaffolding, development tools,
//! testing utilities, and deployment assistance for the MCP ecosystem.
//!
//! ## Overview
//!
//! The UltraFast MCP CLI is designed to streamline MCP development by providing:
//!
//! - **Project Management**: Initialize, build, and manage MCP projects
//! - **Development Tools**: Development servers, hot reloading, and debugging
//! - **Testing Utilities**: Connection testing, schema validation, and integration tests
//! - **Code Generation**: Scaffolding, templates, and boilerplate generation
//! - **Configuration Management**: Server and client configuration management
//! - **Deployment Support**: Build optimization and deployment assistance
//!
//! ## Key Features
//!
//! ### Project Management
//! - **Project Initialization**: Create new MCP projects with templates
//! - **Build System**: Optimized builds with dependency management
//! - **Configuration**: Flexible configuration management
//! - **Dependencies**: Automatic dependency resolution and updates
//!
//! ### Development Tools
//! - **Development Server**: Hot-reloading development server
//! - **Debugging**: Comprehensive debugging and logging support
//! - **Code Generation**: Scaffolding and template generation
//! - **Live Reload**: Automatic reloading on code changes
//!
//! ### Testing and Validation
//! - **Connection Testing**: Test MCP server and client connections
//! - **Schema Validation**: Validate MCP schemas and configurations
//! - **Integration Testing**: End-to-end testing utilities
//! - **Performance Testing**: Benchmark and performance analysis
//!
//! ### Code Generation
//! - **Project Scaffolding**: Generate complete project structures
//! - **Template System**: Customizable templates for different use cases
//! - **Boilerplate Generation**: Reduce repetitive code with generators
//! - **API Documentation**: Automatic documentation generation
//!
//! ## Installation
//!
//! ```bash
//! # Install from crates.io
//! cargo install ultrafast-mcp-cli
//!
//! # Or build from source
//! git clone https://github.com/your-repo/ultrafast-mcp
//! cd ultrafast-mcp/crates/ultrafast-mcp-cli
//! cargo install --path .
//! ```
//!
//! ## Quick Start
//!
//! ### Initialize a New Project
//!
//! ```bash
//! # Create a new MCP server project
//! mcp init my-server --template server
//!
//! # Create a new MCP client project
//! mcp init my-client --template client
//!
//! # Create a full-stack MCP project
//! mcp init my-project --template fullstack
//! ```
//!
//! ### Development Workflow
//!
//! ```bash
//! # Start development server with hot reloading
//! mcp dev --watch
//!
//! # Build the project for production
//! mcp build --release
//!
//! # Test the MCP connection
//! mcp test --server http://localhost:8080
//!
//! # Validate schemas and configurations
//! mcp validate
//! ```
//!
//! ### Project Management
//!
//! ```bash
//! # Show project information
//! mcp info
//!
//! # Generate shell completions
//! mcp completions bash > ~/.bash_completion
//!
//! # Manage server configurations
//! mcp server list
//! mcp server add my-server http://localhost:8080
//!
//! # Manage client configurations
//! mcp client list
//! mcp client add my-client --config client.toml
//! ```
//!
//! ## Commands
//!
//! ### Core Commands
//!
//! #### `mcp init` - Initialize a New Project
//! Creates a new MCP project with the specified template and configuration.
//!
//! ```bash
//! mcp init <project-name> [OPTIONS]
//!
//! Options:
//!   --template <TEMPLATE>    Project template (server, client, fullstack)
//!   --config <CONFIG>        Configuration file path
//!   --force                  Overwrite existing directory
//!   --git                    Initialize git repository
//! ```
//!
//! #### `mcp dev` - Development Server
//! Starts a development server with hot reloading and debugging support.
//!
//! ```bash
//! mcp dev [OPTIONS]
//!
//! Options:
//!   --port <PORT>            Server port (default: 8080)
//!   --host <HOST>            Server host (default: 127.0.0.1)
//!   --watch                  Enable file watching and hot reloading
//!   --debug                  Enable debug mode
//! ```
//!
//! #### `mcp build` - Build Project
//! Builds the project for development or production deployment.
//!
//! ```bash
//! mcp build [OPTIONS]
//!
//! Options:
//!   --release                Build in release mode
//!   --target <TARGET>        Target platform
//!   --optimize               Enable optimizations
//! ```
//!
//! ### Testing and Validation
//!
//! #### `mcp test` - Test Connections
//! Tests MCP server and client connections and functionality.
//!
//! ```bash
//! mcp test [OPTIONS]
//!
//! Options:
//!   --server <URL>           Server URL to test
//!   --client <CONFIG>        Client configuration file
//!   --timeout <SECONDS>      Test timeout in seconds
//!   --verbose                Verbose output
//! ```
//!
//! #### `mcp validate` - Validate Schemas
//! Validates MCP schemas, configurations, and project structure.
//!
//! ```bash
//! mcp validate [OPTIONS]
//!
//! Options:
//!   --schema <PATH>          Schema file path
//!   --config <PATH>          Configuration file path
//!   --strict                 Strict validation mode
//! ```
//!
//! ### Project Management
//!
//! #### `mcp info` - Show Project Information
//! Displays comprehensive information about the current project.
//!
//! ```bash
//! mcp info [OPTIONS]
//!
//! Options:
//!   --format <FORMAT>        Output format (text, json, yaml)
//!   --detailed               Show detailed information
//! ```
//!
//! #### `mcp generate` - Generate Code
//! Generates code, configurations, and project scaffolding.
//!
//! ```bash
//! mcp generate <TYPE> [OPTIONS]
//!
//! Types:
//!   server                   Generate server code
//!   client                   Generate client code
//!   tool                     Generate tool implementation
//!   resource                 Generate resource handler
//!   prompt                   Generate prompt template
//! ```
//!
//! ### Configuration Management
//!
//! #### `mcp server` - Server Management
//! Manages MCP server configurations and deployments.
//!
//! ```bash
//! mcp server <COMMAND> [OPTIONS]
//!
//! Commands:
//!   list                     List configured servers
//!   add <NAME> <URL>         Add a new server
//!   remove <NAME>            Remove a server
//!   test <NAME>              Test server connection
//!   deploy <NAME>            Deploy server
//! ```
//!
//! #### `mcp client` - Client Management
//! Manages MCP client configurations and connections.
//!
//! ```bash
//! mcp client <COMMAND> [OPTIONS]
//!
//! Commands:
//!   list                     List configured clients
//!   add <NAME> <CONFIG>      Add a new client
//!   remove <NAME>            Remove a client
//!   test <NAME>              Test client connection
//!   connect <NAME>           Connect to server
//! ```
//!
//! ## Configuration
//!
//! The CLI supports configuration through multiple sources:
//!
//! ### Configuration File
//! ```toml
//! # mcp.toml
//! [project]
//! name = "my-mcp-project"
//! version = "1.0.0"
//! description = "My MCP project"
//!
//! [development]
//! port = 8080
//! host = "127.0.0.1"
//! watch = true
//! debug = false
//!
//! [build]
//! release = false
//! optimize = true
//! target = "x86_64-unknown-linux-gnu"
//!
//! [testing]
//! timeout = 30
//! verbose = false
//! ```
//!
//! ### Environment Variables
//! ```bash
//! export MCP_CONFIG_PATH="/path/to/config.toml"
//! export MCP_LOG_LEVEL="debug"
//! export MCP_DEVELOPMENT_PORT="8080"
//! ```
//!
//! ### Command Line Options
//! ```bash
//! mcp --config /path/to/config.toml --verbose dev
//! ```
//!
//! ## Templates
//!
//! The CLI provides several project templates:
//!
//! ### Server Template
//! Complete MCP server implementation with:
//! - Tool handlers and implementations
//! - Resource management
//! - Prompt generation
//! - Authentication support
//! - Testing framework
//!
//! ### Client Template
//! MCP client implementation with:
//! - Connection management
//! - Tool calling
//! - Resource access
//! - Error handling
//! - Configuration management
//!
//! ### Fullstack Template
//! Complete MCP application with:
//! - Server and client components
//! - Shared types and utilities
//! - Integration testing
//! - Deployment configuration
//!
//! ## Development Workflow
//!
//! ### 1. Project Initialization
//! ```bash
//! mcp init my-project --template fullstack
//! cd my-project
//! ```
//!
//! ### 2. Development
//! ```bash
//! # Start development server
//! mcp dev --watch
//!
//! # In another terminal, test the server
//! mcp test --server http://localhost:8080
//! ```
//!
//! ### 3. Building and Testing
//! ```bash
//! # Build for production
//! mcp build --release
//!
//! # Validate project
//! mcp validate
//!
//! # Run integration tests
//! mcp test --verbose
//! ```
//!
//! ### 4. Deployment
//! ```bash
//! # Deploy server
//! mcp server add production https://api.example.com
//! mcp server deploy production
//!
//! # Configure client
//! mcp client add production --config client.toml
//! ```
//!
//! ## Best Practices
//!
//! ### Project Structure
//! - Use appropriate templates for your use case
//! - Organize code into logical modules
//! - Follow Rust naming conventions
//! - Implement comprehensive error handling
//!
//! ### Development
//! - Use the development server for rapid iteration
//! - Implement comprehensive testing
//! - Use configuration files for environment-specific settings
//! - Monitor performance and optimize as needed
//!
//! ### Deployment
//! - Use release builds for production
//! - Implement proper logging and monitoring
//! - Use secure configurations
//! - Test thoroughly before deployment
//!
//! ## Troubleshooting
//!
//! ### Common Issues
//!
//! **Connection Errors**
//! ```bash
//! # Check server status
//! mcp server test my-server
//!
//! # Validate configuration
//! mcp validate --config server.toml
//! ```
//!
//! **Build Errors**
//! ```bash
//! # Clean and rebuild
//! cargo clean
//! mcp build
//!
//! # Check dependencies
//! cargo check
//! ```
//!
//! **Validation Errors**
//! ```bash
//! # Validate with verbose output
//! mcp validate --verbose
//!
//! # Check specific files
//! mcp validate --schema schema.json
//! ```
//!
//! ## Performance Considerations
//!
//! - **Development Server**: Optimized for fast reloading and debugging
//! - **Build System**: Parallel compilation and caching
//! - **Testing**: Efficient test execution and reporting
//! - **Validation**: Fast schema validation and error reporting
//!
//! ## Extensibility
//!
//! The CLI is designed to be extensible:
//! - **Custom Templates**: Create custom project templates
//! - **Plugin System**: Extend functionality with plugins
//! - **Configuration**: Flexible configuration management
//! - **Scripting**: Support for custom scripts and automation
//!
//! ## Examples
//!
//! See the `examples/` directory for complete working examples:
//! - Basic server and client projects
//! - Custom templates and configurations
//! - Integration testing scenarios
//! - Deployment configurations

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{debug, info};

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
        .with_env_filter(format!("ultrafast_mcp_cli={log_level}"))
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
