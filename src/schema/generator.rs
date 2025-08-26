use schemars::{Schema, json_schema};
use serde_json::{Map, Value, json};

pub trait TryGet<K, V> {
    fn try_get(&self, key: &K) -> Option<V>;
}

pub struct SchemaGenerator<'store> {
    known_types: &'store dyn TryGet<Value, Schema>,
}

impl<'store> SchemaGenerator<'store> {
    pub fn generate_schema(
        &self,
        instance: &Value,
    ) -> Result<Schema, serde_json::Error> {
        match instance {
            Value::Object(_) => self.generate_object_schema(instance),
            Value::Array(arr) => self.generate_array_schema(arr),
            Value::String(_) => Ok(json_schema!({"type": "string"})),
            Value::Number(n) => {
                if n.is_i64() {
                    Ok(json_schema!({"type": "integer"}))
                } else {
                    Ok(json_schema!({"type": "number"}))
                }
            }
            Value::Bool(_) => Ok(json_schema!({"type": "boolean"})),
            Value::Null => Ok(json_schema!({"type": "null"})),
        }
    }

    fn generate_object_schema(
        &self,
        instance: &Value,
    ) -> Result<Schema, serde_json::Error> {
        if let Some(known_scheme) = self.known_types.try_get(instance) {
            return Ok(known_scheme);
        } else {
            let mut schema = json!({
                "type": "object",
                "properties": {},
                "required": []
            });

            if let Value::Object(obj) = instance {
                for (key, value) in obj {
                    if key == "$ref" {
                        schema["$ref"] = value.clone();
                    } else {
                        let sub_schema = self.generate_schema(value)?;
                        let mut sub_value = sub_schema.to_value();
                        if let Some(obj) = sub_value.as_object_mut() {
                            obj.remove("$schema"); // Remove $schema from nested objects
                        }

                        schema["properties"][key] = sub_value;
                        schema["required"]
                            .as_array_mut()
                            .unwrap()
                            .push(Value::String(key.clone()));
                    }
                }
            }

            // Sort the "required" array
            if let Some(required) = schema["required"].as_array_mut() {
                required.sort_by(|a, b| {
                    a.as_str().unwrap().cmp(b.as_str().unwrap())
                });
            }

            // Add $schema only to the top-level object
            schema["$schema"] =
                json!("http://json-schema.org/draft-07/schema#");

            Schema::try_from(schema)
        }
    }

    fn generate_array_schema(
        &self,
        arr: &[Value],
    ) -> Result<Schema, serde_json::Error> {
        if arr.is_empty() {
            return Ok(json_schema!({
                "type": "array",
                "items": {}
            }));
        }

        let mut item_schemas: Vec<Schema> = Vec::new();

        for item in arr {
            item_schemas.push(self.generate_schema(item)?)
        }

        let common_schema = self.find_common_schema(&item_schemas)?;

        Ok(json_schema!({
            "type": "array",
            "items": common_schema
        }))
    }

    fn find_common_schema(
        &self,
        schemas: &[Schema],
    ) -> Result<Schema, serde_json::Error> {
        if schemas.is_empty() {
            Ok(json_schema!({}))
        } else {
            let mut common = schemas[0].clone();
            for schema in schemas.iter().skip(1) {
                common = self.merge_schemas(&common, schema)?;
            }
            Ok(common)
        }
    }

    fn merge_schemas(
        &self,
        schema1: &Schema,
        schema2: &Schema,
    ) -> Result<Schema, serde_json::Error> {
        if schema1 == schema2 {
            return Ok(schema1.clone());
        }

        let mut merged = json!({
            "oneOf": [schema1, schema2]
        });

        if let (Value::Object(obj1), Value::Object(obj2)) =
            (schema1.as_value(), schema2.as_value())
            && obj1.get("type") == obj2.get("type")
            && obj1.contains_key("properties")
            && obj2.contains_key("properties")
        {
            merged = json!({
                "type": obj1["type"].clone()
            });

            {
                let mut properties = Map::new();
                let props1 = obj1["properties"].as_object().unwrap();
                let props2 = obj2["properties"].as_object().unwrap();

                for (key, value) in props1.iter().chain(props2.iter()) {
                    properties.insert(key.clone(), value.clone());
                }

                merged["properties"] = Value::Object(properties);
            }
        }

        Schema::try_from(merged)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    // Mock implementation of TryGet for testing
    struct MockKnownTypes {
        types: HashMap<Value, Schema>,
    }

    impl MockKnownTypes {
        fn new() -> Self {
            Self {
                types: HashMap::new(),
            }
        }
    }

    impl TryGet<Value, Schema> for MockKnownTypes {
        fn try_get(&self, key: &Value) -> Option<Schema> {
            self.types.get(key).cloned()
        }
    }

    fn create_test_generator() -> MockKnownTypes {
        MockKnownTypes::new()
    }

    fn create_generator_with_mock(
        mock: &MockKnownTypes,
    ) -> SchemaGenerator<'_> {
        SchemaGenerator { known_types: mock }
    }

