use std::collections::{HashMap, HashSet};

use crate::core::blueprint::FieldDefinition;
use crate::core::config::{
    Config, ConfigModule, Field, GraphQL, GraphQLOperationType, Resolver, Type,
};
use crate::core::graphql::RequestTemplate;
use crate::core::helpers;
use crate::core::ir::model::{IO, IR};
use crate::core::ir::RelatedFields;
use crate::core::try_fold::TryFold;
use crate::core::valid::{Valid, ValidationError, Validator};

fn create_related_fields(
    config: &Config,
    type_name: &str,
    visited: &mut HashSet<String>,
) -> RelatedFields {
    let mut map = HashMap::new();
    if visited.contains(type_name) {
        return RelatedFields(map);
    }
    visited.insert(type_name.to_string());

    if let Some(type_) = config.find_type(type_name) {
        for (name, field) in &type_.fields {
            if !field.has_resolver() {
                if let Some(modify) = &field.modify {
                    if let Some(modified_name) = &modify.name {
                        map.insert(
                            modified_name.clone(),
                            (
                                name.clone(),
                                create_related_fields(config, field.type_of.name(), visited),
                            ),
                        );
                    }
                } else {
                    map.insert(
                        name.clone(),
                        (
                            name.clone(),
                            create_related_fields(config, field.type_of.name(), visited),
                        ),
                    );
                }
            }
        }
    } else if let Some(union_) = config.find_union(type_name) {
        for type_name in &union_.types {
            map.extend(create_related_fields(config, type_name, visited).0);
        }
    };

    RelatedFields(map)
}

pub fn compile_graphql(
    config: &ConfigModule,
    operation_type: &GraphQLOperationType,
    type_name: &str,
    graphql: &GraphQL,
) -> Valid<IR, String> {
    let args = graphql.args.as_ref();
    Valid::from_option(
        graphql
            .base_url
            .as_ref()
            .or(config.upstream.base_url.as_ref()),
        "No base URL defined".to_string(),
    )
    .zip(helpers::headers::to_mustache_headers(&graphql.headers))
    .and_then(|(base_url, headers)| {
        Valid::from(
            RequestTemplate::new(
                base_url.to_owned(),
                operation_type,
                &graphql.name,
                args,
                headers,
                create_related_fields(config, type_name, &mut HashSet::new()),
            )
            .map_err(|e| ValidationError::new(e.to_string())),
        )
    })
    .map(|req_template| {
        let field_name = graphql.name.clone();
        let batch = graphql.batch;
        let dedupe = graphql.dedupe.unwrap_or_default();
        IR::IO(IO::GraphQL { req_template, field_name, batch, dl_id: None, dedupe })
    })
}

pub fn update_graphql<'a>(
    operation_type: &'a GraphQLOperationType,
) -> TryFold<'a, (&'a ConfigModule, &'a Field, &'a Type, &'a str), FieldDefinition, String> {
    TryFold::<(&ConfigModule, &Field, &Type, &'a str), FieldDefinition, String>::new(
        |(config, field, type_of, _), b_field| {
            let Some(Resolver::Graphql(graphql)) = &field.resolver else {
                return Valid::succeed(b_field);
            };

            compile_graphql(config, operation_type, field.type_of.name(), graphql)
                .map(|resolver| b_field.resolver(Some(resolver)))
                .and_then(|b_field| b_field.validate_field(type_of, config).map_to(b_field))
        },
    )
}
