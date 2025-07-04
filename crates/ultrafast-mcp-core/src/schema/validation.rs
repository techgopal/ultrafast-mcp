use crate::error::{MCPResult, ToolError};
use serde_json::Value;

/// Validate data against a JSON Schema
pub fn validate_against_schema(data: &Value, schema: &Value) -> MCPResult<()> {
    // For Phase 1, we'll implement basic validation
    // In future phases, we can integrate a full JSON Schema validator

    if let Some(schema_type) = schema.get("type").and_then(|t| t.as_str()) {
        match schema_type {
            "string" => {
                if !data.is_string() {
                    return Err(ToolError::SchemaValidation(format!(
                        "Expected string, got {}",
                        type_name(data)
                    ))
                    .into());
                }
            }
            "number" => {
                if !data.is_number() {
                    return Err(ToolError::SchemaValidation(format!(
                        "Expected number, got {}",
                        type_name(data)
                    ))
                    .into());
                }
            }
            "integer" => {
                if !data.is_i64() && !data.is_u64() {
                    return Err(ToolError::SchemaValidation(format!(
                        "Expected integer, got {}",
                        type_name(data)
                    ))
                    .into());
                }
            }
            "boolean" => {
                if !data.is_boolean() {
                    return Err(ToolError::SchemaValidation(format!(
                        "Expected boolean, got {}",
                        type_name(data)
                    ))
                    .into());
                }
            }
            "array" => {
                if !data.is_array() {
                    return Err(ToolError::SchemaValidation(format!(
                        "Expected array, got {}",
                        type_name(data)
                    ))
                    .into());
                }

                // Validate array items if schema is provided
                if let (Some(items_schema), Some(array)) = (schema.get("items"), data.as_array()) {
                    for (i, item) in array.iter().enumerate() {
                        validate_against_schema(item, items_schema).map_err(|e| {
                            ToolError::SchemaValidation(format!(
                                "Array item {i} validation failed: {e}"
                            ))
                        })?;
                    }
                }
            }
            "object" => {
                if !data.is_object() {
                    return Err(ToolError::SchemaValidation(format!(
                        "Expected object, got {}",
                        type_name(data)
                    ))
                    .into());
                }

                // Validate required properties
                if let (Some(required), Some(obj)) = (schema.get("required"), data.as_object()) {
                    if let Some(required_array) = required.as_array() {
                        for req in required_array {
                            if let Some(prop_name) = req.as_str() {
                                if !obj.contains_key(prop_name) {
                                    return Err(ToolError::SchemaValidation(format!(
                                        "Missing required property: {prop_name}"
                                    ))
                                    .into());
                                }
                            }
                        }
                    }
                }

                // Validate properties if schema is provided
                if let (Some(properties_schema), Some(obj)) =
                    (schema.get("properties"), data.as_object())
                {
                    if let Some(props) = properties_schema.as_object() {
                        for (key, value) in obj {
                            if let Some(prop_schema) = props.get(key) {
                                validate_against_schema(value, prop_schema).map_err(|e| {
                                    ToolError::SchemaValidation(format!(
                                        "Property '{key}' validation failed: {e}"
                                    ))
                                })?;
                            }
                        }
                    }
                }
            }
            _ => {
                // Unknown type, skip validation
            }
        }
    }

    // Validate enum values
    if let Some(enum_values) = schema.get("enum") {
        if let Some(enum_array) = enum_values.as_array() {
            if !enum_array.contains(data) {
                return Err(ToolError::SchemaValidation(format!(
                    "Value must be one of: {enum_array:?}"
                ))
                .into());
            }
        }
    }

    Ok(())
}

/// Get the type name of a JSON value for error messages
fn type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Validate that data conforms to expected input schema for a tool
pub fn validate_tool_input(data: &Value, schema: &Value) -> MCPResult<()> {
    validate_against_schema(data, schema)
}

/// Validate that data conforms to expected output schema for a tool
pub fn validate_tool_output(data: &Value, schema: &Value) -> MCPResult<()> {
    validate_against_schema(data, schema)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_string_validation() {
        let schema = json!({"type": "string"});
        let valid_data = json!("hello");
        let invalid_data = json!(42);

        assert!(validate_against_schema(&valid_data, &schema).is_ok());
        assert!(validate_against_schema(&invalid_data, &schema).is_err());
    }

    #[test]
    fn test_number_validation() {
        let schema = json!({"type": "number"});
        let valid_data = json!(42.5);
        let invalid_data = json!("hello");

        assert!(validate_against_schema(&valid_data, &schema).is_ok());
        assert!(validate_against_schema(&invalid_data, &schema).is_err());
    }

    #[test]
    fn test_object_validation() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "integer"}
            },
            "required": ["name"]
        });

        let valid_data = json!({"name": "John", "age": 30});
        let invalid_data = json!({"age": 30}); // missing required "name"

        assert!(validate_against_schema(&valid_data, &schema).is_ok());
        assert!(validate_against_schema(&invalid_data, &schema).is_err());
    }

    #[test]
    fn test_array_validation() {
        let schema = json!({
            "type": "array",
            "items": {"type": "string"}
        });

        let valid_data = json!(["hello", "world"]);
        let invalid_data = json!(["hello", 42]); // mixed types

        assert!(validate_against_schema(&valid_data, &schema).is_ok());
        assert!(validate_against_schema(&invalid_data, &schema).is_err());
    }

    #[test]
    fn test_enum_validation() {
        let schema = json!({
            "type": "string",
            "enum": ["red", "green", "blue"]
        });

        let valid_data = json!("red");
        let invalid_data = json!("yellow");

        assert!(validate_against_schema(&valid_data, &schema).is_ok());
        assert!(validate_against_schema(&invalid_data, &schema).is_err());
    }
}
