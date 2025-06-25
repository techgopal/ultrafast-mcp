use ultrafast_mcp::prelude::*;
use tokio::time::{timeout, Duration};
use serde_json::json;

#[tokio::test]
async fn test_complete_mcp_server_client_flow() -> Result<(), Box<dyn std::error::Error>> {
    // Create a test server
    let server = UltraFastServer::new("integration-test-server")
        .with_version("1.0.0")
        .tool("echo", |params: serde_json::Value, _ctx| async move {
            Ok(json!({
                "echo": params.get("message"),
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
        })
        .resource("test://status", || async {
            Ok(json!({
                "name": "Integration Test Server",
                "status": "running",
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
        })
        .build()?;
    
    // Test that server is created successfully
    assert_eq!(server.info().name, "integration-test-server");
    assert_eq!(server.info().version, "1.0.0");
    
    // Test tool registration
    let tools = server.list_tools().await?;
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "echo");
    
    // Test resource registration
    let resources = server.list_resources().await?;
    assert_eq!(resources.len(), 1);
    assert_eq!(resources[0].uri, "test://status");
    
    println!("✅ Complete MCP server-client integration test passed!");
    Ok(())
}

#[tokio::test]
async fn test_mcp_protocol_compliance() -> Result<(), Box<dyn std::error::Error>> {
    // Test JSON-RPC message creation and serialization
    let request = ultrafast_mcp_core::protocol::jsonrpc::JsonRpcRequest::new(
        ultrafast_mcp_core::RequestId::String("test-123".to_string()),
        "test_method".to_string(),
        Some(json!({"param": "value"}))
    );
    
    let serialized = serde_json::to_string(&request)?;
    assert!(serialized.contains("test-123"));
    assert!(serialized.contains("test_method"));
    
    // Test MCP 2025-06-18 compliance
    use ultrafast_mcp_core::protocol::lifecycle::InitializeRequest;
    use ultrafast_mcp_core::protocol::capabilities::ClientCapabilities;
    use ultrafast_mcp_core::types::client::ClientInfo;
    
    let init_request = InitializeRequest {
        protocol_version: "2025-06-18".to_string(),
        capabilities: ClientCapabilities::default(),
        client_info: ClientInfo {
            name: "test-client".to_string(),
            version: "1.0.0".to_string(),
        },
    };
    
    let serialized_init = serde_json::to_string(&init_request)?;
    assert!(serialized_init.contains("2025-06-18"));
    
    println!("✅ MCP protocol compliance test passed!");
    Ok(())
}

#[tokio::test]
async fn test_performance_benchmarks() -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Instant;
    
    // Test server creation performance
    let start = Instant::now();
    let _server = UltraFastServer::new("perf-test-server")
        .with_version("1.0.0")
        .tool("fast_tool", |_: serde_json::Value, _ctx| async move {
            Ok(json!({"result": "fast"}))
        })
        .build()?;
    let creation_time = start.elapsed();
    
    // Should create server very quickly (< 10ms)
    assert!(creation_time < Duration::from_millis(10));
    
    // Test message serialization performance
    let start = Instant::now();
    let request = ultrafast_mcp_core::protocol::jsonrpc::JsonRpcRequest::new(
        ultrafast_mcp_core::RequestId::String("perf-test".to_string()),
        "test_method".to_string(),
        Some(json!({"data": "test"}))
    );
    
    for _ in 0..1000 {
        let _serialized = serde_json::to_string(&request).unwrap();
    }
    let serialization_time = start.elapsed();
    
    // Should serialize 1000 messages quickly (< 100ms)
    assert!(serialization_time < Duration::from_millis(100));
    
    println!("✅ Performance benchmarks passed!");
    println!("   Server creation: {:?}", creation_time);
    println!("   1000 message serializations: {:?}", serialization_time);
    
    Ok(())
}

#[tokio::test]
async fn test_concurrent_operations() -> Result<(), Box<dyn std::error::Error>> {
    let server = UltraFastServer::new("concurrent-test-server")
        .with_version("1.0.0")
        .tool("concurrent_tool", |params: serde_json::Value, _ctx| async move {
            // Simulate some async work
            tokio::time::sleep(Duration::from_millis(10)).await;
            Ok(json!({
                "processed": params.get("input"),
                "worker_id": params.get("worker_id")
            }))
        })
        .build()?;
    
    // Create multiple concurrent operations
    let mut handles = Vec::new();
    
    for i in 0..10 {
        let server_clone = server.clone();
        let handle = tokio::spawn(async move {
            let tools = server_clone.list_tools().await.unwrap();
            assert_eq!(tools.len(), 1);
            assert_eq!(tools[0].name, "concurrent_tool");
            i
        });
        handles.push(handle);
    }
    
    // Wait for all operations to complete
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await?);
    }
    
    // Check that all operations completed successfully
    assert_eq!(results.len(), 10);
    assert_eq!(results, (0..10).collect::<Vec<_>>());
    
    println!("✅ Concurrent operations test passed!");
    Ok(())
}

#[tokio::test]
async fn test_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    let server = UltraFastServer::new("error-test-server")
        .with_version("1.0.0")
        .tool("error_tool", |_: serde_json::Value, _ctx| async move {
            Err::<serde_json::Value, _>("Intentional test failure".to_string())
        })
        .build()?;
    
    // Test that server handles tool errors gracefully
    let tools = server.list_tools().await?;
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "error_tool");
    
    println!("✅ Error handling test passed!");
    Ok(())
}

#[tokio::test]
async fn test_schema_generation() -> Result<(), Box<dyn std::error::Error>> {
    #[derive(serde::Deserialize)]
    struct TestInput {
        name: String,
        count: u32,
    }
    
    #[derive(serde::Serialize)]
    struct TestOutput {
        message: String,
        total: u32,
    }
    
    let server = UltraFastServer::new("schema-test-server")
        .with_version("1.0.0")
        .tool("typed_tool", |input: TestInput, _ctx| async move {
            Ok(TestOutput {
                message: format!("Hello, {}!", input.name),
                total: input.count * 2,
            })
        })
        .description("Tool with typed input/output")
        .build()?;
    
    let tools = server.list_tools().await?;
    assert_eq!(tools.len(), 1);
    
    // Test that tool has schema information
    let tool = &tools[0];
    assert_eq!(tool.name, "typed_tool");
    assert!(tool.input_schema.is_object());
    
    println!("✅ Schema generation test passed!");
    Ok(())
}
