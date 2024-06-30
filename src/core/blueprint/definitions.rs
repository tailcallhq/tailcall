use std::collections::HashSet;

use async_graphql_value::ConstValue;
use regex::Regex;
use union_resolver::update_union_resolver;

use crate::core::blueprint::Type::ListType;
use crate::core::blueprint::*;
use crate::core::config::{Config, Enum, Field, GraphQLOperationType, Protected, Union};
use crate::core::directive::DirectiveCodec;
use crate::core::ir::model::{Cache, IR};
use crate::core::try_fold::TryFold;
use crate::core::valid::{Valid, Validator};
use crate::core::{config, scalar};

pub fn to_scalar_type_definition(name: &str) -> Valid<Definition, String> {
    Valid::succeed(Definition::Scalar(ScalarTypeDefinition {
        name: name.to_string(),
        directive: Vec::new(),
        description: None,
        validator: scalar::get_scalar(name),
    }))
}

pub fn to_union_type_definition((name, u): (&String, &Union)) -> Definition {
    Definition::Union(UnionTypeDefinition {
        name: name.to_owned(),
        description: u.doc.clone(),
        directives: Vec::new(),
        types: u.types.clone(),
    })
}

pub fn to_input_object_type_definition(
    definition: ObjectTypeDefinition,
) -> Valid<Definition, String> {
    Valid::succeed(Definition::InputObject(InputObjectTypeDefinition {
        name: definition.name,
        fields: definition
            .fields
            .iter()
            .map(|field| InputFieldDefinition {
                name: field.name.clone(),
                description: field.description.clone(),
                default_value: field.default_value.clone(),
                of_type: field.of_type.clone(),
            })
            .collect(),
        description: definition.description,
    }))
}

pub fn to_interface_type_definition(definition: ObjectTypeDefinition) -> Valid<Definition, String> {
    Valid::succeed(Definition::Interface(InterfaceTypeDefinition {
        name: definition.name,
        fields: definition.fields,
        description: definition.description,
    }))
}

type InvalidPathHandler = dyn Fn(&str, &[String], &[String]) -> Valid<Type, String>;
type PathResolverErrorHandler = dyn Fn(&str, &str, &str, &[String]) -> Valid<Type, String>;

struct ProcessFieldWithinTypeContext<'a> {
    field: &'a config::Field,
    field_name: &'a str,
    remaining_path: &'a [String],
    type_info: &'a config::Type,
    is_required: bool,
    config_module: &'a ConfigModule,
    invalid_path_handler: &'a InvalidPathHandler,
    path_resolver_error_handler: &'a PathResolverErrorHandler,
    original_path: &'a [String],
}

#[derive(Clone)]
struct ProcessPathContext<'a> {
    path: &'a [String],
    field: &'a config::Field,
    type_info: &'a config::Type,
    is_required: bool,
    config_module: &'a ConfigModule,
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
    let config_module = context.config_module;
    let invalid_path_handler = context.invalid_path_handler;
    let path_resolver_error_handler = context.path_resolver_error_handler;

    if let Some(next_field) = type_info.fields.get(field_name) {
        if next_field.has_resolver() {
            let next_dir_http = next_field
                .http
                .as_ref()
                .map(|_| config::Http::directive_name());
            let next_dir_const = next_field
                .const_field
                .as_ref()
                .map(|_| config::Expr::directive_name());
            return path_resolver_error_handler(
                next_dir_http
                    .or(next_dir_const)
                    .unwrap_or(config::JS::directive_name())
                    .as_str(),
                &field.type_of,
                field_name,
                context.original_path,
            )
            .and(process_path(ProcessPathContext {
                type_info,
                is_required,
                config_module,
                invalid_path_handler,
                path_resolver_error_handler,
                path: remaining_path,
                field: next_field,
                original_path: context.original_path,
            }));
        }

        let next_is_required = is_required && next_field.required;
        if scalar::is_predefined_scalar(&next_field.type_of) {
            return process_path(ProcessPathContext {
                type_info,
                config_module,
                invalid_path_handler,
                path_resolver_error_handler,
                path: remaining_path,
                field: next_field,
                is_required: next_is_required,
                original_path: context.original_path,
            });
        }

        if let Some(next_type_info) = config_module.find_type(&next_field.type_of) {
            return process_path(ProcessPathContext {
                config_module,
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
                config_module,
                invalid_path_handler,
                path_resolver_error_handler,
                original_path: context.original_path,
            });
        }
    }

    invalid_path_handler(field_name, remaining_path, context.original_path)
}

