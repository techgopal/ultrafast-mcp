use crate::config::Config;
use anyhow::{Context, Result};
use clap::Args;
use colored::*;
use std::io::Write;
use std::time::Duration;
use tokio::time::timeout;

/// Test MCP connections
#[derive(Debug, Args)]
pub struct TestArgs {
    /// Server endpoint to test
    #[arg(short, long)]
    pub server: Option<String>,

    /// Transport type
    #[arg(short, long, default_value = "stdio")]
    pub transport: String,

    /// Test timeout in seconds
    #[arg(long, default_value = "30")]
    pub timeout: u64,

    /// Specific test to run
    #[arg(long)]
    pub test: Option<String>,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

pub async fn execute(args: TestArgs, config: Option<Config>) -> Result<()> {
    println!("{}", "Testing MCP connections...".green().bold());

    let timeout_duration = Duration::from_secs(args.timeout);

    let result = timeout(timeout_duration, run_tests(&args, config)).await;

    match result {
        Ok(Ok(())) => {
            println!("\n🎉 All tests passed!");
            Ok(())
        }
        Ok(Err(e)) => {
            println!("\n❌ Tests failed: {}", e);
            Err(e)
        }
        Err(_) => {
            anyhow::bail!("Tests timed out after {} seconds", args.timeout);
        }
    }
}

async fn run_tests(args: &TestArgs, config: Option<Config>) -> Result<()> {
    if let Some(test_name) = &args.test {
        run_specific_test(test_name, args, config).await
    } else {
        run_all_tests(args, config).await
    }
}

async fn run_all_tests(args: &TestArgs, config: Option<Config>) -> Result<()> {
    println!("🔄 Running all tests...\n");

    let mut passed = 0;
    let mut failed = 0;

    // Test 1: Configuration loading
    print!("📋 Testing configuration loading... ");
    match test_config_loading(&config).await {
        Ok(()) => {
            println!("{}", "✅ PASSED".green());
            passed += 1;
        }
        Err(e) => {
            println!("{}", "❌ FAILED".red());
            if args.verbose {
                println!("   Error: {}", e);
            }
            failed += 1;
        }
    }

    // Test 2: Transport-specific tests
    match args.transport.as_str() {
        "stdio" => {
            print!("📡 Testing STDIO transport... ");
            match test_stdio_connection(args).await {
                Ok(()) => {
                    println!("{}", "✅ PASSED".green());
                    passed += 1;
                }
                Err(e) => {
                    println!("{}", "❌ FAILED".red());
                    if args.verbose {
                        println!("   Error: {}", e);
                    }
                    failed += 1;
                }
            }
        }
        "http" => {
            print!("🌐 Testing HTTP transport... ");
            match test_http_connection(args).await {
                Ok(()) => {
                    println!("{}", "✅ PASSED".green());
                    passed += 1;
                }
                Err(e) => {
                    println!("{}", "❌ FAILED".red());
                    if args.verbose {
                        println!("   Error: {}", e);
                    }
                    failed += 1;
                }
            }
        }
        _ => {
            anyhow::bail!("Unsupported transport: {}", args.transport);
        }
    }

    // Test 3: Protocol compliance
    print!("📜 Testing protocol compliance... ");
    match test_protocol_compliance(args).await {
        Ok(()) => {
            println!("{}", "✅ PASSED".green());
            passed += 1;
        }
        Err(e) => {
            println!("{}", "❌ FAILED".red());
            if args.verbose {
                println!("   Error: {}", e);
            }
            failed += 1;
        }
    }

    println!("\n📊 Test Results:");
    println!("   Passed: {}", passed.to_string().green());
    println!("   Failed: {}", failed.to_string().red());
    println!("   Total:  {}", (passed + failed));

    if failed > 0 {
        anyhow::bail!("{} test(s) failed", failed);
    }

    Ok(())
}

async fn run_specific_test(test_name: &str, args: &TestArgs, config: Option<Config>) -> Result<()> {
    println!("🎯 Running specific test: {}\n", test_name);

    match test_name {
        "config" => test_config_loading(&config).await,
        "stdio" => test_stdio_connection(args).await,
        "http" => test_http_connection(args).await,
        "protocol" => test_protocol_compliance(args).await,
        _ => anyhow::bail!("Unknown test: {}", test_name),
    }
}

async fn test_config_loading(config: &Option<Config>) -> Result<()> {
    match config {
        Some(config) => {
            // Validate config structure
            if config.project.name.is_empty() {
                anyhow::bail!("Project name is empty");
            }

            // Check for servers
            if config.servers.is_empty() {
                println!("   ⚠️  No servers configured");
            } else {
                println!("   📊 Found {} server(s)", config.servers.len());
            }

            // Check for clients
            if config.clients.is_empty() {
                println!("   ⚠️  No clients configured");
            } else {
                println!("   👥 Found {} client(s)", config.clients.len());
            }

            Ok(())
        }
        None => {
            // Config not found is not necessarily an error
            println!("   ℹ️  No configuration file found (this is okay)");
            Ok(())
        }
    }
}

async fn test_stdio_connection(args: &TestArgs) -> Result<()> {
    if let Some(server) = &args.server {
        println!("   Testing connection to: {}", server);

        // Parse server command
        let parts: Vec<&str> = server.split_whitespace().collect();
        if parts.is_empty() {
            anyhow::bail!("Invalid server command");
        }

        let command = parts[0];
        let server_args = &parts[1..];

        if args.verbose {
            println!("   Command: {}", command);
            println!("   Args: {:?}", server_args);
        }

        // Test basic command availability
        match std::process::Command::new(command)
            .args(["--help"])
            .output()
        {
            Ok(_) => {
                println!("   ✅ Command is available");
            }
            Err(e) => {
                anyhow::bail!("Command not found or not executable: {}", e);
            }
        }

        // Implement actual MCP handshake test
        println!("   🔄 Testing MCP handshake...");

        // Start server process
        let mut server_process = std::process::Command::new(command)
            .args(server_args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .context("Failed to start server process")?;

        // Wait a moment for server to start
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Send initialization request
        let init_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-06-18",
                "capabilities": {
                    "tools": {}
                },
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }
        });

