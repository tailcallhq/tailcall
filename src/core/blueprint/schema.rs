use std::collections::{BTreeMap, HashSet};

use directive::to_directive;
use tailcall_valid::{Valid, Validator};

use crate::core::blueprint::*;
use crate::core::config::{Config, Field, Type};
use crate::core::directive::DirectiveCodec;

fn validate_query(config: &Config) -> Valid<(), BlueprintError> {
    Valid::from_option(
        config.schema.query.clone(),
        BlueprintError::QueryRootIsMissing,
    )
    .and_then(|ref query_type_name| {
        let Some(query) = config.find_type(query_type_name) else {
            return Valid::fail(BlueprintError::QueryTypeNotDefined).trace(query_type_name);
        };
        let mut set = HashSet::new();
        validate_type_has_resolvers(query_type_name, query, &config.types, &mut set)
    })
    .unit()
}

/// Validates that all the root type fields has resolver
/// making into the account the nesting
fn validate_type_has_resolvers(
    name: &str,
    ty: &Type,
    types: &BTreeMap<String, Type>,
    visited: &mut HashSet<String>,
) -> Valid<(), BlueprintError> {
    if ty.scalar() || visited.contains(name) {
        return Valid::succeed(());
    }

    visited.insert(name.to_string());

    Valid::from_iter(ty.fields.iter(), |(name, field)| {
        validate_field_has_resolver(name, field, types, visited)
    })
    .trace(name)
    .unit()
}

pub fn validate_field_has_resolver(
    name: &str,
    field: &Field,
    types: &BTreeMap<String, Type>,
    visited: &mut HashSet<String>,
) -> Valid<(), BlueprintError> {
    Valid::<(), BlueprintError>::fail(BlueprintError::NoResolverFoundInSchema)
        .when(|| {
            if !field.has_resolver() {
                let type_name = field.type_of.name();
                if let Some(ty) = types.get(type_name) {
                    let res = validate_type_has_resolvers(type_name, ty, types, visited);
                    return !res.is_succeed();
                }

                return true;
            }
            false
        })
        .trace(name)
}

fn validate_mutation(config: &Config) -> Valid<(), BlueprintError> {
    let mutation_type_name = config.schema.mutation.as_ref();

    if let Some(mutation_type_name) = mutation_type_name {
        let Some(mutation) = config.find_type(mutation_type_name) else {
            return Valid::fail(BlueprintError::MutationTypeNotDefined).trace(mutation_type_name);
        };
        let mut set = HashSet::new();
        validate_type_has_resolvers(mutation_type_name, mutation, &config.types, &mut set)
    } else {
        Valid::succeed(())
    }
}

pub fn to_schema<'a>() -> TryFoldConfig<'a, SchemaDefinition> {
    TryFoldConfig::new(|config, _| {
        validate_query(config)
            .and(validate_mutation(config))
            .and(Valid::from_option(
                config.schema.query.as_ref(),
                BlueprintError::QueryRootIsMissing,
            ))
            .zip(to_directive(config.server.to_directive()))
            .map(|(query_type_name, directive)| SchemaDefinition {
                query: query_type_name.to_owned(),
                mutation: config.schema.mutation.clone(),
                directives: vec![directive],
            })
    })
}
