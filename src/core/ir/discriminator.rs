use anyhow::{anyhow, bail, Result};
use async_graphql::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum TypeName {
    Single(String),
    Vec(Vec<String>),
}

#[derive(Clone, Debug)]
pub struct Discriminator(Vec<(String, Vec<String>)>);

impl Discriminator {
    pub fn new(types: BTreeMap<String, BTreeSet<String>>) -> Result<Self> {
        let mut types_iter = types.iter();
        let mut common_fields: BTreeSet<_> = types_iter
            .next()
            .ok_or(anyhow!("Types list is empty"))?
            .1
            .clone();

        for (_, fields) in types_iter {
            common_fields = common_fields.intersection(&fields).cloned().collect();
        }

        let mut discriminator = Vec::new();

        for (type_name, type_) in types.iter() {
            let unique_fields: Vec<_> = type_
                .iter()
                .filter(|field| !common_fields.contains(*field))
                .cloned()
                .collect();

            discriminator.push((type_name.clone(), unique_fields));
        }

        // TODO: check for ambiguity and types without additional fields
        dbg!(common_fields);
        dbg!(&discriminator);

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
    use std::collections::{BTreeMap, BTreeSet};

    use async_graphql::Value;
    use serde_json::json;

    use crate::core::ir::discriminator::TypeName;

    use super::Discriminator;

    #[test]
    fn test_foo_bar_single_field() {
        let types = BTreeMap::from_iter([
            ("Foo".to_string(), BTreeSet::from_iter(["foo".to_string()])),
            ("Bar".to_string(), BTreeSet::from_iter(["bar".to_string()])),
        ]);

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
        let types = BTreeMap::from_iter([
            (
                "Foo".to_string(),
                BTreeSet::from_iter(["a".to_string(), "b".to_string(), "foo".to_string()]),
            ),
            (
                "Bar".to_string(),
                BTreeSet::from_iter(["a".to_string(), "b".to_string(), "bar".to_string()]),
            ),
        ]);

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
