use hyper::header::{HeaderName, HeaderValue};
use hyper::HeaderMap;

use crate::blueprint::from_config::to_type;
use crate::blueprint::*;
use crate::config;
use crate::config::group_by::GroupBy;
use crate::config::{Config, Field};
use crate::endpoint::Endpoint;
use crate::http::Method;
use crate::lambda::{Expression, Lambda, Unsafe};
use crate::request_template::RequestTemplate;
use crate::try_fold::TryFold;
use crate::valid::{Valid, ValidationError};

struct MustachePartsValidator<'a> {
  type_of: &'a config::Type,
  config: &'a Config,
  field: &'a FieldDefinition,
}

fn get_value_type(type_of: &config::Type, value: &str) -> Option<Type> {
  if let Some(field) = type_of.fields.get(value) {
    return Some(to_type(field, None));
  }
  None
}

impl<'a> MustachePartsValidator<'a> {
  fn new(type_of: &'a config::Type, config: &'a Config, field: &'a FieldDefinition) -> Self {
    Self { type_of, config, field }
  }
  fn validate(&self, parts: &[String], is_query: bool) -> Valid<(), String> {
    let type_of = self.type_of;
    let config = self.config;
    let args = &self.field.args;

    if parts.len() < 2 {
      return Valid::fail("too few parts in template".to_string());
    }

    let head = parts[0].as_str();
    let tail = parts[1].as_str();

    match head {
      "value" => {
        if let Some(val_type) = get_value_type(type_of, tail) {
          if !is_scalar(val_type.name()) {
            return Valid::fail(format!("value '{tail}' is not of a scalar type"));
          }

          // Queries can use optional values
          if !is_query && val_type.is_nullable() {
            return Valid::fail(format!("value '{tail}' is a nullable type"));
          }
        } else {
          return Valid::fail(format!("no value '{tail}' found"));
        }
      }
      "args" => {
        // XXX this is a linear search but it's cost is less than that of
        // constructing a HashMap since we'd have 3-4 arguments at max in
        // most cases
        if let Some(arg) = args.iter().find(|arg| arg.name == tail) {
          if let Type::ListType { .. } = arg.of_type {
            return Valid::fail(format!("can't use list type '{tail}' here"));
          }

          // we can use non-scalar types in args

          if !is_query && arg.default_value.is_none() && arg.of_type.is_nullable() {
            return Valid::fail(format!("argument '{tail}' is a nullable type"));
          }
        } else {
          return Valid::fail(format!("no argument '{tail}' found"));
        }
      }
      "vars" => {
        if config.server.vars.get(tail).is_none() {
          return Valid::fail(format!("var '{tail}' is not set in the server config"));
        }
      }
      "headers" => {
        // "headers" refers to the header values known at runtime, which we can't
        // validate here
      }
      _ => {
        return Valid::fail(format!("unknown template directive '{head}'"));
      }
    }

    Valid::succeed(())
  }
}

fn validate_field(type_of: &config::Type, config: &Config, field: &FieldDefinition) -> Valid<(), String> {
  // XXX we could use `Mustache`'s `render` method with a mock
  // struct implementing the `PathString` trait encapsulating `validation_map`
  // but `render` simply falls back to the default value for a given
  // type if it doesn't exist, so we wouldn't be able to get enough
  // context from that method alone
  // So we must duplicate some of that logic here :(

  let parts_validator = MustachePartsValidator::new(type_of, config, field);

  if let Some(Expression::Unsafe(Unsafe::Http(req_template, _, _))) = &field.resolver {
    Valid::from_iter(req_template.root_url.expression_segments(), |parts| {
      parts_validator.validate(parts, false).trace("path")
    })
    .and(Valid::from_iter(req_template.query.clone(), |query| {
      let (_, mustache) = query;

      Valid::from_iter(mustache.expression_segments(), |parts| {
        parts_validator.validate(parts, true).trace("query")
      })
    }))
    .unit()
  } else {
    Valid::succeed(())
  }
}

pub fn update_http<'a>() -> TryFold<'a, (&'a Config, &'a Field, &'a config::Type, &'a str), FieldDefinition, String> {
  TryFold::<(&Config, &Field, &config::Type, &'a str), FieldDefinition, String>::new(
    |(config, field, type_of, _), b_field| match field.http.as_ref() {
      Some(http) => match http
        .base_url
        .as_ref()
        .map_or_else(|| config.upstream.base_url.as_ref(), Some)
      {
        Some(base_url) => {
          let mut base_url = base_url.clone();
          if base_url.ends_with('/') {
            base_url.pop();
          }
          base_url.push_str(http.path.clone().as_str());
          let query = http.query.clone().iter().map(|(k, v)| (k.clone(), v.clone())).collect();
          let output_schema = to_json_schema_for_field(field, config);
          let input_schema = to_json_schema_for_args(&field.args, config);

          Valid::<(), String>::fail("GroupBy is only supported for GET requests".to_string())
            .when(|| !http.group_by.is_empty() && http.method != Method::GET)
            .and(Valid::from_iter(http.headers.iter(), |(k, v)| {
              let name =
                Valid::from(HeaderName::from_bytes(k.as_bytes()).map_err(|e| ValidationError::new(e.to_string())));

              let value =
                Valid::from(HeaderValue::from_str(v.as_str()).map_err(|e| ValidationError::new(e.to_string())));

              name.zip(value).map(|(name, value)| (name, value))
            }))
            .map(HeaderMap::from_iter)
            .and_then(|header_map| {
              RequestTemplate::try_from(
                Endpoint::new(base_url.to_string())
                  .method(http.method.clone())
                  .query(query)
                  .output(output_schema)
                  .input(input_schema)
                  .body(http.body.clone())
                  .headers(header_map),
              )
              .map_err(|e| ValidationError::new(e.to_string()))
              .into()
            })
            .map(|req_template| {
              if !http.group_by.is_empty() && http.method == Method::GET {
                b_field.resolver(Some(Expression::Unsafe(Unsafe::Http(
                  req_template,
                  Some(GroupBy::new(http.group_by.clone())),
                  None,
                ))))
              } else {
                b_field.resolver(Some(Lambda::from_request_template(req_template).expression))
              }
            })
            .and_then(|b_field| validate_field(type_of, config, &b_field).map_to(b_field))
        }
        None => Valid::fail("No base URL defined".to_string()),
      },
      None => Valid::succeed(b_field),
    },
  )
}
