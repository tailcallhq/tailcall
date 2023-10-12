use std::collections::HashMap;

use async_graphql::dynamic::{Schema, SchemaBuilder};
use async_graphql::extensions::ApolloTracing;
use async_graphql::*;
use derive_setters::Setters;
use serde_json::Value;

use super::GlobalTimeout;
use crate::config;
use crate::lambda::{Expression, Lambda};
use std::collections::BTreeMap;

/// Blueprint is an intermediary representation that allows us to generate graphQL APIs.
/// It can only be generated from a valid Config.
/// It allows us to choose a different GraphQL Backend, without re-writing all orchestration logic.
/// It's not optimized for REST APIs (yet).
#[derive(Clone, Debug)]
pub struct Blueprint {
  pub definitions: Vec<Definition>,
  pub schema: SchemaDefinition,
  pub entity_resolvers: BTreeMap<String, Option<Expression>>,
}

#[derive(Clone, Debug)]
pub enum Type {
  NamedType { name: String, non_null: bool },
  ListType { of_type: Box<Type>, non_null: bool },
}

impl Type {
  pub fn name(&self) -> &str {
    match self {
      Type::NamedType { name, .. } => name,
      Type::ListType { of_type, .. } => of_type.name(),
    }
  }
}

#[derive(Clone, Debug)]
pub enum Definition {
  InterfaceTypeDefinition(InterfaceTypeDefinition),
  ObjectTypeDefinition(ObjectTypeDefinition),
  InputObjectTypeDefinition(InputObjectTypeDefinition),
  ScalarTypeDefinition(ScalarTypeDefinition),
  EnumTypeDefinition(EnumTypeDefinition),
  UnionTypeDefinition(UnionTypeDefinition),
}
impl Definition {
  pub fn name(&self) -> &str {
    match self {
      Definition::InterfaceTypeDefinition(def) => &def.name,
      Definition::ObjectTypeDefinition(def) => &def.name,
      Definition::InputObjectTypeDefinition(def) => &def.name,
      Definition::ScalarTypeDefinition(def) => &def.name,
      Definition::EnumTypeDefinition(def) => &def.name,
      Definition::UnionTypeDefinition(def) => &def.name,
    }
  }
}

#[derive(Clone, Debug)]
pub struct InterfaceTypeDefinition {
  pub name: String,
  pub fields: Vec<FieldDefinition>,
  pub description: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ObjectTypeDefinition {
  pub name: String,
  pub fields: Vec<FieldDefinition>,
  pub description: Option<String>,
  pub implements: Vec<String>,
  pub key: Option<String>,
}

#[derive(Clone, Debug)]
pub struct InputObjectTypeDefinition {
  pub name: String,
  pub fields: Vec<InputFieldDefinition>,
  pub description: Option<String>,
}

#[derive(Clone, Debug)]
pub struct EnumTypeDefinition {
  pub name: String,
  pub directives: Vec<Directive>,
  pub description: Option<String>,
  pub enum_values: Vec<EnumValueDefinition>,
}

#[derive(Clone, Debug)]
pub struct EnumValueDefinition {
  pub description: Option<String>,
  pub name: String,
  pub directives: Vec<Directive>,
}

#[derive(Clone, Debug)]
pub struct SchemaDefinition {
  pub query: String,
  pub mutation: Option<String>,
  pub directives: Vec<Directive>,
}

#[derive(Clone, Debug)]
pub struct InputFieldDefinition {
  pub name: String,
  pub of_type: Type,
  pub default_value: Option<serde_json::Value>,
  pub description: Option<String>,
}

#[derive(Clone, Debug, Setters)]
pub struct FieldDefinition {
  pub name: String,
  pub args: Vec<InputFieldDefinition>,
  pub of_type: Type,
  pub resolver: Option<Expression>,
  pub directives: Vec<Directive>,
  pub description: Option<String>,
  pub entity_resolver: Option<bool>,
  pub entity_type: Option<String>,
  pub entity_key: Option<String>,
  pub is_federation_key: bool,
}

impl FieldDefinition {
  pub fn to_lambda(self) -> Option<Lambda<serde_json::Value>> {
    self.resolver.map(Lambda::new)
  }

  pub fn resolver_or_default(
    mut self,
    default_res: Lambda<serde_json::Value>,
    other: impl Fn(Lambda<serde_json::Value>) -> Lambda<serde_json::Value>,
  ) -> Self {
    self.resolver = match self.resolver {
      None => Some(default_res.expression),
      Some(expr) => Some(other(Lambda::new(expr)).expression),
    };
    self
  }
}

#[derive(Clone, Debug)]
pub struct Directive {
  pub name: String,
  pub arguments: HashMap<String, Value>,
  pub index: usize,
}

#[derive(Clone, Debug)]
pub struct ScalarTypeDefinition {
  pub name: String,
  pub directive: Vec<Directive>,
  pub description: Option<String>,
}

#[derive(Clone, Debug)]
pub struct UnionTypeDefinition {
  pub name: String,
  pub directives: Vec<Directive>,
  pub description: Option<String>,
  pub types: Vec<String>,
}
impl Blueprint {
  pub fn new(
    schema: SchemaDefinition,
    definitions: Vec<Definition>,
    entity_resolvers: BTreeMap<String, Option<Expression>>,
  ) -> Self {
    Self { schema, definitions, entity_resolvers }
  }

  pub fn query(&self) -> String {
    self.schema.query.clone()
  }

  pub fn mutation(&self) -> Option<String> {
    self.schema.mutation.clone()
  }

  pub fn to_schema(&self, server: &config::Server) -> Schema {
    let mut schema = SchemaBuilder::from(self);

    if server.enable_apollo_tracing.unwrap_or(false) {
      schema = schema.extension(ApolloTracing);
    }

    let global_response_timeout = server.global_response_timeout.unwrap_or(0);
    if global_response_timeout > 0 {
      schema = schema
        .data(async_graphql::Value::from(global_response_timeout))
        .extension(GlobalTimeout);
    }

    if server.enable_query_validation() {
      schema = schema.validation_mode(ValidationMode::Strict);
    } else {
      schema = schema.validation_mode(ValidationMode::Fast);
    }
    if !server.enable_introspection() {
      schema = schema.disable_introspection();
    }

    // We should safely assume the blueprint is correct and,
    // generation of schema cannot fail.
    schema.finish().unwrap()
  }
}
