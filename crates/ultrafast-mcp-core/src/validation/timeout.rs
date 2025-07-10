//! Timeout validation utilities

use std::time::Duration;
use crate::error::{MCPResult, ValidationError};

/// Validate timeout duration
/// 
/// Consolidates implementation from core/config.rs and server validation
pub fn validate_timeout(timeout: Duration) -> MCPResult<()> {
    const MIN_TIMEOUT: Duration = Duration::from_millis(100);
    const MAX_TIMEOUT: Duration = Duration::from_secs(3600); // 1 hour
    
    if timeout < MIN_TIMEOUT {
        return Err(ValidationError::ValueOutOfRange {
            field: "timeout".to_string(),
            min: format!("{}ms", MIN_TIMEOUT.as_millis()),
            max: format!("{}s", MAX_TIMEOUT.as_secs()),
            actual: format!("{}ms", timeout.as_millis()),
        }.into());
    }
    
    if timeout > MAX_TIMEOUT {
        return Err(ValidationError::ValueOutOfRange {
            field: "timeout".to_string(),
            min: format!("{}ms", MIN_TIMEOUT.as_millis()),
            max: format!("{}s", MAX_TIMEOUT.as_secs()),
            actual: format!("{}s", timeout.as_secs()),
        }.into());
    }
    
    Ok(())
}

/// Check if a timeout duration is valid (returns bool for quick checks)
pub fn is_valid_timeout(timeout: Duration) -> bool {
    validate_timeout(timeout).is_ok()
}

/// Validate retry count
pub fn validate_retry_count(retries: u32) -> MCPResult<()> {
    const MAX_RETRIES: u32 = 10;
    
    if retries > MAX_RETRIES {
        return Err(ValidationError::ValueOutOfRange {
            field: "retry_count".to_string(),
            min: "0".to_string(),
            max: MAX_RETRIES.to_string(),
            actual: retries.to_string(),
        }.into());
    }
    
    Ok(())
}

/// Validate backoff multiplier
pub fn validate_backoff_multiplier(multiplier: f64) -> MCPResult<()> {
    const MIN_MULTIPLIER: f64 = 1.0;
    const MAX_MULTIPLIER: f64 = 10.0;
    
    if multiplier < MIN_MULTIPLIER || multiplier > MAX_MULTIPLIER {
        return Err(ValidationError::ValueOutOfRange {
            field: "backoff_multiplier".to_string(),
            min: MIN_MULTIPLIER.to_string(),
            max: MAX_MULTIPLIER.to_string(),
            actual: multiplier.to_string(),
        }.into());
    }
    
    if !multiplier.is_finite() {
        return Err(ValidationError::InvalidFormat {
            field: "backoff_multiplier".to_string(),
            expected: "finite number".to_string(),
        }.into());
    }
    
    Ok(())
}

/// Get recommended timeout for operation type
pub fn get_recommended_timeout(operation: &str) -> Duration {
    match operation {
        "ping" => Duration::from_secs(5),
        "initialize" => Duration::from_secs(30),
        "shutdown" => Duration::from_secs(10),
        "tools/list" => Duration::from_secs(10),
        "tools/call" => Duration::from_secs(300), // 5 minutes for tool execution
        "resources/list" => Duration::from_secs(30),
        "resources/read" => Duration::from_secs(60),
        "prompts/list" => Duration::from_secs(10),
        "prompts/get" => Duration::from_secs(30),
        "sampling/createMessage" => Duration::from_secs(120), // 2 minutes for LLM calls
        "completion/complete" => Duration::from_secs(30),
        _ => Duration::from_secs(30), // Default timeout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_timeout() {
        // Valid timeouts
        assert!(validate_timeout(Duration::from_secs(1)).is_ok());
        assert!(validate_timeout(Duration::from_secs(30)).is_ok());
        assert!(validate_timeout(Duration::from_secs(300)).is_ok());
        
        // Invalid timeouts
        assert!(validate_timeout(Duration::from_millis(50)).is_err()); // Too short
        assert!(validate_timeout(Duration::from_secs(3601)).is_err()); // Too long
    }

    #[test]
    fn test_is_valid_timeout() {
        assert!(is_valid_timeout(Duration::from_secs(30)));
        assert!(!is_valid_timeout(Duration::from_millis(50)));
        assert!(!is_valid_timeout(Duration::from_secs(3601)));
    }

    #[test]
    fn test_validate_retry_count() {
        // Valid retry counts
        assert!(validate_retry_count(0).is_ok());
        assert!(validate_retry_count(3).is_ok());
        assert!(validate_retry_count(10).is_ok());
        
        // Invalid retry counts
        assert!(validate_retry_count(11).is_err());
        assert!(validate_retry_count(100).is_err());
    }

    #[test]
    fn test_validate_backoff_multiplier() {
        // Valid multipliers
        assert!(validate_backoff_multiplier(1.0).is_ok());
        assert!(validate_backoff_multiplier(2.0).is_ok());
        assert!(validate_backoff_multiplier(5.5).is_ok());
        assert!(validate_backoff_multiplier(10.0).is_ok());
        
        // Invalid multipliers
        assert!(validate_backoff_multiplier(0.5).is_err()); // Too small
        assert!(validate_backoff_multiplier(11.0).is_err()); // Too large
        assert!(validate_backoff_multiplier(f64::INFINITY).is_err()); // Not finite
        assert!(validate_backoff_multiplier(f64::NAN).is_err()); // Not finite
    }

    #[test]
    fn test_get_recommended_timeout() {
        assert_eq!(get_recommended_timeout("ping"), Duration::from_secs(5));
        assert_eq!(get_recommended_timeout("initialize"), Duration::from_secs(30));
        assert_eq!(get_recommended_timeout("tools/call"), Duration::from_secs(300));
        assert_eq!(get_recommended_timeout("unknown"), Duration::from_secs(30));
    }
} 