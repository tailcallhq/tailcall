use std::collections::HashMap;

use prost_reflect::{FieldDescriptor, Kind, MessageDescriptor};
use serde::{Deserialize, Serialize};

use crate::mustache::Mustache;
use crate::valid::{Valid, Validator};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename = "schema")]
pub enum JsonSchema {
    Obj(HashMap<String, JsonSchema>),
    Arr(Box<JsonSchema>),
    Opt(Box<JsonSchema>),
    Str,
    Num,
    Bool,
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
                async_graphql::Value::String(s) => Self::check_mustache(s, "expected number"),

                _ => Valid::fail("expected number"),
            },
            JsonSchema::Bool => match value {
                async_graphql::Value::Boolean(_) => Valid::succeed(()),
                async_graphql::Value::String(s) => Self::check_mustache(s, "expected boolean"),
                _ => Valid::fail("expected boolean"),
            },
            JsonSchema::Arr(schema) => match value {
                async_graphql::Value::List(list) => {
                    // TODO: add unit tests
                    Valid::from_iter(list.iter().enumerate(), |(i, item)| {
                        schema.validate(item).trace(i.to_string().as_str())
                    })
                    .unit()
                }
                async_graphql::Value::String(s) => Self::check_mustache(s, "expected array"),
                _ => Valid::fail("expected array"),
            },
            JsonSchema::Obj(fields) => {
                let field_schema_list: Vec<(&String, &JsonSchema)> = fields.iter().collect();
                match value {
                    async_graphql::Value::Object(map) => {
                        Valid::from_iter(field_schema_list, |(name, schema)| {
                            if schema.is_required() {
                                if let Some(field_value) = map.get::<str>(name.as_ref()) {
                                    schema.validate(field_value).trace(name)
                                } else {
                                    Valid::fail("expected field to be non-nullable").trace(name)
                                }
                            } else if let Some(field_value) = map.get::<str>(name.as_ref()) {
                                schema.validate(field_value).trace(name)
                            } else {
                                Valid::succeed(())
                            }
                        })
                        .unit()
                    }
                    async_graphql::Value::String(s) => Self::check_mustache(s, "expected object"),
                    _ => Valid::fail("expected object"),
                }
            }
            JsonSchema::Opt(schema) => match value {
                async_graphql::Value::Null => Valid::succeed(()),
                _ => schema.validate(value),
            },
        }
    }

    fn check_mustache(s: &str, tpe: &'static str) -> Valid<(), &'static str> {
        if let Ok(v) = Mustache::parse(s) {
            if !v.is_const() {
                Valid::succeed(())
            } else {
                Valid::fail(tpe)
            }
        } else {
            Valid::fail(tpe)
        }
    }

    // TODO: add unit tests
    pub fn compare(&self, other: &JsonSchema, name: &str) -> Valid<(), String> {
        match self {
            JsonSchema::Obj(a) => {
                if let JsonSchema::Obj(b) = other {
                    return Valid::from_iter(b.iter(), |(key, b)| {
                        Valid::from_option(a.get(key), format!("missing key: {}", key))
                            .and_then(|a| a.compare(b, key))
                    })
                    .trace(name)
                    .unit();
                } else {
                    return Valid::fail("expected Object type".to_string()).trace(name);
                }
            }
            JsonSchema::Arr(a) => {
                if let JsonSchema::Arr(b) = other {
                    return a.compare(b, name);
                } else {
                    return Valid::fail("expected Non repeatable type".to_string()).trace(name);
                }
            }
            JsonSchema::Opt(a) => {
                if let JsonSchema::Opt(b) = other {
                    return a.compare(b, name);
                } else {
                    return Valid::fail("expected type to be required".to_string()).trace(name);
                }
            }
            JsonSchema::Str => {
                if other != self {
                    return Valid::fail(format!("expected String, got {:?}", other)).trace(name);
                }
            }
            JsonSchema::Num => {
                if other != self {
                    return Valid::fail(format!("expected Number, got {:?}", other)).trace(name);
                }
            }
            JsonSchema::Bool => {
                if other != self {
                    return Valid::fail(format!("expected Boolean, got {:?}", other)).trace(name);
                }
            }
        }
        Valid::succeed(())
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

impl TryFrom<&MessageDescriptor> for JsonSchema {
    type Error = crate::valid::ValidationError<String>;

    fn try_from(value: &MessageDescriptor) -> Result<Self, Self::Error> {
        let mut map = std::collections::HashMap::new();
        let fields = value.fields();

        for field in fields {
            let field_schema = JsonSchema::try_from(&field)?;

            map.insert(field.name().to_string(), field_schema);
        }

        Ok(JsonSchema::Obj(map))
    }
}

impl TryFrom<&FieldDescriptor> for JsonSchema {
    type Error = crate::valid::ValidationError<String>;

    fn try_from(value: &FieldDescriptor) -> Result<Self, Self::Error> {
        let field_schema = match value.kind() {
            Kind::Double => JsonSchema::Num,
            Kind::Float => JsonSchema::Num,
            Kind::Int32 => JsonSchema::Num,
            Kind::Int64 => JsonSchema::Num,
            Kind::Uint32 => JsonSchema::Num,
            Kind::Uint64 => JsonSchema::Num,
            Kind::Sint32 => JsonSchema::Num,
            Kind::Sint64 => JsonSchema::Num,
            Kind::Fixed32 => JsonSchema::Num,
            Kind::Fixed64 => JsonSchema::Num,
            Kind::Sfixed32 => JsonSchema::Num,
            Kind::Sfixed64 => JsonSchema::Num,
            Kind::Bool => JsonSchema::Bool,
            Kind::String => JsonSchema::Str,
            Kind::Bytes => JsonSchema::Str,
            Kind::Message(msg) => JsonSchema::try_from(&msg)?,
            Kind::Enum(_) => {
                todo!("Enum")
            }
        };
        let field_schema = if value
            .cardinality()
            .eq(&prost_reflect::Cardinality::Optional)
        {
            JsonSchema::Opt(Box::new(field_schema))
        } else {
            field_schema
        };
        let field_schema = if value.is_list() {
            JsonSchema::Arr(Box::new(field_schema))
        } else {
            field_schema
        };

        Ok(field_schema)
    }
}

#[cfg(test)]
mod tests {
    use async_graphql::Name;
    use indexmap::IndexMap;

    use crate::json::JsonSchema;
    use crate::valid::{Valid, Validator};

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
        assert_eq!(result, Valid::fail("expected number").trace("age"));
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
}
