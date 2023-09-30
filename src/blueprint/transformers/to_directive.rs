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
fn to_directive(const_directive: ConstDirective) -> Valid<Directive> {
  let arguments = const_directive
    .arguments
    .into_iter()
    .map(|(k, v)| {
      let value = v.node.into_json();
      if let Ok(value) = value {
        return Ok((k.node.to_string(), value));
      }
      Err(value.unwrap_err())
    })
    .collect::<Result<HashMap<String, serde_json::Value>, _>>()
    .map_err(|e| ValidationError::new(e.to_string()))?;

  Ok(Directive { name: const_directive.name.node.clone().to_string(), arguments, index: 0 })
}
