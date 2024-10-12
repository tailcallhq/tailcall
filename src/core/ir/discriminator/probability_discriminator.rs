use std::collections::HashSet;
use std::fmt::Write;

use anyhow::{bail, Result};
use async_graphql::Value;
use derive_more::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};
use indenter::indented;
use indexmap::IndexMap;

use super::TypedValue;
use crate::core::config::Type;
use crate::core::valid::{Cause, Valid, Validator};

/// Resolver for type member of a union or interface.
/// Based on type definitions and the provided value, it can
/// resolve the type of the value.
#[derive(Clone)]
pub struct ProbabilityDiscriminator {
    /// List of all types that are members of the union or interface.
    types: Vec<String>,

    /// Set of all fields that are part of types with
    /// the [FieldInfo] about their relations to types.
    fields_info: IndexMap<String, FieldInfo>,
}

impl ProbabilityDiscriminator {
    pub fn new(union_name: &str, union_types: &[(&str, &Type)]) -> Valid<Self, String> {
        if union_types.len() > usize::BITS as usize {
            return Valid::fail(format!(
                "`{union_name}` defines more than {} types, reaching the maximum limit of types",
                usize::BITS
            ));
        }

        let mut types = Vec::with_capacity(union_types.len());
        let mut fields_info: IndexMap<String, FieldInfo> = IndexMap::new();

        // TODO: do we need to check also added_fields?
        for (i, (type_name, type_)) in union_types.iter().enumerate() {
            types.push(type_name.to_string());
            for (field_name, field) in type_.fields.iter() {
                let info = fields_info.entry(field_name.to_string()).or_default();

                let repr = Repr::from_type_index(i);

                // Add information for this field indicating that it is present in this type.
                info.presented_in |= repr;

                // And information if it is required in this type.
                if !field.type_of.is_nullable() {
                    info.required_in |= repr;
                }
            }
        }

        // Validation to ensure no two types have the same set of fields.
        {
            let mut duplicates = IndexMap::new();

            for (_, type_) in union_types.iter() {
                let mut repr = Repr::all_covered(union_types.len());
                for field_name in type_.fields.keys() {
                    if let Some(info) = fields_info.get(field_name.as_str()) {
                        repr &= info.presented_in;
                    }
                }

                if repr.is_covering_multiple_types() {
                    let types = repr.covered_types(&types);

                    // If every field in this type is also present in some other type,
                    // check if the other types have the same number of fields.
                    let same_types: Vec<_> = types
                        .into_iter()
                        .filter(|type_name| {
                            let other_type = union_types.iter().find(|(name, _)| name == type_name);

                            if let Some((_, other_type)) = other_type {
                                other_type.fields.len() == type_.fields.len()
                            } else {
                                false
                            }
                        })
                        .collect();

                    // One type is already the current type itself.
                    if same_types.len() > 1 {
                        duplicates.insert(same_types[0], same_types);
                    }
                }
            }

            if !duplicates.is_empty() {
                return Valid::from_vec_cause(
                    duplicates
                        .into_iter()
                        .map(|(_, same_types)| {
                            Cause::new(format!(
                                "Union have equal types: {} ",
                                same_types.join(" == ")
                            ))
                        })
                        .collect(),
                )
                .trace(union_name);
            }
        }

        // Strip fields that are not valuable for the discriminator.
        let fields_info = {
            let mut seen_required_in: HashSet<Repr> = HashSet::new();

            fields_info
                .into_iter()
                .filter(|(_, field_info)| {
                    let drop =
                        // If a field is present in all types, it does not help in determining the type of the value.
                        field_info
                        .presented_in
                        .is_covering_all_types(union_types.len())
                        // If multiple fields are required in the same set of types, we can keep only one of these fields.
                        || (!field_info.required_in.is_empty() && seen_required_in.contains(&field_info.required_in));

                    seen_required_in.insert(field_info.required_in);

                    !drop
                })
                .collect()
        };

        let discriminator = Self { fields_info, types };

        tracing::debug!(
            "Generated ProbabilityDiscriminator for type '{union_name}':\n{discriminator:?}",
        );

        Valid::succeed(discriminator)
    }

