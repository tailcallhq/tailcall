use anyhow::{anyhow, bail, Result};
use async_graphql::Value;
use std::collections::{BTreeMap, BTreeSet};

use crate::core::config::Type;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum TypeName {
    Single(String),
    Vec(Vec<String>),
}

#[derive(Clone, Debug)]
pub struct Discriminator(Vec<(String, BTreeSet<String>)>);

impl Discriminator {
    pub fn new(types: BTreeMap<&str, &Type>) -> Result<Self> {
        let mut fields_iter = types
            .values()
            .map(|type_| type_.fields.keys().collect::<BTreeSet<_>>());
        let mut common_fields: BTreeSet<_> = fields_iter
            .next()
            .ok_or(anyhow!("Types list is empty"))?
            .clone();

        for fields in fields_iter {
            common_fields = common_fields.intersection(&fields).cloned().collect();
        }


        let mut discriminator = Vec::new();

        for (type_name, type_) in types.iter() {
            // TODO: do we need to check also addedFields here?
            let unique_fields: BTreeSet<_> = type_
                .fields
                .keys()
                .filter(|field| !common_fields.contains(*field))
                .cloned()
                .collect();

            discriminator.push((type_name.to_string(), unique_fields));
        }

        // TODO: check for ambiguity and types without additional fields

        Ok(Self(discriminator))
    }

    pub fn resolve_type(&self, value: &Value) -> Result<TypeName> {
        if let Value::List(list) = value {
            let results: Result<Vec<_>> = list
                .iter()
                .map(|item| Ok(self.resolve_type_for_single(item)?.to_string()))
                .collect();

            Ok(TypeName::Vec(results?))
        } else {
            Ok(TypeName::Single(
                self.resolve_type_for_single(value)?.to_string(),
            ))
        }
    }

    fn resolve_type_for_single(&self, value: &Value) -> Result<&str> {
        let Value::Object(obj) = value else {
            bail!("Value expected to be object");
        };

        for (type_name, fields) in &self.0 {
            let has_all_fields = fields.iter().all(|field| obj.contains_key(field.as_str()));

            if has_all_fields {
                return Ok(type_name);
            }
        }

        bail!("Failed to find corresponding type for value");
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use async_graphql::Value;
    use serde_json::json;

    use crate::core::{
        config::{Field, Type},
        ir::discriminator::TypeName,
    };

    use super::Discriminator;

    #[test]
    fn test_foo_bar_single_field() {
        let foo = Type::default().fields(vec![("foo", Field::default())]);
        let bar = Type::default().fields(vec![("bar", Field::default())]);
        let types = BTreeMap::from_iter([("Foo", &foo), ("Bar", &bar)]);

        let discriminator = Discriminator::new(types).unwrap();

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "foo": "test" })).unwrap())
                .unwrap(),
            TypeName::Single("Foo".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "bar": "test" })).unwrap())
                .unwrap(),
            TypeName::Single("Bar".to_string())
        );
    }

    #[test]
    fn test_foo_bar_with_shared_fields() {
        let foo = Type::default().fields(vec![
            ("a", Field::default()),
            ("b", Field::default()),
            ("foo", Field::default()),
        ]);
        let bar = Type::default().fields(vec![
            ("a", Field::default()),
            ("b", Field::default()),
            ("bar", Field::default()),
        ]);
        let types = BTreeMap::from_iter([("Foo", &foo), ("Bar", &bar)]);

        let discriminator = Discriminator::new(types).unwrap();

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "a": 123, "b": true, "foo": "test" })).unwrap()
                )
                .unwrap(),
            TypeName::Single("Foo".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "bar": "test" })).unwrap())
                .unwrap(),
            TypeName::Single("Bar".to_string())
        );
    }
}
