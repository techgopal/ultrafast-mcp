//! Authentication Example for UltraFast MCP
//!
//! This example demonstrates how to use various authentication methods
//! with UltraFast MCP client and server.

use anyhow::Result;
use tracing::info;

// Import types from the main crate (these are conditionally available based on features)
#[cfg(feature = "oauth")]
use ultrafast_mcp::{
    ClientCapabilities, ClientInfo, ServerCapabilities, ServerInfo, UltraFastClient,
    UltraFastServer,
};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("UltraFast MCP Authentication Example");
    info!("====================================");

    // Example 1: Server with Bearer Token Authentication
    example_server_with_bearer_auth().await?;

    // Example 2: Client with Bearer Token Authentication
    example_client_with_bearer_auth().await?;

    // Example 3: Client with API Key Authentication
    example_client_with_api_key_auth().await?;

    // Example 4: Client with Basic Authentication
    example_client_with_basic_auth().await?;

    // Example 5: Client with Custom Headers
    example_client_with_custom_auth().await?;

    // Example 6: HTTP Transport with Authentication
    example_http_transport_with_auth().await?;

    // Example 7: Working Authentication Demo
    example_working_auth_demo().await?;

    Ok(())
}

/// Example: Server with Bearer Token Authentication
async fn example_server_with_bearer_auth() -> Result<()> {
    info!("\n1. Server with Bearer Token Authentication");
    info!("-------------------------------------------");

    #[cfg(feature = "oauth")]
    {
        let _server = UltraFastServer::new(
            ServerInfo {
                name: "auth-example-server".to_string(),
                version: "1.0.0".to_string(),
                description: Some("Authentication example server".to_string()),
                authors: None,
                homepage: None,
                license: None,
                repository: None,
            },
            ServerCapabilities::default(),
        )
        .with_bearer_auth(
            "your-jwt-secret-key".to_string(),
            vec!["read".to_string(), "write".to_string()],
        );

        info!("Server configured with Bearer token authentication");
        info!("Required scopes: read, write");
        info!("JWT secret configured");
    }

    #[cfg(not(feature = "oauth"))]
    {
        info!("OAuth feature not enabled. Enable with: cargo run --features oauth");
    }

    Ok(())
}

/// Example: Client with Bearer Token Authentication
async fn example_client_with_bearer_auth() -> Result<()> {
    info!("\n2. Client with Bearer Token Authentication");
    info!("-------------------------------------------");

    #[cfg(feature = "oauth")]
    {
        let _client = UltraFastClient::new(
            ClientInfo {
                name: "auth-example-client".to_string(),
                version: "1.0.0".to_string(),
                authors: None,
                description: Some("Authentication example client".to_string()),
                homepage: None,
                repository: None,
                license: None,
            },
            ClientCapabilities::default(),
        )
        .with_bearer_auth("your-access-token".to_string());

        info!("Client configured with Bearer token authentication");
        info!("Access token: your-access-token");
    }

    #[cfg(not(feature = "oauth"))]
    {
        info!("OAuth feature not enabled. Enable with: cargo run --features oauth");
    }

    Ok(())
}

/// Example: Client with Bearer Token Authentication and Auto-Refresh
#[allow(dead_code)]
async fn example_client_with_bearer_auth_refresh() -> Result<()> {
    info!("\n2b. Client with Bearer Token Authentication and Auto-Refresh");
    info!("-----------------------------------------------------------");

    #[cfg(feature = "oauth")]
    {
        let _client = UltraFastClient::new(
            ClientInfo {
                name: "auth-example-client".to_string(),
                version: "1.0.0".to_string(),
                authors: None,
                description: Some("Authentication example client".to_string()),
                homepage: None,
                repository: None,
                license: None,
            },
            ClientCapabilities::default(),
        )
        .with_bearer_auth_refresh("your-access-token".to_string(), || async {
            // This would typically call your token refresh endpoint
            Ok::<String, ultrafast_mcp::AuthError>("new-refreshed-token".to_string())
        });

        info!("Client configured with Bearer token authentication and auto-refresh");
        info!("Token will be automatically refreshed when needed");
    }

    #[cfg(not(feature = "oauth"))]
    {
        info!("OAuth feature not enabled. Enable with: cargo run --features oauth");
    }

    Ok(())
}

/// Example: Client with API Key Authentication
async fn example_client_with_api_key_auth() -> Result<()> {
    info!("\n3. Client with API Key Authentication");
    info!("-------------------------------------");

    #[cfg(feature = "oauth")]
    {
        let _client = UltraFastClient::new(
            ClientInfo {
                name: "api-key-client".to_string(),
                version: "1.0.0".to_string(),
                authors: None,
                description: Some("API key authentication example".to_string()),
                homepage: None,
                repository: None,
                license: None,
            },
            ClientCapabilities::default(),
        )
        .with_api_key_auth("your-api-key".to_string());

        info!("Client configured with API key authentication");
        info!("API key will be sent in X-API-Key header");

        // Example with custom header name
        let _client_custom = UltraFastClient::new(
            ClientInfo {
                name: "api-key-client-custom".to_string(),
                version: "1.0.0".to_string(),
                authors: None,
                description: Some("API key authentication with custom header".to_string()),
                homepage: None,
                repository: None,
                license: None,
            },
            ClientCapabilities::default(),
        )
        .with_api_key_auth_custom("your-api-key".to_string(), "X-Custom-API-Key".to_string());

        info!("Client configured with custom API key header: X-Custom-API-Key");
    }

    #[cfg(not(feature = "oauth"))]
    {
        info!("OAuth feature not enabled. Enable with: cargo run --features oauth");
    }

    Ok(())
}

