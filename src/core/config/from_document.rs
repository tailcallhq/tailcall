use std::collections::BTreeMap;

use async_graphql::parser::types::{
    BaseType, ConstDirective, EnumType, FieldDefinition, InputObjectType, InputValueDefinition,
    InterfaceType, ObjectType, SchemaDefinition, ServiceDocument, Type, TypeDefinition, TypeKind,
    TypeSystemDefinition, UnionType,
};
use async_graphql::parser::Positioned;
use async_graphql::Name;

use super::telemetry::Telemetry;
use super::{Tag, JS};
use crate::core::config::{
    self, Cache, Call, Config, Enum, GraphQL, Grpc, Link, Modify, Omit, Protected, RootSchema,
    Server, Union, Upstream,
};
use crate::core::directive::DirectiveCodec;
use crate::core::valid::{Valid, Validator};

const DEFAULT_SCHEMA_DEFINITION: &SchemaDefinition = &SchemaDefinition {
    extend: false,
    directives: Vec::new(),
    query: None,
    mutation: None,
    subscription: None,
};

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
    let enums = to_enum_types(&type_definitions);
    let schema = schema_definition(&doc).map(to_root_schema);
    schema_definition(&doc).and_then(|sd| {
        server(sd)
            .fuse(upstream(sd))
            .fuse(types)
            .fuse(unions)
            .fuse(enums)
            .fuse(schema)
            .fuse(links(sd))
            .fuse(telemetry(sd))
            .map(
                |(server, upstream, types, unions, enums, schema, links, telemetry)| Config {
                    server,
                    upstream,
                    types,
                    unions,
                    enums,
                    schema,
                    links,
                    telemetry,
                },
            )
    })
}

