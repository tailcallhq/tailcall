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
fn to_field(
  type_of: &config::Type,
  config: &Config,
  name: &str,
  field: &Field,
) -> Valid<Option<blueprint::FieldDefinition>> {
  let field_type = &field.type_of;
  let args = to_args(field)?;

  let field_definition = FieldDefinition {
    name: name.to_owned(),
    description: field.doc.clone(),
    args,
    of_type: to_type(field_type, &field.list, &field.required, &field.list_type_required),
    directives: Vec::new(),
    resolver: None,
  };

  let field_definition = update_http(field, field_definition, config).trace("@http")?;
  let field_definition = update_unsafe(field.clone(), field_definition);
  let maybe_field_definition = update_modify(field, field_definition, type_of, config).trace("@modify")?;
  let maybe_field_definition = match maybe_field_definition {
    Some(field_definition) => {
      Some(update_inline_field(type_of, name, field, field_definition, config).trace("@inline")?)
    }
    None => None,
  };

  Ok(maybe_field_definition)
}

