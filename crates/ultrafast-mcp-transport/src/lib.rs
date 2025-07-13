//! UltraFast MCP Transport Layer
//!
//! This crate provides high-performance transport implementations for the Model Context Protocol (MCP).
//! It supports multiple transport types including STDIO and HTTP with advanced features like
//! connection pooling, rate limiting, and request optimization.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use ultrafast_mcp_core::protocol::JsonRpcMessage;

pub mod stdio;

#[cfg(feature = "http")]
pub mod streamable_http;

/// Result type for transport operations
pub type Result<T> = std::result::Result<T, TransportError>;

/// Connection state for transport lifecycle management
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    /// Transport is disconnected
    Disconnected,
    /// Transport is connecting
    Connecting,
    /// Transport is connected and ready
    Connected,
    /// Transport is reconnecting after a failure
    Reconnecting,
    /// Transport is shutting down gracefully
    ShuttingDown,
    /// Transport has failed and needs recovery
    Failed(String),
}

impl fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionState::Disconnected => write!(f, "disconnected"),
            ConnectionState::Connecting => write!(f, "connecting"),
            ConnectionState::Connected => write!(f, "connected"),
            ConnectionState::Reconnecting => write!(f, "reconnecting"),
            ConnectionState::ShuttingDown => write!(f, "shutting down"),
            ConnectionState::Failed(reason) => write!(f, "failed: {reason}"),
        }
    }
}

/// Transport health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportHealth {
    pub state: ConnectionState,
    pub last_activity: Option<std::time::SystemTime>,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub connection_duration: Option<std::time::Duration>,
    pub error_count: u64,
    pub last_error: Option<String>,
}

impl Default for TransportHealth {
    fn default() -> Self {
        Self {
            state: ConnectionState::Disconnected,
            last_activity: None,
            messages_sent: 0,
            messages_received: 0,
            connection_duration: None,
            error_count: 0,
            last_error: None,
        }
    }
}

/// Transport lifecycle events
#[derive(Debug, Clone)]
pub enum TransportEvent {
    Connected,
    Disconnected,
    Reconnecting,
    MessageSent,
    MessageReceived,
    Error(String),
    ShutdownRequested,
    ShutdownComplete,
}

/// Callback trait for transport lifecycle events
#[async_trait]
pub trait TransportEventHandler: Send + Sync {
    async fn handle_event(&self, event: TransportEvent);
}

/// Configuration for transport recovery
#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    pub max_retries: u32,
    pub initial_delay: std::time::Duration,
    pub max_delay: std::time::Duration,
    pub backoff_multiplier: f64,
    pub enable_jitter: bool,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            max_retries: 5,
            initial_delay: std::time::Duration::from_millis(100),
            max_delay: std::time::Duration::from_secs(30),
            backoff_multiplier: 2.0,
            enable_jitter: true,
        }
    }
}

/// Transport shutdown configuration
#[derive(Debug, Clone)]
pub struct ShutdownConfig {
    pub graceful_timeout: std::time::Duration,
    pub force_timeout: std::time::Duration,
    pub drain_pending_messages: bool,
}

impl Default for ShutdownConfig {
    fn default() -> Self {
        Self {
            graceful_timeout: std::time::Duration::from_secs(5),
            force_timeout: std::time::Duration::from_secs(10),
            drain_pending_messages: true,
        }
    }
}

/// Transport error types
#[derive(Debug, Error)]
pub enum TransportError {
    #[error("Connection error: {message}")]
    ConnectionError { message: String },

    #[error("Connection closed")]
    ConnectionClosed,

    #[error("Connection timeout")]
    ConnectionTimeout,

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

    #[error("Recovery failed after {attempts} attempts: {message}")]
    RecoveryFailed { attempts: u32, message: String },

    #[error("Shutdown timeout: {message}")]
    ShutdownTimeout { message: String },

    #[error("Transport not ready: current state is {state}")]
    NotReady { state: ConnectionState },
}

/// Enhanced transport trait with lifecycle management
#[async_trait]
pub trait Transport: Send + Sync {
    /// Send a message through the transport
    async fn send_message(&mut self, message: JsonRpcMessage) -> Result<()>;

    /// Receive a message from the transport
    async fn receive_message(&mut self) -> Result<JsonRpcMessage>;

