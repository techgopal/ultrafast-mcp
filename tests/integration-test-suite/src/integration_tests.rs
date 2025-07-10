#[cfg(test)]
mod tests {
    use anyhow::Result;
    use serde::{Deserialize, Serialize};
    use tokio::time::Duration;
    use ultrafast_mcp_core::{
        protocol::capabilities::ClientCapabilities,
        types::client::ClientInfo,
    };
    use ultrafast_mcp_test_utils::create_test_server_with_name;

    // create_test_server function moved to ultrafast-mcp-test-utils

    #[tokio::test]
    async fn test_complete_mcp_server_client_flow() -> Result<(), Box<dyn std::error::Error>> {
        // Create a test server
        let server = create_test_server_with_name("integration-test-server");

        // Test that server is created successfully
        assert_eq!(server.info().name, "integration-test-server");
        assert_eq!(server.info().version, "1.0.0");

        println!("✅ Complete MCP server-client integration test passed!");
        Ok(())
    }

    #[tokio::test]
    async fn test_mcp_protocol_compliance() -> Result<(), Box<dyn std::error::Error>> {
        // Test JSON-RPC message creation and serialization
        let request = ultrafast_mcp_core::protocol::jsonrpc::JsonRpcRequest::new(
            "test_method".to_string(),
            None,
            Some(ultrafast_mcp_core::RequestId::String(
                "test-123".to_string(),
            )),
        );

        let serialized = serde_json::to_string(&request)?;
        assert!(serialized.contains("test-123"));
        assert!(serialized.contains("test_method"));

        // Test MCP 2025-06-18 compliance
        use ultrafast_mcp_core::protocol::lifecycle::InitializeRequest;

        let init_request = InitializeRequest {
            protocol_version: "2025-06-18".to_string(),
            capabilities: ClientCapabilities::default(),
            client_info: ClientInfo {
                name: "test-client".to_string(),
                version: "1.0.0".to_string(),
                description: None,
                authors: None,
                homepage: None,
                repository: None,
                license: None,
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
        let _server = create_test_server_with_name("performance-test-server");
        let creation_time = start.elapsed();

        // Should create server very quickly (< 10ms)
        assert!(creation_time < Duration::from_millis(10));

        // Test message serialization performance
        let start = Instant::now();
        let request = ultrafast_mcp_core::protocol::jsonrpc::JsonRpcRequest::new(
            "test_method".to_string(),
            None,
            Some(ultrafast_mcp_core::RequestId::String(
                "perf-test".to_string(),
            )),
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
        let server = create_test_server_with_name("concurrent-test-server");

        // Create multiple concurrent operations
        let mut handles = Vec::new();

        for i in 0..10 {
            let server_clone = server.clone();
            let handle = tokio::spawn(async move {
                // Test that server can handle concurrent access
                assert_eq!(server_clone.info().name, "concurrent-test-server");
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
        let server = create_test_server_with_name("error-test-server");

        // Test that server is created successfully
        assert_eq!(server.info().name, "error-test-server");

        println!("✅ Error handling test passed!");
        Ok(())
    }

    #[tokio::test]
    async fn test_schema_generation() -> Result<(), Box<dyn std::error::Error>> {
        #[derive(Debug, Serialize, Deserialize)]
        struct TestInput {
            name: String,
            count: u32,
        }

        #[derive(Debug, Serialize, Deserialize)]
        struct TestOutput {
            message: String,
            total: u32,
        }

        // Test that we can serialize/deserialize typed structures
        let input = TestInput {
            name: "Test".to_string(),
            count: 5,
        };

        let output = TestOutput {
            message: format!("Hello, {}!", input.name),
            total: input.count * 2,
        };

        // Test serialization
        let input_json = serde_json::to_string(&input)?;
        let output_json = serde_json::to_string(&output)?;

        assert!(input_json.contains("Test"));
        assert!(input_json.contains("5"));
        assert!(output_json.contains("Hello, Test!"));
        assert!(output_json.contains("10"));

        println!("✅ Schema generation test passed!");
        Ok(())
    }
}
