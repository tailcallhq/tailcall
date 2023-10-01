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
fn to_directive(const_directive: ConstDirective) -> Valid<Directive> {
  let arguments = const_directive
    .arguments
    .into_iter()
    .map(|(k, v)| {
      let value = v.node.into_json();
      if let Ok(value) = value {
        return Ok((k.node.to_string(), value));
      }
      Err(value.unwrap_err())
    })
    .collect::<Result<HashMap<String, serde_json::Value>, _>>()
    .map_err(|e| ValidationError::new(e.to_string()))?;

  Ok(Directive { name: const_directive.name.node.clone().to_string(), arguments, index: 0 })
}
fn to_schema(config: &Config) -> Valid<SchemaDefinition> {
  let query_type_name = config
    .graphql
    .schema
    .query
    .as_ref()
    .validate_some("Query root is missing".to_owned())?;

  validate_query(config).validate_or(validate_mutation(config))?;

  Ok(SchemaDefinition {
    query: query_type_name.clone(),
    mutation: config.graphql.schema.mutation.clone(),
    directives: vec![to_directive(config.server.to_directive("server".to_string()))?],
  })
}
fn to_definitions<'a>(
  config: &Config,
  output_types: HashSet<&'a String>,
  input_types: HashSet<&'a String>,
) -> Valid<Vec<Definition>> {
  let mut types: Vec<Definition> = config.graphql.types.iter().validate_all(|(name, type_)| {
    let dbl_usage = input_types.contains(name) && output_types.contains(name);
    if let Some(variants) = &type_.variants {
      if !variants.is_empty() {
        to_enum_type_definition(name, type_, config, variants.clone()).trace(name)
      } else {
        Valid::fail("No variants found for enum".to_string())
      }
    } else if type_.scalar {
      to_scalar_type_definition(name).trace(name)
    } else if dbl_usage {
      Valid::fail("type is used in input and output".to_string()).trace(name)
    } else {
      let definition = to_object_type_definition(name, type_, config).trace(name)?;
      match definition.clone() {
        Definition::ObjectTypeDefinition(object_type_definition) => {
          if config.input_types().contains(name) {
            to_input_object_type_definition(object_type_definition).trace(name)
          } else if type_.interface {
            to_interface_type_definition(object_type_definition).trace(name)
          } else {
            Valid::Ok(definition)
          }
        }
        _ => Valid::Ok(definition),
      }
    }
  })?;

  let unions = config
    .graphql
    .unions
    .iter()
    .map(to_union_type_definition)
    .map(Definition::UnionTypeDefinition);

  types.extend(unions);
  Ok(types)
}
fn to_scalar_type_definition(name: &str) -> Valid<Definition> {
  Valid::Ok(Definition::ScalarTypeDefinition(ScalarTypeDefinition {
    name: name.to_string(),
    directive: Vec::new(),
    description: None,
  }))
}
fn to_union_type_definition((name, u): (&String, &config::Union)) -> UnionTypeDefinition {
  UnionTypeDefinition {
    name: name.to_owned(),
    description: u.doc.clone(),
    directives: Vec::new(),
    types: u.types.clone(),
  }
}
fn to_enum_type_definition(
  name: &str,
  type_: &config::Type,
  _config: &Config,
  variants: Vec<String>,
) -> Valid<Definition> {
  let enum_type_definition = Definition::EnumTypeDefinition(EnumTypeDefinition {
    name: name.to_string(),
    directives: Vec::new(),
    description: type_.doc.clone(),
    enum_values: variants
      .iter()
      .map(|variant| EnumValueDefinition { description: None, name: variant.clone(), directives: Vec::new() })
      .collect(),
  });
  Valid::Ok(enum_type_definition)
}
fn to_object_type_definition(name: &str, type_of: &config::Type, config: &Config) -> Valid<Definition> {
  to_fields(type_of, config).map(|fields| {
    Definition::ObjectTypeDefinition(ObjectTypeDefinition {
      name: name.to_string(),
      description: type_of.doc.clone(),
      fields,
      implements: type_of.implements.clone(),
    })
  })
}
fn to_input_object_type_definition(definition: ObjectTypeDefinition) -> Valid<Definition> {
  Valid::Ok(Definition::InputObjectTypeDefinition(InputObjectTypeDefinition {
    name: definition.name,
    fields: definition
      .fields
      .iter()
      .map(|field| InputFieldDefinition {
        name: field.name.clone(),
        description: field.description.clone(),
        default_value: None,
        of_type: field.of_type.clone(),
      })
      .collect(),
    description: definition.description,
  }))
}
fn to_interface_type_definition(definition: ObjectTypeDefinition) -> Valid<Definition> {
  Valid::Ok(Definition::InterfaceTypeDefinition(InterfaceTypeDefinition {
    name: definition.name,
    fields: definition.fields,
    description: definition.description,
  }))
}
fn to_fields(type_of: &config::Type, config: &Config) -> Valid<Vec<blueprint::FieldDefinition>> {
  let fields: Vec<Option<blueprint::FieldDefinition>> = type_of.fields.iter().validate_all(|(name, field)| {
    validate_field_type_exist(config, field)
      .validate_or(to_field(type_of, config, name, field))
      .trace(name)
  })?;

  Ok(fields.into_iter().flatten().collect())
}

