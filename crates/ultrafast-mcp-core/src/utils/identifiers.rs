//! Identifier generation utilities
//!
//! This module consolidates all identifier generation functions that were
//! previously scattered across different crates.

use uuid::Uuid;

/// Generate a new session ID for MCP connections
///
/// Consolidates implementations from:
/// - ultrafast-mcp-auth/src/pkce.rs
/// - ultrafast-mcp-transport/src/streamable_http/server.rs
pub fn generate_session_id() -> String {
    Uuid::new_v4().to_string()
}

/// Generate a new state parameter for OAuth flows
///
/// Consolidates implementation from ultrafast-mcp-auth/src/pkce.rs
pub fn generate_state() -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    (0..32)
        .map(|_| {
            let idx = rng.random_range(0..62);
            match idx {
                0..=25 => (b'A' + idx) as char,
                26..=51 => (b'a' + (idx - 26)) as char,
                _ => (b'0' + (idx - 52)) as char,
            }
        })
        .collect()
}

/// Generate a new event ID for Server-Sent Events
///
/// Consolidates implementation from ultrafast-mcp-transport/src/streamable_http/server.rs
pub fn generate_event_id() -> String {
    Uuid::new_v4().to_string()
}

/// Generate a new request ID for JSON-RPC requests
pub fn generate_request_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Generate a cryptographically secure random string of specified length
pub fn generate_secure_random(length: usize) -> String {
    use rand::Rng;
    let charset = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    (0..length)
        .map(|_| {
            let idx = rng.random_range(0..charset.len());
            charset[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_session_id() {
        let id1 = generate_session_id();
        let id2 = generate_session_id();

        assert_ne!(id1, id2);
        assert_eq!(id1.len(), 36); // UUID format
        assert!(id1.contains('-'));
    }

    #[test]
    fn test_generate_state() {
        let state1 = generate_state();
        let state2 = generate_state();

        assert_ne!(state1, state2);
        assert_eq!(state1.len(), 32);
        assert!(state1.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_generate_event_id() {
        let id1 = generate_event_id();
        let id2 = generate_event_id();

        assert_ne!(id1, id2);
        assert_eq!(id1.len(), 36); // UUID format
    }

    #[test]
    fn test_generate_request_id() {
        let id1 = generate_request_id();
        let id2 = generate_request_id();

        assert_ne!(id1, id2);
        assert!(id2 > id1);
    }

    #[test]
    fn test_generate_secure_random() {
        let random1 = generate_secure_random(16);
        let random2 = generate_secure_random(16);

        assert_ne!(random1, random2);
        assert_eq!(random1.len(), 16);
        assert_eq!(random2.len(), 16);
        assert!(random1.chars().all(|c| c.is_ascii_alphanumeric()));
    }
}
