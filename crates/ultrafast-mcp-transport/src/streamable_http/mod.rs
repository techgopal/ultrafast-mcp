//! Streamable HTTP transport implementation
//!
//! This module provides MCP-compliant Streamable HTTP transport for both client and server.
//! It follows the MCP specification for stateless request/response communication.

pub mod client;
pub mod server;

pub use client::{StreamableHttpClient, StreamableHttpClientConfig};
pub use server::{HttpTransportConfig, HttpTransportServer, HttpTransportState};

// Re-export middleware types for convenience
pub use crate::middleware::{
    LoggingMiddleware, MiddlewareTransport, ProgressMiddleware, RateLimitMiddleware,
    TransportMiddleware, ValidationMiddleware,
};

/// Create a Streamable HTTP client with middleware
pub async fn create_streamable_http_client_with_middleware(
    config: StreamableHttpClientConfig,
    middlewares: Vec<Box<dyn TransportMiddleware>>,
) -> crate::Result<MiddlewareTransport<StreamableHttpClient>> {
    let client = StreamableHttpClient::new(config)?;
    let mut transport = MiddlewareTransport::new(client);

    for middleware in middlewares {
        transport = transport.with_middleware(middleware);
    }

    Ok(transport)
}

/// Create a Streamable HTTP client with default middleware stack
pub async fn create_streamable_http_client_default(
    config: StreamableHttpClientConfig,
) -> crate::Result<MiddlewareTransport<StreamableHttpClient>> {
    let client = StreamableHttpClient::new(config)?;

    let transport = MiddlewareTransport::new(client)
        .add_logging()
        .add_rate_limiting(100) // 100 requests per minute
        .add_progress_tracking(30) // 30 second timeout
        .add_validation(true); // strict validation

    Ok(transport)
}

/// Create a Streamable HTTP server with middleware
pub fn create_streamable_http_server_with_middleware(
    config: HttpTransportConfig,
    _middlewares: Vec<Box<dyn TransportMiddleware>>,
) -> HttpTransportServer {
    // Note: Server middleware is applied at the HTTP handler level
    // Individual middleware can be added to the server state if needed

    HttpTransportServer::new(config)
}

/// Create a Streamable HTTP server with default configuration
pub fn create_streamable_http_server_default(host: &str, port: u16) -> HttpTransportServer {
    let config = HttpTransportConfig {
        host: host.to_string(),
        port,
        cors_enabled: true,
        protocol_version: "2025-06-18".to_string(),
        allow_origin: Some("*".to_string()), // Allow all origins for development
        monitoring_enabled: true,
        enable_sse_resumability: true,
    };

    HttpTransportServer::new(config)
}
