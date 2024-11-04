use std::collections::BTreeSet;

use anyhow::{bail, Result};
use async_graphql::Value;
use tailcall_valid::Valid;

use super::TypedValue;

/// Resolver for `__typename` of Union and Interface types.
///
/// The [KeyedDiscriminator] expects an object with one key, the type of the
/// value. For example `{ "Foo": {...} }` the `__typename` will resolve to
/// "Foo".
///
/// This discriminator is used when the type of an object can be determined by
/// its keys.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyedDiscriminator {
    /// List of all types that are members of the union or interface.
    types: BTreeSet<String>,
    /// The name of `KeyedDiscriminator` is used for error reporting
    type_name: String,
}

impl KeyedDiscriminator {
    /// Constructs a new [KeyedDiscriminator] resolver.
    ///
    /// `type_name`: The name of the type that this discriminator is applied at.
    /// `types`: The possible types that this discriminator can resolve.
    pub fn new(type_name: String, types: BTreeSet<String>) -> Valid<Self, String> {
        let discriminator = Self { type_name, types };

        Valid::succeed(discriminator)
    }

    /// Resolves the `__typename` for an object.
    /// If the object has more than one key, or if the key is not in the list of
    /// possible types, an error will be returned.
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
                        let types: Vec<_> = self.types.clone().into_iter().collect();
                        bail!("The type `{}` is not in the list of acceptable types {:?} of KeyedDiscriminator(type=\"{}\")", type_name, types, self.type_name)
                    }
                } else if index_map_len == 0 {
                    bail!("The KeyedDiscriminator(type=\"{}\") cannot discriminate the Value because it contains no keys.", self.type_name)
                } else {
                    bail!("The KeyedDiscriminator(type=\"{}\") cannot discriminate the Value because it contains more than one keys.", self.type_name)
                }
            },
            _ => bail!("The KeyedDiscriminator(type=\"{}\") can only use object values to discriminate, but received a different type.", self.type_name)
        }
    }

    /// Resolves the `__typename` for an object and inserts the value into the
    /// object.
    pub fn resolve_and_set_type(&self, value: Value) -> Result<Value> {
        let type_name = self.resolve_type(&value)?;
        let mut value = match value {
            Value::Object(index_map) => {
                // this is safe to unwrap because we already validated it in `resolve_type``
                let (_, value) = index_map.into_iter().next().unwrap();
                value
            },
            _ => bail!("The KeyedDiscriminator(type=\"{}\") can only use object values to discriminate, but received a different type.", self.type_name)
        };
        value.set_type_name(type_name)?;
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use async_graphql::Value;
    use serde_json::json;
    use tailcall_valid::Validator;
    use test_log::test;

    use super::KeyedDiscriminator;

    #[test]
    fn test_keyed_discriminator_positive() {
        let types = vec!["Foo".to_string(), "Bar".to_string()];
        let discriminator =
            KeyedDiscriminator::new("Test".to_string(), types.into_iter().collect())
                .to_result()
                .unwrap();

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
        let types = vec!["Foo".to_string(), "Bar".to_string()];
        let discriminator =
            KeyedDiscriminator::new("Test".to_string(), types.into_iter().collect())
                .to_result()
                .unwrap();

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "Fizz": { "foo": "test" } })).unwrap())
                .unwrap_err()
                .to_string(),
            "The type `Fizz` is not in the list of acceptable types [\"Bar\", \"Foo\"] of KeyedDiscriminator(type=\"Test\")"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({})).unwrap())
                .unwrap_err()
                .to_string(),
            "The KeyedDiscriminator(type=\"Test\") cannot discriminate the Value because it contains no keys."
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!(false)).unwrap())
                .unwrap_err()
                .to_string(),
            "The KeyedDiscriminator(type=\"Test\") can only use object values to discriminate, but received a different type."
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "Fizz": { "foo": "test" }, "Buzz": { "bar": "test" } })).unwrap())
                .unwrap_err()
                .to_string(),
            "The KeyedDiscriminator(type=\"Test\") cannot discriminate the Value because it contains more than one keys."
        );
    }
}
