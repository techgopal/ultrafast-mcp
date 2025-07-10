//! Common test assertion helpers

use std::fmt::Debug;
use ultrafast_mcp_core::MCPResult;

/// Assert that a result is an MCP error of a specific type
pub fn assert_mcp_error<T: Debug>(result: MCPResult<T>, expected_error_contains: &str) {
    match result {
        Ok(value) => panic!(
            "Expected error containing '{}', but got Ok({:?})",
            expected_error_contains, value
        ),
        Err(error) => {
            let error_string = error.to_string();
            assert!(
                error_string.contains(expected_error_contains),
                "Expected error to contain '{}', but got: {}",
                expected_error_contains,
                error_string
            );
        }
    }
}

/// Assert that a result is an MCP error of a specific variant
pub fn assert_mcp_error_variant<T: Debug>(result: MCPResult<T>, variant_name: &str) {
    match result {
        Ok(value) => panic!("Expected {} error, but got Ok({:?})", variant_name, value),
        Err(error) => {
            let error_debug = format!("{:?}", error);
            assert!(
                error_debug.contains(variant_name),
                "Expected {} error, but got: {:?}",
                variant_name,
                error
            );
        }
    }
}

/// Assert that a result is successful
pub fn assert_mcp_success<T: Debug>(result: MCPResult<T>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("Expected success, but got error: {}", error),
    }
}

/// Assert that two JSON values are equal with better error messages
pub fn assert_json_eq(left: &serde_json::Value, right: &serde_json::Value) {
    if left != right {
        panic!(
            "JSON values are not equal:\nLeft:  {}\nRight: {}",
            serde_json::to_string_pretty(left).unwrap_or_else(|_| format!("{:?}", left)),
            serde_json::to_string_pretty(right).unwrap_or_else(|_| format!("{:?}", right))
        );
    }
}

/// Assert that a string contains expected substring
pub fn assert_contains(haystack: &str, needle: &str) {
    assert!(
        haystack.contains(needle),
        "Expected '{}' to contain '{}', but it doesn't",
        haystack,
        needle
    );
}

/// Assert that a string does not contain a substring
pub fn assert_not_contains(haystack: &str, needle: &str) {
    assert!(
        !haystack.contains(needle),
        "Expected '{}' to not contain '{}', but it does",
        haystack,
        needle
    );
}

/// Assert that a value is within a specific range
pub fn assert_in_range<T>(value: T, min: T, max: T)
where
    T: PartialOrd + Debug,
{
    assert!(
        value >= min && value <= max,
        "Expected {:?} to be in range [{:?}, {:?}]",
        value,
        min,
        max
    );
}

/// Assert that two durations are approximately equal (within tolerance)
pub fn assert_duration_approx_eq(
    left: std::time::Duration,
    right: std::time::Duration,
    tolerance: std::time::Duration,
) {
    let diff = if left > right {
        left - right
    } else {
        right - left
    };
    assert!(
        diff <= tolerance,
        "Durations {:?} and {:?} differ by {:?}, which exceeds tolerance {:?}",
        left,
        right,
        diff,
        tolerance
    );
}

/// Assert that a collection has expected length
pub fn assert_len<T>(collection: &[T], expected_len: usize) {
    assert_eq!(
        collection.len(),
        expected_len,
        "Expected collection to have length {}, but got {}",
        expected_len,
        collection.len()
    );
}

/// Assert that a collection is empty
pub fn assert_empty<T>(collection: &[T]) {
    assert!(
        collection.is_empty(),
        "Expected collection to be empty, but it has {} items",
        collection.len()
    );
}

/// Assert that a collection is not empty
pub fn assert_not_empty<T>(collection: &[T]) {
    assert!(
        !collection.is_empty(),
        "Expected collection to not be empty, but it is"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ultrafast_mcp_core::{error::ProtocolError, MCPError};

    #[test]
    fn test_assert_mcp_error() {
        let result: MCPResult<()> = Err(MCPError::Protocol(ProtocolError::InvalidRequest(
            "test".to_string(),
        )));
        assert_mcp_error(result, "Invalid request");
    }

    #[test]
    #[should_panic(expected = "Expected error containing")]
    fn test_assert_mcp_error_panic_on_success() {
        let result: MCPResult<()> = Ok(());
        assert_mcp_error(result, "some error");
    }

    #[test]
    fn test_assert_mcp_success() {
        let result: MCPResult<i32> = Ok(42);
        let value = assert_mcp_success(result);
        assert_eq!(value, 42);
    }

    #[test]
    #[should_panic(expected = "Expected success")]
    fn test_assert_mcp_success_panic_on_error() {
        let result: MCPResult<()> = Err(MCPError::Protocol(ProtocolError::InvalidRequest(
            "test".to_string(),
        )));
        assert_mcp_success(result);
    }

    #[test]
    fn test_assert_json_eq() {
        let left = serde_json::json!({"key": "value"});
        let right = serde_json::json!({"key": "value"});
        assert_json_eq(&left, &right);
    }

    #[test]
    fn test_assert_contains() {
        assert_contains("hello world", "world");
    }

    #[test]
    fn test_assert_not_contains() {
        assert_not_contains("hello world", "foo");
    }

    #[test]
    fn test_assert_in_range() {
        assert_in_range(5, 1, 10);
        assert_in_range(1, 1, 10);
        assert_in_range(10, 1, 10);
    }

    #[test]
    fn test_assert_duration_approx_eq() {
        let dur1 = std::time::Duration::from_millis(1000);
        let dur2 = std::time::Duration::from_millis(1005);
        let tolerance = std::time::Duration::from_millis(10);
        assert_duration_approx_eq(dur1, dur2, tolerance);
    }

    #[test]
    fn test_assert_len() {
        let vec = vec![1, 2, 3];
        assert_len(&vec, 3);
    }

    #[test]
    fn test_assert_empty() {
        let vec: Vec<i32> = vec![];
        assert_empty(&vec);
    }

    #[test]
    fn test_assert_not_empty() {
        let vec = vec![1];
        assert_not_empty(&vec);
    }
}