    /// Close the transport connection gracefully
    async fn close(&mut self) -> Result<()>;

    /// Get current connection state
    fn get_state(&self) -> ConnectionState {
        ConnectionState::Connected // Default implementation for backward compatibility
    }

    /// Get transport health information
    fn get_health(&self) -> TransportHealth {
        TransportHealth {
            state: self.get_state(),
            ..Default::default()
        }
    }

    /// Check if transport is ready for operations
    fn is_ready(&self) -> bool {
        matches!(self.get_state(), ConnectionState::Connected)
    }

    /// Initiate graceful shutdown
    async fn shutdown(&mut self, config: ShutdownConfig) -> Result<()> {
        // Default implementation just calls close()
        tokio::time::timeout(config.graceful_timeout, self.close())
            .await
            .map_err(|_| TransportError::ShutdownTimeout {
                message: "Graceful shutdown timeout".to_string(),
            })?
    }

    /// Force immediate shutdown
    async fn force_shutdown(&mut self) -> Result<()> {
        // Default implementation just calls close()
        self.close().await
    }

    /// Attempt to reconnect the transport
    async fn reconnect(&mut self) -> Result<()> {
        // Default implementation: close and let the caller handle reconnection
        self.close().await?;
        Err(TransportError::ConnectionError {
            message: "Reconnection not supported by this transport".to_string(),
        })
    }

    /// Reset transport state and clear any cached data
    async fn reset(&mut self) -> Result<()> {
        // Default implementation just calls close()
        self.close().await
    }
}

/// Enhanced transport with automatic recovery
pub struct RecoveringTransport {
    inner: Box<dyn Transport>,
    recovery_config: RecoveryConfig,
    health: TransportHealth,
    event_handler: Option<Box<dyn TransportEventHandler>>,
    retry_count: u32,
    last_error: Option<String>,
}

impl RecoveringTransport {
    pub fn new(transport: Box<dyn Transport>, recovery_config: RecoveryConfig) -> Self {
        Self {
            inner: transport,
            recovery_config,
            health: TransportHealth::default(),
            event_handler: None,
            retry_count: 0,
            last_error: None,
        }
    }

    pub fn with_event_handler(mut self, handler: Box<dyn TransportEventHandler>) -> Self {
        self.event_handler = Some(handler);
        self
    }

    async fn emit_event(&self, event: TransportEvent) {
        if let Some(handler) = &self.event_handler {
            handler.handle_event(event).await;
        }
    }

    async fn attempt_recovery(&mut self) -> Result<()> {
        if self.retry_count >= self.recovery_config.max_retries {
            let error_msg = format!(
                "Max retries ({}) exceeded. Last error: {}",
                self.recovery_config.max_retries,
                self.last_error.as_deref().unwrap_or("unknown")
            );
            self.health.state = ConnectionState::Failed(error_msg.clone());
            return Err(TransportError::RecoveryFailed {
                attempts: self.retry_count,
                message: error_msg,
            });
        }

        self.health.state = ConnectionState::Reconnecting;
        self.emit_event(TransportEvent::Reconnecting).await;

        // Calculate delay with exponential backoff
        let delay = self.calculate_retry_delay();
        tokio::time::sleep(delay).await;

        // Attempt reconnection
        match self.inner.reconnect().await {
            Ok(()) => {
                self.health.state = ConnectionState::Connected;
                self.retry_count = 0;
                self.last_error = None;
                self.emit_event(TransportEvent::Connected).await;
                Ok(())
            }
            Err(e) => {
                self.retry_count += 1;
                self.last_error = Some(e.to_string());
                self.health.error_count += 1;
                self.health.last_error = Some(e.to_string());
                Err(e)
            }
        }
    }

    fn calculate_retry_delay(&self) -> std::time::Duration {
        let base_delay = self.recovery_config.initial_delay.as_millis() as f64;
        let multiplier = self
            .recovery_config
            .backoff_multiplier
            .powi(self.retry_count as i32);
        let mut delay_ms = base_delay * multiplier;

        // Add jitter if enabled
        if self.recovery_config.enable_jitter {
            use rand::Rng;
            let mut rng = rand::rng();
            let jitter: f64 = rng.random_range(0.8..1.2);
            delay_ms *= jitter;
        }

        // Cap at max delay
        let max_delay_ms = self.recovery_config.max_delay.as_millis() as f64;
        delay_ms = delay_ms.min(max_delay_ms);

        std::time::Duration::from_millis(delay_ms as u64)
    }
}