fn to_field(
  type_of: &config::Type,
  config: &Config,
  name: &str,
  field: &Field,
) -> Valid<Option<blueprint::FieldDefinition>> {
  let field_type = &field.type_of;
  let args = to_args(field)?;

  let field_definition = FieldDefinition {
    name: name.to_owned(),
    description: field.doc.clone(),
    args,
    of_type: to_type(field_type, field.list, field.required, field.list_type_required),
    directives: Vec::new(),
    resolver: None,
  };

  let field_definition = update_http(field, field_definition, config).trace("@http")?;
  let field_definition = update_unsafe(field.clone(), field_definition);
  let field_definition = update_inline_field(type_of, field, field_definition, config).trace("@inline")?;
  let maybe_field_definition = update_modify(field, field_definition, type_of, config).trace("@modify")?;
  Ok(maybe_field_definition)
}

fn to_type(name: &str, list: bool, non_null: bool, list_type_required: bool) -> Type {
  if list {
    Type::ListType {
      of_type: Box::new(Type::NamedType { name: name.to_string(), non_null: list_type_required }),
      non_null,
    }
  } else {
    Type::NamedType { name: name.to_string(), non_null }
  }
}

fn validate_query(config: &Config) -> Valid<()> {
  let query_type_name = config
    .graphql
    .schema
    .query
    .as_ref()
    .validate_some("Query root is missing".to_owned())?;

  let Some(query) = config.find_type(query_type_name) else {
    return Valid::fail("Query type is not defined".to_owned()).trace(query_type_name);
  };

  query
    .fields
    .iter()
    .validate_all(validate_field_has_resolver)
    .trace(query_type_name)?;

  Ok(())
}

fn validate_mutation(config: &Config) -> Valid<()> {
  let mutation_type_name = config.graphql.schema.mutation.as_ref();

  if let Some(mutation_type_name) = mutation_type_name {
    let Some(mutation) = config.find_type(mutation_type_name) else {
      return Valid::fail("Mutation type is not defined".to_owned()).trace(mutation_type_name);
    };

    mutation
      .fields
      .iter()
      .validate_all(validate_field_has_resolver)
      .trace(mutation_type_name)?;
  }

  Ok(())
}

fn validate_field_has_resolver((name, field): (&String, &Field)) -> Valid<()> {
  if field.has_resolver() {
    Ok(())
  } else {
    Valid::fail("No resolver has been found in the schema".to_owned()).trace(name)
  }
}

fn validate_field_type_exist(config: &Config, field: &Field) -> Valid<()> {
  let field_type = &field.type_of;

  if !is_scalar(field_type) && !config.contains(field_type) {
    Valid::fail(format!("Undeclared type '{field_type}' was found"))
  } else {
    Valid::Ok(())
  }
}

fn update_unsafe(field: config::Field, mut b_field: FieldDefinition) -> FieldDefinition {
  if let Some(op) = field.unsafe_operation {
    b_field = b_field.resolver_or_default(Lambda::context().to_unsafe_js(op.script.clone()), |r| {
      r.to_unsafe_js(op.script.clone())
    });
  }
  b_field
}

