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
fn to_interface_type_definition(definition: ObjectTypeDefinition) -> Valid<Definition> {
  Valid::Ok(Definition::InterfaceTypeDefinition(InterfaceTypeDefinition {
    name: definition.name,
    fields: definition.fields,
    description: definition.description,
  }))
}
fn to_union_type_definition((name, u): (&String, &config::Union)) -> UnionTypeDefinition {
  UnionTypeDefinition {
    name: name.to_owned(),
    description: u.doc.clone(),
    directives: Vec::new(),
    types: u.types.clone(),
  }
}
fn to_scalar_type_definition(name: &str) -> Valid<Definition> {
  Valid::Ok(Definition::ScalarTypeDefinition(ScalarTypeDefinition {
    name: name.to_string(),
    directive: Vec::new(),
    description: None,
  }))
}
fn is_scalar(type_name: &str) -> bool {
  ["String", "Int", "Float", "Boolean", "ID", "JSON"].contains(&type_name)
}
// Helper function to recursively process the path and return the corresponding type
