//! Transport layer implementations for ULTRAFAST MCP
//! 
//! This crate provides different transport mechanisms for MCP communication:
//! - STDIO: Standard input/output transport for local communication
//! - HTTP: Streamable HTTP transport for web-based communication with OAuth 2.1 support

use async_trait::async_trait;
use ultrafast_mcp_core::protocol::JsonRpcMessage;
use thiserror::Error;

// Define our own Result type for this crate
pub type Result<T> = std::result::Result<T, TransportError>;

pub mod stdio;
pub mod middleware;

// Re-export key types
pub use stdio::StdioTransport;

#[cfg(feature = "http")]
pub mod http;

#[cfg(feature = "http")]
pub use http::{StreamableHttpTransport, HttpSseTransport, HttpTransportServer, HttpTransportConfig};

/// Transport errors
#[derive(Error, Debug)]
pub enum TransportError {
    #[error("Connection error: {message}")]
    ConnectionError { message: String },
    
    #[error("Connection closed")]
    ConnectionClosed,
    
    #[error("Serialization error: {message}")]
    SerializationError { message: String },
    
    #[error("Network error: {message}")]
    NetworkError { message: String },
    
    #[error("Authentication error: {message}")]
    AuthenticationError { message: String },
    
    #[error("Protocol error: {message}")]
    ProtocolError { message: String },
    
    #[error("Initialization error: {message}")]
    InitializationError { message: String },
    
    #[error("Internal error: {message}")]
    InternalError { message: String },
}

/// Transport trait for MCP communication
#[async_trait]
pub trait Transport: Send + Sync {
    /// Send a message through the transport
    async fn send_message(&mut self, message: JsonRpcMessage) -> Result<()>;
    
    /// Receive a message from the transport
    async fn receive_message(&mut self) -> Result<JsonRpcMessage>;
    
    /// Close the transport connection
    async fn close(&mut self) -> Result<()>;
}

/// Transport configuration
#[derive(Debug, Clone)]
pub enum TransportConfig {
    /// Standard input/output transport
    Stdio,
    
    /// Streamable HTTP transport (PRD recommended)
    #[cfg(feature = "http")]
    Streamable {
        base_url: String,
        auth_token: Option<String>,
        session_id: Option<String>,
    },
    
    /// Legacy HTTP+SSE transport (backward compatibility)
    #[cfg(feature = "http")]
    HttpSse {
        base_url: String,
        auth_token: Option<String>,
        session_id: Option<String>,
    },
}

/// Create a transport from configuration
pub async fn create_transport(config: TransportConfig) -> Result<Box<dyn Transport>> {
    match config {
        TransportConfig::Stdio => {
            let transport = stdio::StdioTransport::new().await?;
            Ok(Box::new(transport))
        },
        
        #[cfg(feature = "http")]
        TransportConfig::Streamable { base_url, auth_token, session_id } => {
            let client_config = http::streamable::StreamableHttpClientConfig {
                base_url,
                auth_token,
                session_id,
                ..Default::default()
            };
            
            let mut client = http::streamable::StreamableHttpClient::new(client_config)?;
            client.connect().await?;
            Ok(Box::new(client))
        },
        
        #[cfg(feature = "http")]
        TransportConfig::HttpSse { base_url, auth_token, session_id } => {
            let client_config = http::client::HttpClientConfig {
                base_url,
                auth_token,
                session_id,
                ..Default::default()
            };
            
            let mut client = http::client::HttpTransportClient::new(client_config)?;
            client.connect().await?;
            Ok(Box::new(client))
        },
    }
}
