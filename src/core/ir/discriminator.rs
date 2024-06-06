use anyhow::{anyhow, bail, Result};
use async_graphql::Value;
use indexmap::{IndexMap, IndexSet};

use crate::core::config::Type;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum TypeName {
    Single(String),
    Vec(Vec<String>),
}

#[derive(Clone, Debug)]
pub struct Discriminator {
    types: Vec<String>,
    fields_info: IndexMap<String, FieldInfo>,
}

#[derive(Default, Debug, Clone)]
struct FieldInfo {
    present_in: IndexSet<String>,
    required_in: IndexSet<String>,
}

impl Discriminator {
    pub fn new(types: Vec<(&str, &Type)>) -> Result<Self> {
        let mut fields_info: IndexMap<String, FieldInfo> = IndexMap::new();

        // TODO: do we need to check also added_fields?
        for (type_name, type_) in types.iter() {
            for (field_name, field) in type_.fields.iter() {
                let info = fields_info.entry(field_name.to_string()).or_default();

                info.present_in.insert(type_name.to_string());

                if field.required {
                    info.required_in.insert(type_name.to_string());
                }
            }
        }

        dbg!(&fields_info);

        Ok(Self {
            fields_info,
            types: types.iter().map(|(name, _)| name.to_string()).collect(),
        })
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

    fn resolve_type_for_single(&self, value: &Value) -> Result<String> {
        let Value::Object(obj) = value else {
            bail!("Value expected to be object");
        };

        let mut possible_types: IndexSet<_> =
            self.types.iter().map(|name| name.to_string()).collect();

        for (field, info) in &self.fields_info {
            if obj.contains_key(field.as_str()) {
                possible_types = possible_types
                    .intersection(&info.present_in)
                    .cloned()
                    .collect();
            } else {
                possible_types = possible_types
                    .difference(&info.required_in)
                    .cloned()
                    .collect();
            }

            match possible_types.len() {
                0 => bail!("Failed to find corresponding type for value"),
                1 => {
                    return Ok(possible_types
                        .into_iter()
                        .next()
                        .expect("Safe unwrap due to previous check for length"))
                }
                _ => {}
            };
        }

        possible_types
            .into_iter()
            .next()
            .ok_or(anyhow!("Empty list of possible types"))
    }
}

#[cfg(test)]
mod tests {
    use async_graphql::Value;
    use serde_json::json;

    use crate::core::{
        config::{Field, Type},
        ir::discriminator::TypeName,
    };

    use super::Discriminator;

