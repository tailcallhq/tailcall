use std::collections::HashMap;

use async_graphql::parser::types::ConstDirective;

use crate::config::{Config, Field};
use crate::directive::DirectiveCodec;
use crate::valid::{Valid, ValidationError, Validator};
use crate::{
    blueprint::*,
    config::{ObjectType, TypeKind},
};

fn validate_query(config: &Config) -> Valid<(), String> {
    Valid::from_option(
        config.schema.query.clone(),
        "Query root is missing".to_owned(),
    )
    .and_then(|ref query_type_name| {
        let Some(query) = config.find_type(query_type_name) else {
            return Valid::fail("Query type is not defined".to_owned()).trace(query_type_name);
        };

        if let TypeKind::Object(query) = &query.kind {
            validate_type_has_resolvers(query_type_name, query, &config)
        } else {
            return Valid::fail("Query is not an object type".to_owned()).trace(query_type_name);
        }
    })
    .unit()
}

/// Validates that all the root type fields has resolver
/// making into the account the nesting
fn validate_type_has_resolvers(name: &str, ty: &ObjectType, config: &Config) -> Valid<(), String> {
    Valid::from_iter(ty.fields.iter(), |(name, field)| {
        validate_field_has_resolver(name, field, config, ty)
    })
    .trace(name)
    .unit()
}

pub fn validate_field_has_resolver(
    name: &str,
    field: &Field,
    config: &Config,
    parent_ty: &ObjectType,
) -> Valid<(), String> {
    Valid::<(), String>::fail("No resolver has been found in the schema".to_owned())
        .when(|| {
            // TODO: type of the field is equal to parent type? is that recursive type? how should it work?
            // if config.find_type(&field.type_of).eq(&Some(parent_ty)) {
            //     return true;
            // }
            if !field.has_resolver() {
                let type_name = &field.type_of;
                if let Some(ty) = config.find_type(type_name) {
                    match &ty.kind {
                        TypeKind::Scalar | TypeKind::Enum(_) => true,
                        TypeKind::Object(ty) => {
                            let res = validate_type_has_resolvers(type_name, ty, config);
                            !res.is_succeed()
                        }
                        TypeKind::Union(_) => todo!(),
                    }
                } else {
                    // It's a built-in Scalar
                    true
                }
            } else {
                false
            }
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
        let Some(mutation) = config.find_type(mutation_type_name) else {
            return Valid::fail("Mutation type is not defined".to_owned())
                .trace(mutation_type_name);
        };

        if let TypeKind::Object(mutation) = &mutation.kind {
            validate_type_has_resolvers(&mutation_type_name, mutation, &config)
        } else {
            return Valid::fail("Mutation is not an object type".to_owned())
                .trace(mutation_type_name);
        }
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