/// Example: Client with Basic Authentication
async fn example_client_with_basic_auth() -> Result<()> {
    info!("\n4. Client with Basic Authentication");
    info!("-----------------------------------");

    #[cfg(feature = "oauth")]
    {
        let _client = UltraFastClient::new(
            ClientInfo {
                name: "basic-auth-client".to_string(),
                version: "1.0.0".to_string(),
                authors: None,
                description: Some("Basic authentication example".to_string()),
                homepage: None,
                repository: None,
                license: None,
            },
            ClientCapabilities::default(),
        )
        .with_basic_auth("username".to_string(), "password".to_string());

        info!("Client configured with Basic authentication");
        info!("Credentials will be base64 encoded and sent in Authorization header");
    }

    #[cfg(not(feature = "oauth"))]
    {
        info!("OAuth feature not enabled. Enable with: cargo run --features oauth");
    }

    Ok(())
}

/// Example: Client with Custom Header Authentication
async fn example_client_with_custom_auth() -> Result<()> {
    info!("\n5. Client with Custom Header Authentication");
    info!("-------------------------------------------");

    #[cfg(feature = "oauth")]
    {
        let _client = UltraFastClient::new(
            ClientInfo {
                name: "custom-auth-client".to_string(),
                version: "1.0.0".to_string(),
                authors: None,
                description: Some("Custom authentication example".to_string()),
                homepage: None,
                repository: None,
                license: None,
            },
            ClientCapabilities::default(),
        )
        .with_custom_auth()
        .with_auth(ultrafast_mcp::AuthMethod::Custom(
            ultrafast_mcp::CustomHeaderAuth::new()
                .with_header("X-Custom-Header".to_string(), "custom-value".to_string())
                .with_header("X-Another-Header".to_string(), "another-value".to_string()),
        ));

        info!("Client configured with custom header authentication");
        info!("Custom headers will be sent with each request");
    }

    #[cfg(not(feature = "oauth"))]
    {
        info!("OAuth feature not enabled. Enable with: cargo run --features oauth");
    }

    Ok(())
}

/// Example: HTTP Transport with Authentication
async fn example_http_transport_with_auth() -> Result<()> {
    info!("\n6. HTTP Transport with Authentication");
    info!("-------------------------------------");

    #[cfg(feature = "oauth")]
    {
        use ultrafast_mcp::streamable_http::client::StreamableHttpClientConfig;

        // Bearer token authentication
        let _config_bearer =
            StreamableHttpClientConfig::default().with_bearer_auth("your-access-token".to_string());

        info!("HTTP client configured with Bearer token authentication");

        // API key authentication
        let _config_api_key =
            StreamableHttpClientConfig::default().with_api_key_auth("your-api-key".to_string());

        info!("HTTP client configured with API key authentication");

        // Basic authentication
        let _config_basic = StreamableHttpClientConfig::default()
            .with_basic_auth("username".to_string(), "password".to_string());

        info!("HTTP client configured with Basic authentication");

        // OAuth authentication
        let oauth_config = ultrafast_mcp::OAuthConfig {
            client_id: "your-client-id".to_string(),
            client_secret: "your-client-secret".to_string(),
            auth_url: "https://auth.example.com/oauth/authorize".to_string(),
            token_url: "https://auth.example.com/oauth/token".to_string(),
            redirect_uri: "http://localhost:8080/callback".to_string(),
            scopes: vec!["read".to_string(), "write".to_string()],
        };

        let _config_oauth = StreamableHttpClientConfig::default().with_oauth_auth(oauth_config);

        info!("HTTP client configured with OAuth authentication");

        // Custom authentication
        let _config_custom = StreamableHttpClientConfig::default()
            .with_custom_auth()
            .with_auth_method(ultrafast_mcp::AuthMethod::Custom(
                ultrafast_mcp::CustomHeaderAuth::new()
                    .with_header("X-Custom-Header".to_string(), "custom-value".to_string()),
            ));

        info!("HTTP client configured with custom header authentication");
    }

    #[cfg(not(feature = "oauth"))]
    {
        info!("OAuth feature not enabled. Enable with: cargo run --features oauth");
    }

    Ok(())
}

