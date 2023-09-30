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
fn process_path(
  path: &[String],
  field: &config::Field,
  type_info: &config::Type,
  is_required: bool,
  config: &Config,
  invalid_path_handler: &dyn Fn(&str, &[String]) -> Valid<Type>,
) -> Valid<Type> {
  if let Some((field_name, remaining_path)) = path.split_first() {
    if field_name.parse::<usize>().is_ok() {
      let mut modified_field = field.clone();
      modified_field.list = Some(false);
      return process_path(
        remaining_path,
        &modified_field,
        type_info,
        false,
        config,
        invalid_path_handler,
      )
      .trace(field_name);
    }
    if let Some(next_field) = type_info.fields.get(field_name) {
      let next_is_required = is_required && next_field.required.unwrap_or(false);
      if is_scalar(&next_field.type_of) {
        return process_path(
          remaining_path,
          next_field,
          type_info,
          next_is_required,
          config,
          invalid_path_handler,
        )
        .trace(field_name);
      }
      if let Some(next_type_info) = config.find_type(&next_field.type_of) {
        let of_type = process_path(
          remaining_path,
          next_field,
          next_type_info,
          next_is_required,
          config,
          invalid_path_handler,
        )
        .trace(field_name)?;

        return if field.list.unwrap_or(false) {
          Valid::Ok(ListType { of_type: Box::new(of_type), non_null: is_required })
        } else {
          Ok(of_type)
        };
      }
    }
    return invalid_path_handler(field_name, path).trace(field_name);
  }
  Valid::Ok(to_type(
    &field.type_of,
    &field.list,
    &Some(is_required),
    &field.list_type_required,
  ))
}

// Main function to update an inline field
