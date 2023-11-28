use crate::blueprint::FieldDefinition;
use crate::config::{self, Config, Field, GraphQLOperationType};
use crate::graphql_request_template::GraphqlRequestTemplate;
use crate::helpers;
use crate::lambda::Lambda;
use crate::try_fold::TryFold;
use crate::valid::{Valid, ValidationError};

pub fn update_graphql<'a>(
  operation_type: &'a GraphQLOperationType,
  object_name: &'a str,
) -> TryFold<'a, (&'a Config, &'a Field, &'a config::Type, &'a str), FieldDefinition, String> {
  TryFold::<(&Config, &Field, &config::Type, &'a str), FieldDefinition, String>::new(
    |(config, field, type_of, _), b_field| {
      let Some(graphql) = &field.graphql else {
        return Valid::succeed(b_field);
      };

      let Some(base_url) = graphql.base_url.as_ref().or(config.upstream.base_url.as_ref()) else {
        return Valid::fail("No base URL defined".to_string());
      };

      let args = graphql.args.as_ref();

      helpers::headers::to_headermap(&graphql.headers)
        .and_then(|header_map| {
          Valid::from(
            GraphqlRequestTemplate::new(
              base_url.to_owned(),
              operation_type,
              &graphql.name,
              args,
              header_map,
              graphql.federate.unwrap_or(false),
              field.type_of.clone(),
              type_of.join_types.clone(),
              object_name.to_string(),
              b_field.name.clone(),
              graphql.filter_selection_set.unwrap_or(false),
            )
            .map_err(|e| ValidationError::new(e.to_string())),
          )
        })
        .map(|req_template| {
          let field_name = graphql.name.clone();
          b_field.resolver(Some(
            Lambda::from_graphql_request_template(req_template, field_name, graphql.batch).expression,
          ))
        })
    },
  )
}
