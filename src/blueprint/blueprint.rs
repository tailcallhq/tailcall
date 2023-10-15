use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::net::{AddrParseError, IpAddr};

use async_graphql::dynamic::{Schema, SchemaBuilder};
use async_graphql::extensions::ApolloTracing;
use async_graphql::*;
use derive_setters::Setters;
use hyper::HeaderMap;
use reqwest::header::{HeaderName, HeaderValue};
use serde_json::Value;

use super::GlobalTimeout;
use crate::config;
use crate::lambda::{Expression, Lambda};
use crate::valid::{Valid, ValidExtensions, ValidationError, VectorExtension};

/// Blueprint is an intermediary representation that allows us to generate graphQL APIs.
/// It can only be generated from a valid Config.
/// It allows us to choose a different GraphQL Backend, without re-writing all orchestration logic.
/// It's not optimized for REST APIs (yet).
#[derive(Clone, Debug)]
pub struct Blueprint {
  pub definitions: Vec<Definition>,
  pub schema: SchemaDefinition,
  pub server: Server,
}

#[derive(Clone, Debug, Setters)]
pub struct Server {
  pub enable_apollo_tracing: bool,
  pub enable_cache_control_header: bool,
  pub enable_graphiql: Option<String>,
  pub enable_introspection: bool,
  pub enable_query_validation: bool,
  pub enable_response_validation: bool,
  pub global_response_timeout: i64,
  pub port: u16,
  pub hostname: IpAddr,
  pub upstream: crate::config::Upstream,
  pub vars: BTreeMap<String, String>,
  pub response_headers: HeaderMap,
}

impl TryFrom<crate::config::Server> for Server {
  type Error = ValidationError<String>;

  fn try_from(config_server: config::Server) -> Valid<Self, String> {
    // Configure other server settings
    let mut server = configure_server(&config_server)?;
    server.upstream.base_url = handle_base_url(config_server.upstream.base_url.clone())?;
    Valid::Ok(server.clone())
  }
}
fn validate_hostname(hostname: String) -> Valid<IpAddr, String> {
  let host = if hostname == "localhost" {
    IpAddr::from([127, 0, 0, 1])
  } else {
    hostname
      .parse()
      .map_err(|e: AddrParseError| ValidationError::new(format!("Parsing failed because of {}", e)))
      .trace("hostname")
      .trace("@server")
      .trace("schema")?
  };
  Ok(host)
}
const RESTRICTED_ROUTES: &[&str] = &["/", "/graphql"];

fn handle_graphiql(graphiql: Option<String>) -> Valid<Option<String>, String> {
  let mut graph = None;
  if let Some(enable_graphiql) = graphiql.clone() {
    let lowered_route = enable_graphiql.to_lowercase();
    if RESTRICTED_ROUTES.contains(&lowered_route.as_str()) {
      return Err(
        ValidationError::new(format!(
          "Cannot use restricted routes '{}' for enabling graphiql",
          enable_graphiql
        ))
        .trace("enableGraphiql")
        .trace("@server")
        .trace("schema"),
      );
    } else {
      graph = Some(enable_graphiql);
    }
  };
  Ok(graph)
}

fn handle_response_headers(resp_headers: BTreeMap<String, String>) -> Valid<HeaderMap, String> {
  let headers = resp_headers
    .validate_all(|(k, v)| {
      let name = HeaderName::from_bytes(k.as_bytes())
        .map_err(|e| ValidationError::new(format!("Parsing failed because of {}", e)));
      let value =
        HeaderValue::from_str(v.as_str()).map_err(|e| ValidationError::new(format!("Parsing failed because of {}", e)));
      name.validate_both(value)
    })
    .trace("responseHeaders")
    .trace("@server")
    .trace("schema")?;

  let mut response_headers = HeaderMap::new();
  response_headers.extend(headers);
  Ok(response_headers)
}

fn handle_base_url(base_url: Option<String>) -> Valid<Option<String>, String> {
  let base_url = if let Some(base_url) = base_url {
    Valid::Ok(reqwest::Url::parse(base_url.as_str()).map_err(|e| ValidationError::new(e.to_string()))?)?;
    Some(base_url)
  } else {
    None
  };
  Ok(base_url)
}

fn configure_server(config_server: &config::Server) -> Valid<Server, String> {
  Ok(Server {
    enable_apollo_tracing: config_server.enable_apollo_tracing(),
    enable_cache_control_header: config_server.enable_cache_control(),
    enable_graphiql: handle_graphiql(config_server.enable_graphiql())?,
    enable_introspection: config_server.enable_introspection(),
    enable_query_validation: config_server.enable_query_validation(),
    enable_response_validation: config_server.enable_http_validation(),
    global_response_timeout: config_server.get_global_response_timeout(),
    port: config_server.get_port(),
    hostname: validate_hostname(config_server.get_hostname().to_lowercase())?,
    upstream: config_server.get_upstream(),
    vars: config_server.get_vars(),
    response_headers: handle_response_headers(config_server.get_response_headers().0)?,
  })
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
  pub implements: BTreeSet<String>,
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
  pub fn new(schema: SchemaDefinition, definitions: Vec<Definition>, server: Server) -> Self {
    Self { schema, definitions, server }
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
