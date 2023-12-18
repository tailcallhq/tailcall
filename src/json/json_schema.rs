use std::collections::HashMap;

use async_graphql::Name;
use serde::{Deserialize, Serialize};

use crate::valid::Valid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
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
        _ => Valid::fail("expected number"),
      },
      JsonSchema::Bool => match value {
        async_graphql::Value::Boolean(_) => Valid::succeed(()),
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
        _ => Valid::fail("expected array"),
      },
      JsonSchema::Obj(fields) => {
        let field_schema_list: Vec<(&String, &JsonSchema)> = fields.iter().collect();
        match value {
          async_graphql::Value::Object(map) => Valid::from_iter(field_schema_list, |(name, schema)| {
            let key = Name::new(name);
            if schema.is_required() {
              if let Some(field_value) = map.get(&key) {
                schema.validate(field_value).trace(name)
              } else {
                Valid::fail("expected field to be non-nullable").trace(name)
              }
            } else if let Some(field_value) = map.get(&key) {
              schema.validate(field_value).trace(name)
            } else {
              Valid::succeed(())
            }
          })
          .unit(),
          _ => Valid::fail("expected object"),
        }
      }
      JsonSchema::Opt(schema) => match value {
        async_graphql::Value::Null => Valid::succeed(()),
        _ => schema.validate(value),
      },
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

#[cfg(test)]
mod tests {
  use async_graphql::Name;
  use indexmap::IndexMap;

  use crate::json::JsonSchema;
  use crate::valid::Valid;

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
      map.insert(Name::new("name"), async_graphql::Value::String("hello".to_string()));
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
      map.insert(Name::new("name"), async_graphql::Value::String("hello".to_string()));
      map.insert(Name::new("age"), async_graphql::Value::String("1".to_string()));
      map
    });
    let result = schema.validate(&value);
    assert_eq!(result, Valid::fail("expected number").trace("age"));
  }

  #[test]
  fn test_null_key() {
    let schema = JsonSchema::from([("name", JsonSchema::Str.optional()), ("age", JsonSchema::Num)]);
    let value = async_graphql::Value::Object({
      let mut map = IndexMap::new();
      map.insert(Name::new("age"), async_graphql::Value::Number(1.into()));
      map
    });

    let result = schema.validate(&value);
    assert_eq!(result, Valid::succeed(()));
  }
}