// Helper function to recursively process the path and return the corresponding
// type
fn process_path(context: ProcessPathContext) -> Valid<Type, String> {
    let path = context.path;
    let field = context.field;
    let type_info = context.type_info;
    let is_required = context.is_required;
    let config_module = context.config_module;
    let invalid_path_handler = context.invalid_path_handler;
    let path_resolver_error_handler = context.path_resolver_error_handler;
    if let Some((field_name, remaining_path)) = path.split_first() {
        if field_name.parse::<usize>().is_ok() {
            let mut modified_field = field.clone();
            modified_field.list = false;
            return process_path(ProcessPathContext {
                config_module,
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
            .or_else(|| config_module.find_type(&field.type_of));

        if let Some(type_info) = target_type_info {
            return process_field_within_type(ProcessFieldWithinTypeContext {
                field,
                field_name,
                remaining_path,
                type_info,
                is_required,
                config_module,
                invalid_path_handler,
                path_resolver_error_handler,
                original_path: context.original_path,
            });
        }
        return invalid_path_handler(field_name, path, context.original_path);
    }

    Valid::succeed(to_type(field, Some(is_required)))
}

fn to_enum_type_definition((name, eu): (&String, &Enum)) -> Definition {
    Definition::Enum(EnumTypeDefinition {
        name: name.to_owned(),
        directives: Vec::new(),
        description: eu.doc.to_owned(),
        enum_values: eu
            .variants
            .iter()
            .map(|variant| EnumValueDefinition {
                description: None,
                name: variant.name.clone(),
                directives: vec![],
            })
            .collect(),
    })
}

fn to_object_type_definition(
    name: &str,
    type_of: &config::Type,
    config_module: &ConfigModule,
) -> Valid<Definition, String> {
    to_fields(name, type_of, config_module).map(|fields| {
        Definition::Object(ObjectTypeDefinition {
            name: name.to_string(),
            description: type_of.doc.clone(),
            fields,
            implements: type_of.implements.clone(),
        })
    })
}

fn update_args<'a>(
) -> TryFold<'a, (&'a ConfigModule, &'a Field, &'a config::Type, &'a str), FieldDefinition, String>
{
    TryFold::<(&ConfigModule, &Field, &config::Type, &str), FieldDefinition, String>::new(
        move |(_, field, _typ, name), _| {
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
                default_value: field.default_value.clone(),
            })
        },
    )
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
        let resolver = IR::ContextPath(context.path.to_owned());
        if has_index {
            updated_base_field.of_type =
                Type::NamedType { name: of_type.name().to_string(), non_null: false }
        } else {
            updated_base_field.of_type = of_type;
        }
        let resolver = match updated_base_field.resolver.clone() {
            None => resolver,
            Some(resolver) => IR::Path(Box::new(resolver), context.path.to_owned()),
        };
        Valid::succeed(updated_base_field.resolver(Some(resolver)))
    })
}

/// This function iterates over all types and their fields identifying paths to
/// fields with dangling resolvers and fixes them. Dangling resolvers are those
/// resolvers that cannot be resolved from the root of the schema. This function
/// finds such dangling resolvers and creates a resolvable path from the root
/// schema.
pub fn fix_dangling_resolvers<'a>(
) -> TryFold<'a, (&'a ConfigModule, &'a Field, &'a config::Type, &'a str), FieldDefinition, String>
{
    TryFold::<(&ConfigModule, &Field, &config::Type, &str), FieldDefinition, String>::new(
        move |(config, field, ty, name), mut b_field| {
            let mut set = HashSet::new();
            if !field.has_resolver()
                && validate_field_has_resolver(name, field, &config.types, ty, &mut set)
                    .is_succeed()
            {
                b_field = b_field.resolver(Some(IR::Dynamic(DynamicValue::Value(
                    ConstValue::Object(Default::default()),
                ))));
            }

            Valid::succeed(b_field)
        },
    )
}

/// Wraps the IO Expression with Expression::Cached
/// if `Field::cache` is present for that field
pub fn update_cache_resolvers<'a>(
) -> TryFold<'a, (&'a ConfigModule, &'a Field, &'a config::Type, &'a str), FieldDefinition, String>
{
    TryFold::<(&ConfigModule, &Field, &config::Type, &str), FieldDefinition, String>::new(
        move |(_config, field, typ, _name), mut b_field| {
            if let Some(config::Cache { max_age }) = field.cache.as_ref().or(typ.cache.as_ref()) {
                b_field.map_expr(|expression| Cache::wrap(*max_age, expression))
            }

            Valid::succeed(b_field)
        },
    )
}

fn validate_field_type_exist(config: &Config, field: &Field) -> Valid<(), String> {
    let field_type = &field.type_of;
    if !scalar::is_predefined_scalar(field_type) && !config.contains(field_type) {
        Valid::fail(format!("Undeclared type '{field_type}' was found"))
    } else {
        Valid::succeed(())
    }
}

