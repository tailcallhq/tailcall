use std::collections::{BTreeMap, HashMap};

use async_graphql::parser::types::ConstDirective;

use crate::core::blueprint::*;
use crate::core::config::{Field, Type};
use crate::core::valid::{Valid, ValidationError, Validator};

/// Validates that all the root type fields has resolver
/// making into the account the nesting
fn validate_type_has_resolvers(
    name: &str,
    ty: &Type,
    types: &BTreeMap<String, Type>,
) -> Valid<(), String> {
    Valid::from_iter(ty.fields.iter(), |(name, field)| {
        validate_field_has_resolver(name, field, types, ty)
    })
    .trace(name)
    .unit()
}

pub fn validate_field_has_resolver(
    name: &str,
    field: &Field,
    types: &BTreeMap<String, Type>,
    parent_ty: &Type,
) -> Valid<(), String> {
    Valid::<(), String>::fail("No resolver has been found in the schema".to_owned())
        .when(|| {
            if types.get(&field.type_of).eq(&Some(parent_ty)) {
                return true;
            }
            if !field.has_resolver() {
                let type_name = &field.type_of;
                if let Some(ty) = types.get(type_name) {
                    if ty.scalar() {
                        return true;
                    }
                    let res = validate_type_has_resolvers(type_name, ty, types);
                    return !res.is_succeed();
                } else {
                    // It's a Scalar
                    return true;
                }
            }
            false
        })
        .trace(name)
}

pub fn to_directive(const_directive: ConstDirective) -> Valid<Directive, String> {
    const_directive
        .arguments
        .into_iter()
        .map(|(k, v)| {
            let value = v.node.into_json();
            if let Ok(value) = value {
                return Ok((k.node.to_string(), value));
            }
            Err(value.unwrap_err())
        })
        .collect::<Result<HashMap<String, serde_json::Value>, _>>()
        .map_err(|e| ValidationError::new(e.to_string()))
        .map(|arguments| Directive {
            name: const_directive.name.node.clone().to_string(),
            arguments,
            index: 0,
        })
        .into()
}
