use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use ultrafast_mcp_core::protocol::JsonRpcMessage;

/// HTTP session management for MCP connections
#[derive(Debug, Clone)]
pub struct HttpSession {
    pub session_id: String,
    pub created_at: std::time::SystemTime,
    pub last_activity: std::time::SystemTime,
    pub metadata: HashMap<String, String>,
}

impl HttpSession {
    pub fn new(session_id: String) -> Self {
        let now = std::time::SystemTime::now();
        Self {
            session_id,
            created_at: now,
            last_activity: now,
            metadata: HashMap::new(),
        }
    }

    pub fn update_activity(&mut self) {
        self.last_activity = std::time::SystemTime::now();
    }

    pub fn is_expired(&self, timeout_secs: u64) -> bool {
        self.last_activity
            .elapsed()
            .map(|d| d.as_secs() > timeout_secs)
            .unwrap_or(true)
    }
}

/// Session store for HTTP transport
#[derive(Clone)]
pub struct SessionStore {
    sessions: Arc<RwLock<HashMap<String, HttpSession>>>,
    timeout_secs: u64,
}

impl SessionStore {
    pub fn new(timeout_secs: u64) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            timeout_secs,
        }
    }

    pub async fn create_session(&self, session_id: String) -> HttpSession {
        let session = HttpSession::new(session_id.clone());
        self.sessions
            .write()
            .await
            .insert(session_id, session.clone());
        session
    }

    pub async fn get_session(&self, session_id: &str) -> Option<HttpSession> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            if session.is_expired(self.timeout_secs) {
                sessions.remove(session_id);
                None
            } else {
                session.update_activity();
                Some(session.clone())
            }
        } else {
            None
        }
    }

    pub async fn remove_session(&self, session_id: &str) {
        self.sessions.write().await.remove(session_id);
    }

    pub async fn cleanup_expired_sessions(&self) {
        let mut sessions = self.sessions.write().await;
        sessions.retain(|_, session| !session.is_expired(self.timeout_secs));
    }
}

/// Message queue for HTTP transport (for message redelivery)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedMessage {
    pub id: String,
    pub message: JsonRpcMessage,
    pub timestamp: u64,
    pub retry_count: u32,
}

#[derive(Clone)]
pub struct MessageQueue {
    messages: Arc<RwLock<HashMap<String, Vec<QueuedMessage>>>>,
    max_retries: u32,
}

impl MessageQueue {
    pub fn new(max_retries: u32) -> Self {
        Self {
            messages: Arc::new(RwLock::new(HashMap::new())),
            max_retries,
        }
    }

    pub async fn enqueue_message(&self, session_id: String, message: JsonRpcMessage) {
        let queued_message = QueuedMessage {
            id: uuid::Uuid::new_v4().to_string(),
            message,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            retry_count: 0,
        };

        self.messages
            .write()
            .await
            .entry(session_id)
            .or_insert_with(Vec::new)
            .push(queued_message);
    }

    pub async fn get_pending_messages(&self, session_id: &str) -> Vec<QueuedMessage> {
        self.messages
            .read()
            .await
            .get(session_id)
            .cloned()
            .unwrap_or_default()
    }

    pub async fn acknowledge_message(&self, session_id: &str, message_id: &str) {
        if let Some(messages) = self.messages.write().await.get_mut(session_id) {
            messages.retain(|msg| msg.id != message_id);
        }
    }

    pub async fn increment_retry(&self, session_id: &str, message_id: &str) -> bool {
        if let Some(messages) = self.messages.write().await.get_mut(session_id) {
            if let Some(message) = messages.iter_mut().find(|msg| msg.id == message_id) {
                message.retry_count += 1;
                if message.retry_count >= self.max_retries {
                    // Remove message if max retries exceeded
                    messages.retain(|msg| msg.id != message_id);
                    return false;
                }
                return true;
            }
        }
        false
    }
}