fn to_fields(
    object_name: &str,
    type_of: &config::Type,
    config_module: &ConfigModule,
) -> Valid<Vec<FieldDefinition>, String> {
    let operation_type = if config_module
        .schema
        .mutation
        .as_deref()
        .eq(&Some(object_name))
    {
        GraphQLOperationType::Mutation
    } else {
        GraphQLOperationType::Query
    };
    // Process fields that are not marked as `omit`
    let fields = Valid::from_iter(
        type_of
            .fields
            .iter()
            .filter(|(_, field)| !field.is_omitted()),
        |(name, field)| {
            validate_field_type_exist(config_module, field)
                .and(to_field_definition(
                    field,
                    &operation_type,
                    object_name,
                    config_module,
                    type_of,
                    name,
                ))
                .trace(name)
        },
    );

    let to_added_field = |add_field: &config::AddField,
                          type_of: &config::Type|
     -> Valid<blueprint::FieldDefinition, String> {
        let source_field = type_of
            .fields
            .iter()
            .find(|&(field_name, _)| *field_name == add_field.path[0]);
        match source_field {
            Some((_, source_field)) => to_field_definition(
                source_field,
                &operation_type,
                object_name,
                config_module,
                type_of,
                &add_field.name,
            )
            .and_then(|field_definition| {
                let added_field_path = match source_field.http {
                    Some(_) => add_field.path[1..]
                        .iter()
                        .map(|s| s.to_owned())
                        .collect::<Vec<_>>(),
                    None => add_field.path.clone(),
                };
                let invalid_path_handler = |field_name: &str,
                                            _added_field_path: &[String],
                                            original_path: &[String]|
                 -> Valid<Type, String> {
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
                        config_module,
                        invalid_path_handler: &invalid_path_handler,
                        path_resolver_error_handler: &path_resolver_error_handler,
                        original_path: &add_field.path,
                    },
                    field_definition,
                )
            })
            .trace(config::AddField::trace_name().as_str()),
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

#[allow(clippy::too_many_arguments)]
pub fn to_field_definition(
    field: &Field,
    operation_type: &GraphQLOperationType,
    object_name: &str,
    config_module: &ConfigModule,
    type_of: &config::Type,
    name: &String,
) -> Valid<FieldDefinition, String> {
    let directives = field.resolvable_directives();

    if directives.len() > 1 {
        return Valid::fail(format!(
            "Multiple resolvers detected [{}]",
            directives.join(", ")
        ));
    }

    update_args()
        .and(update_http().trace(config::Http::trace_name().as_str()))
        .and(update_grpc(operation_type).trace(config::Grpc::trace_name().as_str()))
        .and(update_const_field().trace(config::Expr::trace_name().as_str()))
        .and(update_js_field().trace(config::JS::trace_name().as_str()))
        .and(update_graphql(operation_type).trace(config::GraphQL::trace_name().as_str()))
        .and(update_modify().trace(config::Modify::trace_name().as_str()))
        .and(update_call(operation_type, object_name).trace(config::Call::trace_name().as_str()))
        .and(fix_dangling_resolvers())
        .and(update_cache_resolvers())
        .and(update_protected(object_name).trace(Protected::trace_name().as_str()))
        .and(update_enum_alias())
        .and(update_union_resolver())
        .try_fold(
            &(config_module, field, type_of, name),
            FieldDefinition::default(),
        )
}

pub fn to_definitions<'a>() -> TryFold<'a, ConfigModule, Vec<Definition>, String> {
    TryFold::<ConfigModule, Vec<Definition>, String>::new(|config_module, _| {
        Valid::from_iter(config_module.types.iter(), |(name, type_)| {
            if type_.scalar() {
                to_scalar_type_definition(name).trace(name)
            } else {
                to_object_type_definition(name, type_, config_module)
                    .trace(name)
                    .and_then(|definition| match definition.clone() {
                        Definition::Object(object_type_definition) => {
                            if config_module.input_types().contains(name) {
                                to_input_object_type_definition(object_type_definition).trace(name)
                            } else if config_module.interface_types().contains(name) {
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
            types.extend(config_module.unions.iter().map(to_union_type_definition));
            types
        })
        .fuse(Valid::from_iter(
            config_module.enums.iter(),
            |(name, type_)| {
                if type_.variants.is_empty() {
                    Valid::fail("No variants found for enum".to_string())
                } else {
                    Valid::succeed(to_enum_type_definition((name, type_)))
                }
            },
        ))
        .map(|tp| {
            let mut v = tp.0;
            v.extend(tp.1);
            v
        })
    })
}
