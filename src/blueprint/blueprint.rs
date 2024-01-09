use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeSet, HashMap};
use std::num::NonZeroU64;
use std::time::Duration;

use async_graphql::dynamic::{Schema, SchemaBuilder};
use async_graphql::extensions::ApolloTracing;
use async_graphql::*;
use derive_setters::Setters;
use serde_json::Value;

use super::GlobalTimeout;
use crate::blueprint::from_config::Server;
use crate::config::{self, Upstream};
use crate::lambda::{Expression, Lambda};
use crate::mustache::Mustache;
use crate::rate_limiter::RateLimit;

/// Blueprint is an intermediary representation that allows us to generate graphQL APIs.
/// It can only be generated from a valid Config.
/// It allows us to choose a different GraphQL Backend, without re-writing all orchestration logic.
/// It's not optimized for REST APIs (yet).
#[derive(Clone, Debug, Default, Setters)]
pub struct Blueprint {
  pub definitions: Vec<Definition>,
  pub schema: SchemaDefinition,
  pub server: Server,
  pub upstream: Upstream,
  pub global_rate_limit: Option<GlobalRateLimit>,
}

#[derive(Clone, Debug)]
pub enum Type {
  NamedType { name: String, non_null: bool },
  ListType { of_type: Box<Type>, non_null: bool },
}

impl Default for Type {
  fn default() -> Self {
    Type::NamedType { name: "JSON".to_string(), non_null: false }
  }
}

impl Type {
  pub fn name(&self) -> &str {
    match self {
      Type::NamedType { name, .. } => name,
      Type::ListType { of_type, .. } => of_type.name(),
    }
  }

  pub fn is_nullable(&self) -> bool {
    !match self {
      Type::NamedType { non_null, .. } => *non_null,
      Type::ListType { non_null, .. } => *non_null,
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
  pub implements: BTreeSet<String>,
  pub rate_limit: Option<LocalRateLimit>,
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

#[derive(Clone, Debug, Default)]
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

#[derive(Clone, Debug)]
pub struct Cache {
  pub max_age: NonZeroU64,
  pub hasher: DefaultHasher,
}

#[derive(Clone, Debug)]
pub struct LocalRateLimit {
  pub duration: Duration,
  pub requests: NonZeroU64,
  pub group_by: Option<String>,
}

impl RateLimit for LocalRateLimit {
  fn duration(&self) -> Duration {
    self.duration
  }

  fn requests(&self) -> NonZeroU64 {
    self.requests
  }
}

#[derive(Clone, Debug)]
pub struct GlobalRateLimit {
  pub duration: Duration,
  pub requests: NonZeroU64,
  pub group_by: Option<GroupBy>,
}

impl RateLimit for GlobalRateLimit {
  fn duration(&self) -> Duration {
    self.duration
  }

  fn requests(&self) -> NonZeroU64 {
    self.requests
  }
}

#[derive(Clone, Debug)]
pub struct GroupBy {
  pub base: String,
  pub rest: Vec<String>,
}

impl GroupBy {
  pub fn get_global_key<T>(&self, req: &hyper::Request<T>) -> anyhow::Result<String> {
    Ok(match self.base.as_str() {
      "method" => req.method().to_string(),
      "uri" => req.uri().to_string(),
      "version" => format!("{:?}", req.version()),
      "headers" => match self.rest.first() {
        Some(key) => req
          .headers()
          .get(key)
          .map(|val| format!("{val:?}"))
          .ok_or(anyhow::anyhow!("Header key {key} not found"))?,
        _ => Err(anyhow::anyhow!("{:?} invalid path", self.rest))?,
      },
      x => Err(anyhow::anyhow!("{x} field is not supported"))?,
    })
  }
}

impl TryFrom<&String> for GroupBy {
  type Error = anyhow::Error;

  fn try_from(value: &String) -> anyhow::Result<Self> {
    let segments = Mustache::parse(value)?
      .expression_segments_owned()
      .into_iter()
      .next()
      .ok_or(anyhow::anyhow!("No Expression found in mustache"))?;
    let mut segments_iter = segments.into_iter();
    let first = segments_iter.next();
    first
      .map(|name| name.as_str() == "request")
      .ok_or(anyhow::anyhow!("Invalid name"))?;
    let second = segments_iter
      .next()
      .ok_or(anyhow::anyhow!("No field in request object specified"))?;

    match second.as_str() {
      "method" | "uri" | "version" | "headers" => {}
      x => Err(anyhow::anyhow!("`request` doesn't support field name `{x}`"))?,
    }

    Ok(Self { base: second, rest: segments_iter.collect() })
  }
}

impl TryFrom<&config::GlobalRateLimit> for GlobalRateLimit {
  type Error = anyhow::Error;

  fn try_from(
    config::GlobalRateLimit { unit, requests_per_unit, group_by }: &config::GlobalRateLimit,
  ) -> anyhow::Result<Self> {
    let duration = Duration::from_secs(unit.into_secs());
    Ok(GlobalRateLimit {
      duration,
      requests: *requests_per_unit,
      group_by: group_by.as_ref().map(GroupBy::try_from).transpose()?,
    })
  }
}

impl From<&config::LocalRateLimit> for LocalRateLimit {
  fn from(config::LocalRateLimit { unit, requests_per_unit, group_by }: &config::LocalRateLimit) -> Self {
    let duration = Duration::from_secs(unit.into_secs());
    LocalRateLimit { duration, requests: *requests_per_unit, group_by: group_by.as_ref().map(String::clone) }
  }
}

#[derive(Clone, Debug, Setters, Default)]
pub struct FieldDefinition {
  pub name: String,
  pub args: Vec<InputFieldDefinition>,
  pub of_type: Type,
  pub resolver: Option<Expression>,
  pub directives: Vec<Directive>,
  pub description: Option<String>,
  pub cache: Option<Cache>,
  pub rate_limit: Option<LocalRateLimit>,
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
  pub types: BTreeSet<String>,
}
impl Blueprint {
  pub fn query(&self) -> String {
    self.schema.query.clone()
  }

  pub fn mutation(&self) -> Option<String> {
    self.schema.mutation.clone()
  }

  pub fn to_schema(&self) -> Schema {
    let server = &self.server;
    let mut schema = SchemaBuilder::from(self);

    if server.enable_apollo_tracing {
      schema = schema.extension(ApolloTracing);
    }

    if server.global_response_timeout > 0 {
      schema = schema
        .data(async_graphql::Value::from(server.global_response_timeout))
        .extension(GlobalTimeout);
    }

    if server.get_enable_query_validation() {
      schema = schema.validation_mode(ValidationMode::Strict);
    } else {
      schema = schema.validation_mode(ValidationMode::Fast);
    }
    if !server.get_enable_introspection() {
      schema = schema.disable_introspection();
    }

    // We should safely assume the blueprint is correct and,
    // generation of schema cannot fail.
    schema.finish().unwrap()
  }
}
