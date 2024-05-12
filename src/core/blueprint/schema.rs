use std::collections::HashMap;

use async_graphql::parser::types::ConstDirective;

use crate::core::blueprint::*;
use crate::core::config::{Config, Field, Type};
use crate::core::directive::DirectiveCodec;
use crate::core::getter::Getter;
use crate::core::valid::{Valid, ValidationError, Validator};

fn validate_query(config: &Config) -> Valid<(), String> {
    Valid::from_option(
        config.schema.query.clone(),
        "Query root is missing".to_owned(),
    )
    .and_then(|ref query_type_name| {
        let Some(query) = config.types.get(query_type_name) else {
            return Valid::fail("Query type is not defined".to_owned()).trace(query_type_name);
        };

        validate_type_has_resolvers(query_type_name, query, &config.types)
    })
    .unit()
}

/// Validates that all the root type fields has resolver
/// making into the account the nesting
fn validate_type_has_resolvers(name: &str, ty: &Type, types: &Vec<Type>) -> Valid<(), String> {
    Valid::from_iter(ty.fields.iter(), |(name, field)| {
        validate_field_has_resolver(name, field, types, ty)
    })
    .trace(name)
    .unit()
}

pub fn validate_field_has_resolver(
    name: &str,
    field: &Field,
    types: &Vec<Type>,
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

fn validate_mutation(config: &Config) -> Valid<(), String> {
    let mutation_type_name = config.schema.mutation.as_ref();

    if let Some(mutation_type_name) = mutation_type_name {
        let Some(mutation) = config.types.get(mutation_type_name) else {
            return Valid::fail("Mutation type is not defined".to_owned())
                .trace(mutation_type_name);
        };

        validate_type_has_resolvers(mutation_type_name, mutation, &config.types)
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
                "Query root is missing".to_owned(),
            ))
            .zip(to_directive(config.server.to_directive()))
            .map(|(query_type_name, directive)| SchemaDefinition {
                query: query_type_name.to_owned(),
                mutation: config.schema.mutation.clone(),
                directives: vec![directive],
            })
    })
}
