use crate::blueprint::*;
use crate::config;
use crate::config::{Config, Field, GraphQLOperationType};
use crate::directive::DirectiveCodec;
// use crate::mustache::Mustache;
use crate::try_fold::TryFold;
use crate::valid::Valid;
// use crate::valid::ValidationError;

pub fn validate_call(
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

      Valid::from_option(call.query.clone(), "call must have query".to_string())
        .and_then(|field_name| {
          Valid::from_option(config.find_type("Query"), "Query type not found on config".to_string())
            .zip(Valid::succeed(field_name))
        })
        .and_then(|(query_type, field_name)| {
          Valid::from_option(
            query_type.fields.get(&field_name),
            format!("{} field not found", field_name),
          )
          .and_then(|field| {
            if field.has_resolver() {
              Valid::succeed((field, field_name, call.args.iter()))
            } else {
              Valid::fail(format!("{} field has no resolver", field_name))
            }
          })
        })
        .map_to(b_field)
      // .and_then(|(field, field_name, args)| {
      //   args.fold(Valid::succeed(field.clone()), |field, (key, value)| {
      //     field.and_then(|field| {
      //       // not sure if the code below will be useful
      //       // TO-DO: remove if not needed
      //       // let mustache = Mustache::parse(value.as_str()).map_err(|e| ValidationError::new(e.to_string())).unwrap();
      //       // println!("mustache: {:?}", mustache);
      //       // println!("field: {:?}", field);

      //       if let Some(http) = field.clone().http.as_mut() {
      //         let value = value.replace("{{", "").replace("}}", "");

      //         http.path = http.path.replace(format!("args.{}", key).as_str(), value.as_str());

      //         let field = Field { http: Some(http.clone()), ..field.clone() };

      //         Valid::succeed(field)
      //       } else if let Some(graphql) = field.clone().graphql.as_mut() {
      //         graphql.args = graphql.args.clone().map(|mut args| {
      //           args.0.iter_mut().for_each(|(k, v)| {
      //             if k == key {
      //               *v = value.clone();
      //             }
      //           });

      //           args
      //         });

      //         let field = Field { graphql: Some(graphql.clone()), ..field.clone() };

      //         Valid::succeed(field)
      //       } else if let Some(_grpc) = field.clone().grpc.as_mut() {
      //         todo!("grpc not implemented yet");
      //       } else {
      //         Valid::fail(format!("{} field does not have an http resolver", field_name))
      //       }
      //     })
      //   })
      // })
      // .and_then(|_field| {
      //   TryFold::<(&Config, &Field, &config::Type, &str), FieldDefinition, String>::new(|_, b_field| {
      //     Valid::succeed(b_field)
      //   })
      //   .and(update_http().trace(config::Http::trace_name().as_str()))
      //   .and(update_grpc(operation_type).trace(config::Grpc::trace_name().as_str()))
      //   .and(update_graphql(operation_type).trace(config::GraphQL::trace_name().as_str()))
      //   .try_fold(&(config, &_field, type_of, name), b_field)
      // })
    },
  )
}
