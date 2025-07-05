//! Comprehensive MCP Compliance Integration Tests
//!
//! This test suite validates that the ultrafast-mcp implementation is fully compliant
//! with the MCP specification and works correctly with real MCP clients.

use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::process::Command;
use tokio::time::timeout;
use serde_json::{json, Value};

/// Test the full MCP initialization sequence
#[tokio::test]
async fn test_mcp_initialization_sequence() {
    let mut child = Command::new("./target/release/basic-echo-server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn basic-echo-server");

    let stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    
    let mut writer = BufWriter::new(stdin);
    let mut reader = BufReader::new(stdout);

    // Step 1: Send initialize request
    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }
    });

    writer.write_all(initialize_request.to_string().as_bytes()).await.unwrap();
    writer.write_all(b"\n").await.unwrap();
    writer.flush().await.unwrap();

    // Read initialize response
    let mut response_line = String::new();
    timeout(Duration::from_secs(5), reader.read_line(&mut response_line))
        .await
        .expect("Timeout reading initialize response")
        .expect("Failed to read initialize response");

    let response: Value = serde_json::from_str(&response_line).unwrap();
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"].is_object());
    assert!(response["result"]["capabilities"].is_object());

    // Step 2: Send initialized notification
    let initialized_notification = json!({
        "jsonrpc": "2.0",
        "method": "initialized",
        "params": {}
    });

    writer.write_all(initialized_notification.to_string().as_bytes()).await.unwrap();
    writer.write_all(b"\n").await.unwrap();
    writer.flush().await.unwrap();

    // Step 3: Test tools/list after initialization
    let tools_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });

    writer.write_all(tools_request.to_string().as_bytes()).await.unwrap();
    writer.write_all(b"\n").await.unwrap();
    writer.flush().await.unwrap();

    // Read tools response
    let mut tools_response_line = String::new();
    timeout(Duration::from_secs(5), reader.read_line(&mut tools_response_line))
        .await
        .expect("Timeout reading tools response")
        .expect("Failed to read tools response");

    let tools_response: Value = serde_json::from_str(&tools_response_line).unwrap();
    assert_eq!(tools_response["jsonrpc"], "2.0");
    assert_eq!(tools_response["id"], 2);
    assert!(tools_response["result"]["tools"].is_array());

    // Verify echo tool is present
    let tools = tools_response["result"]["tools"].as_array().unwrap();
    assert!(!tools.is_empty());
    let echo_tool = tools.iter().find(|tool| tool["name"] == "echo").unwrap();
    assert_eq!(echo_tool["name"], "echo");
    assert!(echo_tool["description"].is_string());

    // Step 4: Test tool call
    let tool_call_request = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "echo",
            "arguments": {
                "message": "Hello, MCP!"
            }
        }
    });

    writer.write_all(tool_call_request.to_string().as_bytes()).await.unwrap();
    writer.write_all(b"\n").await.unwrap();
    writer.flush().await.unwrap();

    // Read tool call response
    let mut tool_response_line = String::new();
    timeout(Duration::from_secs(5), reader.read_line(&mut tool_response_line))
        .await
        .expect("Timeout reading tool response")
        .expect("Failed to read tool response");

    let tool_response: Value = serde_json::from_str(&tool_response_line).unwrap();
    assert_eq!(tool_response["jsonrpc"], "2.0");
    assert_eq!(tool_response["id"], 3);
    assert!(tool_response["result"]["content"].is_array());

    // Step 5: Test graceful shutdown
    let shutdown_request = json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "shutdown",
        "params": {}
    });

    writer.write_all(shutdown_request.to_string().as_bytes()).await.unwrap();
    writer.write_all(b"\n").await.unwrap();
    writer.flush().await.unwrap();

    // Read shutdown response
    let mut shutdown_response_line = String::new();
    timeout(Duration::from_secs(5), reader.read_line(&mut shutdown_response_line))
        .await
        .expect("Timeout reading shutdown response")
        .expect("Failed to read shutdown response");

    let shutdown_response: Value = serde_json::from_str(&shutdown_response_line).unwrap();
    assert_eq!(shutdown_response["jsonrpc"], "2.0");
    assert_eq!(shutdown_response["id"], 4);
    assert!(shutdown_response["result"].is_object());

    // Clean up
    child.kill().await.ok();
}

