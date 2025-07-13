//! Streamable HTTP transport implementation
//!
//! This module implements a MCP-compliant Streamable HTTP transport that follows
//! the MCP specification for stateless request/response communication.

use crate::{Result, Transport, TransportError};
use async_trait::async_trait;

use ultrafast_mcp_core::protocol::JsonRpcMessage;
use ultrafast_mcp_core::utils::generate_state;

/// Streamable HTTP client configuration
#[derive(Debug, Clone)]
pub struct StreamableHttpClientConfig {
    pub base_url: String,
    pub session_id: Option<String>,
    pub protocol_version: String,
    pub timeout: std::time::Duration,
    pub max_retries: u32,
    pub auth_token: Option<String>,
    pub oauth_config: Option<ultrafast_mcp_auth::OAuthConfig>,
    pub auth_method: Option<ultrafast_mcp_auth::AuthMethod>,
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
            auth_method: None,
        }
    }
}

impl StreamableHttpClientConfig {
    /// Set Bearer token authentication
    pub fn with_bearer_auth(mut self, token: String) -> Self {
        self.auth_method = Some(ultrafast_mcp_auth::AuthMethod::bearer(token));
        self
    }

    /// Set OAuth authentication
    pub fn with_oauth_auth(mut self, config: ultrafast_mcp_auth::OAuthConfig) -> Self {
        self.auth_method = Some(ultrafast_mcp_auth::AuthMethod::oauth(config));
        self
    }

    /// Set API key authentication
    pub fn with_api_key_auth(mut self, api_key: String) -> Self {
        self.auth_method = Some(ultrafast_mcp_auth::AuthMethod::api_key(api_key));
        self
    }

    /// Set API key authentication with custom header name
    pub fn with_api_key_auth_custom(mut self, api_key: String, header_name: String) -> Self {
        let api_key_auth =
            ultrafast_mcp_auth::ApiKeyAuth::new(api_key).with_header_name(header_name);
        let auth_method = ultrafast_mcp_auth::AuthMethod::ApiKey(api_key_auth);
        self.auth_method = Some(auth_method);
        self
    }

    /// Set Basic authentication
    pub fn with_basic_auth(mut self, username: String, password: String) -> Self {
        self.auth_method = Some(ultrafast_mcp_auth::AuthMethod::basic(username, password));
        self
    }

    /// Set custom header authentication
    pub fn with_custom_auth(mut self) -> Self {
        self.auth_method = Some(ultrafast_mcp_auth::AuthMethod::custom());
        self
    }

    /// Set authentication method
    pub fn with_auth_method(mut self, auth_method: ultrafast_mcp_auth::AuthMethod) -> Self {
        self.auth_method = Some(auth_method);
        self
    }
}

/// Streamable HTTP client - MCP-compliant request/response implementation
pub struct StreamableHttpClient {
    client: reqwest::Client,
    config: StreamableHttpClientConfig,
    session_id: Option<String>,
    pending_response: Option<JsonRpcMessage>,
    oauth_client: Option<ultrafast_mcp_auth::OAuthClient>,
    access_token: Option<String>,
    token_expiry: Option<std::time::SystemTime>,
    auth_middleware: Option<ultrafast_mcp_auth::ClientAuthMiddleware>,
}

impl StreamableHttpClient {
    pub fn new(config: StreamableHttpClientConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|e| TransportError::InitializationError {
                message: format!("Failed to create HTTP client: {e}"),
            })?;

        let oauth_client = config
            .oauth_config
            .as_ref()
            .map(|config| ultrafast_mcp_auth::OAuthClient::from_config(config.clone()));

        let access_token = config.auth_token.clone();

        let auth_middleware = config
            .auth_method
            .as_ref()
            .map(|auth_method| ultrafast_mcp_auth::ClientAuthMiddleware::new(auth_method.clone()));

