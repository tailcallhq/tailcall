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

  let typecheck = TryFoldConfig::<Blueprint>::new(|_, blueprint| {
    blueprint
      .definitions
      .iter()
      .fold(Valid::succeed(()), |acc, def| {
        acc.and(typecheck_definition(def, &blueprint))
      })
      .map(|_| blueprint)
  });

  server
    .and(schema)
    .and(definitions)
    .and(upstream)
    .and(typecheck)
    .update(apply_batching)
    .update(compress)
}

fn typecheck_definition(def: &Definition, blueprint: &Blueprint) -> Valid<(), String> {
  let maybe_upstream_query = &blueprint.upstream.query;
  if let Some(upstream_query) = maybe_upstream_query {
    let query_def = blueprint.definitions.iter().find(|d| d.name() == upstream_query);
    match def {
      Definition::ObjectTypeDefinition(ObjectTypeDefinition { name, fields, .. }) => {
        if name != "Query" {
          Valid::succeed(()) // TODO: handle other cases
        } else {
          fields.iter().fold(Valid::succeed(()), |acc, field| {
            let valid_field = if let Some(resolver) = &field.resolver {
              match resolver {
                Expression::Unsafe(Unsafe::GraphQLEndpoint { req_template, .. }) => {
                  match query_def {
                    Some(Definition::ObjectTypeDefinition(ObjectTypeDefinition { fields, .. })) => {
                      if let Some(base_url) = &blueprint.upstream.base_url {
                        if base_url == &req_template.url {
                          // this hits a user defined field
                          if let Some(q) = fields.iter().find(|f| f.name == req_template.operation_name) {
                            if q.of_type == field.of_type {
                              Valid::succeed(())
                            } else {
                              Valid::from_validation_err(ValidationError::new(format!("Mismatched return type")))
                            }
                          } else {
                            Valid::from_validation_err(ValidationError::new(format!(
                              "No GraphQL endpoint with name {}",
                              req_template.operation_name
                            )))
                          }
                        } else {
                          // this is an external call
                          Valid::succeed(())
                        }
                      } else {
                        // no base url defined in upstream, this must be an
                        // external call
                        Valid::succeed(())
                      }
                    }
                    None => Valid::from_validation_err(ValidationError::new(format!("No query object defined"))),
                    _ => Valid::succeed(()),
                  }
                }
                _ => Valid::succeed(()), // TODO
              }
            } else {
              Valid::succeed(())
            };
            acc.and(valid_field)
          })
        }
      }
      _ => Valid::succeed(()), // TODO: what should happen for other cases?
    }
  } else {
    log::warn!("No schema provided for upstream, skipping type check for @graphQL");
    Valid::succeed(())
  }
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
