use std::collections::BTreeMap;

use async_graphql::parser::types::{
  BaseType, ConstDirective, EnumType, FieldDefinition, InputObjectType, InputValueDefinition, InterfaceType,
  ObjectType, SchemaDefinition, ServiceDocument, Type, TypeDefinition, TypeKind, TypeSystemDefinition, UnionType,
};
use async_graphql::parser::Positioned;
use async_graphql::Name;

use super::Cache;
use crate::config::{self, Config, GraphQL, RootSchema, Server, Union, Upstream};
use crate::directive::DirectiveCodec;
use crate::valid::Valid;

pub fn from_document(doc: ServiceDocument) -> Valid<Config, String> {
  let type_definitions: Vec<_> = doc
    .definitions
    .iter()
    .filter_map(|def| match def {
      TypeSystemDefinition::Type(td) => Some(td),
      _ => None,
    })
    .collect();

  let types = to_types(&type_definitions);
  let unions = to_union_types(&type_definitions);
  let schema = schema_definition(&doc).map(to_root_schema);

  schema_definition(&doc)
    .and_then(|sd| server(sd).zip(upstream(sd)).zip(types).zip(unions).zip(schema))
    .map(|((((server, upstream), types), unions), schema)| Config { server, upstream, types, unions, schema })
}

fn schema_definition(doc: &ServiceDocument) -> Valid<&SchemaDefinition, String> {
  let p = doc.definitions.iter().find_map(|def| match def {
    TypeSystemDefinition::Schema(schema_definition) => Some(&schema_definition.node),
    _ => None,
  });
  p.map_or_else(
    || Valid::fail("schema not found".to_string()).trace("schema"),
    Valid::succeed,
  )
}

fn process_schema_directives<T: DirectiveCodec<T> + Default>(
  schema_definition: &SchemaDefinition,
  directive_name: &str,
) -> Valid<T, String> {
  let mut res = Valid::succeed(T::default());
  for directive in schema_definition.directives.iter() {
    if directive.node.name.node.as_ref() == directive_name {
      res = T::from_directive(&directive.node);
    }
  }
  res
}

fn server(schema_definition: &SchemaDefinition) -> Valid<Server, String> {
  process_schema_directives(schema_definition, config::Server::directive_name().as_str())
}
fn upstream(schema_definition: &SchemaDefinition) -> Valid<Upstream, String> {
  process_schema_directives(schema_definition, config::Upstream::directive_name().as_str())
}
fn to_root_schema(schema_definition: &SchemaDefinition) -> RootSchema {
  let query = schema_definition.query.as_ref().map(pos_name_to_string);
  let mutation = schema_definition.mutation.as_ref().map(pos_name_to_string);
  let subscription = schema_definition.subscription.as_ref().map(pos_name_to_string);

  RootSchema { query, mutation, subscription }
}
fn pos_name_to_string(pos: &Positioned<Name>) -> String {
  pos.node.to_string()
}
fn to_types(type_definitions: &Vec<&Positioned<TypeDefinition>>) -> Valid<BTreeMap<String, config::Type>, String> {
  Valid::from_iter(type_definitions, |type_definition| {
    let type_name = pos_name_to_string(&type_definition.node.name);
    let directives = &type_definition.node.directives;
    let cache: Option<Cache> = Cache::from_directives_slice(directives);

    match type_definition.node.kind.clone() {
      TypeKind::Object(object_type) => to_object_type(
        &object_type,
        &type_definition.node.description,
        &type_definition.node.directives,
        cache,
      )
      .some(),
      TypeKind::Interface(interface_type) => to_object_type(
        &interface_type,
        &type_definition.node.description,
        &type_definition.node.directives,
        cache,
      )
      .some(),
      TypeKind::Enum(enum_type) => Valid::succeed(Some(to_enum(enum_type))),
      TypeKind::InputObject(input_object_type) => to_input_object(input_object_type, cache).some(),
      TypeKind::Union(_) => Valid::none(),
      TypeKind::Scalar => Valid::succeed(Some(to_scalar_type())),
    }
    .map(|option| (type_name, option))
  })
  .map(|vec| {
    BTreeMap::from_iter(
      vec
        .into_iter()
        .filter_map(|(name, option)| option.map(|tpe| (name, tpe))),
    )
  })
}
fn to_scalar_type() -> config::Type {
  config::Type { scalar: true, ..Default::default() }
}
fn to_union_types(type_definitions: &Vec<&Positioned<TypeDefinition>>) -> Valid<BTreeMap<String, Union>, String> {
  let mut unions = BTreeMap::new();
  for type_definition in type_definitions {
    let type_name = pos_name_to_string(&type_definition.node.name);
    let type_opt = match type_definition.node.kind.clone() {
      TypeKind::Union(union_type) => to_union(
        union_type,
        &type_definition.node.description.as_ref().map(|pos| pos.node.clone()),
      ),
      _ => continue,
    };
    unions.insert(type_name, type_opt);
  }

  Valid::succeed(unions)
}

