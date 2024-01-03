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

      if validate_field_has_resolver(name, field, &config.types).is_succeed() {
        return Valid::fail(format!(
          "@call directive is not allowed on field {} because it already has a resolver",
          name
        ));
      }

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
        .zip(Valid::succeed(call.args.iter()))
        .and_then(|(field, args)| {
          args.fold(Valid::succeed(field.clone()), |field, (key, value)| {
            field.and_then(|field| {
              let value = value.replace("{{", "").replace("}}", "");

              if let Some(http) = field.clone().http.as_mut() {
                http.path = http.path.replace(format!("args.{}", key).as_str(), value.as_str());

                let field = Field { http: Some(http.clone()), ..field.clone() };

                return Valid::succeed(field);
              }

              Valid::succeed(field)
            })
          })
        })
        .and_then(|_field| {
          TryFold::<(&Config, &Field, &config::Type, &str), FieldDefinition, String>::new(|_, b_field| {
            Valid::succeed(b_field)
          })
          .and(update_http().trace(config::Http::trace_name().as_str()))
          .and(update_grpc(operation_type).trace(config::Grpc::trace_name().as_str()))
          .and(update_graphql(operation_type).trace(config::GraphQL::trace_name().as_str()))
          .try_fold(&(config, &_field, type_of, name), b_field)
        })
    },
  )
}
