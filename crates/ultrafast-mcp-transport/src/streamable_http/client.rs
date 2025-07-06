//! Streamable HTTP transport implementation
//!
//! This module implements a MCP-compliant Streamable HTTP transport that follows
//! the MCP specification for stateless request/response communication.

use crate::{Result, Transport, TransportError};
use async_trait::async_trait;
use ultrafast_mcp_core::protocol::JsonRpcMessage;
use ultrafast_mcp_auth::{OAuthClient, OAuthConfig};

/// Streamable HTTP client configuration
#[derive(Debug, Clone)]
pub struct StreamableHttpClientConfig {
    pub base_url: String,
    pub session_id: Option<String>,
    pub protocol_version: String,
    pub timeout: std::time::Duration,
    pub max_retries: u32,
    pub auth_token: Option<String>,
    pub oauth_config: Option<OAuthConfig>,
}

impl Default for StreamableHttpClientConfig {
    fn default() -> Self {
        Self {
            base_url: "http://127.0.0.1:8080".to_string(),
            session_id: None,
            protocol_version: "2025-06-18".to_string(),
            timeout: std::time::Duration::from_secs(30),
            max_retries: 3,
            auth_token: None,
            oauth_config: None,
        }
    }
}

/// Streamable HTTP client - MCP-compliant request/response implementation
pub struct StreamableHttpClient {
    client: reqwest::Client,
    config: StreamableHttpClientConfig,
    session_id: Option<String>,
    pending_response: Option<JsonRpcMessage>,
    oauth_client: Option<OAuthClient>,
    access_token: Option<String>,
    token_expiry: Option<std::time::SystemTime>,
}

impl StreamableHttpClient {
    pub fn new(config: StreamableHttpClientConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|e| TransportError::InitializationError {
                message: format!("Failed to create HTTP client: {}", e),
            })?;

        let oauth_client = config.oauth_config.as_ref()
            .map(|config| OAuthClient::from_config(config.clone()));

        let access_token = config.auth_token.clone();

