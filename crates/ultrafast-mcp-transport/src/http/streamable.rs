//! Streamable HTTP transport implementation
//!
//! This module implements the PRD-specified Streamable HTTP transport that provides:
//! - Single unified endpoint with optional SSE upgrade
//! - Superior performance vs HTTP+SSE
//! - Stateless architecture support
//! - Session resumability

use crate::http::server::{HttpTransportConfig, HttpTransportServer};
use crate::{Result, Transport, TransportError};
use async_trait::async_trait;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::broadcast;
use ultrafast_mcp_core::protocol::JsonRpcMessage;

/// Streamable HTTP client configuration
#[derive(Debug, Clone)]
pub struct StreamableHttpClientConfig {
    pub base_url: String,
    pub session_id: Option<String>,
    pub protocol_version: String,
    pub timeout: Duration,
    pub max_retries: u32,
    pub auth_token: Option<String>,
}

impl Default for StreamableHttpClientConfig {
    fn default() -> Self {
        Self {
            base_url: "http://127.0.0.1:8080".to_string(),
            session_id: None,
            protocol_version: "2025-06-18".to_string(),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            auth_token: None,
        }
    }
}

/// Streamable HTTP client - implements PRD specification for client-side
pub struct StreamableHttpClient {
    client: Client,
    config: StreamableHttpClientConfig,
    session_id: Option<String>,
    pending_messages: Vec<JsonRpcMessage>,
    #[allow(dead_code)]
    last_message_id: Option<String>,
}

impl StreamableHttpClient {
    pub fn new(config: StreamableHttpClientConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|e| TransportError::InitializationError {
                message: format!("Failed to create HTTP client: {}", e),
            })?;

        Ok(Self {
            client,
            config,
            session_id: None,
            pending_messages: Vec::new(),
            last_message_id: None,
        })
    }

    /// Connect to the Streamable HTTP server
    pub async fn connect(&mut self) -> Result<String> {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", str_to_header_value("application/json")?);

        if let Some(token) = &self.config.auth_token {
            headers.insert("authorization", str_to_header_value(token)?);
        }

        // Only include session_id if present
        let request_body = if let Some(ref session_id) = self.config.session_id {
            StreamableMcpRequest {
                session_id: Some(session_id.clone()),
                message: None,
                upgrade_to_stream: Some(false),
            }
        } else {
            StreamableMcpRequest {
                session_id: None, // Will be omitted by serde if using skip_serializing_if
                message: None,
                upgrade_to_stream: Some(false),
            }
        };

        let url = format!("{}/mcp", self.config.base_url);
        let response = self
            .client
            .post(&url)
            .headers(headers)
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

        let connect_response: StreamableMcpResponse =
            response
                .json()
                .await
                .map_err(|e| TransportError::SerializationError {
                    message: format!("Failed to parse connect response: {}", e),
                })?;

        if !connect_response.success {
            return Err(TransportError::ConnectionError {
                message: connect_response
                    .error
                    .unwrap_or_else(|| "Connection failed".to_string()),
            });
        }

        self.session_id = Some(connect_response.session_id.clone());

        // Store any pending messages from the connection
        if let Some(pending_messages) = connect_response.pending_messages {
            self.pending_messages.extend(pending_messages);
        }

        Ok(connect_response.session_id)
    }

    /// Send a message via Streamable HTTP and return any immediate responses
    async fn send_message_internal(
        &mut self,
        message: JsonRpcMessage,
    ) -> Result<Vec<JsonRpcMessage>> {
        let session_id =
            self.session_id
                .as_ref()
                .ok_or_else(|| TransportError::ConnectionError {
                    message: "Not connected".to_string(),
                })?;

        let mut headers = HeaderMap::new();
        headers.insert("content-type", str_to_header_value("application/json")?);

        if let Some(token) = &self.config.auth_token {
            headers.insert("authorization", str_to_header_value(token)?);
        }

        let request_body = StreamableMcpRequest {
            session_id: Some(session_id.clone()),
            message: Some(message),
            upgrade_to_stream: Some(false),
        };

        let url = format!("{}/mcp", self.config.base_url);
        let response = self
            .client
            .post(&url)
            .headers(headers)
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

        let send_response: StreamableMcpResponse =
            response
                .json()
                .await
                .map_err(|e| TransportError::SerializationError {
                    message: format!("Failed to parse send response: {}", e),
                })?;

        // Return any pending messages from the response instead of storing them
        if let Some(pending_messages) = send_response.pending_messages {
            tracing::debug!(
                "Returning {} pending messages from send response",
                pending_messages.len()
            );
            for (i, msg) in pending_messages.iter().enumerate() {
                tracing::debug!("Pending message {}: {:?}", i, msg);
            }
            Ok(pending_messages)
        } else {
            Ok(Vec::new())
        }
    }

    /// Poll for new messages
    async fn poll_messages(&mut self) -> Result<()> {
        let session_id =
            self.session_id
                .as_ref()
                .ok_or_else(|| TransportError::ConnectionError {
                    message: "Not connected".to_string(),
                })?;

        let mut headers = HeaderMap::new();
        headers.insert("content-type", str_to_header_value("application/json")?);

        if let Some(token) = &self.config.auth_token {
            headers.insert("authorization", str_to_header_value(token)?);
        }

        let request_body = StreamableMcpRequest {
            session_id: Some(session_id.clone()),
            message: None,
            upgrade_to_stream: Some(false),
        };

        let url = format!("{}/mcp", self.config.base_url);
        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| TransportError::NetworkError {
                message: format!("Failed to poll messages: {}", e),
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(TransportError::NetworkError {
                message: format!("Poll failed: {}", error_text),
            });
        }

        let poll_response: StreamableMcpResponse =
            response
                .json()
                .await
                .map_err(|e| TransportError::SerializationError {
                    message: format!("Failed to parse poll response: {}", e),
                })?;

        // Store any pending messages from the response
        if let Some(pending_messages) = poll_response.pending_messages {
            tracing::debug!(
                "Storing {} pending messages from poll response",
                pending_messages.len()
            );
            for (i, msg) in pending_messages.iter().enumerate() {
                tracing::debug!("Pending message {}: {:?}", i, msg);
            }
            self.pending_messages.extend(pending_messages);
        }

        Ok(())
    }
}