#[async_trait]
impl Transport for RecoveringTransport {
    async fn send_message(&mut self, message: JsonRpcMessage) -> Result<()> {
        loop {
            match self.inner.send_message(message.clone()).await {
                Ok(()) => {
                    self.health.messages_sent += 1;
                    self.health.last_activity = Some(std::time::SystemTime::now());
                    self.emit_event(TransportEvent::MessageSent).await;
                    return Ok(());
                }
                Err(e) => {
                    self.emit_event(TransportEvent::Error(e.to_string())).await;

                    // Try recovery for connection errors
                    if matches!(
                        e,
                        TransportError::ConnectionClosed | TransportError::ConnectionError { .. }
                    ) {
                        match self.attempt_recovery().await {
                            Ok(()) => continue, // Retry the send
                            Err(recovery_err) => return Err(recovery_err),
                        }
                    } else {
                        return Err(e);
                    }
                }
            }
        }
    }

    async fn receive_message(&mut self) -> Result<JsonRpcMessage> {
        loop {
            match self.inner.receive_message().await {
                Ok(message) => {
                    self.health.messages_received += 1;
                    self.health.last_activity = Some(std::time::SystemTime::now());
                    self.emit_event(TransportEvent::MessageReceived).await;
                    return Ok(message);
                }
                Err(e) => {
                    self.emit_event(TransportEvent::Error(e.to_string())).await;

                    // Try recovery for connection errors
                    if matches!(
                        e,
                        TransportError::ConnectionClosed | TransportError::ConnectionError { .. }
                    ) {
                        match self.attempt_recovery().await {
                            Ok(()) => continue, // Retry the receive
                            Err(recovery_err) => return Err(recovery_err),
                        }
                    } else {
                        return Err(e);
                    }
                }
            }
        }
    }

    async fn close(&mut self) -> Result<()> {
        self.health.state = ConnectionState::ShuttingDown;
        self.emit_event(TransportEvent::ShutdownRequested).await;

        let result = self.inner.close().await;

        self.health.state = ConnectionState::Disconnected;
        self.emit_event(TransportEvent::ShutdownComplete).await;

        result
    }

    fn get_state(&self) -> ConnectionState {
        self.health.state.clone()
    }

    fn get_health(&self) -> TransportHealth {
        self.health.clone()
    }

    async fn shutdown(&mut self, config: ShutdownConfig) -> Result<()> {
        self.inner.shutdown(config).await
    }

    async fn force_shutdown(&mut self) -> Result<()> {
        self.inner.force_shutdown().await
    }

    async fn reconnect(&mut self) -> Result<()> {
        self.attempt_recovery().await
    }

    async fn reset(&mut self) -> Result<()> {
        self.health = TransportHealth::default();
        self.retry_count = 0;
        self.last_error = None;
        self.inner.reset().await
    }
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
}

/// Create a transport from configuration
pub async fn create_transport(config: TransportConfig) -> Result<Box<dyn Transport>> {
    match config {
        TransportConfig::Stdio => {
            let transport = stdio::StdioTransport::new().await?;
            Ok(Box::new(transport))
        }

        #[cfg(feature = "http")]
        TransportConfig::Streamable {
            base_url,
            auth_token,
            session_id,
        } => {
            let client_config = streamable_http::client::StreamableHttpClientConfig {
                base_url,
                auth_token,
                session_id,
                auth_method: None,
                ..Default::default()
            };

            let mut client = streamable_http::client::StreamableHttpClient::new(client_config)?;
            client.connect().await?;
            Ok(Box::new(client))
        }
    }
}

/// Create a transport with automatic recovery
pub async fn create_recovering_transport(
    config: TransportConfig,
    recovery_config: RecoveryConfig,
) -> Result<Box<dyn Transport>> {
    let transport = create_transport(config).await?;
    let recovering_transport = RecoveringTransport::new(transport, recovery_config);
    Ok(Box::new(recovering_transport))
}