/// Test protocol version negotiation
#[tokio::test]
async fn test_protocol_version_negotiation() {
    let mut child = Command::new("./target/release/basic-echo-server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn basic-echo-server");

    let stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    
    let mut writer = BufWriter::new(stdin);
    let mut reader = BufReader::new(stdout);

    // Test with older supported version
    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }
    });

    writer.write_all(initialize_request.to_string().as_bytes()).await.unwrap();
    writer.write_all(b"\n").await.unwrap();
    writer.flush().await.unwrap();

    let mut response_line = String::new();
    timeout(Duration::from_secs(5), reader.read_line(&mut response_line))
        .await
        .expect("Timeout reading response")
        .expect("Failed to read response");

    let response: Value = serde_json::from_str(&response_line).unwrap();
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"].is_object());
    
    // Should negotiate to the client's version
    assert_eq!(response["result"]["protocolVersion"], "2024-11-05");

    child.kill().await.ok();
}

/// Test error handling for invalid requests
#[tokio::test]
async fn test_error_handling() {
    let mut child = Command::new("./target/release/basic-echo-server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn basic-echo-server");

    let stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    
    let mut writer = BufWriter::new(stdin);
    let mut reader = BufReader::new(stdout);

    // Test invalid JSON
    writer.write_all(b"invalid json\n").await.unwrap();
    writer.flush().await.unwrap();

    // Should receive a parse error response
    let mut response_line = String::new();
    timeout(Duration::from_secs(5), reader.read_line(&mut response_line))
        .await
        .expect("Timeout reading error response")
        .expect("Failed to read error response");

    let response: Value = serde_json::from_str(&response_line).unwrap();
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["error"].is_object());
    assert_eq!(response["error"]["code"], -32700); // Parse error

    // Test method not found
    let invalid_method_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "nonexistent/method",
        "params": {}
    });

    writer.write_all(invalid_method_request.to_string().as_bytes()).await.unwrap();
    writer.write_all(b"\n").await.unwrap();
    writer.flush().await.unwrap();

    let mut method_error_line = String::new();
    timeout(Duration::from_secs(5), reader.read_line(&mut method_error_line))
        .await
        .expect("Timeout reading method error")
        .expect("Failed to read method error");

    let method_error: Value = serde_json::from_str(&method_error_line).unwrap();
    assert_eq!(method_error["jsonrpc"], "2.0");
    assert_eq!(method_error["id"], 1);
    assert!(method_error["error"].is_object());
    assert_eq!(method_error["error"]["code"], -32601); // Method not found

    child.kill().await.ok();
}

/// Test that notifications don't receive responses
#[tokio::test]
async fn test_notification_no_response() {
    let mut child = Command::new("./target/release/basic-echo-server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn basic-echo-server");

    let stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    
    let mut writer = BufWriter::new(stdin);
    let mut reader = BufReader::new(stdout);

    // Initialize first
    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }
    });

    writer.write_all(initialize_request.to_string().as_bytes()).await.unwrap();
    writer.write_all(b"\n").await.unwrap();
    writer.flush().await.unwrap();

    // Read initialize response
    let mut init_response = String::new();
    reader.read_line(&mut init_response).await.unwrap();

    // Send initialized notification (no id = notification)
    let initialized_notification = json!({
        "jsonrpc": "2.0",
        "method": "initialized",
        "params": {}
    });

    writer.write_all(initialized_notification.to_string().as_bytes()).await.unwrap();
    writer.write_all(b"\n").await.unwrap();
    writer.flush().await.unwrap();

    // Send a request immediately after to verify server is still responsive
    let tools_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });

    writer.write_all(tools_request.to_string().as_bytes()).await.unwrap();
    writer.write_all(b"\n").await.unwrap();
    writer.flush().await.unwrap();

    // Should only receive the tools response, not a response to the notification
    let mut response_line = String::new();
    timeout(Duration::from_secs(5), reader.read_line(&mut response_line))
        .await
        .expect("Timeout reading tools response")
        .expect("Failed to read tools response");

    let response: Value = serde_json::from_str(&response_line).unwrap();
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 2); // Should be the tools response, not notification response
    assert!(response["result"]["tools"].is_array());

    child.kill().await.ok();
}

