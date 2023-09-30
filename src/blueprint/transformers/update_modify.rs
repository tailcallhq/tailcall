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
fn update_modify(
  field: &config::Field,
  mut b_field: FieldDefinition,
  type_: &config::Type,
  config: &Config,
) -> Valid<Option<FieldDefinition>> {
  match field.modify.as_ref() {
    Some(modify) => {
      if modify.omit.as_ref().is_some() {
        Ok(None)
      } else if let Some(new_name) = &modify.name {
        if let Some(interface_names) = type_.implements.clone() {
          for name in interface_names {
            let interface = config.find_type(&name);
            if let Some(interface) = interface {
              if interface.fields.iter().any(|(name, _)| name == new_name) {
                return Valid::fail("Field is already implemented from interface".to_string());
              }
            }
          }
        }

        let lambda = Lambda::context_field(b_field.name.clone());
        b_field = b_field.resolver_or_default(lambda, |r| r);
        b_field = b_field.name(new_name.clone());
        Valid::Ok(Some(b_field))
      } else {
        Valid::Ok(Some(b_field))
      }
    }
    None => Valid::Ok(Some(b_field)),
  }
}
