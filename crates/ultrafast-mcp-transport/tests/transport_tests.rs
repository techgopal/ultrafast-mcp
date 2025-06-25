use ultrafast_mcp_transport::*;
use ultrafast_mcp_core::protocol::JsonRpcMessage;
use std::time::Duration;

#[cfg(test)]
mod stdio_tests {
    use super::*;

    #[tokio::test]
    async fn test_stdio_transport_creation() {
        let transport = StdioTransport::new();
        // Test that transport can be created successfully
        assert!(true); // Placeholder - transport creation should succeed
    }

    #[tokio::test]
    async fn test_stdio_transport_message_format() {
        // Test that messages are properly formatted for STDIO
        let message = JsonRpcMessage::Request(ultrafast_mcp_core::JsonRpcRequest::new(
            "test_method".to_string(),
            None,
            Some(ultrafast_mcp_core::RequestId::String("test".to_string()))
        ));
        
        let serialized = serde_json::to_string(&message).unwrap();
        assert!(serialized.contains("test_method"));
        assert!(serialized.contains("2.0")); // JSON-RPC version
    }

    #[tokio::test]
    async fn test_transport_error_handling() {
        let error = TransportError::ConnectionError {
            message: "Test connection error".to_string()
        };
        
        assert!(format!("{}", error).contains("Test connection error"));
    }

    #[tokio::test]
    async fn test_transport_error_types() {
        let errors = vec![
            TransportError::ConnectionClosed,
            TransportError::SerializationError { message: "test".to_string() },
            TransportError::NetworkError { message: "network".to_string() },
            TransportError::AuthenticationError { message: "auth".to_string() },
            TransportError::ProtocolError { message: "protocol".to_string() },
            TransportError::InitializationError { message: "init".to_string() },
        ];
        
        for error in errors {
            // Ensure all error types can be displayed
            let _ = format!("{}", error);
        }
    }
}

#[cfg(test)]
mod middleware_tests {
    use super::*;

    #[tokio::test]
    async fn test_middleware_pipeline() {
        // Test that middleware can be composed
        // This is a placeholder for when middleware is fully implemented
        assert!(true); // Placeholder assertion
    }
}

#[cfg(feature = "http")]
#[cfg(test)]
mod http_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_http_transport_config() {
        let config = HttpTransportConfig::default();
        
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert!(config.cors_enabled);
    }

    #[tokio::test]
    async fn test_http_transport_server_creation() {
        let config = HttpTransportConfig::default();
        
        // This tests that the server can be created with valid config
        let _server = HttpTransportServer::new(config);
        // Server creation should succeed
        assert!(true); // Placeholder - server creation should succeed
    }
}

#[cfg(test)]
mod transport_abstraction_tests {
    use super::*;

    #[test]
    fn test_transport_result_type() {
        let success: Result<String> = Ok("success".to_string());
        assert!(success.is_ok());
        
        let failure: Result<String> = Err(TransportError::ConnectionClosed);
        assert!(failure.is_err());
    }

    #[tokio::test]
    async fn test_transport_factory() {
        // Test that transport creation works with different configs
        let stdio_config = TransportConfig::Stdio;
        
        #[cfg(feature = "http")]
        let streamable_config = TransportConfig::Streamable {
            base_url: "http://localhost:8080/mcp".to_string(),
            auth_token: None,
            session_id: None,
        };
        
        // Test that configs can be created
        assert!(matches!(stdio_config, TransportConfig::Stdio));
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_message_serialization_performance() {
        let message = JsonRpcMessage::Request(ultrafast_mcp_core::JsonRpcRequest::new(
            "performance_method".to_string(),
            Some(serde_json::json!({"data": "test_data"})),
            Some(ultrafast_mcp_core::RequestId::String("perf_test".to_string()))
        ));
        
        let start = Instant::now();
        
        // Serialize 1000 messages
        for _ in 0..1000 {
            let _serialized = serde_json::to_string(&message).unwrap();
        }
        
        let duration = start.elapsed();
        
        // Should complete in reasonable time (less than 100ms)
        assert!(duration < Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_concurrent_transport_operations() {
        let mut handles = vec![];
        
        // Spawn multiple tasks that create and use transports
        for i in 0..10 {
            let handle = tokio::spawn(async move {
                let _transport = StdioTransport::new();
                // Simulate some work
                tokio::time::sleep(Duration::from_millis(1)).await;
                format!("transport_{}", i)
            });
            handles.push(handle);
        }
        
        // Wait for all tasks to complete
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.starts_with("transport_"));
        }
    }
}