#[allow(clippy::too_many_arguments)]
fn to_object_type<T>(
  object: &T,
  description: &Option<Positioned<String>>,
  directives: &[Positioned<ConstDirective>],
  cache: Option<Cache>,
) -> Valid<config::Type, String>
where
  T: ObjectLike,
{
  let fields = object.fields();
  let implements = object.implements();
  let interface = object.is_interface();

  to_fields(fields, cache).map(|fields| {
    let doc = description.as_ref().map(|pos| pos.node.clone());
    let implements = implements.iter().map(|pos| pos.node.to_string()).collect();
    let added_fields = to_add_fields_from_directives(directives);
    config::Type { fields, added_fields, doc, interface, implements, ..Default::default() }
  })
}
fn to_enum(enum_type: EnumType) -> config::Type {
  let variants = enum_type
    .values
    .iter()
    .map(|value| value.node.value.to_string())
    .collect();
  config::Type { variants: Some(variants), ..Default::default() }
}
fn to_input_object(input_object_type: InputObjectType, cache: Option<Cache>) -> Valid<config::Type, String> {
  to_input_object_fields(&input_object_type.fields, cache).map(|fields| config::Type { fields, ..Default::default() })
}

fn to_fields_inner<T, F>(
  fields: &Vec<Positioned<T>>,
  cache: Option<Cache>,
  transform: F,
) -> Valid<BTreeMap<String, config::Field>, String>
where
  F: Fn(&T, Option<Cache>) -> Valid<config::Field, String>,
  T: HasName,
{
  Valid::from_iter(fields, |field| {
    let field_name = pos_name_to_string(field.node.name());
    transform(&field.node, cache.clone()).map(|field| (field_name, field))
  })
  .map(BTreeMap::from_iter)
}
fn to_fields(
  fields: &Vec<Positioned<FieldDefinition>>,
  cache: Option<Cache>,
) -> Valid<BTreeMap<String, config::Field>, String> {
  to_fields_inner(fields, cache, to_field)
}
fn to_input_object_fields(
  input_object_fields: &Vec<Positioned<InputValueDefinition>>,
  cache: Option<Cache>,
) -> Valid<BTreeMap<String, config::Field>, String> {
  to_fields_inner(input_object_fields, cache, to_input_object_field)
}
fn to_field(field_definition: &FieldDefinition, cache: Option<Cache>) -> Valid<config::Field, String> {
  to_common_field(field_definition, to_args(field_definition), cache)
}
fn to_input_object_field(
  field_definition: &InputValueDefinition,
  cache: Option<Cache>,
) -> Valid<config::Field, String> {
  to_common_field(field_definition, BTreeMap::new(), cache)
}
fn to_common_field<F>(
  field: &F,
  args: BTreeMap<String, config::Arg>,
  parent_cache: Option<Cache>,
) -> Valid<config::Field, String>
where
  F: Fieldlike,
{
  let type_ = field.ty();
  let base = &type_.base;
  let nullable = &type_.nullable;
  let description = field.description();
  let directives = field.directives();

  let type_of = to_type_of(type_);
  let list = matches!(&base, BaseType::List(_));
  let list_type_required = matches!(&base, BaseType::List(ty) if !ty.nullable);
  let doc = description.as_ref().map(|pos| pos.node.clone());
  let modify = to_modify(directives);

  config::Http::from_directives(directives.iter())
    .zip(GraphQL::from_directives(directives.iter()))
    .map(|(http, graphql)| {
      let unsafe_operation = to_unsafe_operation(directives);
      let const_field = to_const_field(directives);
      let cache: Option<Cache> = Cache::from_directives_slice(directives);
      config::Field {
        type_of,
        list,
        required: !nullable,
        list_type_required,
        args,
        doc,
        modify,
        http,
        unsafe_operation,
        const_field,
        graphql,
        cache: cache.or(parent_cache),
      }
    })
}
fn to_unsafe_operation(directives: &[Positioned<ConstDirective>]) -> Option<config::Unsafe> {
  directives.iter().find_map(|directive| {
    if directive.node.name.node == config::Unsafe::directive_name() {
      config::Unsafe::from_directive(&directive.node).to_result().ok()
    } else {
      None
    }
  })
}
fn to_type_of(type_: &Type) -> String {
  match &type_.base {
    BaseType::Named(name) => name.to_string(),
    BaseType::List(ty) => to_type_of(ty),
  }
}
fn to_args(field_definition: &FieldDefinition) -> BTreeMap<String, config::Arg> {
  let mut args: BTreeMap<String, config::Arg> = BTreeMap::new();

  for arg in field_definition.arguments.iter() {
    let arg_name = pos_name_to_string(&arg.node.name);
    let arg_val = to_arg(&arg.node);
    args.insert(arg_name, arg_val);
  }

  args
}
fn to_arg(input_value_definition: &InputValueDefinition) -> config::Arg {
  let type_of = to_type_of(&input_value_definition.ty.node);
  let list = matches!(&input_value_definition.ty.node.base, BaseType::List(_));
  let required = !input_value_definition.ty.node.nullable;
  let doc = input_value_definition.description.as_ref().map(|pos| pos.node.clone());
  let modify = to_modify(&input_value_definition.directives);
  let default_value = if let Some(pos) = input_value_definition.default_value.as_ref() {
    let value = &pos.node;
    serde_json::to_value(value).ok()
  } else {
    None
  };
  config::Arg { type_of, list, required, doc, modify, default_value }
}
fn to_modify(directives: &[Positioned<ConstDirective>]) -> Option<config::Modify> {
  directives.iter().find_map(|directive| {
    if directive.node.name.node == config::Modify::directive_name() {
      config::Modify::from_directive(&directive.node).to_result().ok()
    } else {
      None
    }
  })
}

