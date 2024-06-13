use std::collections::BTreeMap;
use std::slice::Iter;

use async_graphql::parser::types::{
    BaseType, ConstDirective, EnumType, FieldDefinition, InputObjectType, InputValueDefinition,
    InterfaceType, ObjectType, SchemaDefinition, ServiceDocument, Type, TypeDefinition, TypeKind,
    TypeSystemDefinition, UnionType,
};
use async_graphql::parser::{Pos as ParserPos, Positioned};
use async_graphql::Name;

use super::position::Pos;
use super::positioned_config::PositionedConfig;
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

pub fn from_document(input_path: &str, doc: ServiceDocument) -> Valid<Config, String> {
    let type_definitions: Vec<_> = doc
        .definitions
        .iter()
        .filter_map(|def| match def {
            TypeSystemDefinition::Type(td) => Some(td),
            _ => None,
        })
        .collect();

    let types = to_types(&type_definitions, input_path);
    let unions = to_union_types(&type_definitions, input_path);
    let enums = to_enum_types(&type_definitions, input_path);
    let schema = schema_definition(&doc)
        .map(|schema_definition| to_root_schema(schema_definition, input_path));

    schema_definition(&doc).and_then(|sd| {
        server(sd, input_path)
            .fuse(upstream(sd, input_path))
            .fuse(types)
            .fuse(unions)
            .fuse(enums)
            .fuse(schema)
            .fuse(links(sd, input_path))
            .fuse(telemetry(sd, input_path))
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

fn process_schema_directives<T: DirectiveCodec<T> + Default + Clone + PositionedConfig>(
    mut directives: Iter<'_, Positioned<ConstDirective>>,
    directive_name: &str,
    input_path: &str,
) -> Valid<Pos<T>, String> {
    if let Some(directive) =
        directives.find(|directive| directive.node.name.node.as_ref() == directive_name)
    {
        T::from_directive(&directive.node).and_then(|mut config| {
            directive.node.arguments.iter().for_each(|(key, _)| {
                config.set_field_position(
                    key.node.as_str(),
                    (key.pos.line, key.pos.column, input_path),
                )
            });
            Valid::succeed(Pos::new(
                directive.pos.line,
                directive.pos.column,
                Some(input_path.to_owned()),
                config,
            ))
        })
    } else {
        Valid::succeed(Default::default())
    }
}

fn process_schema_multiple_directives<T: DirectiveCodec<T> + Default>(
    schema_definition: &SchemaDefinition,
    directive_name: &str,
    input_path: &str,
) -> Valid<Vec<Pos<T>>, String> {
    let directives: Vec<Valid<Pos<T>, String>> = schema_definition
        .directives
        .iter()
        .filter_map(|directive| {
            if directive.node.name.node.as_ref() == directive_name {
                Some(T::from_directive(&directive.node).and_then(|config| {
                    Valid::succeed(Pos::new(
                        directive.pos.line,
                        directive.pos.column,
                        Some(input_path.to_owned()),
                        config,
                    ))
                }))
            } else {
                None
            }
        })
        .collect();

    Valid::from_iter(directives, |item| item)
}

fn process_schema_optional_directives<T: DirectiveCodec<T> + Clone + PositionedConfig>(
    mut directives: Iter<'_, Positioned<ConstDirective>>,
    directive_name: &str,
    input_path: &str,
) -> Valid<Option<Pos<T>>, String> {
    if let Some(directive) =
        directives.find(|directive| directive.node.name.node.as_ref() == directive_name)
    {
        T::from_directive(&directive.node)
            .and_then(|mut config| {
                directive.node.arguments.iter().for_each(|(key, _)| {
                    config.set_field_position(
                        key.node.as_str(),
                        (key.pos.line, key.pos.column, input_path),
                    )
                });
                Valid::succeed(Pos::new(
                    directive.pos.line,
                    directive.pos.column,
                    Some(input_path.to_owned()),
                    config,
                ))
            })
            .map(Some)
    } else {
        Valid::succeed(Default::default())
    }
}

fn server(schema_definition: &SchemaDefinition, input_path: &str) -> Valid<Pos<Server>, String> {
    process_schema_directives(
        schema_definition.directives.iter(),
        config::Server::directive_name().as_str(),
        input_path,
    )
}

fn upstream(
    schema_definition: &SchemaDefinition,
    input_path: &str,
) -> Valid<Pos<Upstream>, String> {
    process_schema_directives(
        schema_definition.directives.iter(),
        config::Upstream::directive_name().as_str(),
        input_path,
    )
}

fn links(schema_definition: &SchemaDefinition, input_path: &str) -> Valid<Vec<Pos<Link>>, String> {
    process_schema_multiple_directives(
        schema_definition,
        config::Link::directive_name().as_str(),
        input_path,
    )
}

fn telemetry(
    schema_definition: &SchemaDefinition,
    input_path: &str,
) -> Valid<Pos<Telemetry>, String> {
    process_schema_directives(
        schema_definition.directives.iter(),
        config::telemetry::Telemetry::directive_name().as_str(),
        input_path,
    )
}

fn to_root_schema(schema_definition: &SchemaDefinition, input_path: &str) -> RootSchema {
    let query = schema_definition.query.as_ref().map(|query| {
        Pos::new(
            query.pos.line,
            query.pos.column,
            Some(input_path.to_owned()),
            query.node.to_string(),
        )
    });
    let mutation = schema_definition.mutation.as_ref().map(|mutation| {
        Pos::new(
            mutation.pos.line,
            mutation.pos.column,
            Some(input_path.to_owned()),
            mutation.node.to_string(),
        )
    });
    let subscription = schema_definition.subscription.as_ref().map(|subscription| {
        Pos::new(
            subscription.pos.line,
            subscription.pos.column,
            Some(input_path.to_owned()),
            subscription.node.to_string(),
        )
    });

    RootSchema { query, mutation, subscription }
}
fn pos_name_to_string(pos: &Positioned<Name>) -> String {
    pos.node.to_string()
}
fn to_types(
    type_definitions: &Vec<&Positioned<TypeDefinition>>,
    input_path: &str,
) -> Valid<BTreeMap<String, Pos<config::Type>>, String> {
    Valid::from_iter(type_definitions, |type_definition| {
        let type_name = pos_name_to_string(&type_definition.node.name);
        match type_definition.node.kind.clone() {
            TypeKind::Object(object_type) => to_object_type(
                &object_type,
                &type_definition.node.description,
                &type_definition.node.directives,
                &type_definition.pos,
                input_path,
            )
            .some(),
            TypeKind::Interface(interface_type) => to_object_type(
                &interface_type,
                &type_definition.node.description,
                &type_definition.node.directives,
                &type_definition.pos,
                input_path,
            )
            .some(),
            TypeKind::Enum(_) => Valid::none(),
            TypeKind::InputObject(input_object_type) => to_input_object(
                input_object_type,
                &type_definition.node.description,
                &type_definition.node.directives,
                &type_definition.pos,
                input_path,
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
fn to_scalar_type() -> Pos<config::Type> {
    Default::default()
}
fn to_union_types(
    type_definitions: &[&Positioned<TypeDefinition>],
    input_path: &str,
) -> Valid<BTreeMap<String, Pos<Union>>, String> {
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
                        &type_definition.pos,
                        input_path,
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
    input_path: &str,
) -> Valid<BTreeMap<String, Pos<Enum>>, String> {
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
                        &type_definition.pos,
                        input_path,
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
    type_position: &ParserPos,
    input_path: &str,
) -> Valid<Pos<config::Type>, String>
where
    T: ObjectLike,
{
    let fields = object.fields();
    let implements = object.implements();

    process_schema_optional_directives(
        directives.iter(),
        Cache::directive_name().as_str(),
        input_path,
    )
    .fuse(to_fields(fields, input_path))
    .fuse(process_schema_optional_directives(
        directives.iter(),
        Protected::directive_name().as_str(),
        input_path,
    ))
    .fuse(process_schema_optional_directives(
        directives.iter(),
        Tag::directive_name().as_str(),
        input_path,
    ))
    .map(|(cache, fields, protected, tag)| {
        let doc = description.to_owned().map(|pos| pos.node);
        let implements = implements.iter().map(|pos| pos.node.to_string()).collect();
        let added_fields = to_add_fields_from_directives(directives, input_path);
        Pos::new(
            type_position.line,
            type_position.column,
            Some(input_path.to_owned()),
            config::Type { fields, added_fields, doc, implements, cache, protected, tag },
        )
    })
}

#[allow(clippy::too_many_arguments)]
fn to_input_object(
    input_object_type: InputObjectType,
    description: &Option<Positioned<String>>,
    directives: &[Positioned<ConstDirective>],
    position: &ParserPos,
    input_path: &str,
) -> Valid<Pos<config::Type>, String> {
    to_input_object_fields(&input_object_type.fields, input_path)
        .fuse(process_schema_optional_directives(
            directives.iter(),
            Protected::directive_name().as_str(),
            input_path,
        ))
        .map(|(fields, protected)| {
            let doc = description.to_owned().map(|pos| pos.node);
            Pos::new(
                position.line,
                position.column,
                Some(input_path.to_owned()),
                config::Type { fields, protected, doc, ..Default::default() },
            )
        })
}

fn to_fields_inner<T, F>(
    fields: &Vec<Positioned<T>>,
    transform: F,
    input_path: &str,
) -> Valid<BTreeMap<String, Pos<config::Field>>, String>
where
    F: Fn(&T, &ParserPos, &str) -> Valid<Pos<config::Field>, String>,
    T: HasName,
{
    Valid::from_iter(fields, |field| {
        let field_name = pos_name_to_string(field.node.name());
        transform(&field.node, &field.pos, input_path).map(|field| (field_name, field))
    })
    .map(BTreeMap::from_iter)
}
fn to_fields(
    fields: &Vec<Positioned<FieldDefinition>>,
    input_path: &str,
) -> Valid<BTreeMap<String, Pos<config::Field>>, String> {
    to_fields_inner(fields, to_field, input_path)
}
fn to_input_object_fields(
    input_object_fields: &Vec<Positioned<InputValueDefinition>>,
    input_path: &str,
) -> Valid<BTreeMap<String, Pos<config::Field>>, String> {
    to_fields_inner(input_object_fields, to_input_object_field, input_path)
}
fn to_field(
    field_definition: &FieldDefinition,
    field_position: &ParserPos,
    input_path: &str,
) -> Valid<Pos<config::Field>, String> {
    to_common_field(
        field_definition,
        field_position,
        to_args(field_definition, input_path),
        input_path,
    )
}
fn to_input_object_field(
    field_definition: &InputValueDefinition,
    field_position: &ParserPos,
    input_path: &str,
) -> Valid<Pos<config::Field>, String> {
    to_common_field(
        field_definition,
        field_position,
        BTreeMap::new(),
        input_path,
    )
}
fn to_common_field<F>(
    field: &F,
    field_position: &ParserPos,
    args: BTreeMap<String, config::Arg>,
    input_path: &str,
) -> Valid<Pos<config::Field>, String>
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

    process_schema_optional_directives(
        directives.iter(),
        config::Http::directive_name().as_str(),
        input_path,
    )
    .fuse(process_schema_optional_directives(
        directives.iter(),
        GraphQL::directive_name().as_str(),
        input_path,
    ))
    .fuse(process_schema_optional_directives(
        directives.iter(),
        Cache::directive_name().as_str(),
        input_path,
    ))
    .fuse(process_schema_optional_directives(
        directives.iter(),
        Grpc::directive_name().as_str(),
        input_path,
    ))
    .fuse(process_schema_optional_directives(
        directives.iter(),
        Omit::directive_name().as_str(),
        input_path,
    ))
    .fuse(process_schema_optional_directives(
        directives.iter(),
        Modify::directive_name().as_str(),
        input_path,
    ))
    .fuse(process_schema_optional_directives(
        directives.iter(),
        JS::directive_name().as_str(),
        input_path,
    ))
    .fuse(process_schema_optional_directives(
        directives.iter(),
        Call::directive_name().as_str(),
        input_path,
    ))
    .fuse(process_schema_optional_directives(
        directives.iter(),
        Protected::directive_name().as_str(),
        input_path,
    ))
    .map(
        |(http, graphql, cache, grpc, omit, modify, script, call, protected)| {
            let const_field = to_const_field(directives, input_path);
            Pos::new(
                field_position.line,
                field_position.column,
                Some(input_path.to_owned()),
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
                },
            )
        },
    )
}

fn to_type_of(type_: &Type) -> String {
    match &type_.base {
        BaseType::Named(name) => name.to_string(),
        BaseType::List(ty) => to_type_of(ty),
    }
}
fn to_args(field_definition: &FieldDefinition, input_path: &str) -> BTreeMap<String, config::Arg> {
    let mut args: BTreeMap<String, config::Arg> = BTreeMap::new();

    for arg in field_definition.arguments.iter() {
        let arg_name = pos_name_to_string(&arg.node.name);
        let arg_val = to_arg(&arg.node, input_path);
        args.insert(arg_name, arg_val);
    }

    args
}
fn to_arg(input_value_definition: &InputValueDefinition, input_path: &str) -> config::Arg {
    let type_of = to_type_of(&input_value_definition.ty.node);
    let list = matches!(&input_value_definition.ty.node.base, BaseType::List(_));
    let required = !input_value_definition.ty.node.nullable;
    let doc = input_value_definition
        .description
        .to_owned()
        .map(|pos| pos.node);

    let modify = process_schema_optional_directives(
        input_value_definition.directives.iter(),
        Modify::directive_name().as_str(),
        input_path,
    )
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

fn to_union(
    union_type: UnionType,
    doc: &Option<String>,
    position: &ParserPos,
    input_path: &str,
) -> Pos<Union> {
    let types = union_type
        .members
        .iter()
        .map(|member| member.node.to_string())
        .collect();

    Pos::new(
        position.line,
        position.column,
        Some(input_path.to_owned()),
        Union { types, doc: doc.clone() },
    )
}

fn to_enum(
    enum_type: EnumType,
    doc: Option<String>,
    position: &ParserPos,
    input_path: &str,
) -> Pos<Enum> {
    let variants = enum_type
        .values
        .iter()
        .map(|member| member.node.value.node.as_str().to_owned())
        .collect();
    Pos::new(
        position.line,
        position.column,
        Some(input_path.to_owned()),
        Enum { variants, doc },
    )
}
fn to_const_field(
    directives: &[Positioned<ConstDirective>],
    input_path: &str,
) -> Option<Pos<config::Expr>> {
    directives.iter().find_map(|directive| {
        if directive.node.name.node == config::Expr::directive_name() {
            config::Expr::from_directive(&directive.node)
                .and_then(|config| {
                    Valid::succeed(Pos::new(
                        directive.pos.line,
                        directive.pos.column,
                        Some(input_path.to_owned()),
                        config,
                    ))
                })
                .to_result()
                .ok()
        } else {
            None
        }
    })
}

fn to_add_fields_from_directives(
    directives: &[Positioned<ConstDirective>],
    input_path: &str,
) -> Vec<Pos<config::AddField>> {
    directives
        .iter()
        .filter_map(|directive| {
            if directive.node.name.node == config::AddField::directive_name() {
                config::AddField::from_directive(&directive.node)
                    .and_then(|mut field| {
                        directive.node.arguments.iter().for_each(|(key, _)| {
                            field.set_field_position(
                                key.node.as_str(),
                                (key.pos.line, key.pos.column, input_path),
                            )
                        });
                        Valid::succeed(Pos::new(
                            directive.pos.line,
                            directive.pos.column,
                            Some(input_path.to_owned()),
                            field,
                        ))
                    })
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
