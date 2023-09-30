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
fn to_input_object_type_definition(definition: ObjectTypeDefinition) -> Valid<Definition> {
  Valid::Ok(Definition::InputObjectTypeDefinition(InputObjectTypeDefinition {
    name: definition.name,
    fields: definition
      .fields
      .iter()
      .map(|field| InputFieldDefinition {
        name: field.name.clone(),
        description: field.description.clone(),
        default_value: None,
        of_type: field.of_type.clone(),
      })
      .collect(),
    description: definition.description,
  }))
}
fn to_type(name: &str, list: &Option<bool>, required: &Option<bool>, list_type_required: &Option<bool>) -> Type {
  let non_null = required.unwrap_or(false);
  if list.unwrap_or(false) {
    Type::ListType {
      of_type: Box::new(Type::NamedType { name: name.to_string(), non_null: list_type_required.unwrap_or(false) }),
      non_null,
    }
  } else {
    Type::NamedType { name: name.to_string(), non_null }
  }
}