fn update_http(field: &config::Field, b_field: FieldDefinition, config: &Config) -> Valid<FieldDefinition> {
  let mut b_field = b_field;
  match field.http.as_ref() {
    Some(http) => match http
      .base_url
      .as_ref()
      .map_or_else(|| config.server.base_url.as_ref(), Some)
    {
      Some(base_url) => {
        let mut base_url = base_url.clone();
        if base_url.ends_with('/') {
          base_url.pop();
        }
        base_url.push_str(http.path.clone().as_str());
        let query = http.query.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        let output_schema = to_json_schema_for_field(field, config);
        let input_schema = to_json_schema_for_args(&field.args, config);
        let mut header_map = HeaderMap::new();
        for (k, v) in http.headers.iter() {
          header_map.insert(
            HeaderName::from_bytes(k.as_bytes()).map_err(|e| ValidationError::new(e.to_string()))?,
            HeaderValue::from_str(v.as_str()).map_err(|e| ValidationError::new(e.to_string()))?,
          );
        }
        let req_template = RequestTemplate::try_from(
          Endpoint::new(base_url.to_string())
            .method(http.method.clone())
            .query(query)
            .output(output_schema)
            .input(input_schema)
            .body(http.body.clone())
            .headers(header_map),
        )
        .map_err(|e| ValidationError::new(e.to_string()))?;

        b_field.resolver = Some(Lambda::from_request_template(req_template).expression);

        Valid::Ok(b_field)
      }
      None => Valid::fail("No base URL defined".to_string()),
    },
    None => Valid::Ok(b_field),
  }
}
fn update_modify(
  field: &config::Field,
  mut b_field: FieldDefinition,
  type_: &config::Type,
  config: &Config,
) -> Valid<Option<FieldDefinition>> {
  match field.modify.as_ref() {
    Some(modify) => {
      if modify.omit {
        Ok(None)
      } else if let Some(new_name) = &modify.name {
        for name in type_.implements.iter() {
          let interface = config.find_type(name);
          if let Some(interface) = interface {
            if interface.fields.iter().any(|(name, _)| name == new_name) {
              return Valid::fail("Field is already implemented from interface".to_string());
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
fn needs_resolving(field: &config::Field) -> bool {
  field.unsafe_operation.is_some() || field.http.is_some()
}
fn is_scalar(type_name: &str) -> bool {
  ["String", "Int", "Float", "Boolean", "ID", "JSON"].contains(&type_name)
}
// Helper function to recursively process the path and return the corresponding type
fn process_path(
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
    if needs_resolving(next_field) {
      return Valid::<Type>::validate_or(
        Valid::fail(format!(
          "Inline can't be done because of {} resolver at [{}.{}]",
          next_field.http.as_ref().map(|_| "http").unwrap_or_else(|| "unsafe"),
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

// Main function to update an inline field
fn update_inline_field(
  type_info: &config::Type,
  field: &config::Field,
  base_field: FieldDefinition,
  config: &Config,
) -> Valid<FieldDefinition> {
  let inlined_path = field.inline.as_ref().map(|x| x.path.clone()).unwrap_or_default();
  let handle_invalid_path = |_field_name: &str, _inlined_path: &[String]| -> Valid<Type> {
    Valid::fail("Inline can't be done because provided path doesn't exist".to_string())
  };
  let has_index = inlined_path.iter().any(|s| {
    let re = Regex::new(r"^\d+$").unwrap();
    re.is_match(s)
  });
  if let Some(InlineType { path }) = field.clone().inline {
    return match process_path(&inlined_path, field, type_info, false, config, &handle_invalid_path) {
      Valid::Ok(of_type) => {
        let mut updated_base_field = base_field;
        let resolver = Lambda::context_path(path.clone());
        if has_index {
          updated_base_field.of_type = Type::NamedType { name: of_type.name().to_string(), non_null: false }
        } else {
          updated_base_field.of_type = of_type;
        }

        updated_base_field = updated_base_field.resolver_or_default(resolver, |r| r.to_input_path(path.clone()));
        Valid::Ok(updated_base_field)
      }
      Valid::Err(err) => Valid::Err(err),
    };
  }
  Valid::Ok(base_field)
}
fn to_args(field: &config::Field) -> Valid<Vec<InputFieldDefinition>> {
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

impl TryFrom<&Config> for Blueprint {
  type Error = ValidationError<String>;

  fn try_from(config: &Config) -> Result<Self, Self::Error> {
    config_blueprint(config)
  }
}