/// Test server state management
#[tokio::test]
async fn test_server_state_management() {
    let mut child = Command::new("./target/release/basic-echo-server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn basic-echo-server");

    let stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    
    let mut writer = BufWriter::new(stdin);
    let mut reader = BufReader::new(stdout);

    // Test that tools/list fails before initialization
    let tools_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {}
    });

    writer.write_all(tools_request.to_string().as_bytes()).await.unwrap();
    writer.write_all(b"\n").await.unwrap();
    writer.flush().await.unwrap();

    let mut response_line = String::new();
    timeout(Duration::from_secs(5), reader.read_line(&mut response_line))
        .await
        .expect("Timeout reading error response")
        .expect("Failed to read error response");

    let response: Value = serde_json::from_str(&response_line).unwrap();
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["error"].is_object());
    assert!(response["error"]["message"].as_str().unwrap().contains("not ready"));

    child.kill().await.ok();
}

/// Test concurrent request handling
#[tokio::test]
async fn test_concurrent_requests() {
    let mut child = Command::new("./target/release/basic-echo-server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn basic-echo-server");

    let stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    
    let mut writer = BufWriter::new(stdin);
    let mut reader = BufReader::new(stdout);

    // Initialize server
    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }
    });

    writer.write_all(initialize_request.to_string().as_bytes()).await.unwrap();
    writer.write_all(b"\n").await.unwrap();
    writer.flush().await.unwrap();

    // Read initialize response
    let mut init_response = String::new();
    reader.read_line(&mut init_response).await.unwrap();

    // Send initialized notification
    let initialized_notification = json!({
        "jsonrpc": "2.0",
        "method": "initialized",
        "params": {}
    });

    writer.write_all(initialized_notification.to_string().as_bytes()).await.unwrap();
    writer.write_all(b"\n").await.unwrap();
    writer.flush().await.unwrap();

    // Send multiple requests with different IDs
    for i in 2..=5 {
        let request = json!({
            "jsonrpc": "2.0",
            "id": i,
            "method": "tools/call",
            "params": {
                "name": "echo",
                "arguments": {
                    "message": format!("Message {}", i)
                }
            }
        });

        writer.write_all(request.to_string().as_bytes()).await.unwrap();
        writer.write_all(b"\n").await.unwrap();
        writer.flush().await.unwrap();
    }

    // Read all responses and verify they match the requests
    let mut received_ids = Vec::new();
    for _ in 0..4 {
        let mut response_line = String::new();
        timeout(Duration::from_secs(5), reader.read_line(&mut response_line))
            .await
            .expect("Timeout reading response")
            .expect("Failed to read response");

        let response: Value = serde_json::from_str(&response_line).unwrap();
        assert_eq!(response["jsonrpc"], "2.0");
        assert!(response["result"].is_object());
        
        let id = response["id"].as_i64().unwrap();
        received_ids.push(id);
    }

    // All requests should have been processed
    received_ids.sort();
    assert_eq!(received_ids, vec![2, 3, 4, 5]);

    child.kill().await.ok();
}

