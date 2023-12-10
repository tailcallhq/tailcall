use std::collections::{BTreeMap, HashMap};

use super::{Server, TypeLike};
use crate::blueprint::compress::compress;
use crate::blueprint::*;
use crate::config::{Arg, Batch, Config, Field};
use crate::json::JsonSchema;
use crate::lambda::{Expression, Unsafe};
use crate::try_fold::TryFold;
use crate::valid::{Valid, ValidationError};

pub fn config_blueprint<'a>() -> TryFold<'a, Config, Blueprint, String> {
  let server = TryFoldConfig::<Blueprint>::new(|config, blueprint| {
    Valid::from(Server::try_from(config.server.clone())).map(|server| blueprint.server(server))
  });

  let schema = to_schema().transform::<Blueprint>(
    |schema, blueprint| blueprint.schema(schema),
    |blueprint| blueprint.schema,
  );

  let definitions = to_definitions().transform::<Blueprint>(
    |definitions, blueprint| blueprint.definitions(definitions),
    |blueprint| blueprint.definitions,
  );

  let upstream = to_upstream().transform::<Blueprint>(
    |upstream, blueprint| blueprint.upstream(upstream),
    |blueprint| blueprint.upstream,
  );

  server
    .and(schema)
    .and(definitions)
    .and(upstream)
    .update(apply_batching)
    .update(compress)
}

// Apply batching if any of the fields have a @http directive with groupBy field

pub fn apply_batching(mut blueprint: Blueprint) -> Blueprint {
  for def in blueprint.definitions.iter() {
    if let Definition::ObjectTypeDefinition(object_type_definition) = def {
      for field in object_type_definition.fields.iter() {
        if let Some(Expression::Unsafe(Unsafe::Http { group_by: Some(_), .. })) = field.resolver.clone() {
          blueprint.upstream.batch = blueprint.upstream.batch.or(Some(Batch::default()));
          return blueprint;
        }
      }
    }
  }
  blueprint
}

pub fn to_json_schema_for_field(field: &Field, config: &Config) -> JsonSchema {
  to_json_schema(field, config)
}
pub fn to_json_schema_for_args(args: &BTreeMap<String, Arg>, config: &Config) -> JsonSchema {
  let mut schema_fields = HashMap::new();
  for (name, arg) in args.iter() {
    schema_fields.insert(name.clone(), to_json_schema(arg, config));
  }
  JsonSchema::Obj(schema_fields)
}
fn to_json_schema<T>(field: &T, config: &Config) -> JsonSchema
where
  T: TypeLike,
{
  let type_of = field.name();
  let list = field.list();
  let required = field.non_null();
  let type_ = config.find_type(type_of);
  let schema = match type_ {
    Some(type_) => {
      let mut schema_fields = HashMap::new();
      for (name, field) in type_.fields.iter() {
        if field.unsafe_operation.is_none() && field.http.is_none() {
          schema_fields.insert(name.clone(), to_json_schema_for_field(field, config));
        }
      }
      JsonSchema::Obj(schema_fields)
    }
    None => match type_of {
      "String" => JsonSchema::Str {},
      "Int" => JsonSchema::Num {},
      "Boolean" => JsonSchema::Bool {},
      "JSON" => JsonSchema::Obj(HashMap::new()),
      _ => JsonSchema::Str {},
    },
  };

  if !required {
    if list {
      JsonSchema::Opt(Box::new(JsonSchema::Arr(Box::new(schema))))
    } else {
      JsonSchema::Opt(Box::new(schema))
    }
  } else if list {
    JsonSchema::Arr(Box::new(schema))
  } else {
    schema
  }
}

impl TryFrom<&Config> for Blueprint {
  type Error = ValidationError<String>;

  fn try_from(config: &Config) -> Result<Self, Self::Error> {
    config_blueprint().try_fold(config, Blueprint::default()).to_result()
  }
}