        Ok(Self {
            client,
            config,
            session_id: None,
            pending_response: None,
            oauth_client,
            access_token,
            token_expiry: None,
        })
    }

    /// Authenticate using OAuth 2.1 if configured
    pub async fn authenticate(&mut self) -> Result<()> {
        if let Some(oauth_client) = &self.oauth_client {
            // Generate PKCE parameters
            let pkce_params = ultrafast_mcp_auth::generate_pkce_params()
                .map_err(|e| TransportError::AuthenticationError {
                    message: format!("Failed to generate PKCE: {}", e),
                })?;

            // Generate state for CSRF protection
            let state = ultrafast_mcp_auth::generate_state();

            // Get authorization URL
            let auth_url = oauth_client.get_authorization_url_with_pkce(state, pkce_params.clone())
                .await
                .map_err(|e| TransportError::AuthenticationError {
                    message: format!("Failed to get auth URL: {}", e),
                })?;

            // In a real implementation, you would:
            // 1. Open the auth_url in a browser
            // 2. Wait for user to complete authorization
            // 3. Receive the authorization code via callback
            // For now, we'll simulate this with a placeholder
            
            tracing::info!("OAuth authentication URL: {}", auth_url);
            tracing::warn!("OAuth authentication requires manual user interaction. Please complete the flow manually.");
            
            // For testing purposes, we'll use a mock token
            self.access_token = Some("mock_oauth_token".to_string());
            self.token_expiry = Some(std::time::SystemTime::now() + std::time::Duration::from_secs(3600));
        }

        Ok(())
    }

    /// Refresh OAuth token if needed
    async fn refresh_token_if_needed(&mut self) -> Result<()> {
        if let Some(expiry) = self.token_expiry {
            if std::time::SystemTime::now() >= expiry {
                tracing::info!("OAuth token expired, refreshing...");
                self.authenticate().await?;
            }
        }
        Ok(())
    }

    /// Get current authentication headers
    async fn get_auth_headers(&mut self) -> Result<Vec<(String, String)>> {
        let mut headers = Vec::new();

        // Refresh token if needed
        self.refresh_token_if_needed().await?;

        // Add OAuth token if available
        if let Some(token) = &self.access_token {
            headers.push(("Authorization".to_string(), format!("Bearer {}", token)));
        }

        Ok(headers)
    }

    /// Connect to the Streamable HTTP server
    pub async fn connect(&mut self) -> Result<String> {
        // Authenticate if OAuth is configured
        if self.oauth_client.is_some() {
            self.authenticate().await?;
        }

        // For Streamable HTTP, we establish a session by sending an initialize request
        let initialize_request = JsonRpcMessage::Request(ultrafast_mcp_core::protocol::JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "initialize".to_string(),
            params: Some(serde_json::json!({
                "protocolVersion": self.config.protocol_version,
                "capabilities": {},
                "clientInfo": {
                    "name": "ultrafast-mcp-client",
                    "version": "1.0.0"
                }
            })),
            id: Some(ultrafast_mcp_core::protocol::RequestId::String("1".to_string())),
            meta: std::collections::HashMap::new(),
        });

        let session_id = self.config.session_id.clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let url = format!("{}/mcp", self.config.base_url);
        
        // Get authentication headers
        let auth_headers = self.get_auth_headers().await?;
        
        let mut request_builder = self.client
            .post(&url)
            .header("content-type", "application/json")
            .header("mcp-session-id", &session_id)
            .header("mcp-protocol-version", &self.config.protocol_version)
            .json(&initialize_request);

        // Add authentication headers
        for (key, value) in auth_headers {
            request_builder = request_builder.header(key, value);
        }

        let response = request_builder
            .send()
            .await
            .map_err(|e| TransportError::ConnectionError {
                message: format!("Failed to connect: {}", e),
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(TransportError::ConnectionError {
                message: format!("Connection failed: {}", error_text),
            });
        }

        // Parse the initialize response
        let response_message: JsonRpcMessage = response
            .json()
            .await
            .map_err(|e| TransportError::SerializationError {
                message: format!("Failed to parse initialize response: {}", e),
            })?;

        // Store session ID and response
        self.session_id = Some(session_id.clone());
        self.pending_response = Some(response_message);
        
        Ok(session_id)
    }

    /// Send a message and get immediate response
    async fn send_message_internal(
        &mut self,
        message: JsonRpcMessage,
    ) -> Result<JsonRpcMessage> {
        let session_id = self.session_id.clone().ok_or_else(|| {
            TransportError::ConnectionError {
                message: "Not connected".to_string(),
            }
        })?;

        let url = format!("{}/mcp", self.config.base_url);
        
        // Get authentication headers
        let auth_headers = self.get_auth_headers().await?;
        
        let mut request_builder = self.client
            .post(&url)
            .header("content-type", "application/json")
            .header("mcp-session-id", session_id)
            .header("mcp-protocol-version", &self.config.protocol_version)
            .json(&message); // Send direct JSON-RPC message

        // Add authentication headers
        for (key, value) in auth_headers {
            request_builder = request_builder.header(key, value);
        }

        let response = request_builder
            .send()
            .await
            .map_err(|e| TransportError::NetworkError {
                message: format!("Failed to send message: {}", e),
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(TransportError::NetworkError {
                message: format!("Send failed: {}", error_text),
            });
        }

        // Parse the response - it should be a single JSON-RPC message
        let response_message: JsonRpcMessage = response
            .json()
            .await
            .map_err(|e| TransportError::SerializationError {
                message: format!("Failed to parse response: {}", e),
            })?;

        Ok(response_message)
    }

    /// Get current connection health
    pub async fn get_health(&self) -> crate::TransportHealth {
        crate::TransportHealth {
            state: if self.session_id.is_some() {
                crate::ConnectionState::Connected
            } else {
                crate::ConnectionState::Disconnected
            },
            last_activity: None, // TODO: Track last activity
            messages_sent: 0, // TODO: Track message counts
            messages_received: 0,
            connection_duration: None, // TODO: Track connection duration
            error_count: 0, // TODO: Track error count
            last_error: None, // TODO: Track last error
        }
    }

    /// Check if the client is healthy
    pub async fn is_healthy(&self) -> bool {
        self.session_id.is_some() && self.pending_response.is_none()
    }

    /// Attempt to reconnect the client
    pub async fn reconnect(&mut self) -> Result<()> {
        tracing::info!("Attempting to reconnect Streamable HTTP client");
        
        // Clear current session
        self.session_id = None;
        self.pending_response = None;
        
        // Attempt to connect again
        self.connect().await?;
        
        tracing::info!("Successfully reconnected Streamable HTTP client");
        Ok(())
    }

    /// Reset the client state
    pub async fn reset(&mut self) -> Result<()> {
        tracing::info!("Resetting Streamable HTTP client");
        
        // Close current connection
        self.close().await?;
        
        // Clear all state
        self.session_id = None;
        self.pending_response = None;
        self.access_token = None;
        self.token_expiry = None;
        
        tracing::info!("Streamable HTTP client reset completed");
        Ok(())
    }
}

#[async_trait]
impl Transport for StreamableHttpClient {
    async fn send_message(&mut self, message: JsonRpcMessage) -> Result<()> {
        // For Streamable HTTP, we send and get immediate response
        // Store the response for the next receive_message call
        let response = self.send_message_internal(message).await?;
        self.pending_response = Some(response);
        Ok(())
    }

    async fn receive_message(&mut self) -> Result<JsonRpcMessage> {
        // Return the pending response if available
        if let Some(response) = self.pending_response.take() {
            Ok(response)
        } else {
            // No pending response, connection is closed
            Err(TransportError::ConnectionClosed)
        }
    }

    async fn close(&mut self) -> Result<()> {
        // Close the session using DELETE method
        if let Some(session_id) = self.session_id.clone() {
            let url = format!("{}/mcp", self.config.base_url);
            
            // Get authentication headers
            let auth_headers = self.get_auth_headers().await?;
            
            let mut request_builder = self.client
                .delete(&url)
                .header("mcp-session-id", session_id)
                .header("mcp-protocol-version", &self.config.protocol_version);

            // Add authentication headers
            for (key, value) in auth_headers {
                request_builder = request_builder.header(key, value);
            }

            let _ = request_builder.send().await;
        }

        Ok(())
    }

    fn get_state(&self) -> crate::ConnectionState {
        if self.session_id.is_some() {
            crate::ConnectionState::Connected
        } else {
            crate::ConnectionState::Disconnected
        }
    }

    fn get_health(&self) -> crate::TransportHealth {
        // This is a blocking call, so we can't use the async version
        crate::TransportHealth {
            state: self.get_state(),
            last_activity: None,
            messages_sent: 0,
            messages_received: 0,
            connection_duration: None,
            error_count: 0,
            last_error: None,
        }
    }

    async fn reconnect(&mut self) -> Result<()> {
        self.reconnect().await
    }

    async fn reset(&mut self) -> Result<()> {
        self.reset().await
    }
}
