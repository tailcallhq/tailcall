use crate::core::config::Type;
use crate::core::valid::Valid;
use anyhow::{bail, Result};
use async_graphql::Value;

use super::TypedValue;

/// Resolver for type member of a union or interface.
#[derive(Debug, Clone)]
pub struct KeyedDiscriminator {
    /// List of all types that are members of the union or interface.
    types: Vec<String>,
    /// The name of KeyedDiscriminator is used for error reporting
    name: String,
}

impl KeyedDiscriminator {
    pub fn new(union_name: &str, union_types: &[(&str, &Type)]) -> Valid<Self, String> {
        let types: Vec<_> = union_types
            .iter()
            .map(|(name, _)| name.to_string())
            .collect();

        let discriminator = Self { name: union_name.to_string(), types };

        tracing::debug!("Generated KeyedDiscriminator for type '{union_name}':\n{discriminator:?}",);

        Valid::succeed(discriminator)
    }

    pub fn resolve_type(&self, value: &Value) -> Result<String> {
        match value {
            // INFO: when a value is null you cannot use __typename so we are safe returning whatever
            Value::Null => Ok("NULL".to_string()),
            Value::Object(index_map) => {
                let index_map_len = index_map.len();
                if index_map_len == 1 {
                    let (name, _) = index_map.first().unwrap();
                    let type_name = name.to_string();
                    if self.types.contains(&type_name) {
                        Ok(type_name)
                    } else {
                        bail!("The type `{}` is not in the list of acceptable types {:?} of KeyedDiscriminator(type=\"{}\")", type_name, self.types, self.name)
                    }
                } else if index_map_len == 0 {
                    bail!("The KeyedDiscriminator(type=\"{}\") cannot discriminate the Value `{}` because it contains no keys.", self.name, value.to_string())
                } else {
                    bail!("The KeyedDiscriminator(type=\"{}\") cannot discriminate the Value `{}` because it contains more than one keys.", self.name, value.to_string())
                }
            },
            _ => bail!("The KeyedDiscriminator(type=\"{}\") uses object values to discriminate, but got `{}` instead", self.name, value.to_string())
        }
    }

    pub fn resolve_and_set_type(&self, value: Value) -> Result<Value> {
        let type_name = self.resolve_type(&value)?;
        let mut value = match value {
            Value::Object(index_map) => {
                // this is safe to unwrap because we already validated it in `resolve_type``
                let (_, value) = index_map.into_iter().next().unwrap();
                value
            },
            _ => bail!("The KeyedDiscriminator(type=\"{}\") uses object values to discriminate, but got `{}` instead", self.name, value.to_string())
        };
        value.set_type_name(type_name)?;
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use async_graphql::Value;
    use serde_json::json;
    use test_log::test;

    use super::KeyedDiscriminator;
    use crate::core::config;
    use crate::core::config::Field;
    use crate::core::valid::Validator;

    #[test]
    fn test_keyed_discriminator_positive() {
        let foo = config::Type::default().fields(vec![("foo", Field::default())]);
        let bar = config::Type::default().fields(vec![("bar", Field::default())]);
        let types = vec![("Foo", &foo), ("Bar", &bar)];

        let discriminator = KeyedDiscriminator::new("Test", &types).to_result().unwrap();

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "Foo": { "foo": "test" } })).unwrap())
                .unwrap(),
            "Foo"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "Bar": { "bar": "test" } })).unwrap())
                .unwrap(),
            "Bar"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!(null)).unwrap())
                .unwrap(),
            "NULL"
        );
    }

    #[test]
    fn test_keyed_discriminator_negative() {
        let foo = config::Type::default().fields(vec![("foo", Field::default())]);
        let bar = config::Type::default().fields(vec![("bar", Field::default())]);
        let types = vec![("Foo", &foo), ("Bar", &bar)];

        let discriminator = KeyedDiscriminator::new("Test", &types).to_result().unwrap();

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "Fizz": { "foo": "test" } })).unwrap())
                .unwrap_err()
                .to_string(),
            "The type `Fizz` is not in the list of acceptable types [\"Foo\", \"Bar\"] of KeyedDiscriminator(type=\"Test\")"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({})).unwrap())
                .unwrap_err()
                .to_string(),
            "The KeyedDiscriminator(type=\"Test\") cannot discriminate the Value `{}` because it contains no keys."
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!(false)).unwrap())
                .unwrap_err()
                .to_string(),
            "The KeyedDiscriminator(type=\"Test\") uses object values to discriminate, but got `false` instead"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "Fizz": { "foo": "test" }, "Buzz": { "bar": "test" } })).unwrap())
                .unwrap_err()
                .to_string(),
            "The KeyedDiscriminator(type=\"Test\") cannot discriminate the Value `{Fizz: {foo: \"test\"}, Buzz: {bar: \"test\"}}` because it contains more than one keys."
        );
    }
}
