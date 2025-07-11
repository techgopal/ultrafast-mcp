//! Mock implementations for testing

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use ultrafast_mcp_core::protocol::JsonRpcMessage;
use ultrafast_mcp_transport::{ConnectionState, Transport, TransportError, TransportHealth};

/// Mock transport for testing without real network connections
pub struct MockTransport {
    pub sent_messages: Arc<Mutex<Vec<JsonRpcMessage>>>,
    pub receive_queue: Arc<Mutex<VecDeque<JsonRpcMessage>>>,
    pub state: Arc<Mutex<ConnectionState>>,
    pub should_fail_send: Arc<Mutex<bool>>,
    pub should_fail_receive: Arc<Mutex<bool>>,
}

impl MockTransport {
    pub fn new() -> Self {
        Self {
            sent_messages: Arc::new(Mutex::new(Vec::new())),
            receive_queue: Arc::new(Mutex::new(VecDeque::new())),
            state: Arc::new(Mutex::new(ConnectionState::Connected)),
            should_fail_send: Arc::new(Mutex::new(false)),
            should_fail_receive: Arc::new(Mutex::new(false)),
        }
    }

    /// Add a message to the receive queue
    pub fn add_receive_message(&self, message: JsonRpcMessage) {
        let mut queue = self.receive_queue.lock().unwrap();
        queue.push_back(message);
    }

    /// Get all sent messages
    pub fn get_sent_messages(&self) -> Vec<JsonRpcMessage> {
        self.sent_messages.lock().unwrap().clone()
    }

    /// Clear sent messages
    pub fn clear_sent_messages(&self) {
        self.sent_messages.lock().unwrap().clear();
    }

    /// Set whether send operations should fail
    pub fn set_fail_send(&self, should_fail: bool) {
        *self.should_fail_send.lock().unwrap() = should_fail;
    }

    /// Set whether receive operations should fail
    pub fn set_fail_receive(&self, should_fail: bool) {
        *self.should_fail_receive.lock().unwrap() = should_fail;
    }

    /// Set the connection state
    pub fn set_state(&self, state: ConnectionState) {
        *self.state.lock().unwrap() = state;
    }
}

impl Default for MockTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Transport for MockTransport {
    async fn send_message(&mut self, message: JsonRpcMessage) -> Result<(), TransportError> {
        if *self.should_fail_send.lock().unwrap() {
            return Err(TransportError::ConnectionError {
                message: "Mock send failure".to_string(),
            });
        }

        let mut sent = self.sent_messages.lock().unwrap();
        sent.push(message);
        Ok(())
    }

    async fn receive_message(&mut self) -> Result<JsonRpcMessage, TransportError> {
        if *self.should_fail_receive.lock().unwrap() {
            return Err(TransportError::ConnectionError {
                message: "Mock receive failure".to_string(),
            });
        }

        let mut queue = self.receive_queue.lock().unwrap();
        queue.pop_front().ok_or(TransportError::ConnectionClosed)
    }

    async fn close(&mut self) -> Result<(), TransportError> {
        self.set_state(ConnectionState::Disconnected);
        Ok(())
    }

    fn get_state(&self) -> ConnectionState {
        self.state.lock().unwrap().clone()
    }

    fn get_health(&self) -> TransportHealth {
        TransportHealth {
            state: self.get_state(),
            last_activity: Some(std::time::SystemTime::now()),
            messages_sent: self.sent_messages.lock().unwrap().len() as u64,
            messages_received: 0, // Mock doesn't track this
            connection_duration: Some(std::time::Duration::from_secs(10)),
            error_count: 0,
            last_error: None,
        }
    }
}

/// Create a mock transport with predefined messages
pub fn create_mock_transport_with_messages(messages: Vec<JsonRpcMessage>) -> MockTransport {
    let transport = MockTransport::new();
    for message in messages {
        transport.add_receive_message(message);
    }
    transport
}

/// Create a mock transport that fails on send
pub fn create_failing_send_transport() -> MockTransport {
    let transport = MockTransport::new();
    transport.set_fail_send(true);
    transport
}

/// Create a mock transport that fails on receive
pub fn create_failing_receive_transport() -> MockTransport {
    let transport = MockTransport::new();
    transport.set_fail_receive(true);
    transport
}

#[cfg(test)]
mod tests {
    use super::*;
    use ultrafast_mcp_core::protocol::jsonrpc::{JsonRpcRequest, RequestId};

    #[tokio::test]
    async fn test_mock_transport_send() {
        let mut transport = MockTransport::new();
        let message = JsonRpcMessage::Request(JsonRpcRequest::new(
            "test".to_string(),
            None,
            Some(RequestId::Number(1)),
        ));

        let result = transport.send_message(message.clone()).await;
        assert!(result.is_ok());

        let sent = transport.get_sent_messages();
        assert_eq!(sent.len(), 1);
        // Note: JsonRpcMessage doesn't implement PartialEq due to HashMap<String, Value>
        // We can only check that a message was sent, not compare content
    }

    #[tokio::test]
    async fn test_mock_transport_receive() {
        let mut transport = MockTransport::new();
        let message = JsonRpcMessage::Request(JsonRpcRequest::new(
            "test".to_string(),
            None,
            Some(RequestId::Number(1)),
        ));

        transport.add_receive_message(message.clone());

        let received = transport.receive_message().await;
        assert!(received.is_ok());
        // Note: JsonRpcMessage doesn't implement PartialEq due to HashMap<String, Value>
        // We can only check that a message was received, not compare content
    }

    #[tokio::test]
    async fn test_mock_transport_fail_send() {
        let mut transport = MockTransport::new();
        transport.set_fail_send(true);

        let message = JsonRpcMessage::Request(JsonRpcRequest::new(
            "test".to_string(),
            None,
            Some(RequestId::Number(1)),
        ));

        let result = transport.send_message(message).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_transport_fail_receive() {
        let mut transport = MockTransport::new();
        transport.set_fail_receive(true);

        let result = transport.receive_message().await;
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_transport_state() {
        let transport = MockTransport::new();
        assert_eq!(transport.get_state(), ConnectionState::Connected);

        transport.set_state(ConnectionState::Disconnected);
        assert_eq!(transport.get_state(), ConnectionState::Disconnected);
    }

    #[test]
    fn test_create_mock_transport_with_messages() {
        let messages = vec![
            JsonRpcMessage::Request(JsonRpcRequest::new(
                "test1".to_string(),
                None,
                Some(RequestId::Number(1)),
            )),
            JsonRpcMessage::Request(JsonRpcRequest::new(
                "test2".to_string(),
                None,
                Some(RequestId::Number(2)),
            )),
        ];

        let transport = create_mock_transport_with_messages(messages);
        assert_eq!(transport.receive_queue.lock().unwrap().len(), 2);
    }
}
