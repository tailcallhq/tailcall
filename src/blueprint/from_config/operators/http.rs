use crate::blueprint::from_config::to_type;
use crate::blueprint::*;
use crate::config::group_by::GroupBy;
use crate::config::{Config, Field};
use crate::endpoint::Endpoint;
use crate::http::{Method, RequestTemplate};
use crate::lambda::{Expression, Lambda, Unsafe};
use crate::try_fold::TryFold;
use crate::valid::{Valid, ValidationError};
use crate::{config, helpers};

struct MustachePartsValidator<'a> {
  type_of: &'a config::Type,
  config: &'a Config,
  field: &'a FieldDefinition,
}

impl<'a> MustachePartsValidator<'a> {
  fn new(type_of: &'a config::Type, config: &'a Config, field: &'a FieldDefinition) -> Self {
    Self { type_of, config, field }
  }

  fn validate_type(&self, parts: &[String], is_query: bool) -> Result<(), String> {
    let mut len = parts.len();
    let mut type_of = self.type_of;
    for item in parts {
      let field = type_of.fields.get(item).ok_or_else(|| {
        format!(
          "no value '{}' found",
          parts[0..parts.len() - len + 1].join(".").as_str()
        )
      })?;
      let val_type = to_type(field, None);

      if !is_query && val_type.is_nullable() {
        return Err(format!("value '{}' is a nullable type", item.as_str()));
      } else if len == 1 && !is_scalar(val_type.name()) {
        return Err(format!("value '{}' is not of a scalar type", item.as_str()));
      } else if len == 1 {
        break;
      }

      type_of = self
        .config
        .find_type(&field.type_of)
        .ok_or_else(|| format!("no type '{}' found", parts.join(".").as_str()))?;

      len -= 1;
    }

    Ok(())
  }

  fn validate(&self, parts: &[String], is_query: bool) -> Valid<(), String> {
    let config = self.config;
    let args = &self.field.args;

    if parts.len() < 2 {
      return Valid::fail("too few parts in template".to_string());
    }

    let head = parts[0].as_str();
    let tail = parts[1].as_str();

    match head {
      "value" => {
        // all items on parts except the first one
        let tail = &parts[1..];

        if let Err(e) = self.validate_type(tail, is_query) {
          return Valid::fail(e);
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

  if let Some(Expression::Unsafe(Unsafe::Http { req_template, .. })) = &field.resolver {
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
    |(config, field, type_of, _), b_field| {
      let Some(http) = &field.http else {
        return Valid::succeed(b_field);
      };

      Valid::<(), String>::fail("GroupBy is only supported for GET requests".to_string())
        .when(|| !http.group_by.is_empty() && http.method != Method::GET)
        .and(
          Valid::<(), String>::fail("GroupBy can only be applied if batching is enabled".to_string()).when(|| {
            (config.upstream.get_delay() < 1 || config.upstream.get_max_size() < 1) && !http.group_by.is_empty()
          }),
        )
        .and(Valid::from_option(
          http.base_url.as_ref().or(config.upstream.base_url.as_ref()),
          "No base URL defined".to_string(),
        ))
        .zip(helpers::headers::to_headermap(&http.headers))
        .and_then(|(base_url, header_map)| {
          let mut base_url = base_url.trim_end_matches('/').to_owned();
          base_url.push_str(http.path.clone().as_str());

          let query = http.query.clone().iter().map(|(k, v)| (k.clone(), v.clone())).collect();
          let output_schema = to_json_schema_for_field(field, config);
          let input_schema = to_json_schema_for_args(&field.args, config);

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
            b_field.resolver(Some(Expression::Unsafe(Unsafe::Http {
              req_template,
              group_by: Some(GroupBy::new(http.group_by.clone())),
              dl_id: None,
            })))
          } else {
            b_field.resolver(Some(Lambda::from_request_template(req_template).expression))
          }
        })
        .and_then(|b_field| validate_field(type_of, config, &b_field).map_to(b_field))
    },
  )
}
