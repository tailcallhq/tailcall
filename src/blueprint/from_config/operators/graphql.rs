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
    |(config, field, _, _), b_field| {
      let Some(graphql) = &field.graphql else {
        return Valid::succeed(b_field);
      };

      let args = graphql.args.as_ref();

      Valid::from_option(
        graphql.base_url.as_ref().or(config.upstream.base_url.as_ref()),
        "No base URL defined".to_string(),
      )
      .zip(helpers::headers::to_headermap(&graphql.headers))
      .and_then(|(base_url, header_map)| {
        Valid::from(
          GraphqlRequestTemplate::new(
            base_url.to_owned(),
            operation_type,
            &graphql.name,
            args,
            header_map,
            field.type_of.clone(),
            object_name.to_string(),
            b_field.name.clone(),
            graphql.filter_selection_set.unwrap_or(false),
          )
          .map_err(|e| ValidationError::new(e.to_string())),
        )
      })
      .map(|req_template| {
        let field_name = graphql.name.clone().unwrap_or_default();
        b_field.resolver(Some(
          Lambda::from_graphql_request_template(req_template, field_name, graphql.batch).expression,
        ))
      })
    },
  )
}
