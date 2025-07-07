use crate::error::{MCPResult, ToolError};
use serde_json::{json, Value};
use std::collections::HashSet;

/// Enhanced schema validation with comprehensive JSON Schema support
pub fn validate_against_schema(data: &Value, schema: &Value) -> MCPResult<()> {
    // Handle allOf first: validate all subschemas, then parent schema (without allOf)
    if let Some(all_of) = schema.get("allOf") {
        if let Some(schemas) = all_of.as_array() {
            for schema_item in schemas {
                if let Err(e) = validate_against_schema(data, schema_item) {
                    return Err(ToolError::SchemaValidation(format!(
                        "allOf validation failed: {e}"
                    ))
                    .into());
                }
            }
        }
        // Validate parent schema with allOf removed
        if let Some(mut parent) = schema.as_object().cloned() {
            parent.remove("allOf");
            let parent_schema = Value::Object(parent);
            return validate_against_schema(data, &parent_schema);
        }
    }
    // Handle anyOf/oneOf: validate and return immediately
    if schema.get("anyOf").is_some() || schema.get("oneOf").is_some() {
        validate_combined_schemas(data, schema)?;
        return Ok(());
    }

    // Handle null values
    if data.is_null() {
        if let Some(nullable) = schema.get("nullable") {
            if nullable.as_bool().unwrap_or(false) {
                return Ok(());
            }
        }
        if let Some(schema_type) = schema.get("type").and_then(|t| t.as_str()) {
            if schema_type == "null" {
                return Ok(());
            }
        }
        return Err(ToolError::SchemaValidation("Value cannot be null".to_string()).into());
    }

    // Handle type validation
    if let Some(schema_type) = schema.get("type").and_then(|t| t.as_str()) {
        match schema_type {
            "string" => validate_string(data, schema)?,
            "number" => validate_number(data, schema)?,
            "integer" => validate_integer(data, schema)?,
            "boolean" => validate_boolean(data, schema)?,
            "array" => validate_array(data, schema)?,
            "object" => validate_object(data, schema)?,
            "null" => {
                if !data.is_null() {
                    return Err(ToolError::SchemaValidation(format!(
                        "Expected null, got {}",
                        type_name(data)
                    ))
                    .into());
                }
            }
            _ => {
                // Unknown type, skip validation
            }
        }
    }

    // Type-specific constraints (enforced even if type is not present)
    // String constraints
    if data.is_string() {
        let string_value = data.as_str().unwrap();
        if let Some(min_length) = schema.get("minLength").and_then(|v| v.as_u64()) {
            if string_value.len() < min_length as usize {
                return Err(ToolError::SchemaValidation(format!(
                    "String length {} is less than minimum {}",
                    string_value.len(),
                    min_length
                ))
                .into());
            }
        }
        if let Some(max_length) = schema.get("maxLength").and_then(|v| v.as_u64()) {
            if string_value.len() > max_length as usize {
                return Err(ToolError::SchemaValidation(format!(
                    "String length {} is greater than maximum {}",
                    string_value.len(),
                    max_length
                ))
                .into());
            }
        }
        if let Some(pattern) = schema.get("pattern").and_then(|v| v.as_str()) {
            if let Ok(regex) = regex::Regex::new(pattern) {
                if !regex.is_match(string_value) {
                    return Err(ToolError::SchemaValidation(format!(
                        "String '{}' does not match pattern '{}'",
                        string_value, pattern
                    ))
                    .into());
                }
            }
        }
        if let Some(format) = schema.get("format").and_then(|v| v.as_str()) {
            validate_string_format(string_value, format)?;
        }
    }
    // Number constraints
    if data.is_number() {
        let number_value = data.as_f64().unwrap();
        if let Some(minimum) = schema.get("minimum").and_then(|v| v.as_f64()) {
            let exclusive = schema
                .get("exclusiveMinimum")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if exclusive {
                if number_value <= minimum {
                    return Err(ToolError::SchemaValidation(format!(
                        "Number {} must be greater than {}",
                        number_value, minimum
                    ))
                    .into());
                }
            } else if number_value < minimum {
                return Err(ToolError::SchemaValidation(format!(
                    "Number {} must be greater than or equal to {}",
                    number_value, minimum
                ))
                .into());
            }
        }
        if let Some(maximum) = schema.get("maximum").and_then(|v| v.as_f64()) {
            let exclusive = schema
                .get("exclusiveMaximum")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if exclusive {
                if number_value >= maximum {
                    return Err(ToolError::SchemaValidation(format!(
                        "Number {} must be less than {}",
                        number_value, maximum
                    ))
                    .into());
                }
            } else if number_value > maximum {
                return Err(ToolError::SchemaValidation(format!(
                    "Number {} must be less than or equal to {}",
                    number_value, maximum
                ))
                .into());
            }
        }
        if let Some(multiple_of) = schema.get("multipleOf").and_then(|v| v.as_f64()) {
            if multiple_of != 0.0 && (number_value % multiple_of).abs() > f64::EPSILON {
                return Err(ToolError::SchemaValidation(format!(
                    "Number {} must be a multiple of {}",
                    number_value, multiple_of
                ))
                .into());
            }
        }
    }
    // Integer constraints
    if data.is_i64() || data.is_u64() {
        let integer_value = data
            .as_i64()
            .unwrap_or_else(|| data.as_u64().unwrap() as i64);
        if let Some(minimum) = schema.get("minimum").and_then(|v| v.as_i64()) {
            let exclusive = schema
                .get("exclusiveMinimum")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if exclusive {
                if integer_value <= minimum {
                    return Err(ToolError::SchemaValidation(format!(
                        "Integer {} must be greater than {}",
                        integer_value, minimum
                    ))
                    .into());
                }
            } else if integer_value < minimum {
                return Err(ToolError::SchemaValidation(format!(
                    "Integer {} must be greater than or equal to {}",
                    integer_value, minimum
                ))
                .into());
            }
        }
        if let Some(maximum) = schema.get("maximum").and_then(|v| v.as_i64()) {
            let exclusive = schema
                .get("exclusiveMaximum")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if exclusive {
                if integer_value >= maximum {
                    return Err(ToolError::SchemaValidation(format!(
                        "Integer {} must be less than {}",
                        integer_value, maximum
                    ))
                    .into());
                }
            } else if integer_value > maximum {
                return Err(ToolError::SchemaValidation(format!(
                    "Integer {} must be less than or equal to {}",
                    integer_value, maximum
                ))
                .into());
            }
        }
        if let Some(multiple_of) = schema.get("multipleOf").and_then(|v| v.as_i64()) {
            if multiple_of != 0 && integer_value % multiple_of != 0 {
                return Err(ToolError::SchemaValidation(format!(
                    "Integer {} must be a multiple of {}",
                    integer_value, multiple_of
                ))
                .into());
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

    // Validate const value
    if let Some(const_value) = schema.get("const") {
        if data != const_value {
            return Err(ToolError::SchemaValidation(format!(
                "Value must be exactly: {const_value:?}"
            ))
            .into());
        }
    }

    Ok(())
}

fn validate_string(data: &Value, schema: &Value) -> MCPResult<()> {
    if !data.is_string() {
        return Err(ToolError::SchemaValidation(format!(
            "Expected string, got {}",
            type_name(data)
        ))
        .into());
    }

    let string_value = data.as_str().unwrap();

    // Validate min/max length
    if let Some(min_length) = schema.get("minLength").and_then(|v| v.as_u64()) {
        if string_value.len() < min_length as usize {
            return Err(ToolError::SchemaValidation(format!(
                "String length {} is less than minimum {}",
                string_value.len(),
                min_length
            ))
            .into());
        }
    }

    if let Some(max_length) = schema.get("maxLength").and_then(|v| v.as_u64()) {
        if string_value.len() > max_length as usize {
            return Err(ToolError::SchemaValidation(format!(
                "String length {} is greater than maximum {}",
                string_value.len(),
                max_length
            ))
            .into());
        }
    }

    // Validate pattern
    if let Some(pattern) = schema.get("pattern").and_then(|v| v.as_str()) {
        if let Ok(regex) = regex::Regex::new(pattern) {
            if !regex.is_match(string_value) {
                return Err(ToolError::SchemaValidation(format!(
                    "String '{}' does not match pattern '{}'",
                    string_value, pattern
                ))
                .into());
            }
        }
    }

    // Validate format
    if let Some(format) = schema.get("format").and_then(|v| v.as_str()) {
        validate_string_format(string_value, format)?;
    }

    Ok(())
}

fn validate_number(data: &Value, schema: &Value) -> MCPResult<()> {
    if !data.is_number() {
        return Err(ToolError::SchemaValidation(format!(
            "Expected number, got {}",
            type_name(data)
        ))
        .into());
    }

    let number_value = data.as_f64().unwrap();

    // Validate minimum/maximum
    if let Some(minimum) = schema.get("minimum").and_then(|v| v.as_f64()) {
        let exclusive = schema
            .get("exclusiveMinimum")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if exclusive {
            if number_value <= minimum {
                return Err(ToolError::SchemaValidation(format!(
                    "Number {} must be greater than {}",
                    number_value, minimum
                ))
                .into());
            }
        } else if number_value < minimum {
            return Err(ToolError::SchemaValidation(format!(
                "Number {} must be greater than or equal to {}",
                number_value, minimum
            ))
            .into());
        }
    }

    if let Some(maximum) = schema.get("maximum").and_then(|v| v.as_f64()) {
        let exclusive = schema
            .get("exclusiveMaximum")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if exclusive {
            if number_value >= maximum {
                return Err(ToolError::SchemaValidation(format!(
                    "Number {} must be less than {}",
                    number_value, maximum
                ))
                .into());
            }
        } else if number_value > maximum {
            return Err(ToolError::SchemaValidation(format!(
                "Number {} must be less than or equal to {}",
                number_value, maximum
            ))
            .into());
        }
    }

    // Validate multipleOf
    if let Some(multiple_of) = schema.get("multipleOf").and_then(|v| v.as_f64()) {
        if multiple_of != 0.0 && (number_value % multiple_of).abs() > f64::EPSILON {
            return Err(ToolError::SchemaValidation(format!(
                "Number {} must be a multiple of {}",
                number_value, multiple_of
            ))
            .into());
        }
    }

    Ok(())
}

fn validate_integer(data: &Value, schema: &Value) -> MCPResult<()> {
    if !data.is_i64() && !data.is_u64() {
        return Err(ToolError::SchemaValidation(format!(
            "Expected integer, got {}",
            type_name(data)
        ))
        .into());
    }

    let integer_value = data
        .as_i64()
        .unwrap_or_else(|| data.as_u64().unwrap() as i64);

    // Validate minimum/maximum
    if let Some(minimum) = schema.get("minimum").and_then(|v| v.as_i64()) {
        let exclusive = schema
            .get("exclusiveMinimum")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if exclusive {
            if integer_value <= minimum {
                return Err(ToolError::SchemaValidation(format!(
                    "Integer {} must be greater than {}",
                    integer_value, minimum
                ))
                .into());
            }
        } else if integer_value < minimum {
            return Err(ToolError::SchemaValidation(format!(
                "Integer {} must be greater than or equal to {}",
                integer_value, minimum
            ))
            .into());
        }
    }

    if let Some(maximum) = schema.get("maximum").and_then(|v| v.as_i64()) {
        let exclusive = schema
            .get("exclusiveMaximum")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if exclusive {
            if integer_value >= maximum {
                return Err(ToolError::SchemaValidation(format!(
                    "Integer {} must be less than {}",
                    integer_value, maximum
                ))
                .into());
            }
        } else if integer_value > maximum {
            return Err(ToolError::SchemaValidation(format!(
                "Integer {} must be less than or equal to {}",
                integer_value, maximum
            ))
            .into());
        }
    }

    // Validate multipleOf
    if let Some(multiple_of) = schema.get("multipleOf").and_then(|v| v.as_i64()) {
        if multiple_of != 0 && integer_value % multiple_of != 0 {
            return Err(ToolError::SchemaValidation(format!(
                "Integer {} must be a multiple of {}",
                integer_value, multiple_of
            ))
            .into());
        }
    }

    Ok(())
}

fn validate_boolean(data: &Value, _schema: &Value) -> MCPResult<()> {
    if !data.is_boolean() {
        return Err(ToolError::SchemaValidation(format!(
            "Expected boolean, got {}",
            type_name(data)
        ))
        .into());
    }
    Ok(())
}

fn validate_array(data: &Value, schema: &Value) -> MCPResult<()> {
    if !data.is_array() {
        return Err(ToolError::SchemaValidation(format!(
            "Expected array, got {}",
            type_name(data)
        ))
        .into());
    }

    let array = data.as_array().unwrap();

    // Validate min/max items
    if let Some(min_items) = schema.get("minItems").and_then(|v| v.as_u64()) {
        if array.len() < min_items as usize {
            return Err(ToolError::SchemaValidation(format!(
                "Array has {} items, minimum required is {}",
                array.len(),
                min_items
            ))
            .into());
        }
    }

    if let Some(max_items) = schema.get("maxItems").and_then(|v| v.as_u64()) {
        if array.len() > max_items as usize {
            return Err(ToolError::SchemaValidation(format!(
                "Array has {} items, maximum allowed is {}",
                array.len(),
                max_items
            ))
            .into());
        }
    }

    // Validate unique items
    if let Some(unique_items) = schema.get("uniqueItems").and_then(|v| v.as_bool()) {
        if unique_items {
            let mut seen = HashSet::new();
            for item in array {
                if !seen.insert(item) {
                    return Err(ToolError::SchemaValidation(
                        "Array items must be unique".to_string(),
                    )
                    .into());
                }
            }
        }
    }

    // Validate array items if schema is provided
    if let Some(items_schema) = schema.get("items") {
        for (i, item) in array.iter().enumerate() {
            validate_against_schema(item, items_schema).map_err(|e| {
                ToolError::SchemaValidation(format!("Array item {i} validation failed: {e}"))
            })?;
        }
    }

    // Validate additional items
    if let Some(additional_items) = schema.get("additionalItems") {
        if let Some(items_schema) = schema.get("items") {
            if let Some(items_array) = items_schema.as_array() {
                let max_defined_items = items_array.len();
                if array.len() > max_defined_items {
                    if let Some(additional_schema) = additional_items.as_object() {
                        if additional_schema.is_empty() {
                            return Err(ToolError::SchemaValidation(
                                "Additional items not allowed".to_string(),
                            )
                            .into());
                        }
                    } else if !additional_items.as_bool().unwrap_or(true) {
                        return Err(ToolError::SchemaValidation(
                            "Additional items not allowed".to_string(),
                        )
                        .into());
                    } else {
                        for item in &array[max_defined_items..] {
                            validate_against_schema(item, additional_items)?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn validate_object(data: &Value, schema: &Value) -> MCPResult<()> {
    if !data.is_object() {
        return Err(ToolError::SchemaValidation(format!(
            "Expected object, got {}",
            type_name(data)
        ))
        .into());
    }

    let obj = data.as_object().unwrap();

    // Validate min/max properties
    if let Some(min_properties) = schema.get("minProperties").and_then(|v| v.as_u64()) {
        if obj.len() < min_properties as usize {
            return Err(ToolError::SchemaValidation(format!(
                "Object has {} properties, minimum required is {}",
                obj.len(),
                min_properties
            ))
            .into());
        }
    }

    if let Some(max_properties) = schema.get("maxProperties").and_then(|v| v.as_u64()) {
        if obj.len() > max_properties as usize {
            return Err(ToolError::SchemaValidation(format!(
                "Object has {} properties, maximum allowed is {}",
                obj.len(),
                max_properties
            ))
            .into());
        }
    }

    // Validate required properties
    if let Some(required) = schema.get("required") {
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
    if let Some(properties_schema) = schema.get("properties") {
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

    // Validate additional properties
    if let Some(additional_properties) = schema.get("additionalProperties") {
        if let Some(properties_schema) = schema.get("properties") {
            if let Some(props) = properties_schema.as_object() {
                for (key, value) in obj {
                    if !props.contains_key(key) {
                        if let Some(additional_schema) = additional_properties.as_object() {
                            if additional_schema.is_empty() {
                                return Err(ToolError::SchemaValidation(format!(
                                    "Additional property '{}' not allowed",
                                    key
                                ))
                                .into());
                            }
                        } else if !additional_properties.as_bool().unwrap_or(true) {
                            return Err(ToolError::SchemaValidation(format!(
                                "Additional property '{}' not allowed",
                                key
                            ))
                            .into());
                        } else {
                            validate_against_schema(value, additional_properties)?;
                        }
                    }
                }
            }
        }
    }

    // Validate property names
    if let Some(property_names) = schema.get("propertyNames") {
        for key in obj.keys() {
            let key_value = Value::String(key.clone());
            validate_against_schema(&key_value, property_names).map_err(|e| {
                ToolError::SchemaValidation(format!(
                    "Property name '{}' validation failed: {e}",
                    key
                ))
            })?;
        }
    }

    // Validate dependencies
    if let Some(dependencies) = schema.get("dependencies") {
        if let Some(deps) = dependencies.as_object() {
            for (property, dependency) in deps {
                if obj.contains_key(property) {
                    match dependency {
                        Value::Array(required_props) => {
                            for req_prop in required_props {
                                if let Some(prop_name) = req_prop.as_str() {
                                    if !obj.contains_key(prop_name) {
                                        return Err(ToolError::SchemaValidation(format!(
                                            "Property '{}' requires property '{}'",
                                            property, prop_name
                                        ))
                                        .into());
                                    }
                                }
                            }
                        }
                        Value::Object(schema_dep) => {
                            let schema_value = Value::Object(schema_dep.clone());
                            validate_against_schema(data, &schema_value)?;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}

fn validate_combined_schemas(data: &Value, schema: &Value) -> MCPResult<()> {
    // Validate oneOf (exactly one schema must match)
    if let Some(one_of) = schema.get("oneOf") {
        if let Some(schemas) = one_of.as_array() {
            let mut matches = 0;
            for schema_item in schemas {
                if validate_against_schema(data, schema_item).is_ok() {
                    matches += 1;
                }
            }
            if matches != 1 {
                return Err(ToolError::SchemaValidation(
                    "Value must match exactly one schema from oneOf".to_string(),
                )
                .into());
            }
        }
    }

    // Validate anyOf (at least one schema must match)
    if let Some(any_of) = schema.get("anyOf") {
        if let Some(schemas) = any_of.as_array() {
            let mut has_match = false;
            for schema_item in schemas {
                if validate_against_schema(data, schema_item).is_ok() {
                    has_match = true;
                    break;
                }
            }
            if !has_match {
                return Err(ToolError::SchemaValidation(
                    "Value must match at least one schema from anyOf".to_string(),
                )
                .into());
            }
        }
    }

    // Validate allOf (all schemas must match)
    if let Some(all_of) = schema.get("allOf") {
        if let Some(schemas) = all_of.as_array() {
            for schema_item in schemas {
                if let Err(e) = validate_against_schema(data, schema_item) {
                    return Err(ToolError::SchemaValidation(format!(
                        "allOf validation failed: {e}"
                    ))
                    .into());
                }
            }
        }
    }

    Ok(())
}

fn validate_string_format(value: &str, format: &str) -> MCPResult<()> {
    match format {
        "date-time" => {
            // Basic ISO 8601 date-time validation
            if !value.contains('T') && !value.contains(' ') {
                return Err(
                    ToolError::SchemaValidation("Invalid date-time format".to_string()).into(),
                );
            }
        }
        "date" => {
            // Basic date validation (YYYY-MM-DD)
            if value.matches(r"^\d{4}-\d{2}-\d{2}$").next().is_none() {
                return Err(ToolError::SchemaValidation("Invalid date format".to_string()).into());
            }
        }
        "time" => {
            // Basic time validation (HH:MM:SS)
            if value.matches(r"^\d{2}:\d{2}:\d{2}").next().is_none() {
                return Err(ToolError::SchemaValidation("Invalid time format".to_string()).into());
            }
        }
        "email" => {
            // Basic email validation
            if !value.contains('@') || !value.contains('.') {
                return Err(ToolError::SchemaValidation("Invalid email format".to_string()).into());
            }
        }
        "uri" => {
            // Basic URI validation
            if !value.contains("://") {
                return Err(ToolError::SchemaValidation("Invalid URI format".to_string()).into());
            }
        }
        "uuid" => {
            // Basic UUID validation
            if value
                .matches(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$")
                .next()
                .is_none()
            {
                return Err(ToolError::SchemaValidation("Invalid UUID format".to_string()).into());
            }
        }
        _ => {
            // Unknown format, skip validation
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

/// Validate a complete tool schema definition
pub fn validate_tool_schema(schema: &Value) -> MCPResult<()> {
    // Check if schema is an object
    if !schema.is_object() {
        return Err(
            ToolError::SchemaValidation("Tool schema must be an object".to_string()).into(),
        );
    }

    // Validate required top-level properties for tool schemas
    if let Some(obj) = schema.as_object() {
        // Check for required properties
        if !obj.contains_key("type") {
            return Err(ToolError::SchemaValidation(
                "Tool schema must have 'type' property".to_string(),
            )
            .into());
        }

        // Validate type
        if let Some(schema_type) = obj.get("type").and_then(|t| t.as_str()) {
            if schema_type != "object" {
                return Err(ToolError::SchemaValidation(
                    "Tool schema type must be 'object'".to_string(),
                )
                .into());
            }
        }

        // Validate properties if present
        if let Some(properties) = obj.get("properties") {
            validate_against_schema(properties, &json!({"type": "object"}))?;
        }

        // Validate required properties array if present
        if let Some(required) = obj.get("required") {
            if let Some(required_array) = required.as_array() {
                for req in required_array {
                    if !req.is_string() {
                        return Err(ToolError::SchemaValidation(
                            "Required properties must be strings".to_string(),
                        )
                        .into());
                    }
                }
            } else {
                return Err(ToolError::SchemaValidation(
                    "Required property must be an array".to_string(),
                )
                .into());
            }
        }
    }

    Ok(())
}

/// Validate tool input with detailed error context and path information
pub fn validate_tool_input_with_context(
    data: &Value,
    schema: &Value,
    tool_name: &str,
) -> MCPResult<ValidationContext> {
    let mut context = ValidationContext::new(tool_name.to_string());

    // First validate the schema itself
    validate_tool_schema(schema)?;

    // Then validate the data against the schema with context tracking
    validate_with_context(data, schema, &mut context, "".to_string())?;

    Ok(context)
}

/// Validate tool output with detailed error context and path information  
pub fn validate_tool_output_with_context(
    data: &Value,
    schema: &Value,
    tool_name: &str,
) -> MCPResult<ValidationContext> {
    let mut context = ValidationContext::new(tool_name.to_string());

    // Validate output schema if present
    if schema.as_object().is_some() {
        validate_tool_schema(schema)?;
    }

    // Validate the output data
    validate_with_context(data, schema, &mut context, "".to_string())?;

    Ok(context)
}

/// Comprehensive tool definition validation with security checks
pub fn validate_tool_definition_comprehensive(
    tool: &crate::types::tools::Tool,
) -> MCPResult<ValidationReport> {
    let mut report = ValidationReport::new(tool.name.clone());

    // Basic validation
    if let Err(e) = tool.validate() {
        report.add_error(ValidationError::new(
            "basic_validation".to_string(),
            format!("Basic tool validation failed: {}", e),
            ErrorSeverity::High,
        ));
        return Ok(report);
    }

    // Schema complexity validation
    validate_schema_complexity(&tool.input_schema, &mut report, "input_schema")?;

    if let Some(ref output_schema) = tool.output_schema {
        validate_schema_complexity(output_schema, &mut report, "output_schema")?;
    }

    // Security validation
    validate_tool_security(tool, &mut report)?;

    // Performance validation
    validate_tool_performance(tool, &mut report)?;

    Ok(report)
}

/// Validate schema complexity to prevent DoS attacks
fn validate_schema_complexity(
    schema: &Value,
    report: &mut ValidationReport,
    context: &str,
) -> MCPResult<()> {
    let complexity = calculate_schema_complexity(schema, 0);

    // Check maximum complexity
    if complexity > MAX_SCHEMA_COMPLEXITY {
        report.add_error(ValidationError::new(
            format!("{}_complexity", context),
            format!(
                "Schema complexity {} exceeds maximum {}",
                complexity, MAX_SCHEMA_COMPLEXITY
            ),
            ErrorSeverity::High,
        ));
    } else if complexity > WARN_SCHEMA_COMPLEXITY {
        report.add_warning(ValidationWarning::new(
            format!("{}_complexity", context),
            format!(
                "Schema complexity {} is high, consider simplifying",
                complexity
            ),
        ));
    }

    // Check nesting depth
    let depth = calculate_schema_depth(schema, 0);
    if depth > MAX_SCHEMA_DEPTH {
        report.add_error(ValidationError::new(
            format!("{}_depth", context),
            format!(
                "Schema nesting depth {} exceeds maximum {}",
                depth, MAX_SCHEMA_DEPTH
            ),
            ErrorSeverity::High,
        ));
    }

    Ok(())
}

/// Calculate schema complexity score
fn calculate_schema_complexity(schema: &Value, current_depth: usize) -> usize {
    if current_depth > 20 {
        // Prevent infinite recursion
        return 1000; // High penalty for excessive depth
    }

    match schema {
        Value::Object(obj) => {
            let mut complexity = 1;

            // Add complexity for each property
            if let Some(properties) = obj.get("properties").and_then(|p| p.as_object()) {
                complexity += properties.len();
                for prop_schema in properties.values() {
                    complexity += calculate_schema_complexity(prop_schema, current_depth + 1);
                }
            }

            // Add complexity for array items
            if let Some(items) = obj.get("items") {
                complexity += calculate_schema_complexity(items, current_depth + 1);
            }

            // Add complexity for combined schemas
            for key in &["allOf", "anyOf", "oneOf"] {
                if let Some(schemas) = obj.get(*key).and_then(|s| s.as_array()) {
                    complexity += schemas.len() * 2; // Higher penalty for complex combinations
                    for sub_schema in schemas {
                        complexity += calculate_schema_complexity(sub_schema, current_depth + 1);
                    }
                }
            }

            complexity
        }
        Value::Array(arr) => arr
            .iter()
            .map(|item| calculate_schema_complexity(item, current_depth + 1))
            .sum(),
        _ => 1,
    }
}

/// Calculate schema nesting depth
fn calculate_schema_depth(schema: &Value, current_depth: usize) -> usize {
    match schema {
        Value::Object(obj) => {
            let mut max_depth = current_depth;

            // Check properties
            if let Some(properties) = obj.get("properties").and_then(|p| p.as_object()) {
                for prop_schema in properties.values() {
                    let prop_depth = calculate_schema_depth(prop_schema, current_depth + 1);
                    max_depth = max_depth.max(prop_depth);
                }
            }

            // Check array items
            if let Some(items) = obj.get("items") {
                let items_depth = calculate_schema_depth(items, current_depth + 1);
                max_depth = max_depth.max(items_depth);
            }

            // Check combined schemas
            for key in &["allOf", "anyOf", "oneOf"] {
                if let Some(schemas) = obj.get(*key).and_then(|s| s.as_array()) {
                    for sub_schema in schemas {
                        let sub_depth = calculate_schema_depth(sub_schema, current_depth + 1);
                        max_depth = max_depth.max(sub_depth);
                    }
                }
            }

            max_depth
        }
        _ => current_depth,
    }
}

/// Validate tool security considerations
fn validate_tool_security(
    tool: &crate::types::tools::Tool,
    report: &mut ValidationReport,
) -> MCPResult<()> {
    // Check for potential security risks in tool name
    if tool.name.contains("..") || tool.name.contains("/") || tool.name.contains("\\") {
        report.add_error(ValidationError::new(
            "tool_name_security".to_string(),
            "Tool name contains potentially unsafe characters".to_string(),
            ErrorSeverity::High,
        ));
    }

    // Check for sensitive parameter names
    if let Some(properties) = tool
        .input_schema
        .get("properties")
        .and_then(|p| p.as_object())
    {
        for prop_name in properties.keys() {
            if is_sensitive_parameter_name(prop_name) {
                report.add_warning(ValidationWarning::new(
                    "sensitive_parameter".to_string(),
                    format!(
                        "Parameter '{}' may contain sensitive data, ensure proper handling",
                        prop_name
                    ),
                ));
            }
        }
    }

    // Check for overly permissive schemas
    if is_overly_permissive_schema(&tool.input_schema) {
        report.add_warning(ValidationWarning::new(
            "permissive_schema".to_string(),
            "Input schema is very permissive, consider adding more constraints".to_string(),
        ));
    }

    Ok(())
}

/// Validate tool performance characteristics
fn validate_tool_performance(
    tool: &crate::types::tools::Tool,
    report: &mut ValidationReport,
) -> MCPResult<()> {
    // Check for performance anti-patterns in schema
    if has_performance_antipatterns(&tool.input_schema) {
        report.add_warning(ValidationWarning::new(
            "performance_concern".to_string(),
            "Schema contains patterns that may impact validation performance".to_string(),
        ));
    }

    // Check description length (very long descriptions may impact UI performance)
    if tool.description.len() > 1000 {
        report.add_warning(ValidationWarning::new(
            "long_description".to_string(),
            format!(
                "Tool description is {} characters, consider shortening for better UX",
                tool.description.len()
            ),
        ));
    }

    Ok(())
}

/// Check if parameter name suggests sensitive data
fn is_sensitive_parameter_name(name: &str) -> bool {
    let sensitive_patterns = [
        "password",
        "passwd",
        "secret",
        "key",
        "token",
        "credential",
        "auth",
        "api_key",
        "private",
        "confidential",
        "sensitive",
    ];

    let name_lower = name.to_lowercase();
    sensitive_patterns
        .iter()
        .any(|pattern| name_lower.contains(pattern))
}

/// Check if schema is overly permissive
fn is_overly_permissive_schema(schema: &Value) -> bool {
    // Check for schemas that accept any type without constraints
    if let Some(obj) = schema.as_object() {
        // No type specified and no constraints
        if !obj.contains_key("type")
            && !obj.contains_key("properties")
            && !obj.contains_key("enum")
            && !obj.contains_key("const")
            && !obj.contains_key("pattern")
        {
            return true;
        }

        // Object type with additionalProperties: true and no defined properties
        if obj.get("type").and_then(|t| t.as_str()) == Some("object")
            && obj.get("additionalProperties").and_then(|ap| ap.as_bool()) == Some(true)
            && !obj.contains_key("properties")
        {
            return true;
        }

        // Check for properties with empty schema objects (no constraints)
        if let Some(properties) = obj.get("properties").and_then(|p| p.as_object()) {
            for (_, prop_schema) in properties {
                if let Some(prop_obj) = prop_schema.as_object() {
                    if prop_obj.is_empty() {
                        return true; // Empty schema object is overly permissive
                    }
                }
            }
        }
    } else if schema.as_object().is_some_and(|o| o.is_empty()) {
        return true; // Empty schema object
    }

    false
}

/// Check for performance anti-patterns in schema
fn has_performance_antipatterns(schema: &Value) -> bool {
    // Very complex regex patterns
    if let Some(pattern) = schema.get("pattern").and_then(|p| p.as_str()) {
        if pattern.len() > 200 || pattern.contains(".*.*") || pattern.contains("(.+)+") {
            return true;
        }
    }

    // Very large enum lists
    if let Some(enum_values) = schema.get("enum").and_then(|e| e.as_array()) {
        if enum_values.len() > 100 {
            return true;
        }
    }

    false
}

/// Validate data against schema with detailed context tracking
fn validate_with_context(
    data: &Value,
    schema: &Value,
    context: &mut ValidationContext,
    path: String,
) -> MCPResult<()> {
    // Track validation path
    context.push_path(path.clone());

    // Perform validation with enhanced error reporting
    if let Err(e) = validate_against_schema(data, schema) {
        context.add_error(ValidationError::new(
            path.clone(),
            format!("{e}"),
            ErrorSeverity::High,
        ));
        return Err(e);
    }

    // Additional context-specific validations
    validate_data_size_limits(data, context, &path)?;
    validate_data_security(data, context, &path)?;

    context.pop_path();
    Ok(())
}

/// Validate data size limits to prevent DoS
fn validate_data_size_limits(
    data: &Value,
    context: &mut ValidationContext,
    path: &str,
) -> MCPResult<()> {
    match data {
        Value::String(s) => {
            if s.len() > MAX_STRING_LENGTH {
                context.add_warning(ValidationWarning::new(
                    path.to_string(),
                    format!(
                        "String length {} exceeds recommended maximum {}",
                        s.len(),
                        MAX_STRING_LENGTH
                    ),
                ));
            }
        }
        Value::Array(arr) => {
            if arr.len() > MAX_ARRAY_LENGTH {
                context.add_warning(ValidationWarning::new(
                    path.to_string(),
                    format!(
                        "Array length {} exceeds recommended maximum {}",
                        arr.len(),
                        MAX_ARRAY_LENGTH
                    ),
                ));
            }
        }
        Value::Object(obj) => {
            if obj.len() > MAX_OBJECT_PROPERTIES {
                context.add_warning(ValidationWarning::new(
                    path.to_string(),
                    format!(
                        "Object has {} properties, exceeds recommended maximum {}",
                        obj.len(),
                        MAX_OBJECT_PROPERTIES
                    ),
                ));
            }
        }
        _ => {}
    }
    Ok(())
}

/// Validate data for potential security issues
fn validate_data_security(
    data: &Value,
    context: &mut ValidationContext,
    path: &str,
) -> MCPResult<()> {
    if let Value::String(s) = data {
        // Check for potential script injection
        if contains_script_patterns(s) {
            context.add_warning(ValidationWarning::new(
                path.to_string(),
                "String contains potential script injection patterns".to_string(),
            ));
        }

        // Check for path traversal attempts
        if s.contains("../") || s.contains("..\\") {
            context.add_warning(ValidationWarning::new(
                path.to_string(),
                "String contains potential path traversal patterns".to_string(),
            ));
        }

        // Check for very long strings that might be malicious
        if s.len() > SECURITY_STRING_LENGTH_LIMIT {
            context.add_warning(ValidationWarning::new(
                path.to_string(),
                format!("String length {} may indicate malicious input", s.len()),
            ));
        }
    }
    Ok(())
}

/// Check if string contains script injection patterns
fn contains_script_patterns(s: &str) -> bool {
    let patterns = [
        "<script",
        "javascript:",
        "data:text/html",
        "eval(",
        "setTimeout(",
        "setInterval(",
    ];
    let s_lower = s.to_lowercase();
    patterns.iter().any(|pattern| s_lower.contains(pattern))
}

// Constants for validation limits
const MAX_SCHEMA_COMPLEXITY: usize = 1000;
const WARN_SCHEMA_COMPLEXITY: usize = 500;
const MAX_SCHEMA_DEPTH: usize = 20;
const MAX_STRING_LENGTH: usize = 100_000;
const MAX_ARRAY_LENGTH: usize = 10_000;
const MAX_OBJECT_PROPERTIES: usize = 1_000;
const SECURITY_STRING_LENGTH_LIMIT: usize = 1_000_000;

/// Validation context for detailed error reporting
#[derive(Debug, Clone)]
pub struct ValidationContext {
    pub tool_name: String,
    pub path_stack: Vec<String>,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationContext {
    pub fn new(tool_name: String) -> Self {
        Self {
            tool_name,
            path_stack: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn push_path(&mut self, path: String) {
        self.path_stack.push(path);
    }

    pub fn pop_path(&mut self) {
        self.path_stack.pop();
    }

    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

/// Validation report for comprehensive tool validation
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub tool_name: String,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub performance_metrics: PerformanceMetrics,
}

impl ValidationReport {
    pub fn new(tool_name: String) -> Self {
        Self {
            tool_name,
            errors: Vec::new(),
            warnings: Vec::new(),
            performance_metrics: PerformanceMetrics::default(),
        }
    }

    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn severity_level(&self) -> ErrorSeverity {
        self.errors
            .iter()
            .map(|e| &e.severity)
            .max()
            .cloned()
            .unwrap_or(ErrorSeverity::Low)
    }
}

/// Validation error with detailed context
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
    pub severity: ErrorSeverity,
}

impl ValidationError {
    pub fn new(path: String, message: String, severity: ErrorSeverity) -> Self {
        Self {
            path,
            message,
            severity,
        }
    }
}

/// Validation warning
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    pub path: String,
    pub message: String,
}

impl ValidationWarning {
    pub fn new(path: String, message: String) -> Self {
        Self { path, message }
    }
}

/// Error severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Performance metrics for validation
#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    pub schema_complexity: usize,
    pub validation_time_ms: Option<u64>,
    pub memory_usage_bytes: Option<usize>,
}

/// Comprehensive MCP message validation
pub struct MCPMessageValidator {
    /// Maximum allowed string length in messages
    max_string_length: usize,
    /// Maximum allowed array length
    max_array_length: usize,
    /// Maximum allowed object properties
    max_object_properties: usize,
    /// Maximum allowed message depth
    max_depth: usize,
    /// Whether to allow potentially dangerous content
    allow_dangerous_content: bool,
    /// Custom validation patterns
    blocked_patterns: Vec<regex::Regex>,
}

impl Default for MCPMessageValidator {
    fn default() -> Self {
        let blocked_patterns = vec![
            // Script injection patterns
            regex::Regex::new(r"(?i)<script[^>]*>").unwrap(),
            regex::Regex::new(r"(?i)javascript:").unwrap(),
            regex::Regex::new(r"(?i)data:text/html").unwrap(),
            regex::Regex::new(r"(?i)eval\s*\(").unwrap(),
            // SQL injection patterns
            regex::Regex::new(r"(?i)(union|select|insert|update|delete|drop|create|alter)\s+")
                .unwrap(),
            regex::Regex::new(r"(?i)(or|and)\s+\d+\s*=\s*\d+").unwrap(),
            // Command injection patterns
            regex::Regex::new(r"[;&|`$(){}\[\]\\]").unwrap(),
            // Path traversal patterns
            regex::Regex::new(r"\.\.[\\/]").unwrap(),
            // XML/XXE patterns
            regex::Regex::new(r"(?i)<!entity").unwrap(),
            regex::Regex::new(r"(?i)<!doctype").unwrap(),
        ];

        Self {
            max_string_length: 1_000_000, // 1MB
            max_array_length: 50_000,
            max_object_properties: 5_000,
            max_depth: 50,
            allow_dangerous_content: false,
            blocked_patterns,
        }
    }
}

impl MCPMessageValidator {
    /// Create a new validator with custom limits
    pub fn new(
        max_string_length: usize,
        max_array_length: usize,
        max_object_properties: usize,
        max_depth: usize,
    ) -> Self {
        Self {
            max_string_length,
            max_array_length,
            max_object_properties,
            max_depth,
            ..Default::default()
        }
    }

    /// Enable/disable dangerous content
    pub fn with_dangerous_content(mut self, allow: bool) -> Self {
        self.allow_dangerous_content = allow;
        self
    }

    /// Add custom blocked pattern
    pub fn add_blocked_pattern(&mut self, pattern: &str) -> Result<(), regex::Error> {
        let regex = regex::Regex::new(pattern)?;
        self.blocked_patterns.push(regex);
        Ok(())
    }

    /// Validate complete JSON-RPC message
    pub fn validate_message(
        &self,
        message: &crate::protocol::jsonrpc::JsonRpcMessage,
    ) -> MCPResult<ValidationReport> {
        let mut report = ValidationReport::new("message".to_string());

        match message {
            crate::protocol::jsonrpc::JsonRpcMessage::Request(req) => {
                self.validate_request(req, &mut report)?;
            }
            crate::protocol::jsonrpc::JsonRpcMessage::Response(resp) => {
                self.validate_response(resp, &mut report)?;
            }
            crate::protocol::jsonrpc::JsonRpcMessage::Notification(notif) => {
                self.validate_notification(notif, &mut report)?;
            }
        }

        Ok(report)
    }

    /// Validate JSON-RPC request
    fn validate_request(
        &self,
        request: &crate::protocol::jsonrpc::JsonRpcRequest,
        report: &mut ValidationReport,
    ) -> MCPResult<()> {
        // Validate JSON-RPC version
        if request.jsonrpc != "2.0" {
            report.add_error(ValidationError::new(
                "jsonrpc".to_string(),
                "Invalid JSON-RPC version, must be '2.0'".to_string(),
                ErrorSeverity::High,
            ));
        }

        // Validate method name
        self.validate_method_name(&request.method, report)?;

        // Validate request ID
        if let Some(ref id) = request.id {
            self.validate_request_id(id, report)?;
        }

        // Validate parameters
        if let Some(ref params) = request.params {
            self.validate_value(params, report, "params".to_string(), 0)?;
            self.validate_method_specific_params(&request.method, params, report)?;
        }

        Ok(())
    }

    /// Validate JSON-RPC response
    fn validate_response(
        &self,
        response: &crate::protocol::jsonrpc::JsonRpcResponse,
        report: &mut ValidationReport,
    ) -> MCPResult<()> {
        // Validate JSON-RPC version
        if response.jsonrpc != "2.0" {
            report.add_error(ValidationError::new(
                "jsonrpc".to_string(),
                "Invalid JSON-RPC version, must be '2.0'".to_string(),
                ErrorSeverity::High,
            ));
        }

        // Validate response ID
        if let Some(ref id) = response.id {
            self.validate_request_id(id, report)?;
        }

        // Validate that response has either result or error, but not both
        match (&response.result, &response.error) {
            (Some(_), Some(_)) => {
                report.add_error(ValidationError::new(
                    "response".to_string(),
                    "Response cannot have both result and error".to_string(),
                    ErrorSeverity::High,
                ));
            }
            (None, None) => {
                report.add_error(ValidationError::new(
                    "response".to_string(),
                    "Response must have either result or error".to_string(),
                    ErrorSeverity::High,
                ));
            }
            _ => {}
        }

        // Validate result if present
        if let Some(ref result) = response.result {
            self.validate_value(result, report, "result".to_string(), 0)?;
        }

        // Validate error if present
        if let Some(ref error) = response.error {
            self.validate_error(error, report)?;
        }

        Ok(())
    }

    /// Validate JSON-RPC notification (request without ID)
    fn validate_notification(
        &self,
        notification: &crate::protocol::jsonrpc::JsonRpcRequest,
        report: &mut ValidationReport,
    ) -> MCPResult<()> {
        // Validate JSON-RPC version
        if notification.jsonrpc != "2.0" {
            report.add_error(ValidationError::new(
                "jsonrpc".to_string(),
                "Invalid JSON-RPC version, must be '2.0'".to_string(),
                ErrorSeverity::High,
            ));
        }

        // Validate method name
        self.validate_method_name(&notification.method, report)?;

        // Validate parameters
        if let Some(ref params) = notification.params {
            self.validate_value(params, report, "params".to_string(), 0)?;
            self.validate_method_specific_params(&notification.method, params, report)?;
        }

        Ok(())
    }

    /// Validate method name format and security
    fn validate_method_name(&self, method: &str, report: &mut ValidationReport) -> MCPResult<()> {
        // Check for empty method
        if method.is_empty() {
            report.add_error(ValidationError::new(
                "method".to_string(),
                "Method name cannot be empty".to_string(),
                ErrorSeverity::High,
            ));
            return Ok(());
        }

        // Check method name length
        if method.len() > 100 {
            report.add_error(ValidationError::new(
                "method".to_string(),
                format!(
                    "Method name too long: {} characters (max 100)",
                    method.len()
                ),
                ErrorSeverity::High,
            ));
        }

        // Check for invalid characters
        if !method
            .chars()
            .all(|c| c.is_alphanumeric() || c == '/' || c == '_' || c == '-' || c == '.')
        {
            report.add_error(ValidationError::new(
                "method".to_string(),
                "Method name contains invalid characters".to_string(),
                ErrorSeverity::High,
            ));
        }

        // Check for security patterns
        if method.contains("..") || method.starts_with('/') || method.ends_with('/') {
            report.add_error(ValidationError::new(
                "method".to_string(),
                "Method name contains potentially unsafe patterns".to_string(),
                ErrorSeverity::High,
            ));
        }

        // Check for private/internal methods
        if method.starts_with('_') || method.contains("internal") || method.contains("private") {
            report.add_warning(ValidationWarning::new(
                "method".to_string(),
                "Method name suggests internal/private usage".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate request ID format and security
    fn validate_request_id(
        &self,
        id: &crate::protocol::jsonrpc::RequestId,
        report: &mut ValidationReport,
    ) -> MCPResult<()> {
        match id {
            crate::protocol::jsonrpc::RequestId::String(s) => {
                if s.is_empty() {
                    report.add_error(ValidationError::new(
                        "id".to_string(),
                        "Request ID string cannot be empty".to_string(),
                        ErrorSeverity::Medium,
                    ));
                }

                if s.len() > 200 {
                    report.add_error(ValidationError::new(
                        "id".to_string(),
                        format!(
                            "Request ID string too long: {} characters (max 200)",
                            s.len()
                        ),
                        ErrorSeverity::Medium,
                    ));
                }

                // Check for dangerous characters
                if s.contains('\0') || s.contains('\n') || s.contains('\r') || s.contains('\t') {
                    report.add_error(ValidationError::new(
                        "id".to_string(),
                        "Request ID contains control characters".to_string(),
                        ErrorSeverity::Medium,
                    ));
                }

                // Check for potential injection
                self.validate_string_security(s, report, "id".to_string())?;
            }
            crate::protocol::jsonrpc::RequestId::Number(n) => {
                // Check for reasonable numeric range
                if *n < -9_223_372_036_854_775_807 {
                    report.add_error(ValidationError::new(
                        "id".to_string(),
                        "Request ID number out of reasonable range".to_string(),
                        ErrorSeverity::Low,
                    ));
                }
            }
        }

        Ok(())
    }

    /// Validate JSON-RPC error object
    fn validate_error(
        &self,
        error: &crate::protocol::jsonrpc::JsonRpcError,
        report: &mut ValidationReport,
    ) -> MCPResult<()> {
        // Validate error code is in valid range
        if (error.code < -32999 || error.code > -32000)
            && (error.code < -32700 || error.code > -32600)
        {
            report.add_warning(ValidationWarning::new(
                "error.code".to_string(),
                format!(
                    "Error code {} is outside standard JSON-RPC ranges",
                    error.code
                ),
            ));
        }

        // Validate error message
        if error.message.is_empty() {
            report.add_error(ValidationError::new(
                "error.message".to_string(),
                "Error message cannot be empty".to_string(),
                ErrorSeverity::Medium,
            ));
        }

        if error.message.len() > 1000 {
            report.add_warning(ValidationWarning::new(
                "error.message".to_string(),
                format!(
                    "Error message very long: {} characters",
                    error.message.len()
                ),
            ));
        }

        self.validate_string_security(&error.message, report, "error.message".to_string())?;

        // Validate error data if present
        if let Some(ref data) = error.data {
            self.validate_value(data, report, "error.data".to_string(), 0)?;
        }

        Ok(())
    }

    /// Validate method-specific parameters
    fn validate_method_specific_params(
        &self,
        method: &str,
        params: &serde_json::Value,
        report: &mut ValidationReport,
    ) -> MCPResult<()> {
        match method {
            "initialize" => self.validate_initialize_params(params, report)?,
            "tools/call" => self.validate_tool_call_params(params, report)?,
            "resources/read" => self.validate_resource_read_params(params, report)?,
            "resources/subscribe" => self.validate_resource_subscribe_params(params, report)?,
            "resources/unsubscribe" => self.validate_resource_unsubscribe_params(params, report)?,
            "prompts/get" => self.validate_prompt_get_params(params, report)?,
            "sampling/createMessage" => self.validate_sampling_params(params, report)?,
            "elicitation/request" => self.validate_elicitation_params(params, report)?,
            "logging/log" => self.validate_logging_params(params, report)?,
            _ => {
                // Generic validation for unknown methods
                if let Some(obj) = params.as_object() {
                    if obj.len() > 50 {
                        report.add_warning(ValidationWarning::new(
                            "params".to_string(),
                            format!(
                                "Large parameter object for method '{}': {} properties",
                                method,
                                obj.len()
                            ),
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate initialize method parameters
    fn validate_initialize_params(
        &self,
        params: &serde_json::Value,
        report: &mut ValidationReport,
    ) -> MCPResult<()> {
        let obj = match params.as_object() {
            Some(obj) => obj,
            None => {
                report.add_error(ValidationError::new(
                    "initialize.params".to_string(),
                    "Initialize parameters must be an object".to_string(),
                    ErrorSeverity::High,
                ));
                return Ok(());
            }
        };

        // Validate protocol version
        if let Some(version) = obj.get("protocolVersion") {
            if let Some(version_str) = version.as_str() {
                if version_str != "2025-06-18" && version_str != "2025-03-26" && version_str != "2024-11-05" {
                    report.add_error(ValidationError::new(
                        "initialize.protocolVersion".to_string(),
                        format!("Unsupported protocol version: {}", version_str),
                        ErrorSeverity::High,
                    ));
                }
            } else {
                report.add_error(ValidationError::new(
                    "initialize.protocolVersion".to_string(),
                    "Protocol version must be a string".to_string(),
                    ErrorSeverity::High,
                ));
            }
        }

        // Validate client info
        if let Some(client_info) = obj.get("clientInfo") {
            self.validate_client_info(client_info, report)?;
        }

        // Validate capabilities
        if let Some(capabilities) = obj.get("capabilities") {
            self.validate_capabilities(
                capabilities,
                report,
                "initialize.capabilities".to_string(),
            )?;
        }

        Ok(())
    }

    /// Validate tool call parameters
    fn validate_tool_call_params(
        &self,
        params: &serde_json::Value,
        report: &mut ValidationReport,
    ) -> MCPResult<()> {
        let obj = match params.as_object() {
            Some(obj) => obj,
            None => {
                report.add_error(ValidationError::new(
                    "tools/call.params".to_string(),
                    "Tool call parameters must be an object".to_string(),
                    ErrorSeverity::High,
                ));
                return Ok(());
            }
        };

        // Validate tool name
        if let Some(name) = obj.get("name") {
            if let Some(name_str) = name.as_str() {
                if name_str.is_empty() {
                    report.add_error(ValidationError::new(
                        "tools/call.name".to_string(),
                        "Tool name cannot be empty".to_string(),
                        ErrorSeverity::High,
                    ));
                }

                if name_str.len() > 200 {
                    report.add_error(ValidationError::new(
                        "tools/call.name".to_string(),
                        format!(
                            "Tool name too long: {} characters (max 200)",
                            name_str.len()
                        ),
                        ErrorSeverity::High,
                    ));
                }

                // Check for dangerous tool names
                if name_str.starts_with('_') || name_str.contains("..") || name_str.contains('/') {
                    report.add_error(ValidationError::new(
                        "tools/call.name".to_string(),
                        "Tool name contains potentially unsafe characters".to_string(),
                        ErrorSeverity::High,
                    ));
                }

                self.validate_string_security(name_str, report, "tools/call.name".to_string())?;
            } else {
                report.add_error(ValidationError::new(
                    "tools/call.name".to_string(),
                    "Tool name must be a string".to_string(),
                    ErrorSeverity::High,
                ));
            }
        } else {
            report.add_error(ValidationError::new(
                "tools/call.name".to_string(),
                "Tool name is required".to_string(),
                ErrorSeverity::High,
            ));
        }

        // Validate arguments if present
        if let Some(arguments) = obj.get("arguments") {
            self.validate_value(arguments, report, "tools/call.arguments".to_string(), 0)?;
        }

        Ok(())
    }

    /// Validate resource read parameters
    fn validate_resource_read_params(
        &self,
        params: &serde_json::Value,
        report: &mut ValidationReport,
    ) -> MCPResult<()> {
        let obj = match params.as_object() {
            Some(obj) => obj,
            None => {
                report.add_error(ValidationError::new(
                    "resources/read.params".to_string(),
                    "Resource read parameters must be an object".to_string(),
                    ErrorSeverity::High,
                ));
                return Ok(());
            }
        };

        // Validate URI
        if let Some(uri) = obj.get("uri") {
            if let Some(uri_str) = uri.as_str() {
                self.validate_uri(uri_str, report)?;
            } else {
                report.add_error(ValidationError::new(
                    "resources/read.uri".to_string(),
                    "URI must be a string".to_string(),
                    ErrorSeverity::High,
                ));
            }
        } else {
            report.add_error(ValidationError::new(
                "resources/read.uri".to_string(),
                "URI is required".to_string(),
                ErrorSeverity::High,
            ));
        }

        Ok(())
    }

    /// Validate resource subscribe parameters
    fn validate_resource_subscribe_params(
        &self,
        params: &serde_json::Value,
        report: &mut ValidationReport,
    ) -> MCPResult<()> {
        let obj = match params.as_object() {
            Some(obj) => obj,
            None => {
                report.add_error(ValidationError::new(
                    "resources/subscribe.params".to_string(),
                    "Resource subscribe parameters must be an object".to_string(),
                    ErrorSeverity::High,
                ));
                return Ok(());
            }
        };

        // Validate URI
        if let Some(uri) = obj.get("uri") {
            if let Some(uri_str) = uri.as_str() {
                self.validate_uri(uri_str, report)?;
            } else {
                report.add_error(ValidationError::new(
                    "resources/subscribe.uri".to_string(),
                    "URI must be a string".to_string(),
                    ErrorSeverity::High,
                ));
            }
        } else {
            report.add_error(ValidationError::new(
                "resources/subscribe.uri".to_string(),
                "URI is required".to_string(),
                ErrorSeverity::High,
            ));
        }

        Ok(())
    }

    /// Validate resource unsubscribe parameters
    fn validate_resource_unsubscribe_params(
        &self,
        params: &serde_json::Value,
        report: &mut ValidationReport,
    ) -> MCPResult<()> {
        let obj = match params.as_object() {
            Some(obj) => obj,
            None => {
                report.add_error(ValidationError::new(
                    "resources/unsubscribe.params".to_string(),
                    "Resource unsubscribe parameters must be an object".to_string(),
                    ErrorSeverity::High,
                ));
                return Ok(());
            }
        };

        // Validate URI
        if let Some(uri) = obj.get("uri") {
            if let Some(uri_str) = uri.as_str() {
                self.validate_uri(uri_str, report)?;
            } else {
                report.add_error(ValidationError::new(
                    "resources/unsubscribe.uri".to_string(),
                    "URI must be a string".to_string(),
                    ErrorSeverity::High,
                ));
            }
        } else {
            report.add_error(ValidationError::new(
                "resources/unsubscribe.uri".to_string(),
                "URI is required".to_string(),
                ErrorSeverity::High,
            ));
        }

        Ok(())
    }

    /// Validate prompt get parameters
    fn validate_prompt_get_params(
        &self,
        params: &serde_json::Value,
        report: &mut ValidationReport,
    ) -> MCPResult<()> {
        let obj = match params.as_object() {
            Some(obj) => obj,
            None => {
                report.add_error(ValidationError::new(
                    "prompts/get.params".to_string(),
                    "Prompt get parameters must be an object".to_string(),
                    ErrorSeverity::High,
                ));
                return Ok(());
            }
        };

        // Validate prompt name
        if let Some(name) = obj.get("name") {
            if let Some(name_str) = name.as_str() {
                if name_str.is_empty() {
                    report.add_error(ValidationError::new(
                        "prompts/get.name".to_string(),
                        "Prompt name cannot be empty".to_string(),
                        ErrorSeverity::High,
                    ));
                }

                if name_str.len() > 200 {
                    report.add_error(ValidationError::new(
                        "prompts/get.name".to_string(),
                        format!(
                            "Prompt name too long: {} characters (max 200)",
                            name_str.len()
                        ),
                        ErrorSeverity::High,
                    ));
                }

                self.validate_string_security(name_str, report, "prompts/get.name".to_string())?;
            } else {
                report.add_error(ValidationError::new(
                    "prompts/get.name".to_string(),
                    "Prompt name must be a string".to_string(),
                    ErrorSeverity::High,
                ));
            }
        } else {
            report.add_error(ValidationError::new(
                "prompts/get.name".to_string(),
                "Prompt name is required".to_string(),
                ErrorSeverity::High,
            ));
        }

        // Validate arguments if present
        if let Some(arguments) = obj.get("arguments") {
            self.validate_value(arguments, report, "prompts/get.arguments".to_string(), 0)?;
        }

        Ok(())
    }

    /// Validate sampling parameters
    fn validate_sampling_params(
        &self,
        params: &serde_json::Value,
        report: &mut ValidationReport,
    ) -> MCPResult<()> {
        let obj = match params.as_object() {
            Some(obj) => obj,
            None => {
                report.add_error(ValidationError::new(
                    "sampling/createMessage.params".to_string(),
                    "Sampling parameters must be an object".to_string(),
                    ErrorSeverity::High,
                ));
                return Ok(());
            }
        };

        // Validate messages array if present
        if let Some(messages) = obj.get("messages") {
            if let Some(messages_array) = messages.as_array() {
                if messages_array.len() > 1000 {
                    report.add_error(ValidationError::new(
                        "sampling/createMessage.messages".to_string(),
                        format!("Too many messages: {} (max 1000)", messages_array.len()),
                        ErrorSeverity::High,
                    ));
                }

                for (i, message) in messages_array.iter().enumerate() {
                    self.validate_value(
                        message,
                        report,
                        format!("sampling/createMessage.messages[{}]", i),
                        0,
                    )?;
                }
            } else {
                report.add_error(ValidationError::new(
                    "sampling/createMessage.messages".to_string(),
                    "Messages must be an array".to_string(),
                    ErrorSeverity::High,
                ));
            }
        }

        // Validate model preferences if present
        if let Some(model_prefs) = obj.get("modelPreferences") {
            self.validate_value(
                model_prefs,
                report,
                "sampling/createMessage.modelPreferences".to_string(),
                0,
            )?;
        }

        Ok(())
    }

    /// Validate elicitation parameters
    fn validate_elicitation_params(
        &self,
        params: &serde_json::Value,
        report: &mut ValidationReport,
    ) -> MCPResult<()> {
        let obj = match params.as_object() {
            Some(obj) => obj,
            None => {
                report.add_error(ValidationError::new(
                    "elicitation/request.params".to_string(),
                    "Elicitation parameters must be an object".to_string(),
                    ErrorSeverity::High,
                ));
                return Ok(());
            }
        };

        // Validate prompt if present
        if let Some(prompt) = obj.get("prompt") {
            if let Some(prompt_str) = prompt.as_str() {
                if prompt_str.len() > 10000 {
                    report.add_warning(ValidationWarning::new(
                        "elicitation/request.prompt".to_string(),
                        format!("Very long prompt: {} characters", prompt_str.len()),
                    ));
                }

                self.validate_string_security(
                    prompt_str,
                    report,
                    "elicitation/request.prompt".to_string(),
                )?;
            }
        }

        Ok(())
    }

    /// Validate logging parameters
    fn validate_logging_params(
        &self,
        params: &serde_json::Value,
        report: &mut ValidationReport,
    ) -> MCPResult<()> {
        let obj = match params.as_object() {
            Some(obj) => obj,
            None => {
                report.add_error(ValidationError::new(
                    "logging/log.params".to_string(),
                    "Logging parameters must be an object".to_string(),
                    ErrorSeverity::High,
                ));
                return Ok(());
            }
        };

        // Validate log level if present
        if let Some(level) = obj.get("level") {
            if let Some(level_str) = level.as_str() {
                let valid_levels = ["error", "warning", "info", "debug"];
                if !valid_levels.contains(&level_str) {
                    report.add_error(ValidationError::new(
                        "logging/log.level".to_string(),
                        format!(
                            "Invalid log level: {} (must be one of: {:?})",
                            level_str, valid_levels
                        ),
                        ErrorSeverity::Medium,
                    ));
                }
            } else {
                report.add_error(ValidationError::new(
                    "logging/log.level".to_string(),
                    "Log level must be a string".to_string(),
                    ErrorSeverity::Medium,
                ));
            }
        }

        // Validate message if present
        if let Some(message) = obj.get("message") {
            if let Some(message_str) = message.as_str() {
                if message_str.len() > 10000 {
                    report.add_warning(ValidationWarning::new(
                        "logging/log.message".to_string(),
                        format!("Very long log message: {} characters", message_str.len()),
                    ));
                }

                self.validate_string_security(
                    message_str,
                    report,
                    "logging/log.message".to_string(),
                )?;
            }
        }

        Ok(())
    }

    /// Validate client info object
    fn validate_client_info(
        &self,
        client_info: &serde_json::Value,
        report: &mut ValidationReport,
    ) -> MCPResult<()> {
        if let Some(obj) = client_info.as_object() {
            // Validate name
            if let Some(name) = obj.get("name") {
                if let Some(name_str) = name.as_str() {
                    if name_str.is_empty() {
                        report.add_error(ValidationError::new(
                            "clientInfo.name".to_string(),
                            "Client name cannot be empty".to_string(),
                            ErrorSeverity::Medium,
                        ));
                    }

                    if name_str.len() > 200 {
                        report.add_error(ValidationError::new(
                            "clientInfo.name".to_string(),
                            format!(
                                "Client name too long: {} characters (max 200)",
                                name_str.len()
                            ),
                            ErrorSeverity::Medium,
                        ));
                    }

                    self.validate_string_security(name_str, report, "clientInfo.name".to_string())?;
                }
            }

            // Validate version
            if let Some(version) = obj.get("version") {
                if let Some(version_str) = version.as_str() {
                    if version_str.len() > 100 {
                        report.add_warning(ValidationWarning::new(
                            "clientInfo.version".to_string(),
                            format!(
                                "Client version string very long: {} characters",
                                version_str.len()
                            ),
                        ));
                    }
                }
            }
        } else {
            report.add_error(ValidationError::new(
                "clientInfo".to_string(),
                "Client info must be an object".to_string(),
                ErrorSeverity::Medium,
            ));
        }

        Ok(())
    }

    /// Validate capabilities object
    fn validate_capabilities(
        &self,
        capabilities: &serde_json::Value,
        report: &mut ValidationReport,
        path: String,
    ) -> MCPResult<()> {
        if let Some(obj) = capabilities.as_object() {
            // Check for excessively large capabilities
            if obj.len() > 100 {
                report.add_warning(ValidationWarning::new(
                    path.clone(),
                    format!("Large capabilities object: {} properties", obj.len()),
                ));
            }

            // Validate each capability
            for (key, value) in obj {
                self.validate_value(value, report, format!("{}.{}", path, key), 0)?;
            }
        } else {
            report.add_error(ValidationError::new(
                path,
                "Capabilities must be an object".to_string(),
                ErrorSeverity::Medium,
            ));
        }

        Ok(())
    }

    /// Validate URI format and security
    fn validate_uri(&self, uri: &str, report: &mut ValidationReport) -> MCPResult<()> {
        // Check URI length
        if uri.len() > 2048 {
            report.add_error(ValidationError::new(
                "uri".to_string(),
                format!("URI too long: {} characters (max 2048)", uri.len()),
                ErrorSeverity::High,
            ));
        }

        // Check for basic URI format
        if !uri.contains("://") && !uri.starts_with("file://") && !uri.starts_with("data:") {
            report.add_error(ValidationError::new(
                "uri".to_string(),
                "URI must contain a valid scheme".to_string(),
                ErrorSeverity::High,
            ));
        }

        // Check for dangerous URI schemes
        let dangerous_schemes = [
            "javascript:",
            "data:text/html",
            "vbscript:",
            "file:///proc",
            "file:///sys",
        ];
        let uri_lower = uri.to_lowercase();

        for scheme in &dangerous_schemes {
            if uri_lower.starts_with(scheme) {
                if !self.allow_dangerous_content {
                    report.add_error(ValidationError::new(
                        "uri".to_string(),
                        format!("Potentially dangerous URI scheme: {}", scheme),
                        ErrorSeverity::High,
                    ));
                } else {
                    report.add_warning(ValidationWarning::new(
                        "uri".to_string(),
                        format!("Dangerous URI scheme detected: {}", scheme),
                    ));
                }
            }
        }

        // Check for path traversal in URI
        if uri.contains("../") || uri.contains("..\\") {
            report.add_error(ValidationError::new(
                "uri".to_string(),
                "URI contains path traversal patterns".to_string(),
                ErrorSeverity::High,
            ));
        }

        // Additional security validation
        self.validate_string_security(uri, report, "uri".to_string())?;

        Ok(())
    }

    /// Validate any JSON value recursively
    fn validate_value(
        &self,
        value: &serde_json::Value,
        report: &mut ValidationReport,
        path: String,
        depth: usize,
    ) -> MCPResult<()> {
        // Check depth limit
        if depth > self.max_depth {
            report.add_error(ValidationError::new(
                path.clone(),
                format!("Value depth {} exceeds maximum {}", depth, self.max_depth),
                ErrorSeverity::High,
            ));
            return Ok(());
        }

        match value {
            serde_json::Value::String(s) => {
                self.validate_string_value(s, report, path)?;
            }
            serde_json::Value::Array(arr) => {
                self.validate_array_value(arr, report, path, depth)?;
            }
            serde_json::Value::Object(obj) => {
                self.validate_object_value(obj, report, path, depth)?;
            }
            serde_json::Value::Number(n) => {
                self.validate_number_value(n, report, path)?;
            }
            // Bool and Null are generally safe
            _ => {}
        }

        Ok(())
    }

    /// Validate string value for size and security
    fn validate_string_value(
        &self,
        s: &str,
        report: &mut ValidationReport,
        path: String,
    ) -> MCPResult<()> {
        // Check string length
        if s.len() > self.max_string_length {
            report.add_error(ValidationError::new(
                path.clone(),
                format!(
                    "String too long: {} characters (max {})",
                    s.len(),
                    self.max_string_length
                ),
                ErrorSeverity::High,
            ));
        }

        // Check for extremely large strings that might be DoS attempts
        if s.len() > 100_000 {
            report.add_warning(ValidationWarning::new(
                path.clone(),
                format!("Very large string: {} characters", s.len()),
            ));
        }

        // Security validation
        self.validate_string_security(s, report, path)?;

        Ok(())
    }

    /// Validate string for security issues
    fn validate_string_security(
        &self,
        s: &str,
        report: &mut ValidationReport,
        path: String,
    ) -> MCPResult<()> {
        // Check for null bytes
        if s.contains('\0') {
            report.add_error(ValidationError::new(
                path.clone(),
                "String contains null bytes".to_string(),
                ErrorSeverity::High,
            ));
        }

        // Check against blocked patterns
        for pattern in &self.blocked_patterns {
            if pattern.is_match(s) {
                if !self.allow_dangerous_content {
                    report.add_error(ValidationError::new(
                        path.clone(),
                        format!("String matches blocked pattern: {}", pattern.as_str()),
                        ErrorSeverity::High,
                    ));
                } else {
                    report.add_warning(ValidationWarning::new(
                        path.clone(),
                        format!(
                            "String matches potentially dangerous pattern: {}",
                            pattern.as_str()
                        ),
                    ));
                }
            }
        }

        // Check for suspicious character sequences
        let suspicious_patterns = [
            ("\r\n\r\n", "HTTP header injection"),
            ("\n\n", "Potential header injection"),
            ("%%", "URL encoding attack"),
            ("%00", "Null byte encoding"),
            ("${", "Variable substitution attack"),
            ("#{", "Expression language injection"),
        ];

        for (pattern, description) in &suspicious_patterns {
            if s.contains(pattern) {
                report.add_warning(ValidationWarning::new(
                    path.clone(),
                    format!(
                        "String contains suspicious pattern '{}': {}",
                        pattern, description
                    ),
                ));
            }
        }

        Ok(())
    }

    /// Validate array value
    fn validate_array_value(
        &self,
        arr: &[serde_json::Value],
        report: &mut ValidationReport,
        path: String,
        depth: usize,
    ) -> MCPResult<()> {
        // Check array length
        if arr.len() > self.max_array_length {
            report.add_error(ValidationError::new(
                path.clone(),
                format!(
                    "Array too large: {} elements (max {})",
                    arr.len(),
                    self.max_array_length
                ),
                ErrorSeverity::High,
            ));
        }

        // Validate each element
        for (i, item) in arr.iter().enumerate() {
            self.validate_value(item, report, format!("{}[{}]", path, i), depth + 1)?;
        }

        Ok(())
    }

    /// Validate object value
    fn validate_object_value(
        &self,
        obj: &serde_json::Map<String, serde_json::Value>,
        report: &mut ValidationReport,
        path: String,
        depth: usize,
    ) -> MCPResult<()> {
        // Check object size
        if obj.len() > self.max_object_properties {
            report.add_error(ValidationError::new(
                path.clone(),
                format!(
                    "Object too large: {} properties (max {})",
                    obj.len(),
                    self.max_object_properties
                ),
                ErrorSeverity::High,
            ));
        }

        // Validate each property
        for (key, value) in obj {
            // Validate key
            if key.is_empty() {
                report.add_error(ValidationError::new(
                    path.clone(),
                    "Object key cannot be empty".to_string(),
                    ErrorSeverity::Medium,
                ));
            }

            if key.len() > 200 {
                report.add_error(ValidationError::new(
                    path.clone(),
                    format!("Object key too long: {} characters (max 200)", key.len()),
                    ErrorSeverity::Medium,
                ));
            }

            // Check for potentially dangerous keys
            if key.starts_with('_') && key != "_meta" {
                report.add_warning(ValidationWarning::new(
                    path.clone(),
                    format!("Private key detected: {}", key),
                ));
            }

            // Validate key security
            self.validate_string_security(key, report, format!("{}.{}", path, key))?;

            // Validate value
            self.validate_value(value, report, format!("{}.{}", path, key), depth + 1)?;
        }

        Ok(())
    }

    /// Validate number value
    fn validate_number_value(
        &self,
        n: &serde_json::Number,
        report: &mut ValidationReport,
        path: String,
    ) -> MCPResult<()> {
        // Check for special float values
        if let Some(f) = n.as_f64() {
            if f.is_infinite() || f.is_nan() {
                report.add_error(ValidationError::new(
                    path,
                    "Number cannot be infinite or NaN".to_string(),
                    ErrorSeverity::Medium,
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod mcp_message_validator_tests {
    use super::*;
    use crate::protocol::jsonrpc::{
        JsonRpcError, JsonRpcMessage, JsonRpcRequest, JsonRpcResponse, RequestId,
    };
    use serde_json::json;

    fn create_test_validator() -> MCPMessageValidator {
        MCPMessageValidator::default()
    }

    #[test]
    fn test_validate_valid_request() {
        let validator = create_test_validator();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "test_tool",
                "arguments": {"input": "hello"}
            })),
            id: Some(RequestId::String("1".to_string())),
            meta: std::collections::HashMap::new(),
        };
        let message = JsonRpcMessage::Request(request);

        let result = validator.validate_message(&message);
        assert!(result.is_ok());
        let report = result.unwrap();
        assert!(
            report.is_valid(),
            "Expected no validation errors: {:?}",
            report.errors
        );
    }

    #[test]
    fn test_validate_invalid_jsonrpc_version() {
        let validator = create_test_validator();
        let request = JsonRpcRequest {
            jsonrpc: "1.0".to_string(),
            method: "test".to_string(),
            params: None,
            id: Some(RequestId::String("1".to_string())),
            meta: std::collections::HashMap::new(),
        };
        let message = JsonRpcMessage::Request(request);

        let result = validator.validate_message(&message);
        assert!(result.is_ok());
        let report = result.unwrap();
        assert!(!report.is_valid());
        assert_eq!(report.errors.len(), 1);
        assert!(report.errors[0]
            .message
            .contains("Invalid JSON-RPC version"));
    }

    #[test]
    fn test_validate_dangerous_method_name() {
        let validator = create_test_validator();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "../system/admin".to_string(),
            params: None,
            id: Some(RequestId::String("1".to_string())),
            meta: std::collections::HashMap::new(),
        };
        let message = JsonRpcMessage::Request(request);

        let result = validator.validate_message(&message);
        assert!(result.is_ok());
        let report = result.unwrap();
        assert!(!report.is_valid());
        assert!(report
            .errors
            .iter()
            .any(|e| e.message.contains("unsafe patterns")));
    }

    #[test]
    fn test_validate_script_injection_in_params() {
        let validator = create_test_validator();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "test_tool",
                "arguments": {"input": "<script>alert('xss')</script>"}
            })),
            id: Some(RequestId::String("1".to_string())),
            meta: std::collections::HashMap::new(),
        };
        let message = JsonRpcMessage::Request(request);

        let result = validator.validate_message(&message);
        assert!(result.is_ok());
        let report = result.unwrap();
        assert!(!report.is_valid());
        assert!(report
            .errors
            .iter()
            .any(|e| e.message.contains("blocked pattern")));
    }

    #[test]
    fn test_validate_tool_call_params() {
        let validator = create_test_validator();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "_private_tool",
                "arguments": {"input": "test"}
            })),
            id: Some(RequestId::String("1".to_string())),
            meta: std::collections::HashMap::new(),
        };
        let message = JsonRpcMessage::Request(request);

        let result = validator.validate_message(&message);
        assert!(result.is_ok());
        let report = result.unwrap();
        assert!(!report.is_valid());
        assert!(report
            .errors
            .iter()
            .any(|e| e.message.contains("unsafe characters")));
    }

    #[test]
    fn test_validate_initialize_params() {
        let validator = create_test_validator();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "initialize".to_string(),
            params: Some(json!({
                "protocolVersion": "1.0.0",
                "clientInfo": {"name": "test", "version": "1.0"}
            })),
            id: Some(RequestId::String("1".to_string())),
            meta: std::collections::HashMap::new(),
        };
        let message = JsonRpcMessage::Request(request);

        let result = validator.validate_message(&message);
        assert!(result.is_ok());
        let report = result.unwrap();
        assert!(!report.is_valid());
        assert!(report
            .errors
            .iter()
            .any(|e| e.message.contains("Unsupported protocol version")));
    }

    #[test]
    fn test_validate_response_with_both_result_and_error() {
        let validator = create_test_validator();
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(json!({"status": "ok"})),
            error: Some(JsonRpcError::new(-32600, "Invalid request".to_string())),
            id: Some(RequestId::String("1".to_string())),
            meta: std::collections::HashMap::new(),
        };
        let message = JsonRpcMessage::Response(response);

        let result = validator.validate_message(&message);
        assert!(result.is_ok());
        let report = result.unwrap();
        assert!(!report.is_valid());
        assert!(report
            .errors
            .iter()
            .any(|e| e.message.contains("cannot have both result and error")));
    }

    #[test]
    fn test_validate_large_array() {
        let validator = create_test_validator();
        let large_array: Vec<serde_json::Value> = (0..60000).map(|i| json!(i)).collect();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "test".to_string(),
            params: Some(json!({"items": large_array})),
            id: Some(RequestId::String("1".to_string())),
            meta: std::collections::HashMap::new(),
        };
        let message = JsonRpcMessage::Request(request);

        let result = validator.validate_message(&message);
        assert!(result.is_ok());
        let report = result.unwrap();
        assert!(!report.is_valid());
        assert!(report
            .errors
            .iter()
            .any(|e| e.message.contains("Array too large")));
    }

    #[test]
    fn test_validate_deep_nested_object() {
        let validator = create_test_validator();

        // Create deeply nested object
        let mut nested = json!("deep_value");
        for _ in 0..60 {
            nested = json!({"nested": nested});
        }

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "test".to_string(),
            params: Some(nested),
            id: Some(RequestId::String("1".to_string())),
            meta: std::collections::HashMap::new(),
        };
        let message = JsonRpcMessage::Request(request);

        let result = validator.validate_message(&message);
        assert!(result.is_ok());
        let report = result.unwrap();
        assert!(!report.is_valid());
        assert!(report
            .errors
            .iter()
            .any(|e| e.message.contains("depth") && e.message.contains("exceeds maximum")));
    }

    #[test]
    fn test_validate_uri_security() {
        let validator = create_test_validator();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "resources/read".to_string(),
            params: Some(json!({
                "uri": "file:///../../etc/passwd"
            })),
            id: Some(RequestId::String("1".to_string())),
            meta: std::collections::HashMap::new(),
        };
        let message = JsonRpcMessage::Request(request);

        let result = validator.validate_message(&message);
        assert!(result.is_ok());
        let report = result.unwrap();
        assert!(!report.is_valid());
        assert!(report
            .errors
            .iter()
            .any(|e| e.message.contains("path traversal")));
    }

    #[test]
    fn test_validate_long_strings() {
        let validator = create_test_validator();
        let long_string = "a".repeat(2_000_000); // 2MB string
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "test".to_string(),
            params: Some(json!({"data": long_string})),
            id: Some(RequestId::String("1".to_string())),
            meta: std::collections::HashMap::new(),
        };
        let message = JsonRpcMessage::Request(request);

        let result = validator.validate_message(&message);
        assert!(result.is_ok());
        let report = result.unwrap();
        assert!(!report.is_valid());
        assert!(report
            .errors
            .iter()
            .any(|e| e.message.contains("String too long")));
    }

    #[test]
    fn test_validate_custom_blocked_pattern() {
        let mut validator = create_test_validator();
        validator.add_blocked_pattern(r"SECRET_\w+").unwrap();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "test".to_string(),
            params: Some(json!({"token": "SECRET_API_KEY_12345"})),
            id: Some(RequestId::String("1".to_string())),
            meta: std::collections::HashMap::new(),
        };
        let message = JsonRpcMessage::Request(request);

        let result = validator.validate_message(&message);
        assert!(result.is_ok());
        let report = result.unwrap();
        assert!(!report.is_valid());
        assert!(report
            .errors
            .iter()
            .any(|e| e.message.contains("blocked pattern")));
    }

    #[test]
    fn test_validate_dangerous_content_allowed() {
        let validator = MCPMessageValidator::default().with_dangerous_content(true);
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "test".to_string(),
            params: Some(json!({"script": "<script>alert('test')</script>"})),
            id: Some(RequestId::String("1".to_string())),
            meta: std::collections::HashMap::new(),
        };
        let message = JsonRpcMessage::Request(request);

        let result = validator.validate_message(&message);
        assert!(result.is_ok());
        let report = result.unwrap();
        // Should have warnings but be valid
        assert!(report.is_valid());
        assert!(!report.warnings.is_empty());
    }

    #[test]
    fn test_validate_notification() {
        let validator = create_test_validator();
        let notification = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "logging/log".to_string(),
            params: Some(json!({
                "level": "info",
                "message": "Test message"
            })),
            id: None, // Notification has no ID
            meta: std::collections::HashMap::new(),
        };
        let message = JsonRpcMessage::Notification(notification);

        let result = validator.validate_message(&message);
        assert!(result.is_ok());
        let report = result.unwrap();
        assert!(
            report.is_valid(),
            "Expected valid notification: {:?}",
            report.errors
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_resource_unsubscribe_params() {
        let validator = MCPMessageValidator::new(1000, 100, 50, 10);
        let mut report = ValidationReport::new("test".to_string());

        // Test valid unsubscribe request
        let valid_params = serde_json::json!({
            "uri": "test://static/resource/0"
        });
        let result = validator.validate_resource_unsubscribe_params(&valid_params, &mut report);
        assert!(result.is_ok());
        assert!(report.errors.is_empty());

        // Test missing URI
        let mut report = ValidationReport::new("test".to_string());
        let invalid_params = serde_json::json!({});
        let result = validator.validate_resource_unsubscribe_params(&invalid_params, &mut report);
        assert!(result.is_ok());
        assert!(!report.errors.is_empty());
        assert!(report.errors.iter().any(|e| e.path == "resources/unsubscribe.uri"));

        // Test non-string URI
        let mut report = ValidationReport::new("test".to_string());
        let invalid_params = serde_json::json!({
            "uri": 123
        });
        let result = validator.validate_resource_unsubscribe_params(&invalid_params, &mut report);
        assert!(result.is_ok());
        assert!(!report.errors.is_empty());
        assert!(report.errors.iter().any(|e| e.path == "resources/unsubscribe.uri"));
    }
}