    pub fn resolve_type(&self, value: &Value) -> Result<String> {
        let Value::Object(obj) = value else {
            bail!("Value expected to be object");
        };

        let mut possible_types = Repr::all_covered(self.types.len());

        for (field, info) in &self.fields_info {
            if obj.contains_key(field.as_str()) {
                possible_types &= info.presented_in;
            } else {
                possible_types &= !info.required_in;
            }

            if possible_types.is_empty() {
                // No possible types. Something is wrong with the resolved value.
                bail!("Failed to find corresponding type for value")
            }

            if !possible_types.is_covering_multiple_types() {
                // We've got only one possible type, so return it,
                // even though the value could be completely wrong if we check other fields.
                // We want to cover positive cases and do it as soon as possible,
                // and the wrong value will likely be incorrect to use later anyway.
                return Ok(possible_types.first_covered_type(&self.types).to_string());
            }
        }

        // We have multiple possible types. Return the first one
        // that is defined earlier in the config.
        Ok(possible_types.first_covered_type(&self.types).to_string())
    }

    pub fn resolve_and_set_type(&self, mut value: Value) -> Result<Value> {
        let type_name = self.resolve_type(&value)?;
        value.set_type_name(type_name)?;
        Ok(value)
    }
}

impl std::fmt::Debug for ProbabilityDiscriminator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ProbabilityDiscriminator {\n")?;
        f.write_str("types: ")?;
        f.write_fmt(format_args!("{:?}\n", &self.types))?;
        f.write_str("fields_info:\n")?;

        {
            let f = &mut indented(f);
            for (field_name, field_info) in &self.fields_info {
                f.write_fmt(format_args!("{field_name}:\n"))?;
                field_info.display_types(&mut indented(f), &self.types)?;
            }
        }

        f.write_str("}\n")?;

        Ok(())
    }
}

/// Represents the relations between a field and a type:
/// - `presented_in` - the field is part of the type definition, regardless of
///   nullability.
/// - `required_in` - the field is part of the type and is non-nullable.
#[derive(Default, Debug, Clone)]
struct FieldInfo {
    presented_in: Repr,
    required_in: Repr,
}

impl FieldInfo {
    /// Displays the [Repr] data inside FieldInfo as type names instead of the
    /// raw underlying representation.
    fn display_types(&self, f: &mut dyn Write, types: &[String]) -> std::fmt::Result {
        f.write_str("presented_in: ")?;
        f.write_fmt(format_args!(
            "{:?}\n",
            self.presented_in.covered_types(types)
        ))?;
        f.write_str("required_in: ")?;
        f.write_fmt(format_args!(
            "{:?}\n",
            self.required_in.covered_types(types)
        ))?;

        Ok(())
    }
}

/// Representation for a set of types if some condition is met.
/// The condition is represented as a bit inside the `usize` number,
/// where the bit position from the right in the binary representation of
/// `usize` is the index of the type in the set. If the value of the bit is
/// 1, then the condition is met.
#[derive(
    Copy,
    Clone,
    Default,
    PartialEq,
    Eq,
    Hash,
    BitAnd,
    BitOr,
    BitXor,
    BitAndAssign,
    BitOrAssign,
    BitXorAssign,
    Not,
)]
struct Repr(usize);

impl std::fmt::Debug for Repr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:0b}", self.0))
    }
}

impl Repr {
    /// Create a new Repr where the condition is met for every type.
    fn all_covered(len: usize) -> Self {
        Self((1 << len) - 1)
    }

    /// Create a new Repr where the condition is met
    /// for the type with the given index.
    fn from_type_index(index: usize) -> Self {
        Self(1 << index)
    }

    /// Search for the first type in the list for which the condition is met.
    fn first_covered_type<'types>(&self, types: &'types [String]) -> &'types str {
        &types[self.0.trailing_zeros() as usize]
    }

    /// Returns a list of all types for which the condition is met.
    fn covered_types<'types>(&self, types: &'types [String]) -> Vec<&'types str> {
        let mut x = *self;
        let mut result = Vec::new();

        while x.0 != 0 {
            result.push(x.first_covered_type(types));

            x.0 = x.0 & (x.0 - 1);
        }

        result
    }

    /// Check if the condition is not met for any type.
    fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// Check if the condition is met for every type.
    fn is_covering_all_types(&self, len: usize) -> bool {
        self.0.trailing_ones() == len as u32
    }

    /// Check if the condition is met for more than one type.
    fn is_covering_multiple_types(&self) -> bool {
        !self.0.is_power_of_two()
    }
}

