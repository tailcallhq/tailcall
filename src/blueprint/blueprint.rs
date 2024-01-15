use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeSet, HashMap};
use std::num::NonZeroU64;

use async_graphql::dynamic::{Schema, SchemaBuilder};
use async_graphql::extensions::ApolloTracing;
use async_graphql::ValidationMode;
use derive_setters::Setters;
use serde_json::Value;

use super::{is_scalar, to_type, GlobalTimeout};
use crate::blueprint::from_config::Server;
use crate::config::{self, Config, Upstream};
use crate::lambda::{Expression, Lambda, Unsafe};
use crate::valid::Valid;

struct MustachePartsValidator<'a> {
  type_of: &'a config::Type,
  config: &'a Config,
  field: &'a FieldDefinition,
}

impl<'a> MustachePartsValidator<'a> {
  fn new(type_of: &'a config::Type, config: &'a Config, field: &'a FieldDefinition) -> Self {
    Self { type_of, config, field }
  }

  fn validate_type(&self, parts: &[String], is_query: bool) -> Result<(), String> {
    let mut len = parts.len();
    let mut type_of = self.type_of;
    for item in parts {
      let field = type_of.fields.get(item).ok_or_else(|| {
        format!(
          "no value '{}' found",
          parts[0..parts.len() - len + 1].join(".").as_str()
        )
      })?;
      let val_type = to_type(field, None);

      if !is_query && val_type.is_nullable() {
        return Err(format!("value '{}' is a nullable type", item.as_str()));
      } else if len == 1 && !is_scalar(val_type.name()) {
        return Err(format!("value '{}' is not of a scalar type", item.as_str()));
      } else if len == 1 {
        break;
      }

      type_of = self
        .config
        .find_type(&field.type_of)
        .ok_or_else(|| format!("no type '{}' found", parts.join(".").as_str()))?;

      len -= 1;
    }

    Ok(())
  }

  fn validate(&self, parts: &[String], is_query: bool) -> Valid<(), String> {
    let config = self.config;
    let args = &self.field.args;

    if parts.len() < 2 {
      return Valid::fail("too few parts in template".to_string());
    }

    let head = parts[0].as_str();
    let tail = parts[1].as_str();

    match head {
      "value" => {
        // all items on parts except the first one
        let tail = &parts[1..];

        if let Err(e) = self.validate_type(tail, is_query) {
          return Valid::fail(e);
        }
      }
      "args" => {
        // XXX this is a linear search but it's cost is less than that of
        // constructing a HashMap since we'd have 3-4 arguments at max in
        // most cases
        if let Some(arg) = args.iter().find(|arg| arg.name == tail) {
          if let Type::ListType { .. } = arg.of_type {
            return Valid::fail(format!("can't use list type '{tail}' here"));
          }

          // we can use non-scalar types in args

          if !is_query && arg.default_value.is_none() && arg.of_type.is_nullable() {
            return Valid::fail(format!("argument '{tail}' is a nullable type"));
          }
        } else {
          return Valid::fail(format!("no argument '{tail}' found"));
        }
      }
      "vars" => {
        if config.server.vars.get(tail).is_none() {
          return Valid::fail(format!("var '{tail}' is not set in the server config"));
        }
      }
      "headers" | "env" => {
        // "headers" and "env" refers to values known at runtime, which we can't
        // validate here
      }
      _ => {
        return Valid::fail(format!("unknown template directive '{head}'"));
      }
    }

    Valid::succeed(())
  }
}

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

#[derive(Clone, Debug, Setters, Default)]
pub struct FieldDefinition {
  pub name: String,
  pub args: Vec<InputFieldDefinition>,
  pub of_type: Type,
  pub resolver: Option<Expression>,
  pub directives: Vec<Directive>,
  pub description: Option<String>,
  pub cache: Option<Cache>,
}

impl FieldDefinition {
  pub fn validate_field(&self, type_of: &config::Type, config: &Config) -> Valid<(), String> {
    // XXX we could use `Mustache`'s `render` method with a mock
    // struct implementing the `PathString` trait encapsulating `validation_map`
    // but `render` simply falls back to the default value for a given
    // type if it doesn't exist, so we wouldn't be able to get enough
    // context from that method alone
    // So we must duplicate some of that logic here :(

    let parts_validator = MustachePartsValidator::new(type_of, config, self);

    if let Some(Expression::Unsafe(Unsafe::Http { req_template, .. })) = &self.resolver {
      Valid::from_iter(req_template.root_url.expression_segments(), |parts| {
        parts_validator.validate(parts, false).trace("path")
      })
      .and(Valid::from_iter(req_template.query.clone(), |query| {
        let (_, mustache) = query;

        Valid::from_iter(mustache.expression_segments(), |parts| {
          parts_validator.validate(parts, true).trace("query")
        })
      }))
      .unit()
    } else if let Some(Expression::Unsafe(Unsafe::GraphQLEndpoint { req_template, .. })) = &self.resolver {
      Valid::from_iter(req_template.headers.clone(), |(_, mustache)| {
        Valid::from_iter(mustache.expression_segments(), |parts| {
          parts_validator.validate(parts, true).trace("headers")
        })
      })
      .and_then(|_| {
        if let Some(args) = &req_template.operation_arguments {
          Valid::from_iter(args, |(_, mustache)| {
            Valid::from_iter(mustache.expression_segments(), |parts| {
              parts_validator.validate(parts, true).trace("args")
            })
          })
        } else {
          Valid::succeed(Default::default())
        }
      })
      .unit()
    } else if let Some(Expression::Unsafe(Unsafe::Grpc { req_template, .. })) = &self.resolver {
      Valid::from_iter(req_template.url.expression_segments(), |parts| {
        parts_validator.validate(parts, false).trace("path")
      })
      .and(
        Valid::from_iter(req_template.headers.clone(), |(_, mustache)| {
          Valid::from_iter(mustache.expression_segments(), |parts| {
            parts_validator.validate(parts, true).trace("headers")
          })
        })
        .map_to(()),
      )
    } else {
      Valid::succeed(())
    }
  }

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