fn schema_definition(doc: &ServiceDocument) -> Valid<&SchemaDefinition, String> {
    doc.definitions
        .iter()
        .find_map(|def| match def {
            TypeSystemDefinition::Schema(schema_definition) => Some(&schema_definition.node),
            _ => None,
        })
        .map_or_else(|| Valid::succeed(DEFAULT_SCHEMA_DEFINITION), Valid::succeed)
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

fn process_schema_multiple_directives<T: DirectiveCodec<T> + Default>(
    schema_definition: &SchemaDefinition,
    directive_name: &str,
) -> Valid<Vec<T>, String> {
    let directives: Vec<Valid<T, String>> = schema_definition
        .directives
        .iter()
        .filter_map(|directive| {
            if directive.node.name.node.as_ref() == directive_name {
                Some(T::from_directive(&directive.node))
            } else {
                None
            }
        })
        .collect();

    Valid::from_iter(directives, |item| item)
}

fn server(schema_definition: &SchemaDefinition) -> Valid<Server, String> {
    process_schema_directives(schema_definition, config::Server::directive_name().as_str())
}

fn upstream(schema_definition: &SchemaDefinition) -> Valid<Upstream, String> {
    process_schema_directives(
        schema_definition,
        config::Upstream::directive_name().as_str(),
    )
}

fn links(schema_definition: &SchemaDefinition) -> Valid<Vec<Link>, String> {
    process_schema_multiple_directives(schema_definition, config::Link::directive_name().as_str())
}

fn telemetry(schema_definition: &SchemaDefinition) -> Valid<Telemetry, String> {
    process_schema_directives(
        schema_definition,
        config::telemetry::Telemetry::directive_name().as_str(),
    )
}

fn to_root_schema(schema_definition: &SchemaDefinition) -> RootSchema {
    let query = schema_definition.query.as_ref().map(pos_name_to_string);
    let mutation = schema_definition.mutation.as_ref().map(pos_name_to_string);
    let subscription = schema_definition
        .subscription
        .as_ref()
        .map(pos_name_to_string);

    RootSchema { query, mutation, subscription }
}
fn pos_name_to_string(pos: &Positioned<Name>) -> String {
    pos.node.to_string()
}
fn to_types(
    type_definitions: &Vec<&Positioned<TypeDefinition>>,
) -> Valid<BTreeMap<String, config::Type>, String> {
    Valid::from_iter(type_definitions, |type_definition| {
        let type_name = pos_name_to_string(&type_definition.node.name);
        match type_definition.node.kind.clone() {
            TypeKind::Object(object_type) => to_object_type(
                &object_type,
                &type_definition.node.description,
                &type_definition.node.directives,
            )
            .some(),
            TypeKind::Interface(interface_type) => to_object_type(
                &interface_type,
                &type_definition.node.description,
                &type_definition.node.directives,
            )
            .some(),
            TypeKind::Enum(_) => Valid::none(),
            TypeKind::InputObject(input_object_type) => to_input_object(
                input_object_type,
                &type_definition.node.description,
                &type_definition.node.directives,
            )
            .some(),
            TypeKind::Union(_) => Valid::none(),
            TypeKind::Scalar => Valid::succeed(Some(to_scalar_type())),
        }
        .map(|option| (type_name, option))
    })
    .map(|vec| {
        BTreeMap::from_iter(
            vec.into_iter()
                .filter_map(|(name, option)| option.map(|tpe| (name, tpe))),
        )
    })
}
fn to_scalar_type() -> config::Type {
    config::Type { ..Default::default() }
}
fn to_union_types(
    type_definitions: &[&Positioned<TypeDefinition>],
) -> Valid<BTreeMap<String, Union>, String> {
    Valid::succeed(
        type_definitions
            .iter()
            .filter_map(|type_definition| {
                let type_name = pos_name_to_string(&type_definition.node.name);
                let type_opt = match type_definition.node.kind.clone() {
                    TypeKind::Union(union_type) => to_union(
                        union_type,
                        &type_definition
                            .node
                            .description
                            .to_owned()
                            .map(|pos| pos.node),
                    ),
                    _ => return None,
                };
                Some((type_name, type_opt))
            })
            .collect(),
    )
}

fn to_enum_types(
    type_definitions: &[&Positioned<TypeDefinition>],
) -> Valid<BTreeMap<String, Enum>, String> {
    Valid::succeed(
        type_definitions
            .iter()
            .filter_map(|type_definition| {
                let type_name = pos_name_to_string(&type_definition.node.name);
                let type_opt = match type_definition.node.kind.clone() {
                    TypeKind::Enum(enum_type) => to_enum(
                        enum_type,
                        type_definition
                            .node
                            .description
                            .to_owned()
                            .map(|pos| pos.node),
                    ),
                    _ => return None,
                };
                Some((type_name, type_opt))
            })
            .collect(),
    )
}

#[allow(clippy::too_many_arguments)]
fn to_object_type<T>(
    object: &T,
    description: &Option<Positioned<String>>,
    directives: &[Positioned<ConstDirective>],
) -> Valid<config::Type, String>
where
    T: ObjectLike,
{
    let fields = object.fields();
    let implements = object.implements();

    Cache::from_directives(directives.iter())
        .fuse(to_fields(fields))
        .fuse(Protected::from_directives(directives.iter()))
        .fuse(Tag::from_directives(directives.iter()))
        .map(|(cache, fields, protected, tag)| {
            let doc = description.to_owned().map(|pos| pos.node);
            let implements = implements.iter().map(|pos| pos.node.to_string()).collect();
            let added_fields = to_add_fields_from_directives(directives);
            config::Type { fields, added_fields, doc, implements, cache, protected, tag }
        })
}
fn to_input_object(
    input_object_type: InputObjectType,
    description: &Option<Positioned<String>>,
    directives: &[Positioned<ConstDirective>],
) -> Valid<config::Type, String> {
    to_input_object_fields(&input_object_type.fields)
        .fuse(Protected::from_directives(directives.iter()))
        .map(|(fields, protected)| {
            let doc = description.to_owned().map(|pos| pos.node);
            config::Type { fields, protected, doc, ..Default::default() }
        })
}

fn to_fields_inner<T, F>(
    fields: &Vec<Positioned<T>>,
    transform: F,
) -> Valid<BTreeMap<String, config::Field>, String>
where
    F: Fn(&T) -> Valid<config::Field, String>,
    T: HasName,
{
    Valid::from_iter(fields, |field| {
        let field_name = pos_name_to_string(field.node.name());
        transform(&field.node).map(|field| (field_name, field))
    })
    .map(BTreeMap::from_iter)
}
fn to_fields(
    fields: &Vec<Positioned<FieldDefinition>>,
) -> Valid<BTreeMap<String, config::Field>, String> {
    to_fields_inner(fields, to_field)
}
fn to_input_object_fields(
    input_object_fields: &Vec<Positioned<InputValueDefinition>>,
) -> Valid<BTreeMap<String, config::Field>, String> {
    to_fields_inner(input_object_fields, to_input_object_field)
}
fn to_field(field_definition: &FieldDefinition) -> Valid<config::Field, String> {
    to_common_field(field_definition, to_args(field_definition))
}
fn to_input_object_field(field_definition: &InputValueDefinition) -> Valid<config::Field, String> {
    to_common_field(field_definition, BTreeMap::new())
}
fn to_common_field<F>(
    field: &F,
    args: BTreeMap<String, config::Arg>,
) -> Valid<config::Field, String>
where
    F: Fieldlike,
{
    let type_of = field.type_of();
    let base = &type_of.base;
    let nullable = &type_of.nullable;
    let description = field.description();
    let directives = field.directives();

    let type_of = to_type_of(type_of);
    let list = matches!(&base, BaseType::List(_));
    let list_type_required = matches!(&base, BaseType::List(type_of) if !type_of.nullable);
    let doc = description.to_owned().map(|pos| pos.node);
    config::Http::from_directives(directives.iter())
        .fuse(GraphQL::from_directives(directives.iter()))
        .fuse(Cache::from_directives(directives.iter()))
        .fuse(Grpc::from_directives(directives.iter()))
        .fuse(Omit::from_directives(directives.iter()))
        .fuse(Modify::from_directives(directives.iter()))
        .fuse(JS::from_directives(directives.iter()))
        .fuse(Call::from_directives(directives.iter()))
        .fuse(Protected::from_directives(directives.iter()))
        .map(
            |(http, graphql, cache, grpc, omit, modify, script, call, protected)| {
                let const_field = to_const_field(directives);
                config::Field {
                    type_of,
                    list,
                    required: !nullable,
                    list_type_required,
                    args,
                    doc,
                    modify,
                    omit,
                    http,
                    grpc,
                    script,
                    const_field,
                    graphql,
                    cache,
                    call,
                    protected,
                }
            },
        )
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
    let doc = input_value_definition
        .description
        .to_owned()
        .map(|pos| pos.node);
    let modify = Modify::from_directives(input_value_definition.directives.iter())
        .to_result()
        .ok()
        .flatten();
    let default_value = if let Some(pos) = input_value_definition.default_value.as_ref() {
        let value = &pos.node;
        serde_json::to_value(value).ok()
    } else {
        None
    };
    config::Arg { type_of, list, required, doc, modify, default_value }
}

fn to_union(union_type: UnionType, doc: &Option<String>) -> Union {
    let types = union_type
        .members
        .iter()
        .map(|member| member.node.to_string())
        .collect();
    Union { types, doc: doc.clone() }
}

fn to_enum(enum_type: EnumType, doc: Option<String>) -> Enum {
    let variants = enum_type
        .values
        .iter()
        .map(|member| member.node.value.node.as_str().to_owned())
        .collect();
    Enum { variants, doc }
}
fn to_const_field(directives: &[Positioned<ConstDirective>]) -> Option<config::Expr> {
    directives.iter().find_map(|directive| {
        if directive.node.name.node == config::Expr::directive_name() {
            config::Expr::from_directive(&directive.node)
                .to_result()
                .ok()
        } else {
            None
        }
    })
}

fn to_add_fields_from_directives(
    directives: &[Positioned<ConstDirective>],
) -> Vec<config::AddField> {
    directives
        .iter()
        .filter_map(|directive| {
            if directive.node.name.node == config::AddField::directive_name() {
                config::AddField::from_directive(&directive.node)
                    .to_result()
                    .ok()
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
    fn type_of(&self) -> &Type;
    fn description(&self) -> &Option<Positioned<String>>;
    fn directives(&self) -> &[Positioned<ConstDirective>];
}
impl Fieldlike for FieldDefinition {
    fn type_of(&self) -> &Type {
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
    fn type_of(&self) -> &Type {
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
}
impl ObjectLike for ObjectType {
    fn fields(&self) -> &Vec<Positioned<FieldDefinition>> {
        &self.fields
    }
    fn implements(&self) -> &Vec<Positioned<Name>> {
        &self.implements
    }
}
impl ObjectLike for InterfaceType {
    fn fields(&self) -> &Vec<Positioned<FieldDefinition>> {
        &self.fields
    }
    fn implements(&self) -> &Vec<Positioned<Name>> {
        &self.implements
    }
}