    #[test]
    fn test_generate_json_schema_string() {
        let mock_types = create_test_generator();
        let generator = create_generator_with_mock(&mock_types);
        let input = json!("test");
        let result = generator.generate_schema(&input).unwrap();
        let expected = json_schema!({"type": "string"});
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_json_schema_integer() {
        let mock_types = create_test_generator();
        let generator = create_generator_with_mock(&mock_types);
        let input = json!(42);
        let result = generator.generate_schema(&input).unwrap();
        let expected = json_schema!({"type": "integer"});
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_json_schema_number() {
        let mock_types = create_test_generator();
        let generator = create_generator_with_mock(&mock_types);
        let input = json!(std::f32::consts::PI);
        let result = generator.generate_schema(&input).unwrap();
        let expected = json_schema!({"type": "number"});
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_json_schema_boolean() {
        let mock_types = create_test_generator();
        let generator = create_generator_with_mock(&mock_types);
        let input = json!(true);
        let result = generator.generate_schema(&input).unwrap();
        let expected = json_schema!({"type": "boolean"});
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_json_schema_null() {
        let mock_types = create_test_generator();
        let generator = create_generator_with_mock(&mock_types);
        let input = json!(null);
        let result = generator.generate_schema(&input).unwrap();
        let expected = json_schema!({"type": "null"});
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_object_schema() {
        let mock_types = create_test_generator();
        let generator = create_generator_with_mock(&mock_types);
        let input = json!({
            "name": "John Doe",
            "age": 30,
            "is_student": false
        });
        let result = generator.generate_schema(&input).unwrap();
        let expected_json = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "integer"},
                "is_student": {"type": "boolean"}
            },
            "required": ["age", "is_student", "name"]
        });
        let expected = Schema::try_from(expected_json).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_array_schema() {
        let mock_types = create_test_generator();
        let generator = create_generator_with_mock(&mock_types);
        let input = json!([1, 2, 3]);
        let result = generator.generate_schema(&input).unwrap();
        let expected = json_schema!({
            "type": "array",
            "items": {"type": "integer"}
        });
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_array_schema_mixed_types() {
        let mock_types = create_test_generator();
        let generator = create_generator_with_mock(&mock_types);
        let input = json!([1, "two", 3.0]);
        let result = generator.generate_schema(&input).unwrap();
        let expected = json_schema!({
            "type": "array",
            "items": {
                "oneOf": [
                    {"oneOf": [
                        {"type": "integer"},
                        {"type": "string"}
                    ]},
                    {"type": "number"}
                ]
            }
        });
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_array_schema_empty() {
        let mock_types = create_test_generator();
        let generator = create_generator_with_mock(&mock_types);
        let input = json!([]);
        let result = generator.generate_schema(&input).unwrap();
        let expected = json_schema!({
            "type": "array",
            "items": {}
        });
        assert_eq!(result, expected);
    }

    #[test]
    fn test_find_common_schema() {
        let mock_types = create_test_generator();
        let generator = create_generator_with_mock(&mock_types);
        let schemas = vec![
            json_schema!({"type": "integer"}),
            json_schema!({"type": "string"}),
            json_schema!({"type": "boolean"}),
        ];
        let result = generator.find_common_schema(&schemas).unwrap();
        let expected = json_schema!({
            "oneOf": [
                {"oneOf": [
                    {"type": "integer"},
                    {"type": "string"}
                ]},
                {"type": "boolean"}
            ]
        });
        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_schemas_same_type() {
        let mock_types = create_test_generator();
        let generator = create_generator_with_mock(&mock_types);
        let schema1 = json_schema!({"type": "object", "properties": {"a": {"type": "string"}}});
        let schema2 = json_schema!({"type": "object", "properties": {"b": {"type": "integer"}}});
        let result = generator.merge_schemas(&schema1, &schema2).unwrap();
        let expected = json_schema!({
            "type": "object",
            "properties": {
                "a": {"type": "string"},
                "b": {"type": "integer"}
            }
        });
        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_schemas_different_types() {
        let mock_types = create_test_generator();
        let generator = create_generator_with_mock(&mock_types);
        let schema1 = json_schema!({"type": "string"});
        let schema2 = json_schema!({"type": "integer"});
        let result = generator.merge_schemas(&schema1, &schema2).unwrap();
        let expected = json_schema!({
            "oneOf": [
                {"type": "string"},
                {"type": "integer"}
            ]
        });
        assert_eq!(result, expected);
    }

    #[test]
    fn test_generate_schema_with_ref() {
        let mock_types = create_test_generator();
        let generator = create_generator_with_mock(&mock_types);
        let input = json!({
            "$ref": "#/definitions/address",
            "address": {
                "street": "123 Main St",
                "city": "New York"
            }
        });
        let result = generator.generate_schema(&input).unwrap();
        let expected_json = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "address": {
                    "type": "object",
                    "properties": {
                        "street": {"type": "string"},
                        "city": {"type": "string"}
                    },
                    "required": ["city", "street"]
                }
            },
            "required": ["address"],
            "$ref": "#/definitions/address"
        });
        let expected = Schema::try_from(expected_json).unwrap();
        assert_eq!(result, expected);
    }
}
