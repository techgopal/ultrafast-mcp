use schemars::{JsonSchema, schema_for};
use serde_json::Value;

/// Generate JSON Schema for a type at runtime
pub fn generate_schema_for<T: JsonSchema>() -> Value {
    let schema = schema_for!(T);
    serde_json::to_value(schema).unwrap_or_else(|_| Value::Object(Default::default()))
}

/// Generate a basic JSON Schema for simple types
pub fn basic_schema(type_name: &str) -> Value {
    match type_name {
        "string" => serde_json::json!({
            "type": "string"
        }),
        "number" => serde_json::json!({
            "type": "number"
        }),
        "integer" => serde_json::json!({
            "type": "integer"
        }),
        "boolean" => serde_json::json!({
            "type": "boolean"
        }),
        "array" => serde_json::json!({
            "type": "array"
        }),
        "object" => serde_json::json!({
            "type": "object"
        }),
        _ => serde_json::json!({
            "type": "object",
            "description": format!("Schema for {}", type_name)
        }),
    }
}

/// Create a schema for a struct with properties
pub fn object_schema(properties: Vec<(&str, Value, bool)>) -> Value {
    let mut props = serde_json::Map::new();
    let mut required = Vec::new();

    for (name, schema, is_required) in properties {
        props.insert(name.to_string(), schema);
        if is_required {
            required.push(name.to_string());
        }
    }

    let mut result = serde_json::json!({
        "type": "object",
        "properties": props
    });

    if !required.is_empty() {
        result["required"] = Value::Array(required.into_iter().map(Value::String).collect());
    }

    result
}

/// Create a schema for an array of items
pub fn array_schema(item_schema: Value) -> Value {
    serde_json::json!({
        "type": "array",
        "items": item_schema
    })
}

/// Create a schema for an enum with specific values
pub fn enum_schema(values: Vec<&str>) -> Value {
    serde_json::json!({
        "type": "string",
        "enum": values
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, JsonSchema)]
    struct TestStruct {
        name: String,
        age: u32,
        optional_field: Option<String>,
    }

    #[test]
    fn test_schema_generation() {
        let schema = generate_schema_for::<TestStruct>();
        assert!(schema.is_object());

        let schema_obj = schema.as_object().unwrap();
        assert!(schema_obj.contains_key("type"));
        assert!(schema_obj.contains_key("properties"));
    }

    #[test]
    fn test_basic_schema() {
        let string_schema = basic_schema("string");
        assert_eq!(string_schema["type"], "string");

        let number_schema = basic_schema("number");
        assert_eq!(number_schema["type"], "number");
    }

    #[test]
    fn test_object_schema() {
        let schema = object_schema(vec![
            ("name", basic_schema("string"), true),
            ("age", basic_schema("integer"), false),
        ]);

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].is_object());
        assert_eq!(schema["required"], serde_json::json!(["name"]));
    }
}
