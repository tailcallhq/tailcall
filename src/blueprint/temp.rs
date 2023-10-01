// #![allow(clippy::too_many_arguments)]

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
fn to_definitions<'a>(
  config: &Config,
  output_types: HashSet<&'a String>,
  input_types: HashSet<&'a String>,
) -> Valid<Vec<Definition>> {
  let mut types: Vec<Definition> = config.graphql.types.iter().validate_all(|(name, type_)| {
    let dbl_usage = input_types.contains(name) && output_types.contains(name);
    if let Some(variants) = &type_.variants {
      if !variants.is_empty() {
        let config = EnumTypeConfig {
            etc_name: name,
            etc_type_: type_,
            etc_variants: variants.clone(),
        };
        to_enum_type_definition(config).trace(name)
    }
     else {
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
struct EnumTypeConfig<'a> {
  etc_name: &'a str,
  etc_type_: &'a config::Type,
  etc_variants: Vec<String>,
}

fn to_enum_type_definition(config: EnumTypeConfig) -> Valid<Definition> {
  let EnumTypeConfig { etc_name, etc_type_, etc_variants } = config;

  let enum_type_definition = Definition::EnumTypeDefinition(EnumTypeDefinition {
      name: etc_name.to_string(),
      directives: Vec::new(),
      description: etc_type_.doc.clone(),
      enum_values: etc_variants
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
          .validate_or({
              let field_config = FieldConfig {
                  fc_type_of: type_of,
                  fc_config: config,
                  fc_name: name,
                  fc_field: field,
              };
              to_field(field_config)
          })
          .trace(name)
  })?;

  Ok(fields.into_iter().flatten().collect())
}


struct FieldConfig<'a> {
  fc_type_of: &'a config::Type,
  fc_config: &'a Config,
  fc_name: &'a str,
  fc_field: &'a Field,
}

fn to_field(config: FieldConfig) -> Valid<Option<blueprint::FieldDefinition>> {
  let FieldConfig { fc_type_of, fc_config, fc_name, fc_field } = config;

  let field_type = &fc_field.type_of;
  let args = to_args(fc_field)?;

  let field_definition = FieldDefinition {
    name: fc_name.to_owned(),
    description: fc_field.doc.clone(),
    args,
    of_type: to_type(TypeDetails {
        td_name: field_type.to_string(),
        td_list: fc_field.list,
        td_non_null: fc_field.required,
        td_list_type_required: fc_field.list_type_required,
    }),
    directives: Vec::new(),
    resolver: None,
};


  let field_definition = update_http(fc_field, field_definition, fc_config).trace("@http")?;
  let field_definition = update_unsafe(fc_field.clone(), field_definition);
  let args = UpdateInlineFieldArgs {
    uifa_type_info: fc_type_of,
    uifa_field: fc_field,
    uifa_base_field: field_definition,
    uifa_config: fc_config,
};

let field_definition = update_inline_field(args).trace("@inline")?;
let field_modifier = FieldModifier::new(fc_field, fc_type_of, fc_config);
let maybe_field_definition = field_modifier.update_modify(field_definition).trace("@modify")?;
  Ok(maybe_field_definition)
}

struct TypeDetails {
  td_name: String,
  td_list: bool,
  td_non_null: bool,
  td_list_type_required: bool,
}

fn to_type(details: TypeDetails) -> Type {
  if details.td_list {
      Type::ListType {
          of_type: Box::new(Type::NamedType { 
              name: details.td_name, 
              non_null: details.td_list_type_required 
          }),
          non_null: details.td_non_null,
      }
  } else {
      Type::NamedType { 
          name: details.td_name, 
          non_null: details.td_non_null 
      }
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
struct FieldModifier<'a> {
  fm_field: &'a config::Field,
  fm_type_: &'a config::Type,
  fm_config: &'a Config,
}

impl<'a> FieldModifier<'a> {
  fn new(fm_field: &'a config::Field, fm_type_: &'a config::Type, fm_config: &'a Config) -> Self {
      Self { fm_field, fm_type_, fm_config }
  }

  fn update_modify(&self, mut b_field: FieldDefinition) -> Valid<Option<FieldDefinition>> {
      match self.fm_field.modify.as_ref() {
          Some(modify) => {
              if modify.omit {
                  Ok(None)
              } else if let Some(new_name) = &modify.name {
                  for name in self.fm_type_.implements.iter() {
                      let interface = self.fm_config.find_type(name);
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
}

fn needs_resolving(field: &config::Field) -> bool {
  field.unsafe_operation.is_some() || field.http.is_some()
}
fn is_scalar(type_name: &str) -> bool {
  ["String", "Int", "Float", "Boolean", "ID", "JSON"].contains(&type_name)
}
// Helper function to recursively process the path and return the corresponding type
struct ProcessPathArgs<'a> {
  ppa_path: &'a [String],
  ppa_field: &'a config::Field,
  ppa_type_info: &'a config::Type,
  ppa_is_required: bool,
  ppa_config: &'a Config,
  ppa_invalid_path_handler: &'a dyn Fn(&str, &[String]) -> Valid<Type>,
}

fn process_path(args: ProcessPathArgs) -> Valid<Type> {
  if let Some((field_name, remaining_path)) = args.ppa_path.split_first() {
      if field_name.parse::<usize>().is_ok() {
          let mut modified_field = args.ppa_field.clone();
          modified_field.list = false;
          return process_path(ProcessPathArgs {
            ppa_path: remaining_path,
            ppa_field: &modified_field,
              ..args
          });
      }
      let target_type_info = args.ppa_type_info
          .fields
          .get(field_name)
          .map(|_| args.ppa_type_info)
          .or_else(|| args.ppa_config.find_type(&args.ppa_field.type_of));

      if let Some(type_info) = target_type_info {
        return process_field_within_type(ProcessArgs {
          pa_field: args.ppa_field,
          pa_field_name: field_name,
          pa_remaining_path: remaining_path,
          pa_type_info: type_info,
          pa_is_required: args.ppa_is_required,
          pa_config: args.ppa_config,
          pa_invalid_path_handler: args.ppa_invalid_path_handler,
      });
      
      }
      return (args.ppa_invalid_path_handler)(field_name, args.ppa_path);
  }

  Valid::Ok(to_type(TypeDetails {
      td_name: args.ppa_field.type_of.to_string(),
      td_list: args.ppa_field.list,
      td_non_null: args.ppa_is_required,
      td_list_type_required: args.ppa_field.list_type_required,
  }))
}

struct ProcessArgs<'a> {
  pa_field: &'a config::Field,
  pa_field_name: &'a str,
  pa_remaining_path: &'a [String],
  pa_type_info: &'a config::Type,
  pa_is_required: bool,
  pa_config: &'a Config,
  pa_invalid_path_handler: &'a dyn Fn(&str, &[String]) -> Valid<Type>,
}

fn process_field_within_type(args: ProcessArgs) -> Valid<Type> {
  if let Some(next_field) = args.pa_type_info.fields.get(args.pa_field_name) {
      if needs_resolving(next_field) {
          return Valid::<Type>::validate_or(
              Valid::fail(format!(
                  "Inline can't be done because of {} resolver at [{}.{}]",
                  next_field.http.as_ref().map(|_| "http").unwrap_or_else(|| "unsafe"),
                  args.pa_field.type_of,
                  args.pa_field_name
              )),
              process_path(ProcessPathArgs {
                  ppa_path: args.pa_remaining_path,
                  ppa_field: next_field,
                  ppa_type_info: args.pa_type_info,
                  ppa_is_required: args.pa_is_required,
                  ppa_config: args.pa_config,
                  ppa_invalid_path_handler: args.pa_invalid_path_handler,
              }),
          );
      }

      let next_is_required = args.pa_is_required && next_field.required;
      if is_scalar(&next_field.type_of) {
          return process_path(ProcessPathArgs {
              ppa_path: args.pa_remaining_path,
              ppa_field: next_field,
              ppa_type_info: args.pa_type_info,
              ppa_is_required: args.pa_is_required,
              ppa_config: args.pa_config,
              ppa_invalid_path_handler: args.pa_invalid_path_handler,
          });
      }

      if let Some(next_type_info) = args.pa_config.find_type(&next_field.type_of) {

          let of_type = process_path(ProcessPathArgs {
              ppa_path: args.pa_remaining_path,
              ppa_field: next_field,
              ppa_type_info: args.pa_type_info,
              ppa_is_required: args.pa_is_required,
              ppa_config: args.pa_config,
              ppa_invalid_path_handler: args.pa_invalid_path_handler,
          })?;

          return if next_field.list {
              Valid::Ok(ListType { of_type: Box::new(of_type), non_null: args.pa_is_required })
          } else {
              Ok(of_type)
          };
      }
  } else if let Some((head, tail)) = args.pa_remaining_path.split_first() {
      if let Some(field) = args.pa_type_info.fields.get(head) {
          return process_path(ProcessPathArgs {
              ppa_path: tail,
              ppa_field: field,
              ppa_type_info: args.pa_type_info,
              ppa_is_required: args.pa_is_required,
              ppa_config: args.pa_config,
              ppa_invalid_path_handler: args.pa_invalid_path_handler,
          });
      }
  }

  (args.pa_invalid_path_handler)(args.pa_field_name, args.pa_remaining_path)
}


// Main function to update an inline field
struct UpdateInlineFieldArgs<'a> {
  uifa_type_info: &'a config::Type,
  uifa_field: &'a config::Field,
  uifa_base_field: FieldDefinition,
  uifa_config: &'a Config,
}

fn update_inline_field(args: UpdateInlineFieldArgs) -> Valid<FieldDefinition> {
  let inlined_path = args.uifa_field.inline.as_ref().map(|x| x.path.clone()).unwrap_or_default();
  let handle_invalid_path = |_field_name: &str, _inlined_path: &[String]| -> Valid<Type> {
      Valid::fail("Inline can't be done because provided path doesn't exist".to_string())
  };
  let has_index = inlined_path.iter().any(|s| {
      let re = Regex::new(r"^\d+$").unwrap();
      re.is_match(s)
  });
  if let Some(InlineType { path }) = args.uifa_field.clone().inline {
      return match process_path(ProcessPathArgs {
          ppa_path: &inlined_path,
          ppa_field: args.uifa_field,
          ppa_type_info: args.uifa_type_info,
          ppa_is_required: false,
          ppa_config: args.uifa_config,
          ppa_invalid_path_handler: &handle_invalid_path,
      }) {
          Valid::Ok(of_type) => {
              let mut updated_base_field = args.uifa_base_field;
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
  Valid::Ok(args.uifa_base_field)
}

fn to_args(field: &config::Field) -> Valid<Vec<InputFieldDefinition>> {
  // TODO! assert type name
  field.args.iter().validate_all(|(name, arg)| {
    Valid::Ok(InputFieldDefinition {
      name: name.clone(),
      description: arg.doc.clone(),
      of_type: to_type(TypeDetails {
        td_name: arg.type_of.to_string(),
        td_list: arg.list,
        td_non_null: arg.required,
        td_list_type_required: false,
    }),
    
      default_value: arg.default_value.clone(),
    })
  })
}
pub struct SchemaParams<'a> {
  type_of: &'a str,
  required: bool,
  list: bool,
  config: &'a Config,
}

pub fn to_json_schema_for_field(field: &Field, config: &Config) -> JsonSchema {
  let params = SchemaParams {
      type_of: &field.type_of,
      required: field.required,
      list: field.list,
      config,
  };
  to_json_schema(params)
}

pub fn to_json_schema_for_args(args: &BTreeMap<String, Arg>, config: &Config) -> JsonSchema {
  let mut schema_fields = HashMap::new();
  for (name, arg) in args.iter() {
      let params = SchemaParams {
          type_of: &arg.type_of,
          required: arg.required,
          list: arg.list,
          config,
      };
      schema_fields.insert(name.clone(), to_json_schema(params));
  }
  JsonSchema::Obj(schema_fields)
}

pub fn to_json_schema(params: SchemaParams) -> JsonSchema {
  let type_ = params.config.find_type(params.type_of);
  let schema = match type_ {
      Some(type_) => {
          let mut schema_fields = HashMap::new();
          for (name, field) in type_.fields.iter() {
              if field.unsafe_operation.is_none() && field.http.is_none() {
                  schema_fields.insert(name.clone(), to_json_schema_for_field(field, params.config));
              }
          }
          JsonSchema::Obj(schema_fields)
      }
      None => match params.type_of {
          "String" => JsonSchema::Str {},
          "Int" => JsonSchema::Num {},
          "Boolean" => JsonSchema::Bool {},
          "JSON" => JsonSchema::Obj(HashMap::new()),
          _ => JsonSchema::Str {},
      },
  };

  if !params.required {
      if params.list {
          JsonSchema::Opt(Box::new(JsonSchema::Arr(Box::new(schema))))
      } else {
          JsonSchema::Opt(Box::new(schema))
      }
  } else if params.list {
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