#[async_trait]
impl Transport for StreamableHttpClient {
    async fn send_message(&mut self, message: JsonRpcMessage) -> Result<()> {
        let immediate_responses = self.send_message_internal(message).await?;
        // Store any immediate responses in pending messages for the message handler
        self.pending_messages.extend(immediate_responses);
        Ok(())
    }
    async fn receive_message(&mut self) -> Result<JsonRpcMessage> {
        // First, check if we have any pending messages
        if let Some(message) = self.pending_messages.pop() {
            tracing::debug!("Returning pending message: {:?}", message);
            return Ok(message);
        }

        // If no pending messages, use optimized polling with immediate checks
        tracing::debug!("No pending messages, starting optimized polling");

        // Try immediate poll first (no delay)
        self.poll_messages().await?;
        if let Some(message) = self.pending_messages.pop() {
            tracing::debug!("Returning message from immediate poll: {:?}", message);
            return Ok(message);
        }

        // Very aggressive initial polling: check every 10ms for first 1 second (100 attempts)
        for attempt in 0..100 {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            self.poll_messages().await?;
            if let Some(message) = self.pending_messages.pop() {
                tracing::debug!(
                    "Returning message from ultra-fast poll attempt {}: {:?}",
                    attempt + 1,
                    message
                );
                return Ok(message);
            }
        }

        // Fast polling: check every 50ms for next 2 seconds (40 attempts)
        for attempt in 0..40 {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            self.poll_messages().await?;
            if let Some(message) = self.pending_messages.pop() {
                tracing::debug!(
                    "Returning message from fast poll attempt {}: {:?}",
                    attempt + 101,
                    message
                );
                return Ok(message);
            }
        }

        // Medium polling: check every 200ms for next 4 seconds (20 attempts)
        for attempt in 0..20 {
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            self.poll_messages().await?;
            if let Some(message) = self.pending_messages.pop() {
                tracing::debug!(
                    "Returning message from medium poll attempt {}: {:?}",
                    attempt + 141,
                    message
                );
                return Ok(message);
            }
        }

        // Slow polling: check every 1000ms for final 3 seconds (3 attempts)
        for attempt in 0..3 {
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            self.poll_messages().await?;
            if let Some(message) = self.pending_messages.pop() {
                tracing::debug!(
                    "Returning message from slow poll attempt {}: {:?}",
                    attempt + 161,
                    message
                );
                return Ok(message);
            }
        }

        // If still no messages after 10 seconds total, return an error
        tracing::debug!("No messages available after optimized polling (10 seconds)");
        Err(TransportError::ProtocolError {
            message: "No messages available".to_string(),
        })
    }

