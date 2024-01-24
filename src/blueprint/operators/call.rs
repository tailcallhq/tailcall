use std::collections::hash_map::Iter;

use crate::blueprint::*;
use crate::config;
use crate::config::{Config, Field, GraphQLOperationType};
use crate::lambda::{Expression, IO};
use crate::mustache::{Mustache, Segment};
use crate::try_fold::TryFold;
use crate::valid::Valid;

fn find_value<'a>(args: &'a Iter<'a, String, String>, key: &'a String) -> Option<&'a String> {
  args
    .clone()
    .find_map(|(k, value)| if k == key { Some(value) } else { None })
}

pub fn update_call(
  operation_type: &GraphQLOperationType,
) -> TryFold<'_, (&Config, &Field, &config::Type, &str), FieldDefinition, String> {
  TryFold::<(&Config, &Field, &config::Type, &str), FieldDefinition, String>::new(
    move |(config, field, _, _), b_field| {
      let Some(call) = &field.call else {
        return Valid::succeed(b_field);
      };

      compile_call(field, config, call, operation_type)
        .and_then(|resolver| Valid::succeed(b_field.resolver(Some(resolver))))
    },
  )
}

pub fn compile_call(
  field: &Field,
  config: &Config,
  call: &config::Call,
  operation_type: &GraphQLOperationType,
) -> Valid<Expression, String> {
  if validate_field_has_resolver(field.name(), field, &config.types).is_succeed() {
    return Valid::fail("@call directive is not allowed on field because it already has a resolver".to_string());
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
        .filter(|(k, _)| !args.clone().any(|(k1, _)| k1.eq(*k)))
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
          Expression::IO(IO::Http { req_template, group_by, dl_id }) => Valid::succeed(
            req_template
              .clone()
              .root_url(replace_url(&req_template.root_url, &args)),
          )
          .map(|req_template| {
            req_template
              .clone()
              .query(req_template.clone().query.iter().map(replace_mustache(&args)).collect())
          })
          .map(|req_template| {
            req_template
              .clone()
              .headers(req_template.headers.iter().map(replace_mustache(&args)).collect())
          })
          .map(|req_template| Expression::IO(IO::Http { req_template, group_by, dl_id })),
          _ => Valid::succeed(expr),
        })
      } else if let Some(graphql) = _field.graphql.clone() {
        compile_graphql(config, operation_type, &graphql).and_then(|expr| match expr {
          Expression::IO(IO::GraphQLEndpoint { req_template, field_name, batch, dl_id }) => Valid::succeed(
            req_template
              .clone()
              .headers(req_template.headers.iter().map(replace_mustache(&args)).collect()),
          )
          .map(|req_template| {
            if req_template.operation_arguments.is_some() {
              let operation_arguments = req_template
                .clone()
                .operation_arguments
                .unwrap()
                .iter()
                .map(replace_mustache(&args))
                .collect();

              req_template.operation_arguments(Some(operation_arguments))
            } else {
              req_template
            }
          })
          .and_then(|req_template| {
            Valid::succeed(Expression::IO(IO::GraphQLEndpoint {
              req_template,
              field_name,
              batch,
              dl_id,
            }))
          }),
          _ => Valid::succeed(expr),
        })
      } else if let Some(grpc) = _field.grpc.clone() {
        // todo!("needs to be implemented");
        let inputs: CompileGrpc<'_> =
          CompileGrpc { config, operation_type, field, grpc: &grpc, validate_with_schema: false };
        compile_grpc(inputs).and_then(|expr| match expr {
          Expression::IO(IO::Grpc { req_template, group_by, dl_id }) => {
            Valid::succeed(req_template.clone().url(replace_url(&req_template.url, &args)))
              .map(|req_template| {
                req_template
                  .clone()
                  .headers(req_template.headers.iter().map(replace_mustache(&args)).collect())
              })
              .map(|req_template| {
                if let Some(body) = req_template.clone().body {
                  req_template.clone().body(Some(replace_url(&body, &args)))
                } else {
                  req_template
                }
              })
              .map(|req_template| Expression::IO(IO::Grpc { req_template, group_by, dl_id }))
          }
          _ => Valid::succeed(expr),
        })
      } else {
        return Valid::fail(format!("{} field has no resolver", field_name));
      }
    })
}

fn replace_url(url: &Mustache, args: &Iter<'_, String, String>) -> Mustache {
  url
    .get_segments()
    .iter()
    .map(|segment| match segment {
      Segment::Literal(literal) => Segment::Literal(literal.clone()),
      Segment::Expression(expression) => {
        if expression[0] == "args" {
          let value = find_value(&args, &expression[1]).unwrap();
          let item = Mustache::parse(value).unwrap();

          let expression = item.get_segments().first().unwrap().to_owned().to_owned();

          expression
        } else {
          Segment::Expression(expression.clone())
        }
      }
    })
    .collect::<Vec<Segment>>()
    .into()
}

fn replace_mustache<'a, T: Clone>(args: &'a Iter<'a, String, String>) -> impl Fn(&(T, Mustache)) -> (T, Mustache) + 'a {
  |(key, value)| {
    let value: Mustache = value
      .expression_segments()
      .iter()
      .map(|expression| {
        if expression[0] == "args" {
          let value = find_value(args, &expression[1]).unwrap();
          let item = Mustache::parse(value).unwrap();

          let expression = item.get_segments().first().unwrap().to_owned().to_owned();

          expression
        } else {
          Segment::Expression(expression.to_owned().to_owned())
        }
      })
      .collect::<Vec<Segment>>()
      .into();

    (key.clone().to_owned(), value)
  }
}
