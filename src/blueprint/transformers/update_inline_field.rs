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
fn update_inline_field(
  type_info: &config::Type,
  field_name: &str,
  field: &config::Field,
  base_field: FieldDefinition,
  config: &Config,
) -> Valid<FieldDefinition> {
  let inlined_path = field.inline.as_ref().map(|x| x.path.clone()).unwrap_or_default();
  let handle_invalid_path = |_field_name: &str, _inlined_path: &[String]| -> Valid<Type> {
    Valid::fail("Field not found at given path".to_string())
  };
  let has_index = inlined_path.iter().any(|s| {
    let re = Regex::new(r"^\d+$").unwrap();
    re.is_match(s)
  });
  let build_path_strings = |name: String| -> Vec<String> {
    let mut path: Vec<String> = inlined_path.iter().map(|s| s.to_string()).collect();
    path.insert(0, name);
    path
  };

  if let Some(InlineType { path }) = field.clone().inline {
    return match process_path(
      &build_path_strings(field_name.to_string()),
      field,
      type_info,
      false,
      config,
      &handle_invalid_path,
    ) {
      Valid::Ok(of_type) => {
        let new_path = if needs_resolving(field) {
          path
        } else {
          let mut new_path = vec![field_name.to_string()];
          new_path.extend(path.iter().cloned());
          new_path
        };
        let mut updated_base_field = base_field;
        let resolver = Lambda::context_path(new_path.clone());
        if has_index {
          updated_base_field.of_type = Type::NamedType { name: of_type.name().to_string(), non_null: false }
        } else {
          updated_base_field.of_type = of_type;
        }

        updated_base_field = updated_base_field.resolver_or_default(resolver, |r| r.to_input_path(new_path.clone()));
        Valid::Ok(updated_base_field)
      }
      Valid::Err(err) => Valid::Err(err),
    };
  }
  Valid::Ok(base_field)
}
