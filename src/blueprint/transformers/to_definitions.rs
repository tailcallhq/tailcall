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
fn to_definitions<'a>(
  config: &Config,
  output_types: HashSet<&'a String>,
  input_types: HashSet<&'a String>,
) -> Valid<Vec<Definition>> {
  let mut types: Vec<Definition> = config.graphql.types.iter().validate_all(|(name, type_)| {
    let dbl_usage = input_types.contains(name) && output_types.contains(name);
    if let Some(variants) = &type_.variants {
      if !variants.is_empty() {
        to_enum_type_definition(name, type_, config, variants.clone()).trace(name)
      } else {
        Valid::fail("No variants found for enum".to_string())
      }
    } else if type_.scalar.is_some() {
      to_scalar_type_definition(name).trace(name)
    } else if dbl_usage {
      Valid::fail("type is used in input and output".to_string()).trace(name)
    } else {
      let definition = to_object_type_definition(name, type_, config).trace(name)?;
      match definition.clone() {
        Definition::ObjectTypeDefinition(object_type_definition) => {
          if config.input_types().contains(name) {
            to_input_object_type_definition(object_type_definition).trace(name)
          } else if type_.interface.unwrap_or(false) {
            to_interface_type_definition(object_type_definition).trace(name)
          } else {
            Valid::Ok(definition)
          }
        }
        _ => Valid::Ok(definition),
      }
    }
  })?;

  let unions = config
    .graphql
    .unions
    .iter()
    .map(to_union_type_definition)
    .map(Definition::UnionTypeDefinition);

  types.extend(unions);
  Ok(types)
}
