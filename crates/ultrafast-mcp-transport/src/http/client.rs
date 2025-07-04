use crate::{Result, Transport, TransportError};
use async_trait::async_trait;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing;
use ultrafast_mcp_core::protocol::JsonRpcMessage;

/// MCP Protocol version constant
pub const MCP_PROTOCOL_VERSION: &str = "2025-06-18";

/// HTTP client configuration
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    pub base_url: String,
    pub session_id: Option<String>,
    pub protocol_version: String,
    pub timeout: Duration,
    pub max_retries: u32,
    pub auth_token: Option<String>,
}

impl HttpClientConfig {
    /// Get the MCP endpoint URL to avoid repeated string allocation
    pub fn mcp_url(&self) -> String {
        format!("{}/mcp", self.base_url)
    }

    /// Get the messages endpoint URL
    pub fn messages_url(&self) -> String {
        format!("{}/mcp/messages", self.base_url)
    }
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            base_url: "http://127.0.0.1:8080".to_string(),
            session_id: None,
            protocol_version: MCP_PROTOCOL_VERSION.to_string(),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            auth_token: None,
        }
    }
}

/// HTTP MCP client implementation
pub struct HttpTransportClient {
    client: Client,
    config: HttpClientConfig,
    session_id: Option<String>,
    message_receiver: mpsc::Receiver<JsonRpcMessage>,
    message_sender: mpsc::Sender<JsonRpcMessage>,
}

impl HttpTransportClient {
    pub fn new(config: HttpClientConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|e| TransportError::InitializationError {
                message: format!("Failed to create HTTP client: {}", e),
            })?;

        let (message_sender, message_receiver) = mpsc::channel(1000);

        Ok(Self {
            client,
            config,
            session_id: None,
            message_receiver,
            message_sender,
        })
    }

    #[cfg(feature = "http")]
    pub fn with_oauth_token(mut self, token: &str) -> Self {
        self.config.auth_token = Some(format!("Bearer {}", token));
        self
    }

    /// Connect to the MCP server
    pub async fn connect(&mut self) -> Result<String> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "mcp-protocol-version",
            str_to_header_value(&self.config.protocol_version)?,
        );

        if let Some(token) = &self.config.auth_token {
            headers.insert("authorization", str_to_header_value(token)?);
        }

        let mut query_params = Vec::new();
        if let Some(session_id) = &self.config.session_id {
            query_params.push(("session_id", session_id.as_str()));
        }

        let url = self.config.mcp_url();
        let response = self
            .client
            .get(&url)
            .headers(headers)
            .query(&query_params)
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

        let connect_response: McpConnectResponse =
            response
                .json()
                .await
                .map_err(|e| TransportError::SerializationError {
                    message: format!("Failed to parse connect response: {}", e),
                })?;

        self.session_id = Some(connect_response.session_id.clone());

        // Process any pending messages
        for message in connect_response.pending_messages {
            let _ = self.message_sender.send(message).await;
        }

        // Start polling for messages
        self.start_message_polling().await;

        Ok(connect_response.session_id)
    }

    /// Start polling for messages from the server
    async fn start_message_polling(&self) {
        let client = self.client.clone();
        let config = self.config.clone();
        let session_id = self.session_id.clone();
        let sender = self.message_sender.clone();

        tokio::spawn(async move {
            if let Some(session_id) = session_id {
                tracing::info!("Starting message polling for session: {}", session_id);
                let mut interval = tokio::time::interval(Duration::from_millis(500));

                loop {
                    interval.tick().await;

                    tracing::debug!("Polling for messages, session: {}", session_id);
                    if let Ok(messages) = poll_messages(&client, &config, &session_id, 0).await {
                        tracing::info!("Received {} messages", messages.len());
                        for message in messages {
                            let _ = sender.send(message).await;
                        }
                    }

                    // Break on channel closed
                    if sender.is_closed() {
                        break;
                    }
                }
            } else {
                tracing::error!("No session ID available for polling");
            }
        });
    }

    /// Send a message and acknowledge it
    async fn send_and_ack(&self, message: JsonRpcMessage) -> Result<()> {
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

        let request_body = McpRequest {
            session_id: session_id.clone(),
            message: message.clone(),
        };

        let url = self.config.mcp_url();
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

        let _send_response: McpResponse =
            response
                .json()
                .await
                .map_err(|e| TransportError::SerializationError {
                    message: format!("Failed to parse send response: {}", e),
                })?;

        Ok(())
    }
}

#[async_trait]
impl Transport for HttpTransportClient {
    async fn send_message(&mut self, message: JsonRpcMessage) -> Result<()> {
        self.send_and_ack(message).await
    }

    async fn receive_message(&mut self) -> Result<JsonRpcMessage> {
        self.message_receiver
            .recv()
            .await
            .ok_or(TransportError::ConnectionClosed)
    }

    async fn close(&mut self) -> Result<()> {
        self.session_id = None;
        Ok(())
    }
}

/// Poll for messages from the server
async fn poll_messages(
    client: &Client,
    config: &HttpClientConfig,
    session_id: &str,
    since: u64,
) -> Result<Vec<JsonRpcMessage>> {
    let mut headers = HeaderMap::new();
    if let Some(token) = &config.auth_token {
        headers.insert("authorization", str_to_header_value(token)?);
    }

    let mut query_params = vec![("session_id", session_id.to_string())];
    if since > 0 {
        query_params.push(("since", since.to_string()));
    }

    let url = config.messages_url();
    let response = client
        .get(&url)
        .headers(headers)
        .query(
            &query_params
                .iter()
                .map(|(k, v)| (k, v.as_str()))
                .collect::<Vec<_>>(),
        )
        .send()
        .await
        .map_err(|e| TransportError::NetworkError {
            message: format!("Failed to poll messages: {}", e),
        })?;

    if !response.status().is_success() {
        tracing::warn!(
            "Polling failed with status {}: {}",
            response.status(),
            response.text().await.unwrap_or_default()
        );
        return Ok(Vec::new()); // Return empty on error
    }

    let messages: Vec<JsonRpcMessage> =
        response
            .json()
            .await
            .map_err(|e| TransportError::SerializationError {
                message: format!("Failed to parse messages: {}", e),
            })?;

    Ok(messages)
}

/// Response types (shared with server)
#[derive(Debug, Serialize, Deserialize)]
pub struct McpConnectResponse {
    pub session_id: String,
    pub protocol_version: String,
    pub pending_messages: Vec<JsonRpcMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpRequest {
    pub session_id: String,
    pub message: JsonRpcMessage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpResponse {
    pub success: bool,
    pub message_id: Option<String>,
    pub error: Option<String>,
}

fn str_to_header_value(s: &str) -> Result<HeaderValue> {
    HeaderValue::from_str(s).map_err(|e| TransportError::InitializationError {
        message: format!("Failed to parse header value: {}", e),
    })
}
