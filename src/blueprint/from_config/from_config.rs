use std::collections::{BTreeMap, BTreeSet, HashMap};

#[allow(unused_imports)]
use async_graphql::InputType;
use regex::Regex;

use super::common::{to_type, TypeLike};
use super::Server;
use crate::blueprint::compress::compress;
use crate::blueprint::Type::ListType;
use crate::blueprint::*;
use crate::config::{Arg, Batch, Config, Field};
use crate::directive::DirectiveCodec;
use crate::json::JsonSchema;
use crate::lambda::{Expression, Lambda, Unsafe};
use crate::try_fold::TryFold;
use crate::valid::{Valid, ValidationError};
use crate::{blueprint, config};

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
        if let Some(Expression::Unsafe(Unsafe::Http(_request_template, Some(_), _dl))) = field.resolver.clone() {
          blueprint.upstream.batch = blueprint.upstream.batch.or(Some(Batch::default()));
          return blueprint;
        }
      }
    }
  }
  blueprint
}

fn to_definitions<'a>() -> TryFold<'a, Config, Vec<Definition>, String> {
  TryFold::<Config, Vec<Definition>, String>::new(|config, _| {
    let output_types = config.output_types();
    let input_types = config.input_types();
    Valid::from_iter(config.graphql.types.iter(), |(name, type_)| {
      let dbl_usage = input_types.contains(name) && output_types.contains(name);
      if let Some(variants) = &type_.variants {
        if !variants.is_empty() {
          to_enum_type_definition(name, type_, variants).trace(name)
        } else {
          Valid::fail("No variants found for enum".to_string())
        }
      } else if type_.scalar {
        to_scalar_type_definition(name).trace(name)
      } else if dbl_usage {
        Valid::fail("type is used in input and output".to_string()).trace(name)
      } else {
        to_object_type_definition(name, type_, config)
          .trace(name)
          .and_then(|definition| match definition.clone() {
            Definition::ObjectTypeDefinition(object_type_definition) => {
              if config.input_types().contains(name) {
                to_input_object_type_definition(object_type_definition).trace(name)
              } else if type_.interface {
                to_interface_type_definition(object_type_definition).trace(name)
              } else {
                Valid::succeed(definition)
              }
            }
            _ => Valid::succeed(definition),
          })
      }
    })
    .map(|mut types| {
      types.extend(config.graphql.unions.iter().map(to_union_type_definition));
      types
    })
  })
}

fn to_enum_type_definition(name: &str, type_: &config::Type, variants: &BTreeSet<String>) -> Valid<Definition, String> {
  let enum_type_definition = Definition::EnumTypeDefinition(EnumTypeDefinition {
    name: name.to_string(),
    directives: Vec::new(),
    description: type_.doc.clone(),
    enum_values: variants
      .iter()
      .map(|variant| EnumValueDefinition { description: None, name: variant.clone(), directives: Vec::new() })
      .collect(),
  });
  Valid::succeed(enum_type_definition)
}

fn to_object_type_definition(name: &str, type_of: &config::Type, config: &Config) -> Valid<Definition, String> {
  to_fields(type_of, config).map(|fields| {
    Definition::ObjectTypeDefinition(ObjectTypeDefinition {
      name: name.to_string(),
      description: type_of.doc.clone(),
      fields,
      implements: type_of.implements.clone(),
    })
  })
}

