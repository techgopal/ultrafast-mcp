//! Subprocess Transport Demo
//!
//! This demonstrates the concept of subprocess transport for MCP servers.
//! It shows how to spawn a server as a subprocess and communicate with it.

use std::process::Stdio;
use tokio::io::AsyncBufReadExt;
use tokio::process::Command;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("🚀 Starting Subprocess Transport Demo");

    info!("🔧 Spawning echo server as subprocess...");

    // Spawn the server as a subprocess
    let mut server_process = Command::new("cargo")
        .args(&["run", "--release", "--bin", "basic-echo-server"])
        .current_dir(".")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to spawn server: {}", e))?;

    info!("✅ Server process spawned (PID: {:?})", server_process.id());

    // Wait a moment for the server to start
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    info!("📋 Server is running as subprocess");
    info!("💡 This demonstrates the subprocess transport pattern:");
    info!("   - Server runs in separate process");
    info!("   - Communication via STDIO pipes");
    info!("   - Process isolation and resource management");
    info!("   - Independent lifecycle management");

    // Show the server's stderr output (if any)
    if let Some(stderr) = server_process.stderr.take() {
        let mut reader = tokio::io::BufReader::new(stderr);
        let mut line = String::new();
        
        // Try to read one line from stderr to see if server started
        if let Ok(_) = reader.read_line(&mut line).await {
            if !line.trim().is_empty() {
                info!("📤 Server output: {}", line.trim());
            }
        }
    }

    info!("⏳ Waiting 3 seconds to demonstrate subprocess running...");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    info!("🛑 Terminating server process...");

    // Terminate the server process
    if let Err(e) = server_process.kill().await {
        warn!("Failed to kill server process: {}", e);
    }

    // Wait for server process to exit
    let exit_status = server_process.wait().await
        .map_err(|e| anyhow::anyhow!("Failed to wait for server: {}", e))?;

    info!("✅ Server process exited with status: {}", exit_status);

    println!();
    println!("🎉 Subprocess Transport Demo Completed!");
    println!();
    println!("📚 Key Benefits of Subprocess Transport:");
    println!("   ✅ Language isolation - run servers in any language");
    println!("   ✅ Process isolation - server crashes don't affect client");
    println!("   ✅ Resource management - independent memory allocation");
    println!("   ✅ Deployment flexibility - standalone executables");
    println!("   ✅ Development workflow - independent testing and debugging");
    println!();
    println!("🔗 Next Steps:");
    println!("   - Implement actual MCP communication via STDIO");
    println!("   - Add proper error handling and retry logic");
    println!("   - Create multi-language server examples");
    println!("   - Add monitoring and health checks");

    Ok(())
} 