        Ok(Self {
            client,
            config,
            session_id: None,
            pending_response: None,
            oauth_client,
            access_token,
            token_expiry: None,
            auth_middleware,
        })
    }

    /// Authenticate using OAuth 2.1 if configured
    pub async fn authenticate(&mut self) -> Result<()> {
        if let Some(oauth_client) = &self.oauth_client {
            // Generate PKCE parameters
            let pkce_params = ultrafast_mcp_auth::generate_pkce_params().map_err(|e| {
                TransportError::AuthenticationError {
                    message: format!("Failed to generate PKCE: {e}"),
                }
            })?;

            // Generate state for CSRF protection
            let state = generate_state();

            // Get authorization URL
            let auth_url = oauth_client
                .get_authorization_url_with_pkce(state, pkce_params.clone())
                .await
                .map_err(|e| TransportError::AuthenticationError {
                    message: format!("Failed to get auth URL: {e}"),
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
            self.token_expiry =
                Some(std::time::SystemTime::now() + std::time::Duration::from_secs(3600));
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

        // Use auth middleware if available
        if let Some(auth_middleware) = &mut self.auth_middleware {
            let auth_headers = auth_middleware.get_headers().await.map_err(|e| {
                TransportError::AuthenticationError {
                    message: format!("Failed to get auth headers: {e}"),
                }
            })?;

            headers.extend(auth_headers.into_iter());
        } else {
            // Fallback to legacy OAuth token handling
            self.refresh_token_if_needed().await?;

            // Add OAuth token if available
            if let Some(token) = &self.access_token {
                headers.push(("Authorization".to_string(), format!("Bearer {token}")));
            }
        }

        Ok(headers)
    }

    /// Connect to the Streamable HTTP server
    pub async fn connect(&mut self) -> Result<String> {
        // Authenticate if OAuth is configured
        if self.oauth_client.is_some() {
            self.authenticate().await?;
        }

        // For Streamable HTTP, we just establish a session ID without sending initialize
        // The client will handle the initialize request separately
        let session_id = self
            .config
            .session_id
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        // Store session ID
        self.session_id = Some(session_id.clone());

        Ok(session_id)
    }

    /// Send a message and get immediate response
    async fn send_message_internal(&mut self, message: JsonRpcMessage) -> Result<JsonRpcMessage> {
        let session_id =
            self.session_id
                .clone()
                .ok_or_else(|| TransportError::ConnectionError {
                    message: "Not connected".to_string(),
                })?;

        let url = format!("{}/mcp", self.config.base_url);

        // Get authentication headers
        let auth_headers = self.get_auth_headers().await?;

        let mut request_builder = self
            .client
            .post(&url)
            .header("content-type", "application/json")
            .header("accept", "application/json, text/event-stream") // Required Accept header
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
                message: format!("Failed to send message: {e}"),
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(TransportError::NetworkError {
                message: format!("Send failed: {error_text}"),
            });
        }

        // Parse the response - it should be a single JSON-RPC message
        let response_message: JsonRpcMessage =
            response
                .json()
                .await
                .map_err(|e| TransportError::SerializationError {
                    message: format!("Failed to parse response: {e}"),
                })?;

        Ok(response_message)
    }

    /// Send a JSON-RPC notification (fire-and-forget, do not wait for response)
    pub async fn send_notification_internal(&mut self, message: JsonRpcMessage) -> Result<()> {
        let session_id =
            self.session_id
                .clone()
                .ok_or_else(|| TransportError::ConnectionError {
                    message: "Not connected".to_string(),
                })?;

        let url = format!("{}/mcp", self.config.base_url);

        // Get authentication headers
        let auth_headers = self.get_auth_headers().await?;

        let mut request_builder = self
            .client
            .post(&url)
            .header("content-type", "application/json")
            .header("accept", "application/json, text/event-stream")
            .header("mcp-session-id", session_id)
            .header("mcp-protocol-version", &self.config.protocol_version)
            .json(&message);

        // Add authentication headers
        for (key, value) in auth_headers {
            request_builder = request_builder.header(key, value);
        }

        // Fire and forget: do not block on response
        let _ = request_builder.send().await;
        Ok(())
    }

    /// Get current connection health
    pub async fn get_health(&mut self) -> crate::TransportHealth {
        crate::TransportHealth {
            state: if self.session_id.is_some() {
                crate::ConnectionState::Connected
            } else {
                crate::ConnectionState::Disconnected
            },
            connection_duration: None,
            messages_sent: 0,
            messages_received: 0,
            error_count: 0,
            last_activity: None,
            last_error: None,
        }
    }

    /// Check if the connection is healthy
    pub async fn is_healthy(&self) -> bool {
        self.session_id.is_some()
    }

    /// Reconnect to the server
    pub async fn reconnect(&mut self) -> Result<()> {
        self.session_id = None;
        self.pending_response = None;
        self.connect().await?;
        Ok(())
    }

    /// Reset the client state
    pub async fn reset(&mut self) -> Result<()> {
        self.session_id = None;
        self.pending_response = None;
        self.access_token = None;
        self.token_expiry = None;
        Ok(())
    }

    /// Start an SSE stream for server-to-client communication
    pub async fn start_sse_stream(&mut self) -> Result<reqwest::Response> {
        let session_id =
            self.session_id
                .clone()
                .ok_or_else(|| TransportError::ConnectionError {
                    message: "Not connected".to_string(),
                })?;

        let url = format!("{}/mcp", self.config.base_url);

        // Get authentication headers
        let auth_headers = self.get_auth_headers().await?;

        let mut request_builder = self
            .client
            .get(&url)
            .header("accept", "text/event-stream") // SSE-specific Accept header
            .header("mcp-session-id", session_id)
            .header("mcp-protocol-version", &self.config.protocol_version);

        // Add authentication headers
        for (key, value) in auth_headers {
            request_builder = request_builder.header(key, value);
        }

        let response = request_builder
            .send()
            .await
            .map_err(|e| TransportError::NetworkError {
                message: format!("Failed to start SSE stream: {e}"),
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(TransportError::NetworkError {
                message: format!("SSE stream failed: {error_text}"),
            });
        }

        Ok(response)
    }

    /// Resume an SSE stream from a specific event ID
    pub async fn resume_sse_stream(&mut self, last_event_id: &str) -> Result<reqwest::Response> {
        let session_id =
            self.session_id
                .clone()
                .ok_or_else(|| TransportError::ConnectionError {
                    message: "Not connected".to_string(),
                })?;

        let url = format!("{}/mcp", self.config.base_url);

        // Get authentication headers
        let auth_headers = self.get_auth_headers().await?;

        let mut request_builder = self
            .client
            .get(&url)
            .header("accept", "text/event-stream")
            .header("mcp-session-id", session_id)
            .header("mcp-protocol-version", &self.config.protocol_version)
            .header("last-event-id", last_event_id); // Resume from specific event

        // Add authentication headers
        for (key, value) in auth_headers {
            request_builder = request_builder.header(key, value);
        }

        let response = request_builder
            .send()
            .await
            .map_err(|e| TransportError::NetworkError {
                message: format!("Failed to resume SSE stream: {e}"),
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(TransportError::NetworkError {
                message: format!("SSE stream resume failed: {error_text}"),
            });
        }

        Ok(response)
    }
}

#[async_trait]
impl Transport for StreamableHttpClient {
    async fn send_message(&mut self, message: JsonRpcMessage) -> Result<()> {
        // For notifications, use fire-and-forget
        if matches!(message, JsonRpcMessage::Notification(_)) {
            self.send_notification_internal(message).await
        } else {
            // For requests, wait for response
            let response = self.send_message_internal(message).await?;
            self.pending_response = Some(response);
            Ok(())
        }
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

            let mut request_builder = self
                .client
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
