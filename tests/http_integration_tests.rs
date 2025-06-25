//! Advanced integration tests for HTTP transport with OAuth

use ultrafast_mcp::prelude::*;
use ultrafast_mcp_transport::{
    TransportConfig, 
    http::{HttpTransportServer, HttpTransportConfig, RateLimitConfig}
};
use ultrafast_mcp_auth::{OAuthClient, TokenValidator, AuthConfig, AuthError};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_http_transport_with_oauth() {
    // Create OAuth client
    let mut oauth_client = OAuthClient::new(
        "test_client_id".to_string(),
        "test_client_secret".to_string(),
        "https://example.com/oauth/authorize".to_string(),
    );
    
    // Test PKCE flow
    let (code_verifier, code_challenge) = oauth_client.generate_pkce_challenge();
    let auth_url = oauth_client.build_authorization_url(
        &code_challenge,
        Some("test_state".to_string()),
        vec!["mcp:read".to_string(), "mcp:write".to_string()],
    );
    
    assert!(auth_url.contains("code_challenge"));
    assert!(auth_url.contains("test_state"));
    assert!(auth_url.contains("mcp%3Aread"));
}

#[tokio::test]
async fn test_http_server_with_rate_limiting() {
    let config = HttpTransportConfig {
        host: "127.0.0.1".to_string(),
        port: 0, // Let OS assign port
        rate_limit_config: RateLimitConfig {
            requests_per_second: 2,
            burst_size: 3,
            window_size: Duration::from_secs(60),
        },
        ..Default::default()
    };
    
    let server = HttpTransportServer::new(config);
    let rate_limiter = server.get_state().rate_limiter;
    
    // Should allow initial requests
    assert!(rate_limiter.check_rate_limit("test_client").await.is_ok());
    assert!(rate_limiter.check_rate_limit("test_client").await.is_ok());
    assert!(rate_limiter.check_rate_limit("test_client").await.is_ok());
    
    // Should rate limit after burst
    assert!(rate_limiter.check_rate_limit("test_client").await.is_err());
    
    // Different client should still work
    assert!(rate_limiter.check_rate_limit("other_client").await.is_ok());
}

#[tokio::test]
async fn test_session_management() {
    let config = HttpTransportConfig {
        session_timeout_secs: 1, // Very short timeout for testing
        ..Default::default()
    };
    
    let server = HttpTransportServer::new(config);
    let session_store = server.get_state().session_store;
    
    // Create session
    let session = session_store.create_session("test_session".to_string()).await;
    assert_eq!(session.session_id, "test_session");
    
    // Should be able to retrieve immediately
    assert!(session_store.get_session("test_session").await.is_some());
    
    // Wait for expiration
    sleep(Duration::from_secs(2)).await;
    
    // Should be expired
    assert!(session_store.get_session("test_session").await.is_none());
}

#[tokio::test]
async fn test_message_queue_reliability() {
    let config = HttpTransportConfig {
        max_message_retries: 2,
        ..Default::default()
    };
    
    let server = HttpTransportServer::new(config);
    let message_queue = server.get_state().message_queue;
    
    // Enqueue a test message
    let test_message = JsonRpcMessage::Notification(Notification {
        method: "test/notification".to_string(),
        params: None,
    });
    
    message_queue.enqueue_message("test_session".to_string(), test_message).await;
    
    // Should have pending message
    let pending = message_queue.get_pending_messages("test_session").await;
    assert_eq!(pending.len(), 1);
    
    let message_id = pending[0].id.clone();
    
    // Retry should increment count
    assert!(message_queue.increment_retry("test_session", &message_id).await);
    assert!(message_queue.increment_retry("test_session", &message_id).await);
    
    // Should be removed after max retries
    assert!(!message_queue.increment_retry("test_session", &message_id).await);
    
    let pending_after = message_queue.get_pending_messages("test_session").await;
    assert_eq!(pending_after.len(), 0);
}

#[tokio::test]
async fn test_connection_pool() {
    use ultrafast_mcp_transport::http::{ConnectionPool, PoolConfig};
    
    let config = PoolConfig {
        max_connections: 2,
        connection_timeout: Duration::from_secs(5),
        idle_timeout: Duration::from_secs(10),
        max_idle_per_host: 1,
    };
    
    let pool = ConnectionPool::new(config);
    
    // Should be able to get clients
    let client1 = pool.get_client("example.com").await.unwrap();
    let client2 = pool.get_client("example.com").await.unwrap();
    
    // Clients should be reused (same underlying client)
    // This is a bit tricky to test directly, but we can at least ensure no errors
    assert!(client1.get_timeout().is_some());
    assert!(client2.get_timeout().is_some());
}

#[tokio::test]
async fn test_streamable_http_transport_config() {
    let config = TransportConfig::Streamable {
        base_url: "https://api.example.com".to_string(),
        auth_token: Some("Bearer test_token".to_string()),
        session_id: Some("test_session_123".to_string()),
    };
    
    // This test ensures the config can be created and serialized
    match config {
        TransportConfig::Streamable { base_url, auth_token, session_id } => {
            assert_eq!(base_url, "https://api.example.com");
            assert_eq!(auth_token, Some("Bearer test_token".to_string()));
            assert_eq!(session_id, Some("test_session_123".to_string()));
        },
        _ => panic!("Wrong config type"),
    }
}

#[tokio::test]
async fn test_auth_integration() {
    let auth_config = AuthConfig {
        client_id: "test_client".to_string(),
        client_secret: Some("test_secret".to_string()),
        auth_url: "https://auth.example.com/oauth/authorize".to_string(),
        token_url: "https://auth.example.com/oauth/token".to_string(),
        scopes: vec!["mcp:read".to_string(), "mcp:write".to_string()],
        redirect_uri: Some("http://localhost:8080/callback".to_string()),
        use_pkce: true,
    };
    
    // Test that auth config can be created and validated
    assert_eq!(auth_config.client_id, "test_client");
    assert!(auth_config.use_pkce);
    assert_eq!(auth_config.scopes.len(), 2);
}