#[cfg(test)]
mod tests {
    use async_graphql::Value;
    use serde_json::json;
    use test_log::test;

    use super::ProbabilityDiscriminator;
    use crate::core::config::Field;
    use crate::core::valid::Validator;
    use crate::core::{config, Type};

    #[test]
    fn test_single_distinct_field_optional() {
        let foo = config::Type::default().fields(vec![("foo", Field::default())]);
        let bar = config::Type::default().fields(vec![("bar", Field::default())]);
        let types = vec![("Foo", &foo), ("Bar", &bar)];

        let discriminator = ProbabilityDiscriminator::new("Test", &types)
            .to_result()
            .unwrap();

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "foo": "test" })).unwrap())
                .unwrap(),
            "Foo"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "bar": "test" })).unwrap())
                .unwrap(),
            "Bar"
        );

        // ambiguous cases
        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "foo": "test", "bar": "test" })).unwrap())
                .unwrap(),
            "Foo"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({})).unwrap())
                .unwrap(),
            "Foo"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "unknown": { "foo": "bar" }})).unwrap())
                .unwrap(),
            "Foo"
        );
    }

    #[test]
    fn test_single_distinct_field_required() {
        let foo = config::Type::default().fields(vec![(
            "foo",
            Field { type_of: Type::default().into_required(), ..Field::default() },
        )]);
        let bar = config::Type::default().fields(vec![(
            "bar",
            Field { type_of: Type::default().into_required(), ..Field::default() },
        )]);
        let types = vec![("Foo", &foo), ("Bar", &bar)];

        let discriminator = ProbabilityDiscriminator::new("Test", &types)
            .to_result()
            .unwrap();

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "foo": "test" })).unwrap())
                .unwrap(),
            "Foo"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "bar": "test" })).unwrap())
                .unwrap(),
            "Bar"
        );

        // ambiguous cases
        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "foo": "test", "bar": "test" })).unwrap())
                .unwrap(),
            "Foo"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({})).unwrap())
                .unwrap(),
            "Bar"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "unknown": { "foo": "bar" }})).unwrap())
                .unwrap(),
            "Bar"
        );
    }

    #[test]
    fn test_multiple_distinct_field_required() {
        let a = config::Type::default().fields(vec![
            (
                "a",
                Field { type_of: Type::default().into_required(), ..Field::default() },
            ),
            (
                "ab",
                Field { type_of: Type::default().into_required(), ..Field::default() },
            ),
            (
                "abab",
                Field { type_of: Type::default().into_required(), ..Field::default() },
            ),
        ]);
        let b = config::Type::default().fields(vec![
            (
                "b",
                Field { type_of: Type::default().into_required(), ..Field::default() },
            ),
            (
                "ab",
                Field { type_of: Type::default().into_required(), ..Field::default() },
            ),
            (
                "abab",
                Field { type_of: Type::default().into_required(), ..Field::default() },
            ),
            (
                "ac",
                Field { type_of: Type::default().into_required(), ..Field::default() },
            ),
        ]);
        let c = config::Type::default().fields(vec![
            (
                "c",
                Field { type_of: Type::default().into_required(), ..Field::default() },
            ),
            (
                "ac",
                Field { type_of: Type::default().into_required(), ..Field::default() },
            ),
        ]);
        let types = vec![("A", &a), ("B", &b), ("C", &c)];

        let discriminator = ProbabilityDiscriminator::new("Test", &types)
            .to_result()
            .unwrap();

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "a": 1, "ab": 1, "abab": 1 })).unwrap())
                .unwrap(),
            "A"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "b": 1, "ab": 1, "abab": 1 })).unwrap())
                .unwrap(),
            "B"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "c": 1, "ac": 1 })).unwrap())
                .unwrap(),
            "C"
        );

        // ambiguous cases
        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "a": 1, "b": 1, "c": 1 })).unwrap())
                .unwrap(),
            "A"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({})).unwrap())
                .unwrap(),
            "C"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "unknown": { "foo": "bar" }})).unwrap())
                .unwrap(),
            "C"
        );
    }

    #[test]
    fn test_single_distinct_field_optional_and_shared_fields() {
        let foo = config::Type::default().fields(vec![
            ("a", Field::default()),
            ("b", Field::default()),
            ("foo", Field::default()),
        ]);
        let bar = config::Type::default().fields(vec![
            ("a", Field::default()),
            ("b", Field::default()),
            ("bar", Field::default()),
        ]);
        let types = vec![("Foo", &foo), ("Bar", &bar)];

        let discriminator = ProbabilityDiscriminator::new("Test", &types)
            .to_result()
            .unwrap();

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "a": 123, "b": true, "foo": "test" })).unwrap()
                )
                .unwrap(),
            "Foo"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "bar": "test" })).unwrap())
                .unwrap(),
            "Bar"
        );

        // ambiguous cases
        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "foo": "test", "bar": "test" })).unwrap())
                .unwrap(),
            "Foo"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({})).unwrap())
                .unwrap(),
            "Foo"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "unknown": { "foo": "bar" }})).unwrap())
                .unwrap(),
            "Foo"
        );

        // ambiguous cases
        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "foo": "test", "bar": "test" })).unwrap())
                .unwrap(),
            "Foo"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({})).unwrap())
                .unwrap(),
            "Foo"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "unknown": { "foo": "bar" }})).unwrap())
                .unwrap(),
            "Foo"
        );
    }

    #[test]
    fn test_multiple_distinct_fields() {
        let foo = config::Type::default().fields(vec![
            ("a", Field::default()),
            ("b", Field::default()),
            ("foo", Field::default()),
        ]);
        let bar = config::Type::default().fields(vec![("bar", Field::default())]);
        let types = vec![("Foo", &foo), ("Bar", &bar)];

        let discriminator = ProbabilityDiscriminator::new("Test", &types)
            .to_result()
            .unwrap();

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "b": 123, "foo": "test" })).unwrap())
                .unwrap(),
            "Foo"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "bar": "test" })).unwrap())
                .unwrap(),
            "Bar"
        );

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "unknown": { "foo": "bar" }, "a": 1 })).unwrap()
                )
                .unwrap(),
            "Foo"
        );

        // ambiguous cases
        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "foo": "test", "bar": "test" })).unwrap())
                .unwrap(),
            "Foo"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({})).unwrap())
                .unwrap(),
            "Foo"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "unknown": { "foo": "bar" }})).unwrap())
                .unwrap(),
            "Foo"
        );
    }

    #[test]
    fn test_fields_intersection() {
        let a = config::Type::default().fields(vec![
            ("shared", Field::default()),
            ("a", Field::default()),
            ("aa", Field::default()),
            ("aaa", Field::default()),
        ]);
        let b = config::Type::default().fields(vec![
            ("shared", Field::default()),
            ("b", Field::default()),
            ("aa", Field::default()),
        ]);
        let c = config::Type::default().fields(vec![
            ("shared", Field::default()),
            ("c", Field::default()),
            ("aaa", Field::default()),
        ]);
        let types = vec![("A", &a), ("B", &b), ("C", &c)];

        let discriminator = ProbabilityDiscriminator::new("Test", &types)
            .to_result()
            .unwrap();

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "a": 1 })).unwrap())
                .unwrap(),
            "A"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "b": 1, "aa": 1 })).unwrap())
                .unwrap(),
            "B"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "c": 1, "aaa": 1 })).unwrap())
                .unwrap(),
            "C"
        );

        // ambiguous cases
        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "shared": 1, "a": 1, "b": 1, "c": 1 })).unwrap()
                )
                .unwrap(),
            "A"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({})).unwrap())
                .unwrap(),
            "A"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "unknown": { "foo": "bar" }})).unwrap())
                .unwrap(),
            "A"
        );
    }

    #[test]
    fn test_fields_protobuf_oneof() {
        let var_var = config::Type::default().fields(vec![("usual", Field::default())]);
        let var0_var = config::Type::default().fields(vec![
            ("usual", Field::default()),
            (
                "payload",
                Field { type_of: Type::default().into_required(), ..Field::default() },
            ),
        ]);
        let var1_var = config::Type::default().fields(vec![
            ("usual", Field::default()),
            (
                "command",
                Field { type_of: Type::default().into_required(), ..Field::default() },
            ),
        ]);
        let var_var0 = config::Type::default().fields(vec![
            ("usual", Field::default()),
            (
                "flag",
                Field { type_of: Type::default().into_required(), ..Field::default() },
            ),
        ]);
        let var_var1 = config::Type::default().fields(vec![
            ("usual", Field::default()),
            (
                "optPayload",
                Field { type_of: Type::default().into_required(), ..Field::default() },
            ),
        ]);
        let var0_var0 = config::Type::default().fields(vec![
            ("usual", Field::default()),
            (
                "payload",
                Field { type_of: Type::default().into_required(), ..Field::default() },
            ),
            (
                "flag",
                Field { type_of: Type::default().into_required(), ..Field::default() },
            ),
        ]);
        let var1_var0 = config::Type::default().fields(vec![
            ("usual", Field::default()),
            (
                "command",
                Field { type_of: Type::default().into_required(), ..Field::default() },
            ),
            (
                "flag",
                Field { type_of: Type::default().into_required(), ..Field::default() },
            ),
        ]);
        let var0_var1 = config::Type::default().fields(vec![
            ("usual", Field::default()),
            (
                "payload",
                Field { type_of: Type::default().into_required(), ..Field::default() },
            ),
            (
                "optPayload",
                Field { type_of: Type::default().into_required(), ..Field::default() },
            ),
        ]);
        let var1_var1 = config::Type::default().fields(vec![
            ("usual", Field::default()),
            (
                "command",
                Field { type_of: Type::default().into_required(), ..Field::default() },
            ),
            (
                "optPayload",
                Field { type_of: Type::default().into_required(), ..Field::default() },
            ),
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

        let discriminator = ProbabilityDiscriminator::new("Test", &types)
            .to_result()
            .unwrap();

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "usual": 1 })).unwrap())
                .unwrap(),
            "Var_Var"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "usual": 1, "payload": 1 })).unwrap())
                .unwrap(),
            "Var0_Var"
        );

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "usual": 1, "command": 2, "useless": 1 })).unwrap()
                )
                .unwrap(),
            "Var1_Var"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "usual": 1, "flag": true })).unwrap())
                .unwrap(),
            "Var_Var0"
        );

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "usual": 1, "optPayload": 1, "a": 1, "b": 2 }))
                        .unwrap()
                )
                .unwrap(),
            "Var_Var1"
        );

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "usual": 1, "payload": 1, "flag": true })).unwrap()
                )
                .unwrap(),
            "Var0_Var0"
        );

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "usual": 1, "payload": 1, "optPayload": 1 }))
                        .unwrap()
                )
                .unwrap(),
            "Var0_Var1"
        );

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "usual": 1, "command": 1, "flag": true })).unwrap()
                )
                .unwrap(),
            "Var1_Var0"
        );

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "usual": 1, "command": 1, "optPayload": 1 }))
                        .unwrap()
                )
                .unwrap(),
            "Var1_Var1"
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
            "Var_Var"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "unknown": { "foo": "bar" }})).unwrap())
                .unwrap(),
            "Var_Var"
        );
    }

    #[test]
    fn test_additional_types() {
        let type_a = config::Type::default().fields(vec![
            ("uniqueA1", Field::default()),
            ("common", Field::default()),
        ]);
        let type_b = config::Type::default().fields(vec![
            (
                "uniqueB1",
                Field { type_of: Type::default().into_required(), ..Field::default() },
            ),
            ("common", Field::default()),
        ]);
        let type_c = config::Type::default().fields(vec![
            ("uniqueC1", Field::default()),
            ("uniqueC2", Field::default()),
        ]);
        let type_d = config::Type::default().fields(vec![
            ("uniqueD1", Field::default()),
            ("common", Field::default()),
            (
                "uniqueD2",
                Field { type_of: Type::default().into_required(), ..Field::default() },
            ),
        ]);

        let types = vec![
            ("TypeA", &type_a),
            ("TypeB", &type_b),
            ("TypeC", &type_c),
            ("TypeD", &type_d),
        ];

        let discriminator = ProbabilityDiscriminator::new("Test", &types)
            .to_result()
            .unwrap();

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "uniqueA1": "value", "common": 1 })).unwrap()
                )
                .unwrap(),
            "TypeA"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "uniqueB1": true, "common": 2 })).unwrap())
                .unwrap(),
            "TypeB"
        );

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "uniqueC1": "value1", "uniqueC2": "value2" }))
                        .unwrap()
                )
                .unwrap(),
            "TypeC"
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
            "TypeD"
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
            "TypeA"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({})).unwrap())
                .unwrap(),
            "TypeA"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "unknown": { "foo": "bar" }})).unwrap())
                .unwrap(),
            "TypeA"
        );
    }

    #[test]
    fn test_combination_of_shared_fields() {
        let type_a = config::Type::default().fields(vec![
            ("field1", Field::default()),
            ("field2", Field::default()),
        ]);
        let type_b = config::Type::default().fields(vec![
            ("field2", Field::default()),
            ("field3", Field::default()),
        ]);
        let type_c = config::Type::default().fields(vec![
            ("field1", Field::default()),
            ("field3", Field::default()),
        ]);
        let type_d = config::Type::default().fields(vec![
            ("field1", Field::default()),
            ("field2", Field::default()),
            (
                "field4",
                Field { type_of: Type::default().into_required(), ..Field::default() },
            ),
        ]);

        let types = vec![
            ("TypeA", &type_a),
            ("TypeB", &type_b),
            ("TypeC", &type_c),
            ("TypeD", &type_d),
        ];

        let discriminator = ProbabilityDiscriminator::new("Test", &types)
            .to_result()
            .unwrap();

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "field1": "value", "field2": "value" })).unwrap()
                )
                .unwrap(),
            "TypeA"
        );

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "field2": "value", "field3": "value" })).unwrap()
                )
                .unwrap(),
            "TypeB"
        );

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "field1": "value", "field3": "value" })).unwrap()
                )
                .unwrap(),
            "TypeC"
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
            "TypeD"
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
            "TypeA"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "unknown": { "foo": "bar" }})).unwrap())
                .unwrap(),
            "TypeA"
        );
    }

    #[test]
    fn validation_number_of_types() {
        let types: Vec<_> = (0..136)
            .map(|i| (i.to_string(), config::Type::default()))
            .collect();
        let union_types: Vec<_> = types
            .iter()
            .map(|(name, type_)| (name.as_str(), type_))
            .collect();

        assert_eq!(
            ProbabilityDiscriminator::new("BigUnion", &union_types)
                .to_result()
                .unwrap_err()
                .to_string(),
            format!(
                "Validation Error
• Union BigUnion defines more than {} types that is not supported
",
                usize::BITS
            )
        );
    }

    #[test]
    fn test_validation_equal_types() {
        let a =
            config::Type::default().fields(vec![("a", Field::default()), ("b", Field::default())]);
        let b = config::Type::default().fields(vec![
            (
                "a",
                Field { type_of: Type::default().into_required(), ..Field::default() },
            ),
            ("b", Field::default()),
        ]);
        let c =
            config::Type::default().fields(vec![("a", Field::default()), ("b", Field::default())]);
        let d = config::Type::default().fields(vec![
            ("a", Field::default()),
            ("b", Field::default()),
            (
                "c",
                Field { type_of: Type::default().into_required(), ..Field::default() },
            ),
        ]);
        let e =
            config::Type::default().fields(vec![("c", Field::default()), ("d", Field::default())]);
        let f = config::Type::default().fields(vec![
            ("c", Field::default()),
            (
                "d",
                Field { type_of: Type::default().into_required(), ..Field::default() },
            ),
        ]);

        let types = vec![
            ("A", &a),
            ("B", &b),
            ("C", &c),
            ("D", &d),
            ("E", &e),
            ("F", &f),
        ];

        assert_eq!(
            ProbabilityDiscriminator::new("Test", &types)
                .to_result()
                .unwrap_err()
                .to_string(),
            "Validation Error
• Union have equal types: A == B == C  [Test]
• Union have equal types: E == F  [Test]
"
        );
    }

    #[test]
    fn test_validation_non_object() {
        let foo = config::Type::default().fields(vec![("foo", Field::default())]);
        let bar = config::Type::default().fields(vec![("bar", Field::default())]);
        let types = vec![("Foo", &foo), ("Bar", &bar)];

        let discriminator = ProbabilityDiscriminator::new("Test", &types)
            .to_result()
            .unwrap();

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!("string")).unwrap())
                .unwrap_err()
                .to_string(),
            "Value expected to be object"
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!(25)).unwrap())
                .unwrap_err()
                .to_string(),
            "Value expected to be object"
        );
    }
}
