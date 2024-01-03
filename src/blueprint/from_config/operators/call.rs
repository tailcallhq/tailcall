use crate::blueprint::*;
use crate::config;
use crate::config::{Config, Field, GraphQLOperationType};
use crate::directive::DirectiveCodec;
use crate::try_fold::TryFold;
use crate::valid::Valid;

pub fn update_call(
  operation_type: &GraphQLOperationType,
) -> TryFold<'_, (&Config, &Field, &config::Type, &str), FieldDefinition, String> {
  TryFold::<(&Config, &Field, &config::Type, &str), FieldDefinition, String>::new(
    move |(config, field, type_of, name), b_field| {
      let Some(call) = &field.call else {
        return Valid::succeed(b_field);
      };

      let type_and_field = if let Some(mutation) = &call.mutation {
        Valid::succeed(("Mutation", mutation.as_str()))
      } else if let Some(query) = &call.query {
        Valid::succeed(("Query", query.as_str()))
      } else {
        Valid::fail("call must have one of mutation or query".to_string())
      };

      type_and_field
        .and_then(|(type_name, field_name)| {
          Valid::from_option(
            config.find_type(type_name),
            format!("{} type not found on config", type_name),
          )
          .zip(Valid::succeed(field_name))
        })
        .and_then(|(query_type, field_name)| {
          Valid::from_option(
            query_type.fields.get(field_name),
            format!("{} field not found", field_name),
          )
          .and_then(|field| {
            if !field.has_resolver() {
              return Valid::fail(format!("{} field has no resolver", field_name));
            }

            Valid::succeed(field)
          })
        })
        .and_then(|field| {
          // TO-DO: parse call.args into a way that `update_http`, `update_grpc` and `update_graphql` can use

          TryFold::<(&Config, &Field, &config::Type, &str), FieldDefinition, String>::new(|_, b_field| {
            Valid::succeed(b_field)
          })
          .and(update_http().trace(config::Http::trace_name().as_str()))
          .and(update_grpc(operation_type).trace(config::Grpc::trace_name().as_str()))
          .and(update_graphql(operation_type).trace(config::GraphQL::trace_name().as_str()))
          .try_fold(&(config, field, type_of, name), b_field)
        })
    },
  )
}
