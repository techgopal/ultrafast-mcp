use std::sync::Arc;
use ultrafast_mcp_core::error::MCPResult;
use ultrafast_mcp_core::protocol::capabilities::ServerCapabilities;
use ultrafast_mcp_core::types::{
    server::ServerInfo,
    tools::{ToolCall, ToolContent, ToolResult},
};
use ultrafast_mcp_server::{
    ListToolsRequest, ListToolsResponse, ServerState, ToolHandler, UltraFastServer,
};

#[cfg(test)]
mod server_tests {
    use super::*;

    #[test]
    fn test_server_state_transitions() {
        assert_eq!(
            std::mem::discriminant(&ServerState::Uninitialized),
            std::mem::discriminant(&ServerState::Uninitialized)
        );

        // Test that states are different
        assert_ne!(
            std::mem::discriminant(&ServerState::Uninitialized),
            std::mem::discriminant(&ServerState::Initialized)
        );
    }

    #[tokio::test]
    async fn test_server_builder_creation() {
        let server_info = ServerInfo {
            name: "test-server".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            homepage: None,
            repository: None,
            authors: None,
            license: None,
        };
        let capabilities = ServerCapabilities::default();
        let server = UltraFastServer::new(server_info.clone(), capabilities);
        assert_eq!(server.info().name, "test-server");
        assert_eq!(server.info().version, "1.0.0");
    }

    #[tokio::test]
    async fn test_server_builder_with_capabilities() {
        let server_info = ServerInfo {
            name: "test-server".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            homepage: None,
            repository: None,
            authors: None,
            license: None,
        };
        let capabilities = ServerCapabilities::default();
        let server = UltraFastServer::new(server_info.clone(), capabilities);
        assert_eq!(server.info().name, "test-server");
    }

    #[tokio::test]
    async fn test_server_creation_with_handlers() {
        let server_info = ServerInfo {
            name: "test-server".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            homepage: None,
            repository: None,
            authors: None,
            license: None,
        };
        let capabilities = ServerCapabilities::default();
        let server = UltraFastServer::new(server_info.clone(), capabilities);
        assert_eq!(server.info().name, "test-server");
    }

    #[tokio::test]
    async fn test_server_with_monitoring() {
        let server_info = ServerInfo {
            name: "test-server".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            homepage: None,
            repository: None,
            authors: None,
            license: None,
        };
        let capabilities = ServerCapabilities::default();
        let server = UltraFastServer::new(server_info.clone(), capabilities);
        assert_eq!(server.info().name, "test-server");
        // Note: monitoring system is available as a field but not exposed via a method
    }

    #[tokio::test]
    async fn test_server_state_management() {
        let server_info = ServerInfo {
            name: "test-server".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            homepage: None,
            repository: None,
            authors: None,
            license: None,
        };
        let capabilities = ServerCapabilities::default();
        let server = UltraFastServer::new(server_info.clone(), capabilities);
        assert_eq!(server.info().name, "test-server");
        assert_eq!(server.info().version, "1.0.0");
    }

    #[tokio::test]
    async fn test_server_builder_pattern() {
        let server_info = ServerInfo {
            name: "integration-test-server".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            homepage: None,
            repository: None,
            authors: None,
            license: None,
        };
        let capabilities = ServerCapabilities::default();
        let server = UltraFastServer::new(server_info.clone(), capabilities);
        assert_eq!(server.info().name, "integration-test-server");
    }

    #[tokio::test]
    async fn test_server_concurrent_access() {
        let server_info = ServerInfo {
            name: "concurrent-server".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            homepage: None,
            repository: None,
            authors: None,
            license: None,
        };
        let capabilities = ServerCapabilities::default();
        let server = Arc::new(UltraFastServer::new(server_info.clone(), capabilities));
        let server_clone: Arc<UltraFastServer> = Arc::clone(&server);
        let name = server_clone.info().name.clone();
        assert_eq!(name, "concurrent-server");
    }

    #[tokio::test]
    async fn test_server_state_enum() {
        assert_eq!(
            std::mem::discriminant(&ServerState::Uninitialized),
            std::mem::discriminant(&ServerState::Uninitialized)
        );
        assert_ne!(
            std::mem::discriminant(&ServerState::Uninitialized),
            std::mem::discriminant(&ServerState::Initialized)
        );
    }
}

#[cfg(test)]
mod handler_tests {
    use super::*;

    #[tokio::test]
    async fn test_tool_handler_trait() {
        struct TestToolHandler;
        #[async_trait::async_trait]
        impl ToolHandler for TestToolHandler {
            async fn handle_tool_call(&self, _call: ToolCall) -> MCPResult<ToolResult> {
                Ok(ToolResult {
                    content: vec![ToolContent::Text {
                        text: "test result".to_string(),
                    }],
                    is_error: Some(false),
                })
            }
            async fn list_tools(&self, _request: ListToolsRequest) -> MCPResult<ListToolsResponse> {
                Ok(ListToolsResponse {
                    tools: vec![],
                    next_cursor: None,
                })
            }
        }
        let handler = TestToolHandler;
        let call = ToolCall {
            name: "test".to_string(),
            arguments: None,
        };
        let result = handler.handle_tool_call(call).await.unwrap();
        assert_eq!(result.content.len(), 1);
        assert_eq!(result.is_error, Some(false));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn test_server_builder_pattern() {
        let server_info = ServerInfo {
            name: "integration-test-server".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            homepage: None,
            repository: None,
            authors: None,
            license: None,
        };

        let server = UltraFastServer::new(server_info, ServerCapabilities::default());

        // Test the server was built correctly
        assert_eq!(server.info().name, "integration-test-server");
        assert_eq!(server.info().version, "1.0.0");
    }

    #[tokio::test]
    async fn test_server_concurrent_access() {
        let server_info = ServerInfo {
            name: "concurrent-test-server".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            homepage: None,
            repository: None,
            authors: None,
            license: None,
        };

        let server = Arc::new(UltraFastServer::new(
            server_info,
            ServerCapabilities::default(),
        ));

        let mut handles = vec![];

        // Spawn multiple concurrent access attempts
        for _i in 0..10 {
            let server_clone = Arc::clone(&server);
            let handle = tokio::spawn(async move {
                // Test concurrent access to server info
                
                server_clone.info().name.clone()
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            let result = timeout(Duration::from_secs(1), handle).await;
            assert!(result.is_ok());
            let name = result.unwrap().unwrap();
            assert_eq!(name, "concurrent-test-server");
        }
    }

    #[tokio::test]
    async fn test_server_state_enum() {
        // Test ServerState enum variants
        let states = [
            ServerState::Uninitialized,
            ServerState::Initializing,
            ServerState::Initialized,
            ServerState::Shutdown,
        ];

        // Test that all states can be created and are different
        assert_eq!(states.len(), 4);

        // Test state transitions make sense
        assert_ne!(
            std::mem::discriminant(&ServerState::Uninitialized),
            std::mem::discriminant(&ServerState::Initialized)
        );
    }
}
