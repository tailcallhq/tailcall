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
fn to_fields(type_of: &config::Type, config: &Config) -> Valid<Vec<blueprint::FieldDefinition>> {
  let fields: Vec<Option<blueprint::FieldDefinition>> = type_of.fields.iter().validate_all(|(name, field)| {
    validate_field_type_exist(config, field)
      .validate_or(to_field(type_of, config, name, field))
      .trace(name)
  })?;

  Ok(fields.into_iter().flatten().collect())
}

fn update_unsafe(field: config::Field, mut b_field: FieldDefinition) -> FieldDefinition {
  if let Some(op) = field.unsafe_operation {
    b_field = b_field.resolver_or_default(Lambda::context().to_unsafe_js(op.script.clone()), |r| {
      r.to_unsafe_js(op.script.clone())
    });
  }
  b_field
}

fn validate_field_type_exist(config: &Config, field: &Field) -> Valid<()> {
  let field_type = &field.type_of;

  if !is_scalar(field_type) && !config.contains(field_type) {
    Valid::fail(format!("Undeclared type '{field_type}' was found"))
  } else {
    Valid::Ok(())
  }
}

