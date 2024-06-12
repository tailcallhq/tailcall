use crate::core::blueprint::FieldDefinition;
use crate::core::config::position::Pos;
use crate::core::config::{self, ConfigModule, Field, GraphQLOperationType};
use crate::core::graphql::RequestTemplate;
use crate::core::helpers;
use crate::core::ir::{IO, IR};
use crate::core::try_fold::TryFold;
use crate::core::valid::{Valid, ValidationError, Validator};

pub fn compile_graphql(
    config: &config::Config,
    operation_type: &config::GraphQLOperationType,
    graphql: &Pos<config::GraphQL>,
) -> Valid<IR, String> {
    let args = graphql.args.as_ref();
    Valid::from_option(
        graphql
            .base_url
            .as_ref()
            .or(config.upstream.base_url.as_ref()),
        "No base URL defined".to_string(),
    )
    .zip(helpers::headers::to_mustache_headers(
        &graphql
            .headers
            .iter()
            .map(|header| header.inner.clone())
            .collect::<Vec<_>>(),
    ))
    .and_then(|(base_url, headers)| {
        Valid::from(
            RequestTemplate::new(
                base_url.inner.to_owned(),
                operation_type,
                &graphql.name,
                args,
                headers,
            )
            .map_err(|e| ValidationError::new(e.to_string())),
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
) -> TryFold<
    'a,
    (
        &'a ConfigModule,
        &'a Pos<Field>,
        &'a Pos<config::Type>,
        &'a str,
    ),
    FieldDefinition,
    String,
> {
    TryFold::<(&ConfigModule, &Pos<Field>, &Pos<config::Type>, &'a str), FieldDefinition, String>::new(
        |(config, field, type_of, _), b_field| {
            let Some(graphql) = &field.graphql else {
                return Valid::succeed(b_field);
            };

            compile_graphql(config, operation_type, graphql).trace(graphql.to_trace_err().as_str())
                .map(|resolver| b_field.resolver(Some(resolver)))
                .and_then(|b_field| b_field.validate_field(type_of, config).trace(graphql.to_trace_err().as_str()).map_to(b_field))
        },
    )
}