/// Example: Server-side Authentication Middleware
#[allow(dead_code)]
async fn example_server_auth_middleware() -> Result<()> {
    info!("\n7. Server-side Authentication Middleware");
    info!("----------------------------------------");

    #[cfg(feature = "oauth")]
    {
        use ultrafast_mcp::{ServerAuthMiddleware, TokenValidator};

        // Create token validator
        let token_validator = TokenValidator::new("your-jwt-secret".to_string());

        // Create auth middleware
        let auth_middleware = ServerAuthMiddleware::new(token_validator)
            .with_required_scopes(vec!["read".to_string(), "write".to_string()]);

        info!("Server auth middleware configured");
        info!("Required scopes: read, write");

        // Example of using auth middleware in a request handler
        let mut headers = std::collections::HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            "Bearer your-jwt-token".to_string(),
        );

        match auth_middleware.validate_request(&headers).await {
            Ok(auth_context) => {
                info!("Authentication successful");
                info!("User ID: {:?}", auth_context.user_id);
                info!("Scopes: {:?}", auth_context.scopes);
                info!("Authenticated: {}", auth_context.is_authenticated);
            }
            Err(e) => {
                info!("Authentication failed: {:?}", e);
            }
        }
    }

    #[cfg(not(feature = "oauth"))]
    {
        info!("OAuth feature not enabled. Enable with: cargo run --features oauth");
    }

    Ok(())
}

/// Example: Client-side Authentication Middleware
#[allow(dead_code)]
async fn example_client_auth_middleware() -> Result<()> {
    info!("\n8. Client-side Authentication Middleware");
    info!("----------------------------------------");

    #[cfg(feature = "oauth")]
    {
        use ultrafast_mcp::{AuthMethod, ClientAuthMiddleware};

        // Create auth middleware with Bearer token
        let auth_method = AuthMethod::bearer("your-access-token".to_string());
        let mut auth_middleware = ClientAuthMiddleware::new(auth_method);

        info!("Client auth middleware configured with Bearer token");

        // Get authentication headers
        match auth_middleware.get_headers().await {
            Ok(headers) => {
                info!("Authentication headers generated:");
                for (key, value) in headers {
                    info!("  {}: {}", key, value);
                }
            }
            Err(e) => {
                info!("Failed to get auth headers: {:?}", e);
            }
        }
    }

    #[cfg(not(feature = "oauth"))]
    {
        info!("OAuth feature not enabled. Enable with: cargo run --features oauth");
    }

    Ok(())
}

/// Example: Working Authentication Demo
/// This demonstrates a complete client-server interaction with authentication
async fn example_working_auth_demo() -> Result<()> {
    use tracing::info;
    info!("\n9. Working Authentication Demo");
    info!("------------------------------");

    #[cfg(feature = "oauth")]
    {
        use std::process::Command;
        use std::time::Duration;
        use tokio::time::sleep;

        info!("Starting authentication demo...");

        // Start the server in a separate process (dummy example, replace with real server if needed)
        let server_handle = tokio::spawn(async {
            let output = Command::new("cargo")
                .args(["run", "--example", "05-lifecycle-compliance"])
                .current_dir("../../")
                .output()
                .expect("Failed to start server");

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("Server error: {}", stderr);
            }
        });

        // Wait a moment for server to start
        sleep(Duration::from_millis(1000)).await;

        // Create client with authentication
        let _client = UltraFastClient::new(
            ClientInfo {
                name: "auth-demo-client".to_string(),
                version: "1.0.0".to_string(),
                authors: None,
                description: Some("Authentication demo client".to_string()),
                homepage: None,
                repository: None,
                license: None,
            },
            ClientCapabilities::default(),
        )
        .with_bearer_auth("demo-token".to_string());

        info!("Client created with Bearer token authentication");

        // Connect to server using stdio
        match _client.connect_stdio().await {
            Ok(()) => {
                info!("Successfully connected to server");

                // Initialize the client
                match _client.initialize().await {
                    Ok(()) => {
                        info!("Successfully initialized client");

                        // List available tools
                        match _client.list_tools_default().await {
                            Ok(tools_response) => {
                                info!("Successfully listed tools");
                                info!("Available tools: {:?}", tools_response.tools.len());

                                // Try to call a tool
                                if let Some(tool) = tools_response.tools.first() {
                                    let tool_call = ultrafast_mcp::ToolCall {
                                        name: tool.name.clone(),
                                        arguments: Some(serde_json::json!({
                                            "message": "Hello from authenticated client!"
                                        })),
                                    };

                                    match _client.call_tool(tool_call).await {
                                        Ok(result) => {
                                            info!("Successfully called tool: {:?}", result);
                                        }
                                        Err(e) => {
                                            info!("Tool call failed: {:?}", e);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                info!("Failed to list tools: {:?}", e);
                            }
                        }

                        // Shutdown the client
                        if let Err(e) = _client.shutdown(None).await {
                            info!("Shutdown error: {:?}", e);
                        }
                    }
                    Err(e) => {
                        info!("Failed to initialize client: {:?}", e);
                    }
                }
            }
            Err(e) => {
                info!("Failed to connect to server: {:?}", e);
            }
        }

        // Wait for server to finish
        if let Err(e) = server_handle.await {
            info!("Server task error: {:?}", e);
        }

        info!("Authentication demo completed");
    }

    #[cfg(not(feature = "oauth"))]
    {
        info!("OAuth feature not enabled. Enable with: cargo run --features oauth");
        info!("To run the full demo: cargo run --features oauth");
    }

    Ok(())
}