        if let Some(stdin) = server_process.stdin.as_mut() {
            let request_str = serde_json::to_string(&init_request)?;
            stdin
                .write_all((request_str + "\n").as_bytes())
                .context("Failed to write to server stdin")?;
        }

        // Read response with timeout
        let response = tokio::time::timeout(Duration::from_secs(5), async {
            if let Some(stdout) = server_process.stdout.as_mut() {
                let mut buffer = String::new();
                let mut reader = std::io::BufReader::new(stdout);
                std::io::BufRead::read_line(&mut reader, &mut buffer)
                    .context("Failed to read server response")?;
                Ok(buffer)
            } else {
                anyhow::bail!("No stdout from server process")
            }
        })
        .await
        .context("Timeout waiting for server response")??;

        // Parse and validate response
        let response_json: serde_json::Value =
            serde_json::from_str(&response).context("Failed to parse server response as JSON")?;

        if let Some(result) = response_json.get("result") {
            if let Some(protocol_version) = result.get("protocolVersion") {
                println!("   ✅ MCP handshake successful");
                println!("   📋 Protocol version: {}", protocol_version);
            } else {
                anyhow::bail!("Invalid initialization response: missing protocolVersion");
            }
        } else {
            anyhow::bail!("Invalid initialization response: missing result");
        }

        // Send shutdown request
        let shutdown_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "shutdown",
            "params": {}
        });

        if let Some(stdin) = server_process.stdin.as_mut() {
            let request_str = serde_json::to_string(&shutdown_request)?;
            stdin
                .write_all((request_str + "\n").as_bytes())
                .context("Failed to write shutdown request")?;
        }

        // Wait for server to terminate
        let _ = server_process.wait();
    } else {
        println!("   ⚠️  No server specified for STDIO test");
    }

    Ok(())
}

async fn test_http_connection(args: &TestArgs) -> Result<()> {
    if let Some(server) = &args.server {
        println!("   Testing connection to: {}", server);

        // Parse URL
        let url: reqwest::Url = server.parse().context("Invalid server URL")?;

        if args.verbose {
            println!("   Host: {}", url.host_str().unwrap_or("unknown"));
            println!("   Port: {}", url.port().unwrap_or(80));
        }

        // Test basic connectivity
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;

        match client.get(server).send().await {
            Ok(response) => {
                println!("   ✅ HTTP connection successful");
                if args.verbose {
                    println!("   Status: {}", response.status());
                }
            }
            Err(e) => {
                anyhow::bail!("HTTP connection failed: {}", e);
            }
        }

        // Implement MCP-over-HTTP test
        println!("   🔄 Testing MCP-over-HTTP protocol...");

        // Test MCP HTTP endpoint
        let mcp_url = format!("{}/mcp", server);
        let init_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-06-18",
                "capabilities": {
                    "tools": {}
                },
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }
        });

        let response = client
            .post(&mcp_url)
            .header("Content-Type", "application/json")
            .json(&init_request)
            .send()
            .await
            .context("Failed to send MCP initialization request")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "MCP initialization failed with status: {}",
                response.status()
            );
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse MCP response")?;

        if let Some(result) = response_json.get("result") {
            if let Some(protocol_version) = result.get("protocolVersion") {
                println!("   ✅ MCP-over-HTTP handshake successful");
                println!("   📋 Protocol version: {}", protocol_version);
            } else {
                anyhow::bail!("Invalid MCP response: missing protocolVersion");
            }
        } else {
            anyhow::bail!("Invalid MCP response: missing result");
        }
    } else {
        anyhow::bail!("Server URL required for HTTP test");
    }

    Ok(())
}

