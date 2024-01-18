use std::collections::BTreeMap;

use crate::blueprint::*;
use crate::config;
use crate::config::KeyValues;
use crate::config::{Config, Field, GraphQLOperationType};
use crate::lambda::Expression;
use crate::lambda::Unsafe;
use crate::mustache::Mustache;
use crate::mustache::Segment;
use crate::try_fold::TryFold;
use crate::valid::Valid;

pub fn update_call(
  operation_type: &GraphQLOperationType,
) -> TryFold<'_, (&Config, &Field, &config::Type, &str), FieldDefinition, String> {
  TryFold::<(&Config, &Field, &config::Type, &str), FieldDefinition, String>::new(
    move |(config, field, _type_of, name), b_field| {
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
          .zip(Valid::succeed(field_name))
          .and_then(|(field, field_name)| {
            if field.has_resolver() {
              Valid::succeed((field, field_name, call.args.iter()))
            } else {
              Valid::fail(format!("{} field has no resolver", field_name))
            }
          })
        })
        .and_then(|(_field, field_name, args)| {
          let empties: Vec<(&String, &config::Arg)> = _field
            .args
            .iter()
            .filter(|(k, _)| args.clone().find(|(k1, _)| k == k1).is_none())
            .collect();

          if empties.len().gt(&0) {
            return Valid::fail(format!(
              "no argument {} found",
              empties
                .iter()
                .map(|(k, _)| format!("'{}'", k))
                .collect::<Vec<String>>()
                .join(", ")
            ))
            .trace(field_name.as_str());
          }

          if let Some(http) = _field.http.clone() {
            compile_http(config, field, &http).and_then(|expr| match expr.clone() {
              Expression::Unsafe(Unsafe::Http { mut req_template, group_by, dl_id }) => {
                req_template = req_template.clone().root_url(
                  req_template
                    .root_url
                    .get_segments()
                    .iter()
                    .map(|segment| {
                      match segment {
                        Segment::Literal(literal) => Segment::Literal(literal.clone()),
                        Segment::Expression(expression) => {
                          if expression[0] == "args" {
                            // this value will always be present because we already checked it
                            let (_, value) = args.clone().find(|(k, _)| **k == expression[1]).unwrap();
                            let item = Mustache::parse(value).unwrap();

                            let expression = item.get_segments().first().unwrap().to_owned().to_owned();

                            expression
                          } else {
                            Segment::Expression(expression.clone())
                          }
                        }
                      }
                    })
                    .collect::<Vec<Segment>>()
                    .into(),
                );

                Valid::succeed(Expression::Unsafe(Unsafe::Http { req_template, group_by, dl_id }))
              }
              _ => Valid::succeed(expr),
            })
          } else if let Some(mut graphql) = _field.graphql.clone() {
            if let Some(mut _args) = graphql.args {
              let mut updated: BTreeMap<String, String> = BTreeMap::new();

              for (key, value) in _args.0 {
                let found = args
                  .clone()
                  .into_iter()
                  .find_map(|(k, v)| if *k == key { Some(v) } else { None });

                if let Some(v) = found {
                  updated.insert(key, v.clone().to_string());
                } else {
                  updated.insert(key, value);
                }
              }

              graphql.args = Some(KeyValues(updated));
            }
            compile_graphql(config, operation_type, &graphql)
          } else if let Some(grpc) = _field.grpc.clone() {
            let inputs: CompileGrpc<'_> =
              CompileGrpc { config, operation_type, field, grpc: &grpc, validate_with_schema: false };
            compile_grpc(inputs)
          } else {
            return Valid::fail(format!("{} field has no resolver", field_name));
          }
          .and_then(|resolver| Valid::succeed(b_field.resolver(Some(resolver))))
        })
    },
  )
}
