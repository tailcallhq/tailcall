use crate::blueprint::FieldDefinition;
use crate::config::{self, Config, Field, GraphQLOperationType};
use crate::graphql::RequestTemplate;
use crate::helpers;
use crate::lambda::{Expression, Lambda};
use crate::try_fold::TryFold;
use crate::valid::{Valid, ValidationError};

pub fn compile_graphql(
  config: &config::Config,
  operation_type: &config::GraphQLOperationType,
  graphql: &config::GraphQL,
) -> Valid<Expression, String> {
  let args = graphql.args.as_ref();
  Valid::from_option(
    graphql.base_url.as_ref().or(config.upstream.base_url.as_ref()),
    "No base URL defined".to_string(),
  )
  .zip(helpers::headers::to_headervec(&graphql.headers))
  .and_then(|(base_url, headers)| {
    Valid::from(
      RequestTemplate::new(base_url.to_owned(), operation_type, &graphql.name, args, headers)
        .map_err(|e| ValidationError::new(e.to_string())),
    )
  })
  .map(|req_template| {
    let field_name = graphql.name.clone();
    Lambda::from_graphql_request_template(req_template, field_name, graphql.batch).expression
  })
}

pub fn update_graphql<'a>(
  operation_type: &'a GraphQLOperationType,
) -> TryFold<'a, (&'a Config, &'a Field, &'a config::Type, &'a str), FieldDefinition, String> {
  TryFold::<(&Config, &Field, &config::Type, &'a str), FieldDefinition, String>::new(
    |(config, field, _, _), b_field| {
      let Some(graphql) = &field.graphql else {
        return Valid::succeed(b_field);
      };

      compile_graphql(config, operation_type, graphql).map(|resolver| b_field.resolver(Some(resolver)))
    },
  )
}