fn to_union(union_type: UnionType, doc: &Option<String>) -> Union {
  let types = union_type
    .members
    .iter()
    .map(|member| member.node.to_string())
    .collect();
  Union { types, doc: doc.clone() }
}
fn to_const_field(directives: &[Positioned<ConstDirective>]) -> Option<config::Const> {
  directives.iter().find_map(|directive| {
    if directive.node.name.node == config::Const::directive_name() {
      config::Const::from_directive(&directive.node).to_result().ok()
    } else {
      None
    }
  })
}

fn to_add_fields_from_directives(directives: &[Positioned<ConstDirective>]) -> Vec<config::AddField> {
  directives
    .iter()
    .filter_map(|directive| {
      if directive.node.name.node == config::AddField::directive_name() {
        config::AddField::from_directive(&directive.node).to_result().ok()
      } else {
        None
      }
    })
    .collect::<Vec<_>>()
}

trait HasName {
  fn name(&self) -> &Positioned<Name>;
}
impl HasName for FieldDefinition {
  fn name(&self) -> &Positioned<Name> {
    &self.name
  }
}
impl HasName for InputValueDefinition {
  fn name(&self) -> &Positioned<Name> {
    &self.name
  }
}

trait Fieldlike {
  fn ty(&self) -> &Type;
  fn description(&self) -> &Option<Positioned<String>>;
  fn directives(&self) -> &[Positioned<ConstDirective>];
}
impl Fieldlike for FieldDefinition {
  fn ty(&self) -> &Type {
    &self.ty.node
  }
  fn description(&self) -> &Option<Positioned<String>> {
    &self.description
  }
  fn directives(&self) -> &[Positioned<ConstDirective>] {
    &self.directives
  }
}
impl Fieldlike for InputValueDefinition {
  fn ty(&self) -> &Type {
    &self.ty.node
  }
  fn description(&self) -> &Option<Positioned<String>> {
    &self.description
  }
  fn directives(&self) -> &[Positioned<ConstDirective>] {
    &self.directives
  }
}

trait ObjectLike {
  fn fields(&self) -> &Vec<Positioned<FieldDefinition>>;
  fn implements(&self) -> &Vec<Positioned<Name>>;
  fn is_interface(&self) -> bool;
}
impl ObjectLike for ObjectType {
  fn fields(&self) -> &Vec<Positioned<FieldDefinition>> {
    &self.fields
  }
  fn implements(&self) -> &Vec<Positioned<Name>> {
    &self.implements
  }
  fn is_interface(&self) -> bool {
    false
  }
}
impl ObjectLike for InterfaceType {
  fn fields(&self) -> &Vec<Positioned<FieldDefinition>> {
    &self.fields
  }
  fn implements(&self) -> &Vec<Positioned<Name>> {
    &self.implements
  }
  fn is_interface(&self) -> bool {
    true
  }
}
