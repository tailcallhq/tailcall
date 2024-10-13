use anyhow::{bail, Result};
use async_graphql::{Name, Value};
use indexmap::IndexSet;

use super::TypedValue;
use crate::core::json::JsonLike;
use crate::core::valid::Valid;

/// Resolver for type member of a union or interface.
#[derive(Debug, Clone)]
pub struct TypeFieldDiscriminator {
    typename_field: Name,
    /// List of all types that are members of the union or interface.
    types: IndexSet<String>,
    /// The name of TypeFieldDiscriminator is used for error reporting
    type_name: String,
}

impl TypeFieldDiscriminator {
    pub fn new(
        type_name: String,
        types: IndexSet<String>,
        typename_field: String,
    ) -> Valid<Self, String> {
        let discriminator = Self { type_name, types, typename_field: Name::new(typename_field) };

        Valid::succeed(discriminator)
    }

    pub fn resolve_type(&self, value: &Value) -> Result<String> {
        if value.is_null() {
            return Ok("NULL".to_string());
        }

        let Some(index_map) = value.as_object() else {
            bail!("The TypeFieldDiscriminator(type=\"{}\") uses object values to discriminate, but got `{}` instead", self.type_name, value.to_string())
        };

        let Some(value) = index_map.get(&self.typename_field) else {
            bail!("The TypeFieldDiscriminator(type=\"{}\") cannot discriminate the Value `{}` because it does not contain the type name field `{}`", self.type_name, value.to_string(), self.typename_field.to_string())
        };

        let Value::String(type_name) = value else {
            bail!("The TypeFieldDiscriminator(type=\"{}\") uses a string type name field to discriminate, but got `{}` instead", self.type_name, value.to_string())
        };

        if self.types.contains(type_name) {
            Ok(type_name.to_string())
        } else {
            bail!("The type `{}` is not in the list of acceptable types {:?} of TypeFieldDiscriminator(type=\"{}\")", type_name, self.types, self.type_name)
        }
    }

    pub fn resolve_and_set_type(&self, mut value: Value) -> Result<Value> {
        let type_name = self.resolve_type(&value)?;
        value.set_type_name(type_name)?;
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use async_graphql::Value;
    use serde_json::json;
    use test_log::test;

    use super::TypeFieldDiscriminator;
    use crate::core::valid::Validator;

    #[test]
    fn test_type_field_positive() {
        let types = vec!["Foo".to_string(), "Bar".to_string()];
        let discriminator = TypeFieldDiscriminator::new(
            "Test".to_string(),
            types.into_iter().collect(),
            "type".to_string(),
        )
        .to_result()
        .unwrap();

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "foo": "test", "type": "Foo" })).unwrap())
                .unwrap(),
            "Foo"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "bar": "test", "type": "Bar" })).unwrap())
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
    fn test_type_field_negative() {
        let types = vec!["Foo".to_string(), "Bar".to_string()];
        let discriminator = TypeFieldDiscriminator::new(
            "Test".to_string(),
            types.into_iter().collect(),
            "type".to_string(),
        )
        .to_result()
        .unwrap();

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!(false)).unwrap())
                .unwrap_err()
                .to_string(),
            "The TypeFieldDiscriminator(type=\"Test\") uses object values to discriminate, but got `false` instead"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "foo": "test" })).unwrap())
                .unwrap_err()
                .to_string(),
            "The TypeFieldDiscriminator(type=\"Test\") cannot discriminate the Value `{foo: \"test\"}` because it does not contain the type name field `type`"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "foo": "test", "type": false })).unwrap())
                .unwrap_err()
                .to_string(),
            "The TypeFieldDiscriminator(type=\"Test\") uses a string type name field to discriminate, but got `false` instead"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "foo": "test", "type": "Buzz" })).unwrap())
                .unwrap_err()
                .to_string(),
            "The type `Buzz` is not in the list of acceptable types [\"Foo\", \"Bar\"] of TypeFieldDiscriminator(type=\"Test\")"
        );
    }
}