/// Test with MCP Inspector compatibility
#[tokio::test]
async fn test_mcp_inspector_compatibility() {
    // This test verifies that our server works with the MCP Inspector
    // by testing the exact message flow that the Inspector uses
    
    let mut child = Command::new("./target/release/basic-echo-server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn basic-echo-server");

    let stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    
    let mut writer = BufWriter::new(stdin);
    let mut reader = BufReader::new(stdout);

    // MCP Inspector sends this exact initialization sequence
    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-06-18",
            "capabilities": {
                "sampling": {}
            },
            "clientInfo": {
                "name": "@modelcontextprotocol/inspector",
                "version": "0.1.0"
            }
        }
    });

    writer.write_all(initialize_request.to_string().as_bytes()).await.unwrap();
    writer.write_all(b"\n").await.unwrap();
    writer.flush().await.unwrap();

    // Read and verify initialize response
    let mut response_line = String::new();
    timeout(Duration::from_secs(5), reader.read_line(&mut response_line))
        .await
        .expect("Timeout reading initialize response")
        .expect("Failed to read initialize response");

    let response: Value = serde_json::from_str(&response_line).unwrap();
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"].is_object());
    assert!(response["result"]["capabilities"].is_object());
    assert!(response["result"]["serverInfo"].is_object());

    // MCP Inspector sends initialized notification
    let initialized_notification = json!({
        "jsonrpc": "2.0",
        "method": "initialized",
        "params": {}
    });

    writer.write_all(initialized_notification.to_string().as_bytes()).await.unwrap();
    writer.write_all(b"\n").await.unwrap();
    writer.flush().await.unwrap();

    // MCP Inspector requests tools list
    let tools_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });

    writer.write_all(tools_request.to_string().as_bytes()).await.unwrap();
    writer.write_all(b"\n").await.unwrap();
    writer.flush().await.unwrap();

    // Read tools response
    let mut tools_response_line = String::new();
    timeout(Duration::from_secs(5), reader.read_line(&mut tools_response_line))
        .await
        .expect("Timeout reading tools response")
        .expect("Failed to read tools response");

    let tools_response: Value = serde_json::from_str(&tools_response_line).unwrap();
    assert_eq!(tools_response["jsonrpc"], "2.0");
    assert_eq!(tools_response["id"], 2);
    assert!(tools_response["result"]["tools"].is_array());

    // Verify the response format matches MCP Inspector expectations
    let tools = tools_response["result"]["tools"].as_array().unwrap();
    assert!(!tools.is_empty());
    
    for tool in tools {
        assert!(tool["name"].is_string());
        assert!(tool["description"].is_string());
        assert!(tool["inputSchema"].is_object());
    }

    child.kill().await.ok();
}

/// Test transport lifecycle and error recovery
#[tokio::test] 
async fn test_transport_lifecycle() {
    let mut child = Command::new("./target/release/basic-echo-server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn basic-echo-server");

    let stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    
    let mut writer = BufWriter::new(stdin);
    let mut reader = BufReader::new(stdout);

    // Test that the server starts in a ready state
    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": {
                "name": "lifecycle-test",
                "version": "1.0.0"
            }
        }
    });

    writer.write_all(initialize_request.to_string().as_bytes()).await.unwrap();
    writer.write_all(b"\n").await.unwrap();
    writer.flush().await.unwrap();

    let mut response_line = String::new();
    timeout(Duration::from_secs(5), reader.read_line(&mut response_line))
        .await
        .expect("Timeout reading response")
        .expect("Failed to read response");

    let response: Value = serde_json::from_str(&response_line).unwrap();
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"].is_object());

    // Test graceful shutdown
    let shutdown_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "shutdown",
        "params": {}
    });

    writer.write_all(shutdown_request.to_string().as_bytes()).await.unwrap();
    writer.write_all(b"\n").await.unwrap();
    writer.flush().await.unwrap();

    let mut shutdown_response_line = String::new();
    timeout(Duration::from_secs(5), reader.read_line(&mut shutdown_response_line))
        .await
        .expect("Timeout reading shutdown response")
        .expect("Failed to read shutdown response");

    let shutdown_response: Value = serde_json::from_str(&shutdown_response_line).unwrap();
    assert_eq!(shutdown_response["jsonrpc"], "2.0");
    assert_eq!(shutdown_response["id"], 2);
    assert!(shutdown_response["result"].is_object());

    child.kill().await.ok();
} 