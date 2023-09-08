use std::collections::BTreeMap;

use async_graphql::parser::types::{
    BaseType, ConstDirective, EnumType, FieldDefinition, InputObjectType, InputValueDefinition, SchemaDefinition,
    ServiceDocument, Type, TypeDefinition, TypeKind, TypeSystemDefinition, UnionType,
};
use async_graphql::parser::Positioned;
use async_graphql::Name;

use crate::batch::Batch;
use crate::config;
use crate::config::Config;
use crate::config::{GraphQL, RootSchema, Server, Union};
use crate::directive::DirectiveCodec;

use anyhow::Result;

fn from_document(doc: ServiceDocument) -> Result<Config> {
    let server = if let Some(defs) = schema_definition(&doc) {
        server(defs)?
    } else {
        Server::default()
    };
    Ok(Config { server, graphql: graphql(&doc)? })
}
fn graphql(doc: &ServiceDocument) -> Result<GraphQL> {
    let type_definitions: Vec<_> = doc
        .definitions
        .iter()
        .filter_map(|def| match def {
            TypeSystemDefinition::Type(type_definition) => Some(type_definition),
            _ => None,
        })
        .collect();

    let may_be_schema = schema_definition(doc);
    let root_schema = if let Some(schema_definition) = may_be_schema {
        to_root_schema(schema_definition)?
    } else {
        RootSchema::default()
    };
    let types = to_types(&type_definitions)?;
    let unions = to_union_types(&type_definitions)?;
    Ok(GraphQL { schema: root_schema, types, unions: Some(unions) })
}
fn schema_definition(doc: &ServiceDocument) -> Option<&SchemaDefinition> {
    doc.definitions.iter().find_map(|def| match def {
        TypeSystemDefinition::Schema(schema_definition) => Some(&schema_definition.node),
        _ => None,
    })
}
fn server(schema_definition: &SchemaDefinition) -> Result<Server> {
    let mut server = Server::default();
    for directive in schema_definition.directives.iter() {
        if directive.node.name.node == "server" {
            server = Server::from_directive(&directive.node)?;
        }
    }
    Ok(server)
}
fn to_root_schema(schema_definition: &SchemaDefinition) -> Result<RootSchema> {
    let query = schema_definition.query.as_ref().map(pos_name_to_string);
    let mutation = schema_definition.mutation.as_ref().map(pos_name_to_string);
    let subscription = schema_definition.subscription.as_ref().map(pos_name_to_string);

    Ok(RootSchema { query, mutation, subscription })
}
fn pos_name_to_string(pos: &Positioned<Name>) -> String {
    pos.node.to_string()
}
fn to_types(type_definitions: &Vec<&Positioned<TypeDefinition>>) -> Result<BTreeMap<String, config::Type>> {
    let mut types = BTreeMap::new();
    for type_definition in type_definitions {
        let type_name = pos_name_to_string(&type_definition.node.name);
        let type_opt = match type_definition.node.kind.clone() {
            TypeKind::Object(object_type) => Some(to_object_type(
                &object_type.fields,
                &type_definition.node.description,
                &false,
                &object_type.implements,
            )?),
            TypeKind::Interface(interface_type) => Some(to_object_type(
                &interface_type.fields,
                &type_definition.node.description,
                &true,
                &interface_type.implements,
            )?),
            TypeKind::Enum(enum_type) => Some(to_enum(enum_type)?),
            TypeKind::InputObject(input_object_type) => Some(to_input_object(input_object_type)?),
            TypeKind::Union(_) => None,
            TypeKind::Scalar => Some(to_scalar_type()?),
        };
        if let Some(type_) = type_opt {
            types.insert(type_name, type_);
        }
    }
    Ok(types)
}
fn to_scalar_type() -> Result<config::Type> {
    Ok(config::Type {
        fields: BTreeMap::new(),
        doc: None,
        interface: None,
        implements: None,
        variants: None,
        scalar: Some(true),
    })
}
fn to_union_types(type_definitions: &Vec<&Positioned<TypeDefinition>>) -> Result<Vec<Union>> {
    let mut unions = Vec::new();
    for type_definition in type_definitions {
        let type_opt = match type_definition.node.kind.clone() {
            TypeKind::Union(union_type) => to_union(
                union_type,
                &type_definition.node.name.node,
                &type_definition.node.description.as_ref().map(|pos| pos.node.clone()),
            )?,
            _ => continue,
        };
        unions.push(type_opt);
    }
    Ok(unions)
}
fn to_object_type(
    fields: &Vec<Positioned<FieldDefinition>>,
    description: &Option<Positioned<String>>,
    is_interface: &bool,
    implements: &[Positioned<Name>],
) -> Result<config::Type> {
    let fields = to_fields(fields)?;
    let doc = description.as_ref().map(|pos| pos.node.clone());
    let interface = Some(*is_interface);
    let implements = Some(implements.iter().map(|pos| pos.node.to_string()).collect());
    Ok(config::Type { fields, doc, interface, implements, variants: None, scalar: None })
}
fn to_enum(enum_type: EnumType) -> Result<config::Type> {
    let variants = enum_type
        .values
        .iter()
        .map(|value| value.node.value.to_string())
        .collect();
    Ok(config::Type {
        fields: BTreeMap::new(),
        doc: None,
        interface: None,
        implements: None,
        variants: Some(variants),
        scalar: None,
    })
}
fn to_input_object(input_object_type: InputObjectType) -> Result<config::Type> {
    let fields = to_input_object_fields(&input_object_type.fields)?;
    Ok(config::Type { fields, doc: None, interface: None, implements: None, variants: None, scalar: None })
}
fn to_fields_inner<T, F>(fields: &Vec<Positioned<T>>, transform: F) -> Result<BTreeMap<String, config::Field>>
where
    F: Fn(&T) -> Result<config::Field>,
    T: HasName,
{
    let mut parsed_fields = BTreeMap::new();
    for field in fields {
        let field_name = pos_name_to_string(field.node.name());
        let field_ = transform(&field.node)?;
        parsed_fields.insert(field_name, field_);
    }
    Ok(parsed_fields)
}
fn to_fields(fields: &Vec<Positioned<FieldDefinition>>) -> Result<BTreeMap<String, config::Field>> {
    to_fields_inner(fields, to_field)
}
fn to_input_object_fields(
    input_object_fields: &Vec<Positioned<InputValueDefinition>>,
) -> Result<BTreeMap<String, config::Field>> {
    to_fields_inner(input_object_fields, to_input_object_field)
}
fn to_field(field_definition: &FieldDefinition) -> Result<config::Field> {
    to_common_field(
        &field_definition.ty.node,
        &field_definition.ty.node.base,
        field_definition.ty.node.nullable,
        Some(to_args(field_definition)?),
        &field_definition.description,
        &field_definition.directives,
    )
}
fn to_input_object_field(field_definition: &InputValueDefinition) -> Result<config::Field> {
    to_common_field(
        &field_definition.ty.node,
        &field_definition.ty.node.base,
        field_definition.ty.node.nullable,
        None,
        &field_definition.description,
        &field_definition.directives,
    )
}
fn to_common_field(
    type_: &Type,
    base: &BaseType,
    nullable: bool,
    args: Option<BTreeMap<String, config::Arg>>,
    description: &Option<Positioned<String>>,
    directives: &[Positioned<ConstDirective>],
) -> Result<config::Field> {
    let type_of = to_type_of(type_)?;
    let list = Some(matches!(&base, BaseType::List(_)));
    let required = if nullable { None } else { Some(false) };
    let list_type_required = Some(matches!(&base, BaseType::List(ty) if !ty.nullable));
    let doc = description.as_ref().map(|pos| pos.node.clone());
    let modify = to_modify(directives);
    let inline = to_inline(directives);
    let http = to_http(directives);
    let unsafe_operation = to_unsafe_operation(directives);
    let batch = to_batch(directives);
    Ok(config::Field {
        type_of,
        list,
        required,
        list_type_required,
        args,
        doc,
        modify,
        inline,
        http,
        unsafe_operation,
        batch,
    })
}
fn to_unsafe_operation(directives: &[Positioned<ConstDirective>]) -> Option<config::Unsafe> {
    directives.iter().find_map(|directive| {
        if directive.node.name.node == "unsafe" {
            config::Unsafe::from_directive(&directive.node).ok()
        } else {
            None
        }
    })
}
fn to_type_of(type_: &Type) -> Result<String> {
    match &type_.base {
        BaseType::Named(name) => Ok(name.to_string()),
        BaseType::List(ty) => match &ty.base {
            BaseType::Named(name) => Ok(name.to_string()),
            _ => Ok("".to_string()),
        },
    }
}
fn to_args(field_definition: &FieldDefinition) -> Result<BTreeMap<String, config::Arg>> {
    let mut args: BTreeMap<String, config::Arg> = BTreeMap::new();

    for arg in field_definition.arguments.iter() {
        let arg_name = pos_name_to_string(&arg.node.name);
        let arg_val = to_arg(&arg.node)?;
        args.insert(arg_name, arg_val);
    }

    Ok(args)
}
fn to_arg(input_value_definition: &InputValueDefinition) -> Result<config::Arg> {
    let type_of = to_type_of(&input_value_definition.ty.node)?;
    let list = Some(matches!(&input_value_definition.ty.node.base, BaseType::List(_)));
    let required = Some(!input_value_definition.ty.node.nullable);
    let doc = input_value_definition.description.as_ref().map(|pos| pos.node.clone());
    let modify = to_modify(&input_value_definition.directives);
    let default_value = if let Some(pos) = input_value_definition.default_value.as_ref() {
        let value = &pos.node;
        Some(serde_json::to_value(value)?)
    } else {
        None
    };
    Ok(config::Arg { type_of, list, required, doc, modify, default_value })
}
fn to_modify(directives: &[Positioned<ConstDirective>]) -> Option<config::ModifyField> {
    directives.iter().find_map(|directive| {
        if directive.node.name.node == "modify" {
            config::ModifyField::from_directive(&directive.node).ok()
        } else {
            None
        }
    })
}
fn to_inline(directives: &[Positioned<ConstDirective>]) -> Option<config::InlineType> {
    directives.iter().find_map(|directive| {
        if directive.node.name.node == "inline" {
            config::InlineType::from_directive(&directive.node).ok()
        } else {
            None
        }
    })
}
fn to_http(directives: &[Positioned<ConstDirective>]) -> Option<config::Http> {
    directives.iter().find_map(|directive| {
        if directive.node.name.node == "http" {
            config::Http::from_directive(&directive.node).ok()
        } else {
            None
        }
    })
}
fn to_union(union_type: UnionType, name: &str, doc: &Option<String>) -> Result<config::Union> {
    let types = union_type
        .members
        .iter()
        .map(|member| member.node.to_string())
        .collect();
    Ok(config::Union { name: name.to_owned(), types, doc: doc.clone() })
}
fn to_batch(directives: &[Positioned<ConstDirective>]) -> Option<Batch> {
    directives.iter().find_map(|directive| {
        if directive.node.name.node == "batch" {
            Batch::from_directive(&directive.node).ok()
        } else {
            None
        }
    })
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
impl TryFrom<ServiceDocument> for Config {
    type Error = anyhow::Error;
    fn try_from(doc: ServiceDocument) -> Result<Self> {
        from_document(doc)
    }
}
