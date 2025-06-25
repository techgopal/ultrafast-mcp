//! STDIO transport implementation for MCP
//! 
//! This module provides a transport that communicates over standard input/output,
//! which is the most common transport for MCP servers.

use crate::{Transport, TransportError, Result};
use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter, AsyncReadExt};
use tracing::{debug, trace};
use ultrafast_mcp_core::protocol::JsonRpcMessage;

/// STDIO transport for MCP communication
pub struct StdioTransport {
    stdin: BufReader<tokio::io::Stdin>,
    stdout: BufWriter<tokio::io::Stdout>,
    connected: bool,
}

impl StdioTransport {
    /// Create a new STDIO transport
    pub async fn new() -> Result<Self> {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        
        Ok(Self {
            stdin: BufReader::new(stdin),
            stdout: BufWriter::new(stdout),
            connected: true,
        })
    }
}

#[async_trait]
impl Transport for StdioTransport {
    async fn send_message(&mut self, message: JsonRpcMessage) -> Result<()> {
        if !self.connected {
            return Err(TransportError::ConnectionClosed);
        }
        
        // Serialize the message to JSON
        let json_str = serde_json::to_string(&message)
            .map_err(|e| TransportError::SerializationError { 
                message: format!("Failed to serialize message: {}", e) 
            })?;
        
        trace!("Sending message: {}", json_str);
        
        // Write Content-Length header followed by empty line and message (MCP spec compliant)
        let content_length = json_str.len();
        let header = format!("Content-Length: {}\r\n\r\n", content_length);
        
        self.stdout.write_all(header.as_bytes()).await
            .map_err(|e| TransportError::NetworkError { 
                message: format!("Failed to write Content-Length header: {}", e) 
            })?;
        
        self.stdout.write_all(json_str.as_bytes()).await
            .map_err(|e| TransportError::NetworkError { 
                message: format!("Failed to write message: {}", e) 
            })?;
        
        self.stdout.flush().await
            .map_err(|e| TransportError::NetworkError { 
                message: format!("Failed to flush stdout: {}", e) 
            })?;
        
        debug!("Successfully sent message with Content-Length: {}", content_length);
        Ok(())
    }
    
    async fn receive_message(&mut self) -> Result<JsonRpcMessage> {
        if !self.connected {
            return Err(TransportError::ConnectionClosed);
        }
        
        // Read headers until we find Content-Length
        let mut content_length = 0;
        loop {
            let mut line = String::new();
            let bytes_read = self.stdin.read_line(&mut line).await
                .map_err(|e| TransportError::NetworkError { 
                    message: format!("Failed to read header line from stdin: {}", e) 
                })?;
            
            if bytes_read == 0 {
                // EOF reached
                self.connected = false;
                return Err(TransportError::ConnectionClosed);
            }
            
            let line = line.trim();
            
            if line.is_empty() {
                // Empty line indicates end of headers
                break;
            }
            
            if let Some(length_str) = line.strip_prefix("Content-Length:") {
                content_length = length_str.trim().parse::<usize>()
                    .map_err(|e| TransportError::SerializationError { 
                        message: format!("Invalid Content-Length header: {}", e) 
                    })?;
            }
        }
        
        if content_length == 0 {
            return Err(TransportError::SerializationError { 
                message: "Missing or invalid Content-Length header".to_string() 
            });
        }
        
        // Read the exact number of bytes specified by Content-Length
        let mut message_bytes = vec![0u8; content_length];
        self.stdin.read_exact(&mut message_bytes).await
            .map_err(|e| TransportError::NetworkError { 
                message: format!("Failed to read message content from stdin: {}", e) 
            })?;
        
        let message_str = String::from_utf8(message_bytes)
            .map_err(|e| TransportError::SerializationError { 
                message: format!("Invalid UTF-8 in message: {}", e) 
            })?;
        
        trace!("Received message: {}", message_str);
        
        // Parse the JSON message
        let message: JsonRpcMessage = serde_json::from_str(&message_str)
            .map_err(|e| TransportError::SerializationError { 
                message: format!("Failed to parse JSON message: {}", e) 
            })?;
        
        debug!("Successfully received message");
        Ok(message)
    }
    
    async fn close(&mut self) -> Result<()> {
        self.connected = false;
        debug!("STDIO transport closed");
        Ok(())
    }
}