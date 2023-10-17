#![allow(clippy::too_many_arguments)]

use std::collections::{BTreeMap, HashMap};

#[allow(unused_imports)]
use async_graphql::InputType;

use crate::blueprint::foldrs::definitions::DefinitionsFold;
use crate::blueprint::foldrs::directive::DirectiveFold;
use crate::blueprint::foldrs::schema::SchemaFold;
use crate::blueprint::foldrs::server::ServerFold;
use crate::blueprint::Type::ListType;
use crate::blueprint::*;
use crate::config;
use crate::config::{Arg, Config, Field};
use crate::json::JsonSchema;
use crate::try_fold::{TryFold, TryFolding};
use crate::valid::{ValidExtensions, ValidationError, VectorExtension};

/// Just [`crate::valid::Valid`] with `String set as error type
pub type Valid<T> = crate::valid::Valid<T, String>;

pub fn config_blueprint(config: &Config) -> Valid<Blueprint> {
  let blueprint = TryFold::try_all(vec![SchemaFold, DefinitionsFold, DirectiveFold, ServerFold])
    .try_fold(config, Blueprint::default())?;
  Ok(compress::compress(blueprint))
}

fn is_scalar(type_name: &str) -> bool {
  ["String", "Int", "Float", "Boolean", "ID", "JSON"].contains(&type_name)
}
// Helper function to recursively process the path and return the corresponding type
pub fn process_path(
  path: &[String],
  field: &config::Field,
  type_info: &config::Type,
  is_required: bool,
  config: &Config,
  invalid_path_handler: &dyn Fn(&str, &[String]) -> Valid<Type>,
) -> Valid<Type> {
  if let Some((field_name, remaining_path)) = path.split_first() {
    if field_name.parse::<usize>().is_ok() {
      let mut modified_field = field.clone();
      modified_field.list = false;
      return process_path(
        remaining_path,
        &modified_field,
        type_info,
        false,
        config,
        invalid_path_handler,
      );
    }
    let target_type_info = type_info
      .fields
      .get(field_name)
      .map(|_| type_info)
      .or_else(|| config.find_type(&field.type_of));

    if let Some(type_info) = target_type_info {
      return process_field_within_type(
        field,
        field_name,
        remaining_path,
        type_info,
        is_required,
        config,
        invalid_path_handler,
      );
    }
    return invalid_path_handler(field_name, path);
  }

  Valid::Ok(to_type(
    &field.type_of,
    field.list,
    is_required,
    field.list_type_required,
  ))
}

fn process_field_within_type(
  field: &config::Field,
  field_name: &str,
  remaining_path: &[String],
  type_info: &config::Type,
  is_required: bool,
  config: &Config,
  invalid_path_handler: &dyn Fn(&str, &[String]) -> Valid<Type>,
) -> Valid<Type> {
  if let Some(next_field) = type_info.fields.get(field_name) {
    if next_field.has_resolver() {
      return Valid::<Type>::validate_or(
        Valid::fail(format!(
          "Inline can't be done because of {} resolver at [{}.{}]",
          {
            let next_dir_http = next_field.http.as_ref().map(|_| "http");
            let next_dir_const = next_field.const_field.as_ref().map(|_| "const");
            next_dir_http.or(next_dir_const).unwrap_or("unsafe")
          },
          field.type_of,
          field_name
        )),
        process_path(
          remaining_path,
          next_field,
          type_info,
          is_required,
          config,
          invalid_path_handler,
        ),
      );
    }

    let next_is_required = is_required && next_field.required;
    if is_scalar(&next_field.type_of) {
      return process_path(
        remaining_path,
        next_field,
        type_info,
        next_is_required,
        config,
        invalid_path_handler,
      );
    }

    if let Some(next_type_info) = config.find_type(&next_field.type_of) {
      let of_type = process_path(
        remaining_path,
        next_field,
        next_type_info,
        next_is_required,
        config,
        invalid_path_handler,
      )?;

      return if next_field.list {
        Valid::Ok(ListType { of_type: Box::new(of_type), non_null: is_required })
      } else {
        Ok(of_type)
      };
    }
  } else if let Some((head, tail)) = remaining_path.split_first() {
    if let Some(field) = type_info.fields.get(head) {
      return process_path(tail, field, type_info, is_required, config, invalid_path_handler);
    }
  }

  invalid_path_handler(field_name, remaining_path)
}

pub(super) fn to_args(field: &config::Field) -> Valid<Vec<InputFieldDefinition>> {
  // TODO! assert type name
  field.args.iter().validate_all(|(name, arg)| {
    Valid::Ok(InputFieldDefinition {
      name: name.clone(),
      description: arg.doc.clone(),
      of_type: to_type(&arg.type_of, arg.list, arg.required, false),
      default_value: arg.default_value.clone(),
    })
  })
}

pub(super) fn to_type(name: &str, list: bool, non_null: bool, list_type_required: bool) -> Type {
  if list {
    Type::ListType {
      of_type: Box::new(Type::NamedType { name: name.to_string(), non_null: list_type_required }),
      non_null,
    }
  } else {
    Type::NamedType { name: name.to_string(), non_null }
  }
}

pub fn to_json_schema_for_field(field: &Field, config: &Config) -> JsonSchema {
  to_json_schema(&field.type_of, field.required, field.list, config)
}
pub fn to_json_schema_for_args(args: &BTreeMap<String, Arg>, config: &Config) -> JsonSchema {
  let mut schema_fields = HashMap::new();
  for (name, arg) in args.iter() {
    schema_fields.insert(
      name.clone(),
      to_json_schema(&arg.type_of, arg.required, arg.list, config),
    );
  }
  JsonSchema::Obj(schema_fields)
}
pub fn to_json_schema(type_of: &str, required: bool, list: bool, config: &Config) -> JsonSchema {
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

pub(super) fn validate_field_has_resolver((name, field): (&String, &Field)) -> Valid<()> {
  if field.has_resolver() {
    Ok(())
  } else {
    Valid::fail("No resolver has been found in the schema".to_owned()).trace(name)
  }
}

pub(super) fn validate_field_type_exist(config: &Config, field: &Field) -> Valid<()> {
  let field_type = &field.type_of;

  if !is_scalar(field_type) && !config.contains(field_type) {
    Valid::fail(format!("Undeclared type '{field_type}' was found"))
  } else {
    Valid::Ok(())
  }
}

impl TryFrom<&Config> for Blueprint {
  type Error = ValidationError<String>;

  fn try_from(config: &Config) -> Result<Self, Self::Error> {
    config_blueprint(config)
  }
}