fn to_fields(type_of: &config::Type, config: &Config) -> Valid<Vec<FieldDefinition>, String> {
  let to_field = |name: &String, field: &Field| {
    let directives = field.resolvable_directives();
    if directives.len() > 1 {
      return Valid::fail(format!("Multiple resolvers detected [{}]", directives.join(", ")));
    }

    update_args()
      .and(update_http().trace("@http"))
      .and(update_unsafe().trace("@unsafe"))
      .and(update_const_field().trace("@const"))
      .and(update_modify().trace("@modify"))
      .try_fold(&(config, field, type_of, name), FieldDefinition::default())
  };

  let fields = Valid::from_iter(
    type_of
      .fields
      .iter()
      .filter(|field| field.1.modify.as_ref().map(|m| !m.omit).unwrap_or(true)),
    |(name, field)| {
      validate_field_type_exist(config, field)
        .and(to_field(name, field))
        .trace(name)
    },
  );

  let to_added_field =
    |add_field: &config::AddField, type_of: &config::Type| -> Valid<blueprint::FieldDefinition, String> {
      let source_field = type_of
        .fields
        .iter()
        .find(|&(field_name, _)| *field_name == add_field.path[0]);
      match source_field {
        Some((_, source_field)) => {
          let new_field = config::Field {
            type_of: source_field.type_of.clone(),
            list: source_field.list,
            required: source_field.required,
            list_type_required: source_field.list_type_required,
            args: source_field.args.clone(),
            doc: None,
            modify: source_field.modify.clone(),
            http: source_field.http.clone(),
            unsafe_operation: source_field.unsafe_operation.clone(),
            const_field: source_field.const_field.clone(),
          };
          to_field(&add_field.name, &new_field)
            .and_then(|field_definition| {
              let added_field_path = match source_field.http {
                Some(_) => add_field.path[1..].iter().map(|s| s.to_owned()).collect::<Vec<_>>(),
                None => add_field.path.clone(),
              };
              let invalid_path_handler =
                |field_name: &str, _added_field_path: &[String], original_path: &[String]| -> Valid<Type, String> {
                  Valid::fail_with(
                    "Cannot add field".to_string(),
                    format!("Path [{}] does not exist", original_path.join(", ")),
                  )
                  .trace(field_name)
                };
              let path_resolver_error_handler = |resolver_name: &str,
                                                 field_type: &str,
                                                 field_name: &str,
                                                 original_path: &[String]|
               -> Valid<Type, String> {
                Valid::<Type, String>::fail_with(
                  "Cannot add field".to_string(),
                  format!(
                    "Path: [{}] contains resolver {} at [{}.{}]",
                    original_path.join(", "),
                    resolver_name,
                    field_type,
                    field_name
                  ),
                )
              };
              update_resolver_from_path(
                &ProcessPathContext {
                  path: &added_field_path,
                  field: source_field,
                  type_info: type_of,
                  is_required: false,
                  config,
                  invalid_path_handler: &invalid_path_handler,
                  path_resolver_error_handler: &path_resolver_error_handler,
                  original_path: &add_field.path,
                },
                field_definition,
              )
            })
            .trace(config::AddField::trace_name().as_str())
        }
        None => Valid::fail(format!(
          "Could not find field {} in path {}",
          add_field.path[0],
          add_field.path.join(",")
        )),
      }
    };

  let added_fields = Valid::from_iter(type_of.added_fields.iter(), |added_field| {
    to_added_field(added_field, type_of)
  });
  fields.zip(added_fields).map(|(mut fields, added_fields)| {
    fields.extend(added_fields);
    fields
  })
}

fn validate_field_type_exist(config: &Config, field: &Field) -> Valid<(), String> {
  let field_type = &field.type_of;
  if !is_scalar(field_type) && !config.contains(field_type) {
    Valid::fail(format!("Undeclared type '{field_type}' was found"))
  } else {
    Valid::succeed(())
  }
}

pub fn is_scalar(type_name: &str) -> bool {
  ["String", "Int", "Float", "Boolean", "ID", "JSON"].contains(&type_name)
}

type InvalidPathHandler = dyn Fn(&str, &[String], &[String]) -> Valid<Type, String>;
type PathResolverErrorHandler = dyn Fn(&str, &str, &str, &[String]) -> Valid<Type, String>;

#[derive(Clone)]
struct ProcessPathContext<'a> {
  path: &'a [String],
  field: &'a config::Field,
  type_info: &'a config::Type,
  is_required: bool,
  config: &'a Config,
  invalid_path_handler: &'a InvalidPathHandler,
  path_resolver_error_handler: &'a PathResolverErrorHandler,
  original_path: &'a [String],
}

// Helper function to recursively process the path and return the corresponding type
fn process_path(context: ProcessPathContext) -> Valid<Type, String> {
  let path = context.path;
  let field = context.field;
  let type_info = context.type_info;
  let is_required = context.is_required;
  let config = context.config;
  let invalid_path_handler = context.invalid_path_handler;
  let path_resolver_error_handler = context.path_resolver_error_handler;
  if let Some((field_name, remaining_path)) = path.split_first() {
    if field_name.parse::<usize>().is_ok() {
      let mut modified_field = field.clone();
      modified_field.list = false;
      return process_path(ProcessPathContext {
        config,
        type_info,
        invalid_path_handler,
        path_resolver_error_handler,
        path: remaining_path,
        field: &modified_field,
        is_required: false,
        original_path: context.original_path,
      });
    }
    let target_type_info = type_info
      .fields
      .get(field_name)
      .map(|_| type_info)
      .or_else(|| config.find_type(&field.type_of));

    if let Some(type_info) = target_type_info {
      return process_field_within_type(ProcessFieldWithinTypeContext {
        field,
        field_name,
        remaining_path,
        type_info,
        is_required,
        config,
        invalid_path_handler,
        path_resolver_error_handler,
        original_path: context.original_path,
      });
    }
    return invalid_path_handler(field_name, path, context.original_path);
  }

  Valid::succeed(to_type(field, Some(is_required)))
}

struct ProcessFieldWithinTypeContext<'a> {
  field: &'a config::Field,
  field_name: &'a str,
  remaining_path: &'a [String],
  type_info: &'a config::Type,
  is_required: bool,
  config: &'a Config,
  invalid_path_handler: &'a InvalidPathHandler,
  path_resolver_error_handler: &'a PathResolverErrorHandler,
  original_path: &'a [String],
}

