mod keyed_discriminator;
mod type_field_discriminator;

use std::collections::BTreeSet;

use anyhow::{bail, Result};
use async_graphql::Value;
use keyed_discriminator::KeyedDiscriminator;
use tailcall_valid::{Valid, Validator};
use type_field_discriminator::TypeFieldDiscriminator;

use crate::core::json::{JsonLike, JsonObjectLike};

/// Resolver for `__typename` of Union and Interface types.
///
/// A discriminator is used to determine the type of an object in a GraphQL
/// schema. It can be used to resolve the `__typename` field of an object.
///
/// There are two types of discriminators:
///
/// * [KeyedDiscriminator]: Uses the keys of an object to determine its type.
/// * [TypeFieldDiscriminator]: Uses a specific field of an object to determine
///   its type.
///
/// The [Discriminator] enum provides a way to construct and use these
/// discriminators.
#[derive(Debug, Clone, PartialEq)]
pub enum Discriminator {
    /// A discriminator that uses the keys of an object to determine its type.
    Keyed(KeyedDiscriminator),
    /// A discriminator that uses a specific field of an object to determine its
    /// type.
    TypeField(TypeFieldDiscriminator),
}

impl Discriminator {
    /// Constructs a new discriminator.
    ///
    /// `type_name`: The name of the type that this discriminator is applied to.
    /// `types`: The possible types that this discriminator can resolve.
    /// `typename_field`: If specified, the discriminator will use this field to
    /// resolve the `__typename`.
    ///
    /// When `typename_field` is present the function Validates that it is not
    /// empty.
    pub fn new(
        type_name: String,
        types: BTreeSet<String>,
        typename_field: Option<String>,
    ) -> Valid<Self, String> {
        if let Some(typename_field) = &typename_field {
            if typename_field.is_empty() {
                return Valid::fail(format!(
                    "The `field` cannot be an empty string for the `@discriminate` of type {}",
                    type_name
                ));
            }
        }

        if let Some(typename_field) = typename_field {
            TypeFieldDiscriminator::new(type_name, types, typename_field).map(Self::TypeField)
        } else {
            KeyedDiscriminator::new(type_name, types).map(Self::Keyed)
        }
    }

    /// Resolves the `__typename` for an object and inserts the value into the
    /// object.
    pub fn resolve_type(&self, value: Value) -> Result<Value> {
        // if typename is already present we return it
        if value.get_type_name().is_some() {
            return Ok(value);
        }

        match value {
            Value::Null => Ok(value),
            Value::List(arr) => {
                let arr = arr.into_iter().map(|i| self.resolve_type(i)).collect::<Result<Vec<_>>>()?;
                Ok(Value::array(arr))
            },
            Value::Object(_) => {
                match self {
                    Discriminator::Keyed(keyed_discriminator) => {
                        keyed_discriminator.resolve_and_set_type(value)
                    }
                    Discriminator::TypeField(type_field_discriminator) => {
                        type_field_discriminator.resolve_and_set_type(value)
                    }
                }
            },
            _ => bail!("Discriminator can only determine the types of arrays or objects but a different type.")
        }
    }
}

pub trait TypedValue<'a> {
    type Error;

    fn get_type_name(&'a self) -> Option<&'a str>;
    fn set_type_name(&'a mut self, type_name: String) -> Result<(), Self::Error>;
}

const TYPENAME_FIELD: &str = "__typename";

impl<'json, T> TypedValue<'json> for T
where
    T: JsonLike<'json>,
{
    type Error = anyhow::Error;

    fn get_type_name(&'json self) -> Option<&'json str> {
        self.as_object()
            .and_then(|obj| obj.get_key(TYPENAME_FIELD))
            .and_then(|val| val.as_str())
    }

    fn set_type_name(&'json mut self, type_name: String) -> Result<(), Self::Error> {
        if self.is_null() {
            Ok(())
        } else if let Some(obj) = self.as_object_mut() {
            obj.insert_key(TYPENAME_FIELD, T::string(type_name.into()));
            Ok(())
        } else {
            bail!("Cannot discriminate the type of a non object type.")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_type_field_is_invalid() {
        let result = Discriminator::new("Test".to_string(), BTreeSet::new(), Some("".to_string()));
        assert!(result.is_fail());
        assert_eq!(result.to_result().unwrap_err().to_string(), "Validation Error\n• The `field` cannot be an empty string for the `@discriminate` of type Test\n");
    }

    #[test]
    fn keyed_discriminator_works() {
        let mut types = BTreeSet::new();
        types.insert("Test1".to_string());
        types.insert("Test2".to_string());

        let result = Discriminator::new("Test".to_string(), types.clone(), None);
        assert!(result.is_succeed());

        let result = result.to_result().unwrap();
        assert_eq!(
            result,
            Discriminator::Keyed(
                KeyedDiscriminator::new("Test".to_string(), types)
                    .to_result()
                    .unwrap()
            )
        );
    }

    #[test]
    fn type_field_discriminator_works() {
        let mut types = BTreeSet::new();
        types.insert("Test1".to_string());
        types.insert("Test2".to_string());

        let result =
            Discriminator::new("Test".to_string(), types.clone(), Some("type".to_string()));
        assert!(result.is_succeed());

        let result = result.to_result().unwrap();
        assert_eq!(
            result,
            Discriminator::TypeField(
                TypeFieldDiscriminator::new("Test".to_string(), types, "type".to_string())
                    .to_result()
                    .unwrap()
            )
        );
    }
}
