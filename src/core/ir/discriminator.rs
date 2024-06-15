use std::collections::HashSet;
use std::fmt::Write;

use anyhow::{bail, Result};
use async_graphql::Value;
use derive_more::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};
use indenter::indented;
use indexmap::IndexMap;

use crate::core::config::Type;
use crate::core::valid::{Cause, Valid, Validator};

/// Represents the type name for the resolved value.
/// It is used when the GraphQL executor needs to resolve values of a union
/// type. In order to select the correct fields, the executor must know the
/// exact type name for each resolved value. And in cases when the output is
/// list of union type it should resolve exact type for every entry in list.
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum TypeName {
    Single(&'static str),
    Vec(Vec<&'static str>),
}

/// Resolver for type member of union.
/// Based on types definitions and provided value can
/// resolve what's the type of the value
///
/// ## Resolution algorithm
///
/// The resolution algorithm is based on following points:
/// - common set of fields it's a set of all fields that are defined in type
///   members of union
/// - if resolved value is a list then the resolution should be run for every
///   entry in list as for separate value
/// - if field from common set if presented in resolved value then the result
///   type is one of the types that have this field
/// - if field from common set is required in some types and this field is not
///   present in resolved value then the result type is not one of those types
/// - by repeating checks from above for every field in common set we will end
///   up with smaller set of possible types and even, what is more likely, with
///   only single possible type
#[derive(Clone)]
pub struct Discriminator {
    /// List of all types that are member of Union
    types: Vec<&'static str>,
    /// Set of all fields that are part of types with
    /// the [FieldInfo] about its relations to types
    fields_info: IndexMap<String, FieldInfo>,
}

impl std::fmt::Debug for Discriminator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Discriminator {\n")?;
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

/// Represents relations between field and type
/// - `presented_in` - the field is a part of the type definition despite
///   nullability
/// - `required_in` - the field is a part of the type and it's non-nullable
#[derive(Default, Debug, Clone)]
struct FieldInfo {
    presented_in: Repr,
    required_in: Repr,
}

impl FieldInfo {
    /// Displays the [Repr] data inside FieldInfo as type names instead of raw
    /// underlying representation
    fn display_types(&self, f: &mut dyn Write, types: &[&'static str]) -> std::fmt::Result {
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

impl Discriminator {
    pub fn new(union_name: &str, union_types: &[(&str, &Type)]) -> Valid<Self, String> {
        if union_types.len() > usize::BITS as usize {
            return Valid::fail(format!(
                "Union {union_name} defines more than {} types that is not supported",
                usize::BITS
            ));
        }

        let mut types: Vec<&'static str> = Vec::with_capacity(union_types.len());
        let mut fields_info: IndexMap<String, FieldInfo> = IndexMap::new();

        // TODO: do we need to check also added_fields?
        for (i, (type_name, type_)) in union_types.iter().enumerate() {
            types.push(type_name.to_string().leak());
            for (field_name, field) in type_.fields.iter() {
                let info = fields_info.entry(field_name.to_string()).or_default();

                let repr = Repr::from_type_index(i);

                // add info for this field that it's presented in this type
                info.presented_in |= repr;

                // and info if it's required in this type
                if field.required {
                    info.required_in |= repr;
                }
            }
        }

        // validation for the same set of fields in types
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

                    // if for every field in this type some other type also have same field set
                    // check if other types have same number of fields
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

                    // one types is already the current type itself
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

        // strip some fields that are not valuable for discriminator
        let fields_info = {
            let mut seen_required_in: HashSet<Repr> = HashSet::new();

            fields_info
                .into_iter()
                .filter(|(_, field_info)| {
                    let drop =
                        // if field is presented in all types then it doesn't help in figuring out the type of value
                        field_info
                        .presented_in
                        .is_covering_all_types(union_types.len())
                        // if multiple fields are required in the same set of types than we can leave only one of such fields
                        || (!field_info.required_in.is_empty() && seen_required_in.contains(&field_info.required_in));

                    seen_required_in.insert(field_info.required_in);

                    !drop
                })
                .collect()
        };

        let discriminator = Self { fields_info, types };

        tracing::debug!(
            "Generated discriminator for union type '{union_name}':\n{discriminator:?}",
        );

        Valid::succeed(discriminator)
    }

    pub fn resolve_type(&self, value: &Value) -> Result<TypeName> {
        if let Value::List(list) = value {
            let results: Result<Vec<_>> = list
                .iter()
                .map(|item| self.resolve_type_for_single(item))
                .collect();

            Ok(TypeName::Vec(results?))
        } else {
            Ok(TypeName::Single(self.resolve_type_for_single(value)?))
        }
    }

    fn resolve_type_for_single(&self, value: &Value) -> Result<&'static str> {
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
                // no possible types, something is wrong with the resolved value
                bail!("Failed to find corresponding type for value")
            }

            if !possible_types.is_covering_multiple_types() {
                // we've got only one possible type so return it
                // even despite the fact the value could be totally wrong if we check other
                // fields. But we want to cover positive cases and we want to do
                // it as soon as possible and the wrong value will be probably
                // wrong anyway to use later
                return Ok(possible_types.first_covered_type(&self.types));
            }
        }

        // we have multiple possible types. Return the first one
        // that is defined earlier in config
        Ok(possible_types.first_covered_type(&self.types))
    }
}

/// Representation for set of types if some condition is hold.
/// The condition is represented as a bit inside the usize number
/// where bit position from the right in binary representation of usize
/// is the index of type in the set and the if the value of the bit is
/// 1 then the condition is hold
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
    /// Create new Repr where condition is hold for every type
    fn all_covered(len: usize) -> Self {
        Self((1 << len) - 1)
    }

    /// Create new Repr where condition is hold
    /// for type with passed index
    fn from_type_index(index: usize) -> Self {
        Self(1 << index)
    }

    /// Search for first type in list for which condition is hold
    fn first_covered_type(&self, types: &[&'static str]) -> &'static str {
        types[self.0.trailing_zeros() as usize]
    }

    /// Returns list of all types for which condition is hold
    fn covered_types(&self, types: &[&'static str]) -> Vec<&'static str> {
        let mut x = *self;
        let mut result = Vec::new();

        while x.0 != 0 {
            result.push(x.first_covered_type(types));

            x.0 = x.0 & (x.0 - 1);
        }

        result
    }

    /// Check if the condition is not hold for any type
    fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// Check if condition is hold for every type
    fn is_covering_all_types(&self, len: usize) -> bool {
        self.0.trailing_ones() == len as u32
    }

    /// Check if condition is hold for more than 1 type
    fn is_covering_multiple_types(&self) -> bool {
        !self.0.is_power_of_two()
    }
}

#[cfg(test)]
mod tests {
    use async_graphql::Value;
    use serde_json::json;
    use test_log::test;

    use super::Discriminator;
    use crate::core::config::{Field, Type};
    use crate::core::ir::discriminator::TypeName;

    #[test]
    fn test_single_distinct_field_optional() {
        let foo = Type::default().fields(vec![("foo", Field::default())]);
        let bar = Type::default().fields(vec![("bar", Field::default())]);
        let types = vec![("Foo", &foo), ("Bar", &bar)];

        let discriminator = Discriminator::new("Test", &types).unwrap();

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "foo": "test" })).unwrap())
                .unwrap(),
            TypeName::Single("Foo")
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "bar": "test" })).unwrap())
                .unwrap(),
            TypeName::Single("Bar")
        );

        // ambiguous cases
        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "foo": "test", "bar": "test" })).unwrap())
                .unwrap(),
            TypeName::Single("Foo")
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({})).unwrap())
                .unwrap(),
            TypeName::Single("Foo")
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "unknown": { "foo": "bar" }})).unwrap())
                .unwrap(),
            TypeName::Single("Foo")
        );
    }

    #[test]
    fn test_single_distinct_field_required() {
        let foo =
            Type::default().fields(vec![("foo", Field { required: true, ..Field::default() })]);
        let bar =
            Type::default().fields(vec![("bar", Field { required: true, ..Field::default() })]);
        let types = vec![("Foo", &foo), ("Bar", &bar)];

        let discriminator = Discriminator::new("Test", &types).unwrap();

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "foo": "test" })).unwrap())
                .unwrap(),
            TypeName::Single("Foo")
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "bar": "test" })).unwrap())
                .unwrap(),
            TypeName::Single("Bar")
        );

        // ambiguous cases
        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "foo": "test", "bar": "test" })).unwrap())
                .unwrap(),
            TypeName::Single("Foo")
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({})).unwrap())
                .unwrap(),
            TypeName::Single("Bar")
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "unknown": { "foo": "bar" }})).unwrap())
                .unwrap(),
            TypeName::Single("Bar")
        );
    }

    #[test]
    fn test_multiple_distinct_field_required() {
        let a = Type::default().fields(vec![
            ("a", Field { required: true, ..Field::default() }),
            ("ab", Field { required: true, ..Field::default() }),
            ("abab", Field { required: true, ..Field::default() }),
        ]);
        let b = Type::default().fields(vec![
            ("b", Field { required: true, ..Field::default() }),
            ("ab", Field { required: true, ..Field::default() }),
            ("abab", Field { required: true, ..Field::default() }),
            ("ac", Field { required: true, ..Field::default() }),
        ]);
        let c = Type::default().fields(vec![
            ("c", Field { required: true, ..Field::default() }),
            ("ac", Field { required: true, ..Field::default() }),
        ]);
        let types = vec![("A", &a), ("B", &b), ("C", &c)];

        let discriminator = Discriminator::new("Test", &types).unwrap();

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "a": 1, "ab": 1, "abab": 1 })).unwrap())
                .unwrap(),
            TypeName::Single("A")
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "b": 1, "ab": 1, "abab": 1 })).unwrap())
                .unwrap(),
            TypeName::Single("B")
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "c": 1, "ac": 1 })).unwrap())
                .unwrap(),
            TypeName::Single("C")
        );

        // ambiguous cases
        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "a": 1, "b": 1, "c": 1 })).unwrap())
                .unwrap(),
            TypeName::Single("A")
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({})).unwrap())
                .unwrap(),
            TypeName::Single("C")
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "unknown": { "foo": "bar" }})).unwrap())
                .unwrap(),
            TypeName::Single("C")
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

        let discriminator = Discriminator::new("Test", &types).unwrap();

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "a": 123, "b": true, "foo": "test" })).unwrap()
                )
                .unwrap(),
            TypeName::Single("Foo")
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "bar": "test" })).unwrap())
                .unwrap(),
            TypeName::Single("Bar")
        );

        // ambiguous cases
        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "foo": "test", "bar": "test" })).unwrap())
                .unwrap(),
            TypeName::Single("Foo")
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({})).unwrap())
                .unwrap(),
            TypeName::Single("Foo")
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "unknown": { "foo": "bar" }})).unwrap())
                .unwrap(),
            TypeName::Single("Foo")
        );

        // ambiguous cases
        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "foo": "test", "bar": "test" })).unwrap())
                .unwrap(),
            TypeName::Single("Foo")
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({})).unwrap())
                .unwrap(),
            TypeName::Single("Foo")
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "unknown": { "foo": "bar" }})).unwrap())
                .unwrap(),
            TypeName::Single("Foo")
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

        let discriminator = Discriminator::new("Test", &types).unwrap();

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "b": 123, "foo": "test" })).unwrap())
                .unwrap(),
            TypeName::Single("Foo")
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "bar": "test" })).unwrap())
                .unwrap(),
            TypeName::Single("Bar")
        );

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "unknown": { "foo": "bar" }, "a": 1 })).unwrap()
                )
                .unwrap(),
            TypeName::Single("Foo")
        );

        // ambiguous cases
        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "foo": "test", "bar": "test" })).unwrap())
                .unwrap(),
            TypeName::Single("Foo")
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({})).unwrap())
                .unwrap(),
            TypeName::Single("Foo")
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "unknown": { "foo": "bar" }})).unwrap())
                .unwrap(),
            TypeName::Single("Foo")
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

        let discriminator = Discriminator::new("Test", &types).unwrap();

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "a": 1 })).unwrap())
                .unwrap(),
            TypeName::Single("A")
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "b": 1, "aa": 1 })).unwrap())
                .unwrap(),
            TypeName::Single("B")
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "c": 1, "aaa": 1 })).unwrap())
                .unwrap(),
            TypeName::Single("C")
        );

        // ambiguous cases
        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "shared": 1, "a": 1, "b": 1, "c": 1 })).unwrap()
                )
                .unwrap(),
            TypeName::Single("A")
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({})).unwrap())
                .unwrap(),
            TypeName::Single("A")
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "unknown": { "foo": "bar" }})).unwrap())
                .unwrap(),
            TypeName::Single("A")
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

        let discriminator = Discriminator::new("Test", &types).unwrap();

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "usual": 1 })).unwrap())
                .unwrap(),
            TypeName::Single("Var_Var")
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "usual": 1, "payload": 1 })).unwrap())
                .unwrap(),
            TypeName::Single("Var0_Var")
        );

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "usual": 1, "command": 2, "useless": 1 })).unwrap()
                )
                .unwrap(),
            TypeName::Single("Var1_Var")
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "usual": 1, "flag": true })).unwrap())
                .unwrap(),
            TypeName::Single("Var_Var0")
        );

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "usual": 1, "optPayload": 1, "a": 1, "b": 2 }))
                        .unwrap()
                )
                .unwrap(),
            TypeName::Single("Var_Var1")
        );

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "usual": 1, "payload": 1, "flag": true })).unwrap()
                )
                .unwrap(),
            TypeName::Single("Var0_Var0")
        );

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "usual": 1, "payload": 1, "optPayload": 1 }))
                        .unwrap()
                )
                .unwrap(),
            TypeName::Single("Var0_Var1")
        );

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "usual": 1, "command": 1, "flag": true })).unwrap()
                )
                .unwrap(),
            TypeName::Single("Var1_Var0")
        );

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "usual": 1, "command": 1, "optPayload": 1 }))
                        .unwrap()
                )
                .unwrap(),
            TypeName::Single("Var1_Var1")
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
            TypeName::Single("Var_Var")
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "unknown": { "foo": "bar" }})).unwrap())
                .unwrap(),
            TypeName::Single("Var_Var")
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

        let discriminator = Discriminator::new("Test", &types).unwrap();

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "uniqueA1": "value", "common": 1 })).unwrap()
                )
                .unwrap(),
            TypeName::Single("TypeA")
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "uniqueB1": true, "common": 2 })).unwrap())
                .unwrap(),
            TypeName::Single("TypeB")
        );

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "uniqueC1": "value1", "uniqueC2": "value2" }))
                        .unwrap()
                )
                .unwrap(),
            TypeName::Single("TypeC")
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
            TypeName::Single("TypeD")
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
            TypeName::Single("TypeA")
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({})).unwrap())
                .unwrap(),
            TypeName::Single("TypeA")
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "unknown": { "foo": "bar" }})).unwrap())
                .unwrap(),
            TypeName::Single("TypeA")
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

        let discriminator = Discriminator::new("Test", &types).unwrap();

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "field1": "value", "field2": "value" })).unwrap()
                )
                .unwrap(),
            TypeName::Single("TypeA")
        );

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "field2": "value", "field3": "value" })).unwrap()
                )
                .unwrap(),
            TypeName::Single("TypeB")
        );

        assert_eq!(
            discriminator
                .resolve_type(
                    &Value::from_json(json!({ "field1": "value", "field3": "value" })).unwrap()
                )
                .unwrap(),
            TypeName::Single("TypeC")
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
            TypeName::Single("TypeD")
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
            TypeName::Single("TypeA")
        );

        assert_eq!(
            discriminator
                .resolve_type(&Value::from_json(json!({ "unknown": { "foo": "bar" }})).unwrap())
                .unwrap(),
            TypeName::Single("TypeA")
        );
    }

    #[test]
    fn validation_number_of_types() {
        let types: Vec<_> = (0..136).map(|i| (i.to_string(), Type::default())).collect();
        let union_types: Vec<_> = types
            .iter()
            .map(|(name, type_)| (name.as_str(), type_))
            .collect();

        assert_eq!(
            Discriminator::new("BigUnion", &union_types)
                .unwrap_err()
                .to_string(),
            "Validation Error
• Union BigUnion defines more than 64 types that is not supported
"
        );
    }

    #[test]
    fn test_validation_equal_types() {
        let a = Type::default().fields(vec![("a", Field::default()), ("b", Field::default())]);
        let b = Type::default().fields(vec![
            ("a", Field { required: true, ..Field::default() }),
            ("b", Field::default()),
        ]);
        let c = Type::default().fields(vec![("a", Field::default()), ("b", Field::default())]);
        let d = Type::default().fields(vec![
            ("a", Field::default()),
            ("b", Field::default()),
            ("c", Field { required: true, ..Field::default() }),
        ]);
        let e = Type::default().fields(vec![("c", Field::default()), ("d", Field::default())]);
        let f = Type::default().fields(vec![
            ("c", Field::default()),
            ("d", Field { required: true, ..Field::default() }),
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
            Discriminator::new("Test", &types).unwrap_err().to_string(),
            "Validation Error
• Union have equal types: A == B == C  [Test]
• Union have equal types: E == F  [Test]
"
        );
    }
}