    #[test]
    fn test_single_distinct_field_optional() {
        let foo = Type::default().fields(vec![("foo", Field::default())]);
        let bar = Type::default().fields(vec![("bar", Field::default())]);
        let types = vec![("Foo", &foo), ("Bar", &bar)];

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

        // ambiguous cases
        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "foo": "test", "bar": "test" })).unwrap())
                .unwrap(),
            TypeName::Single("Foo".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({})).unwrap())
                .unwrap(),
            TypeName::Single("Foo".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "unknown": { "foo": "bar" }})).unwrap())
                .unwrap(),
            TypeName::Single("Foo".to_string())
        );
    }

    #[test]
    fn test_single_distinct_field_required() {
        let foo =
            Type::default().fields(vec![("foo", Field { required: true, ..Field::default() })]);
        let bar =
            Type::default().fields(vec![("bar", Field { required: true, ..Field::default() })]);
        let types = vec![("Foo", &foo), ("Bar", &bar)];

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

        // ambiguous cases
        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "foo": "test", "bar": "test" })).unwrap())
                .unwrap(),
            TypeName::Single("Foo".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({})).unwrap())
                .unwrap(),
            TypeName::Single("Bar".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "unknown": { "foo": "bar" }})).unwrap())
                .unwrap(),
            TypeName::Single("Bar".to_string())
        );
    }

    #[test]
    fn test_single_distinct_field_optional_and_shared_fields() {
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
        let types = vec![("Foo", &foo), ("Bar", &bar)];

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

        // ambiguous cases
        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "foo": "test", "bar": "test" })).unwrap())
                .unwrap(),
            TypeName::Single("Foo".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({})).unwrap())
                .unwrap(),
            TypeName::Single("Foo".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "unknown": { "foo": "bar" }})).unwrap())
                .unwrap(),
            TypeName::Single("Foo".to_string())
        );

        // ambiguous cases
        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "foo": "test", "bar": "test" })).unwrap())
                .unwrap(),
            TypeName::Single("Foo".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({})).unwrap())
                .unwrap(),
            TypeName::Single("Foo".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "unknown": { "foo": "bar" }})).unwrap())
                .unwrap(),
            TypeName::Single("Foo".to_string())
        );
    }

    #[test]
    fn test_multiple_distinct_fields() {
        let foo = Type::default().fields(vec![
            ("a", Field::default()),
            ("b", Field::default()),
            ("foo", Field::default()),
        ]);
        let bar = Type::default().fields(vec![("bar", Field::default())]);
        let types = vec![("Foo", &foo), ("Bar", &bar)];

        let discriminator = Discriminator::new(types).unwrap();

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "b": 123, "foo": "test" })).unwrap())
                .unwrap(),
            TypeName::Single("Foo".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "bar": "test" })).unwrap())
                .unwrap(),
            TypeName::Single("Bar".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "unknown": { "foo": "bar" }, "a": 1 })).unwrap()
                )
                .unwrap(),
            TypeName::Single("Foo".to_string())
        );

        // ambiguous cases
        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "foo": "test", "bar": "test" })).unwrap())
                .unwrap(),
            TypeName::Single("Foo".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({})).unwrap())
                .unwrap(),
            TypeName::Single("Foo".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "unknown": { "foo": "bar" }})).unwrap())
                .unwrap(),
            TypeName::Single("Foo".to_string())
        );
    }

    #[test]
    fn test_fields_intersection() {
        let a = Type::default().fields(vec![
            ("shared", Field::default()),
            ("a", Field::default()),
            ("aa", Field::default()),
            ("aaa", Field::default()),
        ]);
        let b = Type::default().fields(vec![
            ("shared", Field::default()),
            ("b", Field::default()),
            ("aa", Field::default()),
        ]);
        let c = Type::default().fields(vec![
            ("shared", Field::default()),
            ("c", Field::default()),
            ("aaa", Field::default()),
        ]);
        let types = vec![("A", &a), ("B", &b), ("C", &c)];

        let discriminator = Discriminator::new(types).unwrap();

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "a": 1 })).unwrap())
                .unwrap(),
            TypeName::Single("A".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "b": 1, "aa": 1 })).unwrap())
                .unwrap(),
            TypeName::Single("B".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "c": 1, "aaa": 1 })).unwrap())
                .unwrap(),
            TypeName::Single("C".to_string())
        );

        // ambiguous cases
        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "shared": 1, "a": 1, "b": 1, "c": 1 })).unwrap()
                )
                .unwrap(),
            TypeName::Single("A".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({})).unwrap())
                .unwrap(),
            TypeName::Single("A".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "unknown": { "foo": "bar" }})).unwrap())
                .unwrap(),
            TypeName::Single("A".to_string())
        );
    }

    #[test]
    fn test_fields_protobuf_oneof() {
        let var_var = Type::default().fields(vec![("usual", Field::default())]);
        let var0_var = Type::default().fields(vec![
            ("usual", Field::default()),
            ("payload", Field { required: true, ..Field::default() }),
        ]);
        let var1_var = Type::default().fields(vec![
            ("usual", Field::default()),
            ("command", Field { required: true, ..Field::default() }),
        ]);
        let var_var0 = Type::default().fields(vec![
            ("usual", Field::default()),
            ("flag", Field { required: true, ..Field::default() }),
        ]);
        let var_var1 = Type::default().fields(vec![
            ("usual", Field::default()),
            ("optPayload", Field { required: true, ..Field::default() }),
        ]);
        let var0_var0 = Type::default().fields(vec![
            ("usual", Field::default()),
            ("payload", Field { required: true, ..Field::default() }),
            ("flag", Field { required: true, ..Field::default() }),
        ]);
        let var1_var0 = Type::default().fields(vec![
            ("usual", Field::default()),
            ("command", Field { required: true, ..Field::default() }),
            ("flag", Field { required: true, ..Field::default() }),
        ]);
        let var0_var1 = Type::default().fields(vec![
            ("usual", Field::default()),
            ("payload", Field { required: true, ..Field::default() }),
            ("optPayload", Field { required: true, ..Field::default() }),
        ]);
        let var1_var1 = Type::default().fields(vec![
            ("usual", Field::default()),
            ("command", Field { required: true, ..Field::default() }),
            ("optPayload", Field { required: true, ..Field::default() }),
        ]);
        let types = vec![
            ("Var_Var", &var_var),
            ("Var0_Var", &var0_var),
            ("Var1_Var", &var1_var),
            ("Var_Var0", &var_var0),
            ("Var_Var1", &var_var1),
            ("Var0_Var0", &var0_var0),
            ("Var1_Var0", &var1_var0),
            ("Var0_Var1", &var0_var1),
            ("Var1_Var1", &var1_var1),
        ];

        let discriminator = Discriminator::new(types).unwrap();

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "usual": 1 })).unwrap())
                .unwrap(),
            TypeName::Single("Var_Var".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "usual": 1, "payload": 1 })).unwrap())
                .unwrap(),
            TypeName::Single("Var0_Var".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "usual": 1, "command": 2, "useless": 1 })).unwrap()
                )
                .unwrap(),
            TypeName::Single("Var1_Var".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "usual": 1, "flag": true })).unwrap())
                .unwrap(),
            TypeName::Single("Var_Var0".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "usual": 1, "optPayload": 1, "a": 1, "b": 2 }))
                        .unwrap()
                )
                .unwrap(),
            TypeName::Single("Var_Var1".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "usual": 1, "payload": 1, "flag": true })).unwrap()
                )
                .unwrap(),
            TypeName::Single("Var0_Var0".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "usual": 1, "payload": 1, "optPayload": 1 }))
                        .unwrap()
                )
                .unwrap(),
            TypeName::Single("Var0_Var1".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "usual": 1, "command": 1, "flag": true })).unwrap()
                )
                .unwrap(),
            TypeName::Single("Var1_Var0".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "usual": 1, "command": 1, "optPayload": 1 }))
                        .unwrap()
                )
                .unwrap(),
            TypeName::Single("Var1_Var1".to_string())
        );

        // ambiguous cases
        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "usual": 1, "command": 1, "payload": 1 })).unwrap()
                )
                .unwrap_err()
                .to_string(),
            "Failed to find corresponding type for value"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({})).unwrap())
                .unwrap(),
            TypeName::Single("Var_Var".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "unknown": { "foo": "bar" }})).unwrap())
                .unwrap(),
            TypeName::Single("Var_Var".to_string())
        );
    }

    #[test]
    fn test_additional_types() {
        let type_a = Type::default().fields(vec![
            ("uniqueA1", Field::default()),
            ("common", Field::default()),
        ]);
        let type_b = Type::default().fields(vec![
            ("uniqueB1", Field { required: true, ..Field::default() }),
            ("common", Field::default()),
        ]);
        let type_c = Type::default().fields(vec![
            ("uniqueC1", Field::default()),
            ("uniqueC2", Field::default()),
        ]);
        let type_d = Type::default().fields(vec![
            ("uniqueD1", Field::default()),
            ("common", Field::default()),
            ("uniqueD2", Field { required: true, ..Field::default() }),
        ]);

        let types = vec![
            ("TypeA", &type_a),
            ("TypeB", &type_b),
            ("TypeC", &type_c),
            ("TypeD", &type_d),
        ];

        let discriminator = Discriminator::new(types).unwrap();

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "uniqueA1": "value", "common": 1 })).unwrap()
                )
                .unwrap(),
            TypeName::Single("TypeA".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "uniqueB1": true, "common": 2 })).unwrap())
                .unwrap(),
            TypeName::Single("TypeB".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "uniqueC1": "value1", "uniqueC2": "value2" }))
                        .unwrap()
                )
                .unwrap(),
            TypeName::Single("TypeC".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(
                        json!({ "uniqueD1": "value", "common": 3, "uniqueD2": false })
                    )
                    .unwrap()
                )
                .unwrap(),
            TypeName::Single("TypeD".to_string())
        );

        // ambiguous cases
        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(
                        json!({ "uniqueA1": "value", "uniqueB1": true, "common": 4 })
                    )
                    .unwrap()
                )
                .unwrap(),
            TypeName::Single("TypeA".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({})).unwrap())
                .unwrap(),
            TypeName::Single("TypeA".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "unknown": { "foo": "bar" }})).unwrap())
                .unwrap(),
            TypeName::Single("TypeA".to_string())
        );
    }

    #[test]
    fn test_combination_of_shared_fields() {
        let type_a = Type::default().fields(vec![
            ("field1", Field::default()),
            ("field2", Field::default()),
        ]);
        let type_b = Type::default().fields(vec![
            ("field2", Field::default()),
            ("field3", Field::default()),
        ]);
        let type_c = Type::default().fields(vec![
            ("field1", Field::default()),
            ("field3", Field::default()),
        ]);
        let type_d = Type::default().fields(vec![
            ("field1", Field::default()),
            ("field2", Field::default()),
            ("field4", Field { required: true, ..Field::default() }),
        ]);

        let types = vec![
            ("TypeA", &type_a),
            ("TypeB", &type_b),
            ("TypeC", &type_c),
            ("TypeD", &type_d),
        ];

        let discriminator = Discriminator::new(types).unwrap();

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "field1": "value", "field2": "value" })).unwrap()
                )
                .unwrap(),
            TypeName::Single("TypeA".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "field2": "value", "field3": "value" })).unwrap()
                )
                .unwrap(),
            TypeName::Single("TypeB".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "field1": "value", "field3": "value" })).unwrap()
                )
                .unwrap(),
            TypeName::Single("TypeC".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(
                        json!({ "field1": "value", "field2": "value", "field4": "value" })
                    )
                    .unwrap()
                )
                .unwrap(),
            TypeName::Single("TypeD".to_string())
        );

        // ambiguous cases
        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(
                        json!({ "field1": "value", "field2": "value", "field3": "value" })
                    )
                    .unwrap()
                )
                .unwrap_err()
                .to_string(),
            "Failed to find corresponding type for value"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({})).unwrap())
                .unwrap(),
            TypeName::Single("TypeA".to_string())
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "unknown": { "foo": "bar" }})).unwrap())
                .unwrap(),
            TypeName::Single("TypeA".to_string())
        );
    }
}
