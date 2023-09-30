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
fn to_enum_type_definition(
  name: &str,
  type_: &config::Type,
  _config: &Config,
  variants: Vec<String>,
) -> Valid<Definition> {
  let enum_type_definition = Definition::EnumTypeDefinition(EnumTypeDefinition {
    name: name.to_string(),
    directives: Vec::new(),
    description: type_.doc.clone(),
    enum_values: variants
      .iter()
      .map(|variant| EnumValueDefinition { description: None, name: variant.clone(), directives: Vec::new() })
      .collect(),
  });
  Valid::Ok(enum_type_definition)
}
