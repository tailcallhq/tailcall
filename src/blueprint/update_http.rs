#![allow(clippy::too_many_arguments)]

use std::collections::{BTreeMap, HashMap, HashSet};

use async_graphql::parser::types::ConstDirective;
#[allow(unused_imports)]
use async_graphql::InputType;
use hyper::header::{HeaderName, HeaderValue};
use hyper::HeaderMap;
use regex::Regex;

use super::UnionTypeDefinition;
use crate::blueprint::Type::ListType;
use crate::blueprint::*;
use crate::config::{Arg, Config, Field, InlineType};
use crate::directive::DirectiveCodec;
use crate::endpoint::Endpoint;
use crate::http::Method;
use crate::json::JsonSchema;
use crate::lambda::Lambda;
use crate::request_template::RequestTemplate;
use crate::valid::{OptionExtension, Valid as ValidDefault, ValidExtensions, ValidationError, VectorExtension};
use crate::{blueprint, config};

type Valid<A> = ValidDefault<A, String>;

pub fn config_blueprint(config: &Config) -> Valid<Blueprint> {
  let output_types = config.output_types();
  let input_types = config.input_types();
  let schema = to_schema(config)?;
  let definitions = to_definitions(config, output_types, input_types)?;
  Ok(super::compress::compress(Blueprint { schema, definitions }))
}
fn update_http(field: &config::Field, b_field: FieldDefinition, config: &Config) -> Valid<FieldDefinition> {
  let mut b_field = b_field;
  match field.http.as_ref() {
    Some(http) => match http
      .base_url
      .as_ref()
      .map_or_else(|| config.server.base_url.as_ref(), Some)
    {
      Some(base_url) => {
        let mut base_url = base_url.clone();
        if base_url.ends_with('/') {
          base_url.pop();
        }
        base_url.push_str(http.path.clone().as_str());
        let method = http.method.as_ref().unwrap_or(&Method::GET);
        let query = match http.query.as_ref() {
          Some(q) => q.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
          None => Vec::new(),
        };
        let output_schema = to_json_schema_for_field(field, config);
        let input_schema = to_json_schema_for_args(&field.args, config);
        let mut header_map = HeaderMap::new();
        for (k, v) in http.headers.clone().unwrap_or_default().iter() {
          header_map.insert(
            HeaderName::from_bytes(k.as_bytes()).map_err(|e| ValidationError::new(e.to_string()))?,
            HeaderValue::from_str(v.as_str()).map_err(|e| ValidationError::new(e.to_string()))?,
          );
        }
        let req_template = RequestTemplate::try_from(
          Endpoint::new(base_url.to_string())
            .method(method.clone())
            .query(query)
            .output(output_schema)
            .input(input_schema)
            .body(http.body.clone())
            .headers(header_map),
        )
        .map_err(|e| ValidationError::new(e.to_string()))?;

        b_field.resolver = Some(Lambda::from_request_template(req_template).expression);

        Valid::Ok(b_field)
      }
      None => Valid::fail("No base URL defined".to_string()),
    },
    None => Valid::Ok(b_field),
  }
}
