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
fn to_schema(config: &Config) -> Valid<SchemaDefinition> {
  let query = config
    .graphql
    .schema
    .query
    .as_ref()
    .validate_some("Query type is not defined".to_string())?;

  Ok(SchemaDefinition {
    query: query.clone(),
    mutation: config.graphql.schema.mutation.clone(),
    directives: vec![to_directive(config.server.to_directive("server".to_string()))?],
  })
}
fn to_object_type_definition(name: &str, type_of: &config::Type, config: &Config) -> Valid<Definition> {
  to_fields(type_of, config).map(|fields| {
    Definition::ObjectTypeDefinition(ObjectTypeDefinition {
      name: name.to_string(),
      description: type_of.doc.clone(),
      fields,
      implements: type_of.implements.as_ref().unwrap_or(&Vec::new()).to_vec(),
    })
  })
}
