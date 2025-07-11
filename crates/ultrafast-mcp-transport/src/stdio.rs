//! STDIO transport implementation for MCP
//!
//! This module provides a transport that communicates over standard input/output,
//! which is the most common transport for MCP servers.

use crate::{ConnectionState, Result, Transport, TransportError, TransportHealth};
use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tracing::{debug, trace};
use ultrafast_mcp_core::protocol::JsonRpcMessage;

/// STDIO transport for MCP communication
pub struct StdioTransport {
    stdin: BufReader<tokio::io::Stdin>,
    stdout: BufWriter<tokio::io::Stdout>,
    health: TransportHealth,
    connected_at: Option<std::time::SystemTime>,
}

impl StdioTransport {
    /// Create a new STDIO transport
    pub async fn new() -> Result<Self> {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();

        let health = TransportHealth {
            state: ConnectionState::Connected,
            ..Default::default()
        };

        let connected_at = Some(std::time::SystemTime::now());

        Ok(Self {
            stdin: BufReader::new(stdin),
            stdout: BufWriter::new(stdout),
            health,
            connected_at,
        })
    }

    fn update_connection_duration(&mut self) {
        if let Some(connected_at) = self.connected_at {
            self.health.connection_duration = connected_at.elapsed().ok();
        }
    }
}

#[async_trait]
impl Transport for StdioTransport {
    async fn send_message(&mut self, message: JsonRpcMessage) -> Result<()> {
        if !matches!(self.health.state, ConnectionState::Connected) {
            return Err(TransportError::NotReady {
                state: self.health.state.clone(),
            });
        }

        // Serialize the message to JSON
        let json_str = serde_json::to_string(&message).map_err(|e| {
            self.health.error_count += 1;
            self.health.last_error = Some(format!("Serialization error: {}", e));
            TransportError::SerializationError {
                message: format!("Failed to serialize message: {}", e),
            }
        })?;

        trace!("Sending message: {}", json_str);

        // MCP STDIO protocol: newline-delimited JSON (one message per line)
        self.stdout
            .write_all(json_str.as_bytes())
            .await
            .map_err(|e| {
                self.health.error_count += 1;
                self.health.last_error = Some(format!("Write error: {}", e));
                self.health.state = ConnectionState::Failed(format!("Write failed: {}", e));
                TransportError::NetworkError {
                    message: format!("Failed to write message: {}", e),
                }
            })?;

        // Add newline to delimit the message
        self.stdout.write_all(b"\n").await.map_err(|e| {
            self.health.error_count += 1;
            self.health.last_error = Some(format!("Write newline error: {}", e));
            self.health.state = ConnectionState::Failed(format!("Write newline failed: {}", e));
            TransportError::NetworkError {
                message: format!("Failed to write newline: {}", e),
            }
        })?;

        self.stdout.flush().await.map_err(|e| {
            self.health.error_count += 1;
            self.health.last_error = Some(format!("Flush error: {}", e));
            self.health.state = ConnectionState::Failed(format!("Flush failed: {}", e));
            TransportError::NetworkError {
                message: format!("Failed to flush stdout: {}", e),
            }
        })?;

        // Update health metrics
        self.health.messages_sent += 1;
        self.health.last_activity = Some(std::time::SystemTime::now());
        self.update_connection_duration();

        debug!("Successfully sent message with {} bytes", json_str.len());
        Ok(())
    }

    async fn receive_message(&mut self) -> Result<JsonRpcMessage> {
        if !matches!(self.health.state, ConnectionState::Connected) {
            return Err(TransportError::NotReady {
                state: self.health.state.clone(),
            });
        }

        // Read a line from stdin (newline-delimited JSON)
        let mut line = String::new();
        let bytes_read = self.stdin.read_line(&mut line).await.map_err(|e| {
            self.health.error_count += 1;
            self.health.last_error = Some(format!("Read error: {}", e));
            TransportError::NetworkError {
                message: format!("Failed to read line from stdin: {}", e),
            }
        })?;

        if bytes_read == 0 {
            // EOF reached
            self.health.state = ConnectionState::Disconnected;
            return Err(TransportError::ConnectionClosed);
        }

        // Remove trailing newline
        let message_str = line.trim_end();

        if message_str.is_empty() {
            self.health.error_count += 1;
            self.health.last_error = Some("Empty message received".to_string());
            return Err(TransportError::SerializationError {
                message: "Received empty message".to_string(),
            });
        }

        trace!("Received message: {}", message_str);

        // Parse the JSON message
        let message: JsonRpcMessage = serde_json::from_str(message_str).map_err(|e| {
            self.health.error_count += 1;
            self.health.last_error = Some(format!("Parse error: {}", e));
            TransportError::SerializationError {
                message: format!("Failed to parse JSON message: {}", e),
            }
        })?;

        // Update health metrics
        self.health.messages_received += 1;
        self.health.last_activity = Some(std::time::SystemTime::now());
        self.update_connection_duration();

        debug!("Successfully received message");
        Ok(message)
    }

    async fn close(&mut self) -> Result<()> {
        self.health.state = ConnectionState::Disconnected;
        debug!("STDIO transport closed");
        Ok(())
    }

    fn get_state(&self) -> ConnectionState {
        self.health.state.clone()
    }

    fn get_health(&self) -> TransportHealth {
        let mut health = self.health.clone();
        if let Some(connected_at) = self.connected_at {
            health.connection_duration = connected_at.elapsed().ok();
        }
        health
    }

    async fn shutdown(&mut self, _config: crate::ShutdownConfig) -> Result<()> {
        self.health.state = ConnectionState::ShuttingDown;
        debug!("STDIO transport shutting down gracefully");
        self.close().await
    }

    async fn force_shutdown(&mut self) -> Result<()> {
        debug!("STDIO transport force shutdown");
        self.close().await
    }

    async fn reset(&mut self) -> Result<()> {
        self.health = TransportHealth::default();
        self.health.state = ConnectionState::Connected;
        self.connected_at = Some(std::time::SystemTime::now());
        debug!("STDIO transport reset");
        Ok(())
    }
}
