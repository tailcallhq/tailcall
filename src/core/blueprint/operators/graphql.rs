use std::collections::HashMap;

use crate::core::blueprint::FieldDefinition;
use crate::core::config::position::Pos;
use crate::core::config::{Config, ConfigModule, Field, GraphQL, GraphQLOperationType, Type};
use crate::core::directive::DirectiveCodec;
use crate::core::graphql::RequestTemplate;
use crate::core::helpers;
use crate::core::ir::model::{IO, IR};
use crate::core::ir::RelatedFields;
use crate::core::try_fold::TryFold;
use crate::core::valid::{Valid, ValidationError, Validator};

fn create_related_fields(config: &Config, type_name: &str) -> RelatedFields {
    let mut map = HashMap::new();

    if let Some(type_) = config.find_type(type_name) {
        for (name, field) in &type_.fields {
            if !field.has_resolver() {
                map.insert(name.clone(), create_related_fields(config, &field.type_of));
            }
        }
    } else if let Some(union_) = config.find_union(type_name) {
        for type_name in &union_.types {
            map.extend(create_related_fields(config, type_name).0);
        }
    };

    RelatedFields(map)
}

pub fn compile_graphql(
    config: &Config,
    operation_type: &GraphQLOperationType,
    type_name: &str,
    graphql: &Pos<GraphQL>,
) -> Valid<IR, String> {
    let args = graphql.args.as_ref();
    Valid::from_option(
        graphql
            .base_url
            .as_ref()
            .or(config.upstream.base_url.as_ref()),
        "No base URL defined".to_string(),
    )
    .trace(graphql.to_pos_trace_err(GraphQL::trace_name()).as_deref())
    .zip(helpers::headers::to_mustache_headers(
        graphql.headers.as_ref(),
    ))
    .and_then(|(base_url, headers)| {
        Valid::from(
            RequestTemplate::new(
                base_url.inner.to_owned(),
                operation_type,
                &graphql.name,
                args.map(|args| args.inner.as_ref()),
                headers,
                create_related_fields(config, type_name),
            )
            .map_err(|e| {
                ValidationError::new(e.to_string())
                    .trace(graphql.to_pos_trace_err(GraphQL::trace_name()).as_deref())
            }),
        )
    })
    .map(|req_template| {
        let field_name = graphql.name.inner.clone();
        let batch = graphql.batch.inner;
        IR::IO(IO::GraphQL { req_template, field_name, batch, dl_id: None })
    })
}

pub fn update_graphql<'a>(
    operation_type: &'a GraphQLOperationType,
) -> TryFold<'a, (&'a ConfigModule, &'a Pos<Field>, &'a Pos<Type>, &'a str), FieldDefinition, String>
{
    TryFold::<(&ConfigModule, &Pos<Field>, &Pos<Type>, &'a str), FieldDefinition, String>::new(
        |(config, field, type_of, _), b_field| {
            let Some(graphql) = &field.graphql else {
                return Valid::succeed(b_field);
            };

            compile_graphql(config, operation_type, &field.type_of, graphql)
                .map(|resolver| b_field.resolver(Some(resolver)))
                .and_then(|b_field| {
                    b_field
                        .validate_field(type_of, field, config)
                        .map_to(b_field)
                })
        },
    )
}
