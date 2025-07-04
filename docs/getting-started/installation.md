# Installation Guide

Complete installation guide for **ULTRAFAST_MCP**, covering all platforms, dependencies, and installation methods.

## 🚀 Prerequisites

### System Requirements

- **Operating System**: Linux, macOS, or Windows
- **Rust**: 1.70 or higher
- **Cargo**: Latest stable version
- **Memory**: 512MB RAM minimum (2GB recommended)
- **Disk Space**: 100MB for installation

### Rust Installation

If you don't have Rust installed, install it using [rustup](https://rustup.rs/):

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Or on Windows (PowerShell)
winget install Rust.Rust
# Or download from https://rustup.rs/

# Verify installation
rustc --version
cargo --version
```

## 📦 Installation Methods

### 1. Cargo Installation (Recommended)

#### Basic Installation

```bash
# Add ULTRAFAST_MCP to your project
cargo add ultrafast-mcp
```

#### With Specific Features

```bash
# HTTP transport and OAuth authentication
cargo add ultrafast-mcp --features="http-transport,oauth"

# All features for enterprise use
cargo add ultrafast-mcp --features="all"

# Minimal installation (stdio only)
cargo add ultrafast-mcp --no-default-features --features="stdio-transport"
```

#### Feature Flags Reference

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `stdio-transport` | stdio transport (default) | None |
| `http-transport` | HTTP/HTTPS transport | axum, reqwest, tower |
| `oauth` | OAuth 2.1 authentication | oauth2, jsonwebtoken |
| `performance` | Zero-copy optimizations | bytes, smallvec, dashmap |
| `monitoring` | OpenTelemetry observability | opentelemetry, metrics |

#### Convenience Features

```bash
# Web server with authentication
cargo add ultrafast-mcp --features="web"

# Enterprise features (all optimizations)
cargo add ultrafast-mcp --features="enterprise"

# Development features
cargo add ultrafast-mcp --features="dev"
```

### 2. Manual Installation

#### Clone Repository

```bash
# Clone the repository
git clone https://github.com/ultrafast-mcp/ultrafast-mcp.git
cd ultrafast-mcp

# Build all crates
cargo build --workspace

# Install CLI tool
cargo install --path crates/ultrafast-mcp-cli
```

#### Build from Source

```bash
# Build with specific features
cargo build --workspace --features="http-transport,oauth,performance"

# Build release version
cargo build --workspace --release

# Build specific crate
cargo build -p ultrafast-mcp-server
```

### 3. Docker Installation

#### Docker Image

```bash
# Pull the official image
docker pull ultrafast-mcp/ultrafast-mcp:latest

# Run a basic server
docker run -p 8080:8080 ultrafast-mcp/ultrafast-mcp:latest

# Run with custom configuration
docker run -p 8080:8080 \
  -e MCP_HOST=0.0.0.0 \
  -e MCP_PORT=8080 \
  -e MCP_FEATURES="http-transport,oauth" \
  ultrafast-mcp/ultrafast-mcp:latest
```

#### Docker Compose

```yaml
# docker-compose.yml
version: '3.8'
services:
  mcp-server:
    image: ultrafast-mcp/ultrafast-mcp:latest
    ports:
      - "8080:8080"
    environment:
      - MCP_HOST=0.0.0.0
      - MCP_PORT=8080
      - MCP_FEATURES=http-transport,oauth
    volumes:
      - ./config:/app/config
    restart: unless-stopped
```

### 4. Package Manager Installation

#### Homebrew (macOS)

```bash
# Install via Homebrew
brew install ultrafast-mcp/ultrafast-mcp

# Update
brew upgrade ultrafast-mcp
```

#### Cargo Install

```bash
# Install CLI globally
cargo install ultrafast-mcp-cli

# Install specific version
cargo install ultrafast-mcp-cli --version 0.1.0
```

## 🔧 Configuration

### Environment Variables

```bash
# Server configuration
export MCP_HOST=127.0.0.1
export MCP_PORT=8080
export MCP_PROTOCOL_VERSION=2025-06-18
export MCP_FEATURES=http-transport,oauth

# Authentication
export MCP_OAUTH_CLIENT_ID=your-client-id
export MCP_OAUTH_CLIENT_SECRET=your-client-secret
export MCP_OAUTH_REDIRECT_URI=http://localhost:8080/callback

# Monitoring
export MCP_MONITORING_ENABLED=true
export MCP_JAEGER_ENDPOINT=http://localhost:14268/api/traces
export MCP_PROMETHEUS_ENDPOINT=http://localhost:9090
```

### Configuration File

```toml
# config.toml
[server]
host = "127.0.0.1"
port = 8080
protocol_version = "2025-06-18"
features = ["http-transport", "oauth"]

[capabilities]
tools = { list_changed = true }
resources = { subscribe = true, list_changed = true }
prompts = { list_changed = true }
logging = {}

[auth]
oauth_client_id = "your-client-id"
oauth_client_secret = "your-client-secret"
oauth_redirect_uri = "http://localhost:8080/callback"

[monitoring]
enabled = true
jaeger_endpoint = "http://localhost:14268/api/traces"
prometheus_endpoint = "http://localhost:9090"
```

## 🖥️ Platform-Specific Instructions

### Linux

#### Ubuntu/Debian

```bash
# Install dependencies
sudo apt update
sudo apt install build-essential pkg-config libssl-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install ULTRAFAST_MCP
cargo add ultrafast-mcp --features="all"
```

#### CentOS/RHEL

```bash
# Install dependencies
sudo yum groupinstall "Development Tools"
sudo yum install openssl-devel

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install ULTRAFAST_MCP
cargo add ultrafast-mcp --features="all"
```

#### Arch Linux

```bash
# Install from AUR
yay -S ultrafast-mcp

