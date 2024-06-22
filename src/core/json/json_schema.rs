use std::collections::{BTreeSet, HashMap};

use serde::{Deserialize, Serialize};

use super::{JsonScheamWithSourcePosition, PositionedJsonSchema};
use crate::core::valid::{Valid, Validator};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename = "schema")]
pub enum JsonSchema {
    Obj(HashMap<String, JsonSchema>),
    Arr(Box<JsonSchema>),
    Opt(Box<JsonSchema>),
    Enum(BTreeSet<String>),
    Str,
    Num,
    Bool,
    Empty,
    Any,
}

impl<const L: usize> From<[(&'static str, JsonSchema); L]> for JsonSchema {
    fn from(fields: [(&'static str, JsonSchema); L]) -> Self {
        let mut map = HashMap::new();
        for (name, schema) in fields {
            map.insert(name.to_string(), schema);
        }
        JsonSchema::Obj(map)
    }
}

impl Default for JsonSchema {
    fn default() -> Self {
        JsonSchema::Obj(HashMap::new())
    }
}

impl JsonSchema {
    // TODO: validate `JsonLike` instead of fixing on `async_graphql::Value`
    pub fn validate(&self, value: &async_graphql::Value) -> Valid<(), &'static str> {
        match self {
            JsonSchema::Str => match value {
                async_graphql::Value::String(_) => Valid::succeed(()),
                _ => Valid::fail("expected string"),
            },
            JsonSchema::Num => match value {
                async_graphql::Value::Number(_) => Valid::succeed(()),
                _ => Valid::fail("expected number"),
            },
            JsonSchema::Bool => match value {
                async_graphql::Value::Boolean(_) => Valid::succeed(()),
                _ => Valid::fail("expected boolean"),
            },
            JsonSchema::Empty => match value {
                async_graphql::Value::Null => Valid::succeed(()),
                async_graphql::Value::Object(obj) if obj.is_empty() => Valid::succeed(()),
                _ => Valid::fail("expected empty"),
            },
            JsonSchema::Any => Valid::succeed(()),
            JsonSchema::Arr(schema) => match value {
                async_graphql::Value::List(list) => {
                    // TODO: add unit tests
                    Valid::from_iter(list.iter().enumerate(), |(i, item)| {
                        schema.validate(item).trace(Some(i.to_string().as_str()))
                    })
                    .unit()
                }
                _ => Valid::fail("expected array"),
            },
            JsonSchema::Obj(fields) => {
                let field_schema_list: Vec<(&String, &JsonSchema)> = fields.iter().collect();
                match value {
                    async_graphql::Value::Object(map) => {
                        Valid::from_iter(field_schema_list, |(name, schema)| {
                            if schema.is_required() {
                                if let Some(field_value) = map.get::<str>(name.as_ref()) {
                                    schema.validate(field_value).trace(Some(name))
                                } else {
                                    Valid::fail("expected field to be non-nullable")
                                        .trace(Some(name))
                                }
                            } else if let Some(field_value) = map.get::<str>(name.as_ref()) {
                                schema.validate(field_value).trace(Some(name))
                            } else {
                                Valid::succeed(())
                            }
                        })
                        .unit()
                    }
                    _ => Valid::fail("expected object"),
                }
            }
            JsonSchema::Opt(schema) => match value {
                async_graphql::Value::Null => Valid::succeed(()),
                _ => schema.validate(value),
            },
            JsonSchema::Enum(_) => Valid::succeed(()),
        }
    }

    pub fn optional(self) -> JsonSchema {
        JsonSchema::Opt(Box::new(self))
    }

    pub fn is_optional(&self) -> bool {
        matches!(self, JsonSchema::Opt(_))
    }

    pub fn is_required(&self) -> bool {
        !self.is_optional()
    }
}

impl From<PositionedJsonSchema> for JsonSchema {
    fn from(value: PositionedJsonSchema) -> Self {
        match value.schema {
            JsonScheamWithSourcePosition::Obj(map) => {
                let mut new_map = HashMap::new();
                for (k, v) in map {
                    new_map.insert(k, JsonSchema::from(v));
                }
                JsonSchema::Obj(new_map)
            }
            JsonScheamWithSourcePosition::Arr(val) => {
                JsonSchema::Arr(Box::new(JsonSchema::from(*val)))
            }
            JsonScheamWithSourcePosition::Opt(val) => {
                JsonSchema::Opt(Box::new(JsonSchema::from(*val)))
            }
            JsonScheamWithSourcePosition::Enum(val) => JsonSchema::Enum(val),
            JsonScheamWithSourcePosition::Str => JsonSchema::Str,
            JsonScheamWithSourcePosition::Num => JsonSchema::Num,
            JsonScheamWithSourcePosition::Bool => JsonSchema::Bool,
            JsonScheamWithSourcePosition::Empty => JsonSchema::Empty,
            JsonScheamWithSourcePosition::Any => JsonSchema::Any,
        }
    }
}

#[cfg(test)]
mod tests {
    use async_graphql::Name;
    use indexmap::IndexMap;

    use crate::core::json::JsonSchema;
    use crate::core::valid::{Valid, Validator};

    #[test]
    fn test_validate_string() {
        let schema = JsonSchema::Str;
        let value = async_graphql::Value::String("hello".to_string());
        let result = schema.validate(&value);
        assert_eq!(result, Valid::succeed(()));
    }

    #[test]
    fn test_validate_valid_object() {
        let schema = JsonSchema::from([("name", JsonSchema::Str), ("age", JsonSchema::Num)]);
        let value = async_graphql::Value::Object({
            let mut map = IndexMap::new();
            map.insert(
                Name::new("name"),
                async_graphql::Value::String("hello".to_string()),
            );
            map.insert(Name::new("age"), async_graphql::Value::Number(1.into()));
            map
        });
        let result = schema.validate(&value);
        assert_eq!(result, Valid::succeed(()));
    }

    #[test]
    fn test_validate_invalid_object() {
        let schema = JsonSchema::from([("name", JsonSchema::Str), ("age", JsonSchema::Num)]);
        let value = async_graphql::Value::Object({
            let mut map = IndexMap::new();
            map.insert(
                Name::new("name"),
                async_graphql::Value::String("hello".to_string()),
            );
            map.insert(
                Name::new("age"),
                async_graphql::Value::String("1".to_string()),
            );
            map
        });
        let result = schema.validate(&value);
        assert_eq!(result, Valid::fail("expected number").trace(Some("age")));
    }

    #[test]
    fn test_null_key() {
        let schema = JsonSchema::from([
            ("name", JsonSchema::Str.optional()),
            ("age", JsonSchema::Num),
        ]);
        let value = async_graphql::Value::Object({
            let mut map = IndexMap::new();
            map.insert(Name::new("age"), async_graphql::Value::Number(1.into()));
            map
        });

        let result = schema.validate(&value);
        assert_eq!(result, Valid::succeed(()));
    }

    #[test]
    fn test_empty_valid() {
        let schema = JsonSchema::from([
            ("empty1", JsonSchema::Empty.optional()),
            ("empty2", JsonSchema::Empty),
        ]);
        let value = async_graphql::Value::Object({
            let mut map = IndexMap::new();
            map.insert(
                Name::new("empty1"),
                async_graphql::Value::Object(Default::default()),
            );
            map.insert(Name::new("empty2"), async_graphql::Value::Null);
            map
        });

        let result = schema.validate(&value);
        assert_eq!(result, Valid::succeed(()));
    }

    #[test]
    fn test_empty_invalid() {
        let schema = JsonSchema::Empty;
        let value = async_graphql::Value::String("test".to_owned());

        let result = schema.validate(&value);
        assert_eq!(result, Valid::fail("expected empty"));
    }

    #[test]
    fn test_any_valid() {
        let schema = JsonSchema::from([
            ("any1", JsonSchema::Any.optional()),
            ("any2", JsonSchema::Any),
        ]);
        let value = async_graphql::Value::Object({
            let mut map = IndexMap::new();
            map.insert(
                Name::new("any1"),
                async_graphql::Value::Object(Default::default()),
            );
            map.insert(
                Name::new("any2"),
                async_graphql::Value::String("test".to_owned()),
            );
            map
        });

        let result = schema.validate(&value);
        assert_eq!(result, Valid::succeed(()));
    }
}