    async fn close(&mut self) -> Result<()> {
        // Close the session
        if let Some(session_id) = &self.session_id {
            let mut headers = HeaderMap::new();
            headers.insert("content-type", str_to_header_value("application/json")?);

            if let Some(token) = &self.config.auth_token {
                headers.insert("authorization", str_to_header_value(token)?);
            }

            let request_body = StreamableMcpRequest {
                session_id: Some(session_id.clone()),
                message: None,
                upgrade_to_stream: Some(false),
            };

            let url = format!("{}/mcp", self.config.base_url);
            let _ = self
                .client
                .post(&url)
                .headers(headers)
                .json(&request_body)
                .send()
                .await;
        }

        Ok(())
    }
}

/// Streamable HTTP transport - implements PRD specification
pub struct StreamableHttpTransport {
    server: HttpTransportServer,
    message_receiver: broadcast::Receiver<(String, JsonRpcMessage)>,
}

impl StreamableHttpTransport {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        let config = HttpTransportConfig {
            host: host.into(),
            port,
            enable_streamable_http: true,
            enable_legacy_endpoints: false,
            ..Default::default()
        };

        let server = HttpTransportServer::new(config);
        let message_receiver = server.get_message_receiver();

        Self {
            server,
            message_receiver,
        }
    }

    pub async fn run(self) -> Result<()> {
        self.server.run().await
    }
}

#[async_trait]
impl Transport for StreamableHttpTransport {
    async fn send_message(&mut self, message: JsonRpcMessage) -> Result<()> {
        // Queue message for delivery via HTTP polling or SSE streaming
        let _ = self
            .server
            .get_message_sender()
            .send(("*".to_string(), message));
        Ok(())
    }

    async fn receive_message(&mut self) -> Result<JsonRpcMessage> {
        match self.message_receiver.recv().await {
            Ok((_, message)) => Ok(message),
            Err(_) => Err(TransportError::ConnectionClosed),
        }
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Legacy HTTP+SSE transport for backward compatibility
///
/// ⚠️ **DEPRECATED**: SSE transport is deprecated per MCP 2025-03-26 specification.
/// Use StreamableHttpTransport instead for better proxy compatibility and performance.
#[deprecated(
    since = "0.1.0",
    note = "Use StreamableHttpTransport instead. SSE transport is deprecated per MCP 2025-03-26 specification."
)]
pub struct HttpSseTransport {
    server: HttpTransportServer,
    message_receiver: broadcast::Receiver<(String, JsonRpcMessage)>,
}

#[allow(deprecated)]
impl HttpSseTransport {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        let config = HttpTransportConfig {
            host: host.into(),
            port,
            enable_streamable_http: false,
            enable_legacy_endpoints: true,
            ..Default::default()
        };

        let server = HttpTransportServer::new(config);
        let message_receiver = server.get_message_receiver();

        Self {
            server,
            message_receiver,
        }
    }

    pub async fn run(self) -> Result<()> {
        self.server.run().await
    }
}

#[allow(deprecated)]
#[async_trait]
impl Transport for HttpSseTransport {
    async fn send_message(&mut self, message: JsonRpcMessage) -> Result<()> {
        // Use legacy message queuing approach
        let _ = self
            .server
            .get_message_sender()
            .send(("*".to_string(), message));
        Ok(())
    }

    async fn receive_message(&mut self) -> Result<JsonRpcMessage> {
        match self.message_receiver.recv().await {
            Ok((_, message)) => Ok(message),
            Err(_) => Err(TransportError::ConnectionClosed),
        }
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

// Helper types for Streamable HTTP
#[derive(Debug, Serialize, Deserialize)]
pub struct StreamableMcpRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub message: Option<JsonRpcMessage>,
    pub upgrade_to_stream: Option<bool>,
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

fn str_to_header_value(s: &str) -> Result<HeaderValue> {
    HeaderValue::from_str(s).map_err(|e| TransportError::SerializationError {
        message: format!("Invalid header value: {}", e),
    })
}
