//! Streamable HTTP transport implementation
//!
//! This module implements a simple, correct Streamable HTTP transport that follows
//! the MCP specification for stateless request/response communication.

use crate::{Result, Transport, TransportError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use ultrafast_mcp_core::protocol::JsonRpcMessage;

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamableMcpRequest {
    pub session_id: Option<String>,
    pub message: Option<JsonRpcMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamableMcpResponse {
    pub session_id: String,
    pub protocol_version: String,
    pub message_id: Option<String>,
    pub success: bool,
    pub error: Option<String>,
    pub pending_messages: Option<Vec<JsonRpcMessage>>,
}

/// Streamable HTTP client configuration
#[derive(Debug, Clone)]
pub struct StreamableHttpClientConfig {
    pub base_url: String,
    pub session_id: Option<String>,
    pub protocol_version: String,
    pub timeout: std::time::Duration,
    pub max_retries: u32,
    pub auth_token: Option<String>,
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
        }
    }
}

/// Streamable HTTP client - simple request/response implementation
pub struct StreamableHttpClient {
    client: reqwest::Client,
    config: StreamableHttpClientConfig,
    session_id: Option<String>,
    pending_response: Option<JsonRpcMessage>,
}

impl StreamableHttpClient {
    pub fn new(config: StreamableHttpClientConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|e| TransportError::InitializationError {
                message: format!("Failed to create HTTP client: {}", e),
            })?;

        Ok(Self {
            client,
            config,
            session_id: None,
            pending_response: None,
        })
    }

    /// Connect to the Streamable HTTP server
    pub async fn connect(&mut self) -> Result<String> {
        // For Streamable HTTP, we just need to establish a session
        // Send an empty request to get a session ID
        let request_body = StreamableMcpRequest {
            session_id: self.config.session_id.clone(),
            message: None,
        };

        let url = format!("{}/mcp", self.config.base_url);
        let response = self
            .client
            .post(&url)
            .header("content-type", "application/json")
            .json(&request_body)
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

        // Parse the response to get session ID
        let response_data: StreamableMcpResponse = response
            .json()
            .await
            .map_err(|e| TransportError::SerializationError {
                message: format!("Failed to parse connect response: {}", e),
            })?;

        self.session_id = Some(response_data.session_id.clone());
        Ok(response_data.session_id)
    }

    /// Send a message and get immediate response
    async fn send_message_internal(
        &mut self,
        message: JsonRpcMessage,
    ) -> Result<JsonRpcMessage> {
        let session_id = self.session_id.as_ref().ok_or_else(|| {
            TransportError::ConnectionError {
                message: "Not connected".to_string(),
            }
        })?;

        let request_body = StreamableMcpRequest {
            session_id: Some(session_id.clone()),
            message: Some(message),
        };

        let url = format!("{}/mcp", self.config.base_url);
        let response = self
            .client
            .post(&url)
            .header("content-type", "application/json")
            .json(&request_body)
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
        // Close the session
        if let Some(session_id) = &self.session_id {
            let request_body = StreamableMcpRequest {
                session_id: Some(session_id.clone()),
                message: None,
            };

            let url = format!("{}/mcp", self.config.base_url);
            let _ = self
                .client
                .post(&url)
                .header("content-type", "application/json")
                .json(&request_body)
                .send()
                .await;
        }

        Ok(())
    }
}
