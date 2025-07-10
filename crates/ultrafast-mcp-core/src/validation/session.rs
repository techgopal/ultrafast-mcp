//! Session validation utilities

use crate::error::{MCPResult, ValidationError};

/// Validate session ID format (visible ASCII characters only)
/// 
/// Consolidates implementation from transport/streamable_http/server.rs
pub fn validate_session_id(session_id: &str) -> MCPResult<()> {
    if session_id.is_empty() {
        return Err(ValidationError::RequiredField {
            field: "session_id".to_string(),
        }.into());
    }
    
    // Check that all characters are visible ASCII (0x21-0x7E)
    if !session_id.chars().all(|c| {
        let code = c as u8;
        code >= 0x21 && code <= 0x7E
    }) {
        return Err(ValidationError::InvalidFormat {
            field: "session_id".to_string(),
            expected: "visible ASCII characters only (0x21-0x7E)".to_string(),
        }.into());
    }
    
    // Check reasonable length limits
    if session_id.len() < 8 {
        return Err(ValidationError::ValueOutOfRange {
            field: "session_id".to_string(),
            min: "8".to_string(),
            max: "255".to_string(),
            actual: session_id.len().to_string(),
        }.into());
    }
    
    if session_id.len() > 255 {
        return Err(ValidationError::ValueOutOfRange {
            field: "session_id".to_string(),
            min: "8".to_string(),
            max: "255".to_string(),
            actual: session_id.len().to_string(),
        }.into());
    }
    
    Ok(())
}

/// Check if a session ID format is valid (returns bool for quick checks)
pub fn is_valid_session_id(session_id: &str) -> bool {
    validate_session_id(session_id).is_ok()
}

/// Validate event ID for Server-Sent Events
pub fn validate_event_id(event_id: &str) -> MCPResult<()> {
    if event_id.is_empty() {
        return Err(ValidationError::RequiredField {
            field: "event_id".to_string(),
        }.into());
    }
    
    // Event IDs should be alphanumeric with hyphens (UUID format is common)
    if !event_id.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return Err(ValidationError::InvalidFormat {
            field: "event_id".to_string(),
            expected: "alphanumeric characters and hyphens only".to_string(),
        }.into());
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_session_id() {
        // Valid session IDs
        assert!(validate_session_id("abcd1234").is_ok());
        assert!(validate_session_id("session-123-abc").is_ok());
        assert!(validate_session_id("!@#$%^&*()_+-={}[]|\\:;\"'<>?,./").is_ok()); // All visible ASCII
        
        // Invalid session IDs
        assert!(validate_session_id("").is_err()); // Empty
        assert!(validate_session_id("short").is_err()); // Too short
        assert!(validate_session_id(&"a".repeat(256)).is_err()); // Too long
        assert!(validate_session_id("session with spaces").is_err()); // Contains space
        assert!(validate_session_id("session\twith\ttabs").is_err()); // Contains tabs
        assert!(validate_session_id("session\nwith\nnewlines").is_err()); // Contains newlines
    }

    #[test]
    fn test_is_valid_session_id() {
        assert!(is_valid_session_id("valid-session-123"));
        assert!(!is_valid_session_id(""));
        assert!(!is_valid_session_id("short"));
        assert!(!is_valid_session_id("invalid with spaces"));
    }

    #[test]
    fn test_validate_event_id() {
        // Valid event IDs
        assert!(validate_event_id("event123").is_ok());
        assert!(validate_event_id("550e8400-e29b-41d4-a716-446655440000").is_ok()); // UUID format
        assert!(validate_event_id("event-id-123").is_ok());
        
        // Invalid event IDs
        assert!(validate_event_id("").is_err()); // Empty
        assert!(validate_event_id("event with spaces").is_err()); // Contains spaces
        assert!(validate_event_id("event.with.dots").is_err()); // Contains dots
        assert!(validate_event_id("event_with_underscores").is_err()); // Contains underscores
    }
} 