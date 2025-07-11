//! Common validator trait patterns
//!
//! This module defines common validator traits that are used across
//! different MCP implementations to ensure consistency.

use crate::MCPResult;

/// Base trait for all validators
pub trait BaseValidator<T> {
    /// Validate an item and return result
    fn validate(&self, item: &T) -> MCPResult<()>;

    /// Get validator name for debugging/logging
    fn name(&self) -> &'static str;

    /// Check if validator is enabled
    fn is_enabled(&self) -> bool {
        true
    }
}

/// Trait for validators that can provide validation context
pub trait ContextualValidator<T, Context>: BaseValidator<T> {
    /// Validate an item with additional context
    fn validate_with_context(&self, item: &T, context: &Context) -> MCPResult<()>;
}

/// Trait for validators that can provide suggestions for fixing validation errors
pub trait SuggestiveValidator<T>: BaseValidator<T> {
    /// Suggest fixes for validation errors
    fn suggest_fixes(&self, item: &T) -> Vec<String>;
}

/// Composite validator that runs multiple validators
pub struct CompositeValidator<T> {
    validators: Vec<Box<dyn BaseValidator<T>>>,
    #[allow(dead_code)]
    name: String,
}

impl<T> CompositeValidator<T> {
    pub fn new(name: String) -> Self {
        Self {
            validators: Vec::new(),
            name,
        }
    }

    pub fn add_validator(mut self, validator: Box<dyn BaseValidator<T>>) -> Self {
        self.validators.push(validator);
        self
    }
}

impl<T> BaseValidator<T> for CompositeValidator<T> {
    fn validate(&self, item: &T) -> MCPResult<()> {
        for validator in &self.validators {
            if validator.is_enabled() {
                validator.validate(item)?;
            }
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        // This is a limitation - we can't return the dynamic name
        // In practice, you'd want to use a different approach for dynamic names
        "CompositeValidator"
    }
}

/// Macro to implement common validator patterns
#[macro_export]
macro_rules! impl_base_validator {
    ($validator_type:ty, $target_type:ty, $name:expr) => {
        impl $crate::traits::BaseValidator<$target_type> for $validator_type {
            fn validate(&self, item: &$target_type) -> $crate::MCPResult<()> {
                self.validate_impl(item)
            }

            fn name(&self) -> &'static str {
                $name
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{MCPError, ValidationError};

    struct TestValidator;

    impl TestValidator {
        fn validate_impl(&self, item: &str) -> MCPResult<()> {
            if item.is_empty() {
                Err(MCPError::Validation(ValidationError::RequiredField {
                    field: "test_string".to_string(),
                }))
            } else {
                Ok(())
            }
        }
    }

    impl_base_validator!(TestValidator, String, "test_validator");

    #[test]
    fn test_base_validator() {
        let validator = TestValidator;
        assert_eq!(validator.name(), "test_validator");
        assert!(validator.is_enabled());

        // Valid case
        assert!(validator.validate(&"hello".to_string()).is_ok());

        // Invalid case
        assert!(validator.validate(&"".to_string()).is_err());
    }

    struct LengthValidator {
        min_length: usize,
    }

    impl LengthValidator {
        fn new(min_length: usize) -> Self {
            Self { min_length }
        }

        fn validate_impl(&self, item: &str) -> MCPResult<()> {
            if item.len() < self.min_length {
                Err(MCPError::Validation(ValidationError::ValueOutOfRange {
                    field: "string_length".to_string(),
                    min: self.min_length.to_string(),
                    max: "unlimited".to_string(),
                    actual: item.len().to_string(),
                }))
            } else {
                Ok(())
            }
        }
    }

    impl_base_validator!(LengthValidator, String, "length_validator");

    #[test]
    fn test_composite_validator() {
        let composite = CompositeValidator::new("test_composite".to_string())
            .add_validator(Box::new(TestValidator))
            .add_validator(Box::new(LengthValidator::new(3)));

        assert_eq!(composite.name(), "CompositeValidator");

        // Valid case
        assert!(composite.validate(&"hello".to_string()).is_ok());

        // Invalid case - empty string
        assert!(composite.validate(&"".to_string()).is_err());

        // Invalid case - too short
        assert!(composite.validate(&"hi".to_string()).is_err());
    }
}