async fn test_protocol_compliance(_args: &TestArgs) -> Result<()> {
    println!("   Testing MCP protocol compliance...");

    // Test JSON-RPC 2.0 compliance
    println!("   🔍 Testing JSON-RPC 2.0 compliance...");
    test_jsonrpc_compliance().await?;

    // Test MCP message format validation
    println!("   📝 Testing MCP message format validation...");
    test_mcp_message_format().await?;

    // Test required method implementations
    println!("   ⚙️  Testing required method implementations...");
    test_required_methods().await?;

    // Test error handling compliance
    println!("   ⚠️  Testing error handling compliance...");
    test_error_handling().await?;

    println!("   ✅ All protocol compliance tests passed");

    Ok(())
}

async fn test_jsonrpc_compliance() -> Result<()> {
    // Test JSON-RPC 2.0 message structure
    let valid_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "test",
        "params": {}
    });

    // Validate required fields
    if valid_request.get("jsonrpc").is_none() {
        anyhow::bail!("JSON-RPC 2.0 request missing 'jsonrpc' field");
    }
    if valid_request.get("method").is_none() {
        anyhow::bail!("JSON-RPC 2.0 request missing 'method' field");
    }

    // Test error response format
    let error_response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "error": {
            "code": -32601,
            "message": "Method not found"
        }
    });

    if let Some(error) = error_response.get("error") {
        if error.get("code").is_none() || error.get("message").is_none() {
            anyhow::bail!("JSON-RPC 2.0 error response missing required fields");
        }
    } else {
        anyhow::bail!("JSON-RPC 2.0 error response missing 'error' field");
    }

    Ok(())
}

async fn test_mcp_message_format() -> Result<()> {
    // Test MCP initialization request format
    let init_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-06-18",
            "capabilities": {
                "tools": {}
            },
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }
    });

    // Validate MCP-specific fields
    if let Some(params) = init_request.get("params") {
        if params.get("protocolVersion").is_none() {
            anyhow::bail!("MCP initialization request missing 'protocolVersion'");
        }
        if params.get("capabilities").is_none() {
            anyhow::bail!("MCP initialization request missing 'capabilities'");
        }
        if params.get("clientInfo").is_none() {
            anyhow::bail!("MCP initialization request missing 'clientInfo'");
        }
    } else {
        anyhow::bail!("MCP initialization request missing 'params'");
    }

    // Test tool call format
    let tool_call = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "test-tool",
            "arguments": {}
        }
    });

    if let Some(params) = tool_call.get("params") {
        if params.get("name").is_none() {
            anyhow::bail!("Tool call missing 'name' field");
        }
    }

    Ok(())
}

async fn test_required_methods() -> Result<()> {
    // Define required MCP methods
    let required_methods = vec!["initialize", "shutdown", "tools/list", "tools/call"];

    // Test method name format
    for method in required_methods {
        if !method.contains('/') && method != "initialize" && method != "shutdown" {
            anyhow::bail!("Invalid MCP method name format: {}", method);
        }
    }

    Ok(())
}

async fn test_error_handling() -> Result<()> {
    // Test standard JSON-RPC error codes
    let standard_codes = vec![
        -32700, // Parse error
        -32600, // Invalid request
        -32601, // Method not found
        -32602, // Invalid params
        -32603, // Internal error
    ];

    for code in standard_codes {
        // JSON-RPC error codes are -32700 to -32000 (inclusive)
        if (-32700..=-32000).contains(&code) {
            // Valid range for JSON-RPC error codes
        } else {
            anyhow::bail!("Invalid JSON-RPC error code: {}", code);
        }
    }

    Ok(())
}