# Or install manually
sudo pacman -S base-devel openssl
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
cargo add ultrafast-mcp --features="all"
```

### macOS

#### Using Homebrew

```bash
# Install Rust
brew install rust

# Install ULTRAFAST_MCP
brew install ultrafast-mcp/ultrafast-mcp

# Or install manually
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
cargo add ultrafast-mcp --features="all"
```

#### Using MacPorts

```bash
# Install Rust
sudo port install rust

# Install ULTRAFAST_MCP
cargo add ultrafast-mcp --features="all"
```

### Windows

#### Using Chocolatey

```powershell
# Install Rust
choco install rust

# Install ULTRAFAST_MCP
cargo add ultrafast-mcp --features="all"
```

#### Using Scoop

```powershell
# Install Rust
scoop install rust

# Install ULTRAFAST_MCP
cargo add ultrafast-mcp --features="all"
```

#### Manual Installation

1. Download Rust from [https://rustup.rs/](https://rustup.rs/)
2. Run the installer
3. Open PowerShell or Command Prompt
4. Run: `cargo add ultrafast-mcp --features="all"`

## 🔍 Verification

### Check Installation

```bash
# Verify ULTRAFAST_MCP is installed
cargo tree | grep ultrafast-mcp

# Check available features
cargo doc --open

# Run tests
cargo test --workspace
```

### Quick Test

```bash
# Create a test project
cargo new test-mcp
cd test-mcp

# Add ULTRAFAST_MCP
cargo add ultrafast-mcp

# Create a simple server
cat > src/main.rs << 'EOF'
use ultrafast_mcp::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let server = UltraFastServer::new("Test Server")
        .with_capabilities(ServerCapabilities::default());
    
    println!("ULTRAFAST_MCP server created successfully!");
    Ok(())
}
EOF

# Build and run
cargo build
cargo run
```

## 🐛 Troubleshooting

### Common Issues

#### 1. Build Errors

```bash
# Clean and rebuild
cargo clean
cargo build

# Update Rust
rustup update

# Check Rust version
rustc --version
```

#### 2. Feature Flag Issues

```bash
# Check available features
cargo doc --open

# Install with specific features
cargo add ultrafast-mcp --features="stdio-transport"

# Check feature dependencies
cargo tree --features="http-transport"
```

#### 3. SSL/TLS Issues

```bash
# Install OpenSSL development libraries
# Ubuntu/Debian
sudo apt install libssl-dev pkg-config

# CentOS/RHEL
sudo yum install openssl-devel

# macOS
brew install openssl
```

#### 4. Memory Issues

```bash
# Increase memory limit for Cargo
export CARGO_BUILD_JOBS=1
export RUSTFLAGS="-C link-arg=-Wl,-rpath,$(rustc --print sysroot)/lib"

# Or use release build
cargo build --release
```

### Getting Help

```bash
# Check documentation
cargo doc --open

# Run examples
cd examples/01-basic-echo
cargo run --bin server

# Check logs
RUST_LOG=debug cargo run

# Get help
mcp --help
```

## 🔄 Updates

### Updating ULTRAFAST_MCP

```bash
# Update to latest version
cargo update ultrafast-mcp

# Update specific version
cargo update -p ultrafast-mcp --precise 0.1.1

# Check for updates
cargo outdated
```

### Updating Rust

```bash
# Update Rust toolchain
rustup update

# Check for new versions
rustup check

# Update specific component
rustup update stable
```

## 📊 System Requirements

### Minimum Requirements

| Component | Requirement |
|-----------|-------------|
| **CPU** | 1 core, 1GHz |
| **RAM** | 512MB |
| **Storage** | 100MB |
| **Network** | None (stdio) or HTTP |

### Recommended Requirements

| Component | Requirement |
|-----------|-------------|
| **CPU** | 2+ cores, 2GHz+ |
| **RAM** | 2GB+ |
| **Storage** | 1GB+ |
| **Network** | HTTP/HTTPS support |

### Production Requirements

| Component | Requirement |
|-----------|-------------|
| **CPU** | 4+ cores, 3GHz+ |
| **RAM** | 8GB+ |
| **Storage** | 10GB+ SSD |
| **Network** | High-bandwidth, low-latency |

## 🎯 Next Steps

After installation, proceed to:

1. **[Quick Start Guide](./quick-start.md)** - Get up and running in 5 minutes
2. **[First Server](./first-server.md)** - Create your first MCP server
3. **[Examples](./examples/basic-echo.md)** - Explore working examples
4. **[API Reference](./api-reference/server-api.md)** - Learn the complete API

## 📞 Support

If you encounter issues during installation:

- **Documentation**: [Complete Installation Guide](./installation.md)
- **Issues**: [GitHub Issues](https://github.com/ultrafast-mcp/ultrafast-mcp/issues)
- **Discussions**: [GitHub Discussions](https://github.com/ultrafast-mcp/ultrafast-mcp/discussions)
- **Email**: team@ultrafast-mcp.com

---

**ULTRAFAST_MCP** is now ready to use! 🚀 