fn process_field_within_type(context: ProcessFieldWithinTypeContext) -> Valid<Type, String> {
  let field = context.field;
  let field_name = context.field_name;
  let remaining_path = context.remaining_path;
  let type_info = context.type_info;
  let is_required = context.is_required;
  let config = context.config;
  let invalid_path_handler = context.invalid_path_handler;
  let path_resolver_error_handler = context.path_resolver_error_handler;

  if let Some(next_field) = type_info.fields.get(field_name) {
    if next_field.has_resolver() {
      let next_dir_http = next_field.http.as_ref().map(|_| "http");
      let next_dir_const = next_field.const_field.as_ref().map(|_| "const");
      return path_resolver_error_handler(
        next_dir_http.or(next_dir_const).unwrap_or("unsafe"),
        &field.type_of,
        field_name,
        context.original_path,
      )
      .and(process_path(ProcessPathContext {
        type_info,
        is_required,
        config,
        invalid_path_handler,
        path_resolver_error_handler,
        path: remaining_path,
        field: next_field,
        original_path: context.original_path,
      }));
    }

    let next_is_required = is_required && next_field.required;
    if is_scalar(&next_field.type_of) {
      return process_path(ProcessPathContext {
        type_info,
        config,
        invalid_path_handler,
        path_resolver_error_handler,
        path: remaining_path,
        field: next_field,
        is_required: next_is_required,
        original_path: context.original_path,
      });
    }

    if let Some(next_type_info) = config.find_type(&next_field.type_of) {
      return process_path(ProcessPathContext {
        config,
        invalid_path_handler,
        path_resolver_error_handler,
        path: remaining_path,
        field: next_field,
        type_info: next_type_info,
        is_required: next_is_required,
        original_path: context.original_path,
      })
      .and_then(|of_type| {
        if next_field.list {
          Valid::succeed(ListType { of_type: Box::new(of_type), non_null: is_required })
        } else {
          Valid::succeed(of_type)
        }
      });
    }
  } else if let Some((head, tail)) = remaining_path.split_first() {
    if let Some(field) = type_info.fields.get(head) {
      return process_path(ProcessPathContext {
        path: tail,
        field,
        type_info,
        is_required,
        config,
        invalid_path_handler,
        path_resolver_error_handler,
        original_path: context.original_path,
      });
    }
  }

  invalid_path_handler(field_name, remaining_path, context.original_path)
}

fn item_is_numberic(list: &[String]) -> bool {
  list.iter().any(|s| {
    let re = Regex::new(r"^\d+$").unwrap();
    re.is_match(s)
  })
}

fn update_resolver_from_path(
  context: &ProcessPathContext,
  base_field: blueprint::FieldDefinition,
) -> Valid<blueprint::FieldDefinition, String> {
  let has_index = item_is_numberic(context.path);

  process_path(context.clone()).and_then(|of_type| {
    let mut updated_base_field = base_field;
    let resolver = Lambda::context_path(context.path.to_owned());
    if has_index {
      updated_base_field.of_type = Type::NamedType { name: of_type.name().to_string(), non_null: false }
    } else {
      updated_base_field.of_type = of_type;
    }

    updated_base_field = updated_base_field.resolver_or_default(resolver, |r| r.to_input_path(context.path.to_owned()));
    Valid::succeed(updated_base_field)
  })
}

fn update_args<'a>() -> TryFold<'a, (&'a Config, &'a Field, &'a config::Type, &'a str), FieldDefinition, String> {
  TryFold::<(&Config, &Field, &config::Type, &str), FieldDefinition, String>::new(|(_, field, _, name), _| {
    // TODO! assert type name
    Valid::from_iter(field.args.iter(), |(name, arg)| {
      Valid::succeed(InputFieldDefinition {
        name: name.clone(),
        description: arg.doc.clone(),
        of_type: to_type(arg, None),
        default_value: arg.default_value.clone(),
      })
    })
    .map(|args| FieldDefinition {
      name: name.to_string(),
      description: field.doc.clone(),
      args,
      of_type: to_type(*field, None),
      directives: Vec::new(),
      resolver: None,
    })
  })
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

impl TypeLike for Field {
  fn name(&self) -> &str {
    &self.type_of
  }

  fn list(&self) -> bool {
    self.list
  }

  fn non_null(&self) -> bool {
    self.required
  }

  fn list_type_required(&self) -> bool {
    self.list_type_required
  }
}
impl TypeLike for Arg {
  fn name(&self) -> &str {
    &self.type_of
  }

  fn list(&self) -> bool {
    self.list
  }

  fn non_null(&self) -> bool {
    self.required
  }

  fn list_type_required(&self) -> bool {
    false
  }
}
