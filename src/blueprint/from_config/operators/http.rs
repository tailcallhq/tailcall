use crate::blueprint::*;
use crate::config::group_by::GroupBy;
use crate::config::{Config, Field};
use crate::endpoint::Endpoint;
use crate::http::{Method, RequestTemplate};
use crate::lambda::{Expression, Lambda, Unsafe};
use crate::try_fold::TryFold;
use crate::valid::{Valid, ValidationError};
use crate::{config, helpers};

pub fn compile_http(config: &config::Config, field: &config::Field, http: &config::Http) -> Valid<Expression, String> {
  Valid::<(), String>::fail("GroupBy is only supported for GET requests".to_string())
    .when(|| !http.group_by.is_empty() && http.method != Method::GET)
    .and(
      Valid::<(), String>::fail("GroupBy can only be applied if batching is enabled".to_string())
        .when(|| (config.upstream.get_delay() < 1 || config.upstream.get_max_size() < 1) && !http.group_by.is_empty()),
    )
    .and(Valid::from_option(
      http.base_url.as_ref().or(config.upstream.base_url.as_ref()),
      "No base URL defined".to_string(),
    ))
    .zip(helpers::headers::to_mustache_headers(&http.headers))
    .and_then(|(base_url, headers)| {
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
          .encoding(http.encoding.clone()),
      )
      .map(|req_tmpl| req_tmpl.headers(headers))
      .map_err(|e| ValidationError::new(e.to_string()))
      .into()
    })
    .map(|req_template| {
      if !http.group_by.is_empty() && http.method == Method::GET {
        Expression::Unsafe(Unsafe::Http {
          req_template,
          group_by: Some(GroupBy::new(http.group_by.clone())),
          dl_id: None,
        })
      } else {
        Lambda::from_request_template(req_template).expression
      }
    })
}

pub fn update_http<'a>() -> TryFold<'a, (&'a Config, &'a Field, &'a config::Type, &'a str), FieldDefinition, String> {
  TryFold::<(&Config, &Field, &config::Type, &'a str), FieldDefinition, String>::new(
    |(config, field, type_of, _), b_field| {
      let Some(http) = &field.http else {
        return Valid::succeed(b_field);
      };

      compile_http(config, field, http)
        .map(|resolver| b_field.resolver(Some(resolver)))
        .and_then(|b_field| b_field.validate_field(type_of, config).map_to(b_field))
    },
  )
}
