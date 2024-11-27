use std::collections::HashSet;

use async_graphql_value::ConstValue;
use directive::Directive;
use interface_resolver::update_interface_resolver;
use regex::Regex;
use tailcall_valid::{Valid, Validator};
use union_resolver::update_union_resolver;

use crate::core::blueprint::*;
use crate::core::config::{Config, Enum, Field, GraphQLOperationType, Protected, Union};
use crate::core::directive::DirectiveCodec;
use crate::core::ir::model::{Cache, IR};
use crate::core::try_fold::TryFold;
use crate::core::{config, scalar, Type};

pub fn to_scalar_type_definition(name: &str) -> Valid<Definition, BlueprintError> {
    if scalar::Scalar::is_predefined(name) {
        Valid::fail(BlueprintError::ScalarTypeIsPredefined(name.to_string()))
    } else {
        Valid::succeed(Definition::Scalar(ScalarTypeDefinition {
            name: name.to_string(),
            directives: Vec::new(),
            description: None,
            scalar: scalar::Scalar::find(name)
                .unwrap_or(&scalar::Scalar::Empty)
                .clone(),
        }))
    }
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
) -> Valid<Definition, BlueprintError> {
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
        directives: Vec::new(),
    }))
}

pub fn to_interface_type_definition(
    definition: ObjectTypeDefinition,
) -> Valid<Definition, BlueprintError> {
    Valid::succeed(Definition::Interface(InterfaceTypeDefinition {
        name: definition.name,
        fields: definition.fields,
        description: definition.description,
        implements: definition.implements,
        directives: Vec::new(),
    }))
}

type InvalidPathHandler = dyn Fn(&str, &[String], &[String]) -> Valid<Type, BlueprintError>;
type PathResolverErrorHandler = dyn Fn(&str, &str, &str, &[String]) -> Valid<Type, BlueprintError>;

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
    // TODO: does it even used other than as false?
    is_required: bool,
    config_module: &'a ConfigModule,
    invalid_path_handler: &'a InvalidPathHandler,
    path_resolver_error_handler: &'a PathResolverErrorHandler,
    original_path: &'a [String],
}

fn process_field_within_type(
    context: ProcessFieldWithinTypeContext,
) -> Valid<Type, BlueprintError> {
    let field = context.field;
    let field_name = context.field_name;
    let remaining_path = context.remaining_path;
    let type_info = context.type_info;
    let is_required = context.is_required;
    let config_module = context.config_module;
    let invalid_path_handler = context.invalid_path_handler;
    let path_resolver_error_handler = context.path_resolver_error_handler;

    if let Some(next_field) = type_info.fields.get(field_name) {
        if !next_field.resolvers.is_empty() {
            let mut valid = Valid::succeed(field.type_of.clone());

            for resolver in next_field.resolvers.iter() {
                valid = valid.and(path_resolver_error_handler(
                    &resolver.directive_name(),
                    field.type_of.name(),
                    field_name,
                    context.original_path,
                ));
            }

            return valid.and(process_path(ProcessPathContext {
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

        let next_is_required = is_required && !next_field.type_of.is_nullable();
        if scalar::Scalar::is_predefined(next_field.type_of.name()) {
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

        if let Some(next_type_info) = config_module.find_type(next_field.type_of.name()) {
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
                if next_field.type_of.is_list() {
                    Valid::succeed(Type::List { of_type: Box::new(of_type), non_null: is_required })
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
fn process_path(context: ProcessPathContext) -> Valid<Type, BlueprintError> {
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
            // TODO: does it required?
            modified_field.type_of = modified_field.type_of.into_single();
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
            .or_else(|| config_module.find_type(field.type_of.name()));

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

    Valid::succeed(if is_required {
        field.type_of.clone().into_required()
    } else {
        field.type_of.clone().into_nullable()
    })
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
                alias: variant.alias.clone().unwrap_or_default().options,
            })
            .collect(),
    })
}

fn to_object_type_definition(
    name: &str,
    type_of: &config::Type,
    config_module: &ConfigModule,
) -> Valid<Definition, BlueprintError> {
    to_fields(name, type_of, config_module).map(|fields| {
        Definition::Object(ObjectTypeDefinition {
            name: name.to_string(),
            description: type_of.doc.clone(),
            fields,
            implements: type_of.implements.clone(),
            directives: to_directives(&type_of.directives),
        })
    })
}

fn update_args<'a>() -> TryFold<
    'a,
    (&'a ConfigModule, &'a Field, &'a config::Type, &'a str),
    FieldDefinition,
    BlueprintError,
> {
    TryFold::<(&ConfigModule, &Field, &config::Type, &str), FieldDefinition, BlueprintError>::new(
        move |(_, field, _typ, name), _| {
            // TODO: assert type name
            Valid::from_iter(field.args.iter(), |(name, arg)| {
                Valid::succeed(InputFieldDefinition {
                    name: name.clone(),
                    description: arg.doc.clone(),
                    of_type: arg.type_of.clone(),
                    default_value: arg.default_value.clone(),
                })
            })
            .map(|args| FieldDefinition {
                name: name.to_string(),
                description: field.doc.clone(),
                args,
                of_type: field.type_of.clone(),
                directives: to_directives(&field.directives),
                resolver: None,
                default_value: field.default_value.clone(),
            })
        },
    )
}

fn item_is_numeric(list: &[String]) -> bool {
    list.iter().any(|s| {
        let re = Regex::new(r"^\d+$").unwrap();
        re.is_match(s)
    })
}

fn update_resolver_from_path(
    context: &ProcessPathContext,
    base_field: blueprint::FieldDefinition,
) -> Valid<blueprint::FieldDefinition, BlueprintError> {
    let has_index = item_is_numeric(context.path);

    process_path(context.clone()).and_then(|of_type| {
        let mut updated_base_field = base_field;
        let resolver = IR::ContextPath(context.path.to_owned());
        if has_index {
            updated_base_field.of_type =
                Type::Named { name: of_type.name().to_string(), non_null: false }
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
pub fn fix_dangling_resolvers<'a>() -> TryFold<
    'a,
    (&'a ConfigModule, &'a Field, &'a config::Type, &'a str),
    FieldDefinition,
    BlueprintError,
> {
    TryFold::<(&ConfigModule, &Field, &config::Type, &str), FieldDefinition, BlueprintError>::new(
        move |(config, field, _, name), mut b_field| {
            let mut set = HashSet::new();
            if !field.has_resolver()
                && validate_field_has_resolver(name, field, &config.types, &mut set).is_succeed()
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
pub fn update_cache_resolvers<'a>() -> TryFold<
    'a,
    (&'a ConfigModule, &'a Field, &'a config::Type, &'a str),
    FieldDefinition,
    BlueprintError,
> {
    TryFold::<(&ConfigModule, &Field, &config::Type, &str), FieldDefinition, BlueprintError>::new(
        move |(_config, field, typ, _name), mut b_field| {
            if let Some(config::Cache { max_age }) = field.cache.as_ref().or(typ.cache.as_ref()) {
                b_field.map_expr(|expression| Cache::wrap(*max_age, expression))
            }

            Valid::succeed(b_field)
        },
    )
}

fn validate_field_type_exist(config: &Config, field: &Field) -> Valid<(), BlueprintError> {
    let field_type = field.type_of.name();
    if !scalar::Scalar::is_predefined(field_type) && !config.contains(field_type) {
        Valid::fail(BlueprintError::UndeclaredTypeFound(field_type.clone()))
    } else {
        Valid::succeed(())
    }
}

fn to_fields(
    object_name: &str,
    type_of: &config::Type,
    config_module: &ConfigModule,
) -> Valid<Vec<FieldDefinition>, BlueprintError> {
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

    // collect the parent auth ids
    let parent_auth_ids = type_of.protected.as_ref().and_then(|p| p.id.as_ref());
    // collect the field names that have different auth ids than the parent type
    let fields_with_different_auth_ids = type_of
        .fields
        .iter()
        .filter_map(|(k, v)| {
            if let Some(p) = &v.protected {
                if p.id.as_ref() != parent_auth_ids {
                    Some(k)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<HashSet<_>>();

    let fields = Valid::from_iter(
        type_of
            .fields
            .iter()
            .filter(|(_, field)| !field.is_omitted()),
        |(name, field)| {
            let mut result =
                validate_field_type_exist(config_module, field).and(to_field_definition(
                    field,
                    &operation_type,
                    object_name,
                    config_module,
                    type_of,
                    name,
                ));

            if fields_with_different_auth_ids.contains(name) || parent_auth_ids.is_none() {
                // if the field has a different auth id than the parent type or parent has no
                // auth id, we need to add correct trace.
                result = result.trace(name);
            }

            result
        },
    );

    let to_added_field = |add_field: &config::AddField,
                          type_of: &config::Type|
     -> Valid<blueprint::FieldDefinition, BlueprintError> {
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
                let added_field_path = if source_field.resolvers.is_empty() {
                    add_field.path.clone()
                } else {
                    add_field.path[1..]
                        .iter()
                        .map(|s| s.to_owned())
                        .collect::<Vec<_>>()
                };
                let invalid_path_handler = |field_name: &str,
                                            _added_field_path: &[String],
                                            original_path: &[String]|
                 -> Valid<Type, BlueprintError> {
                    Valid::fail_with(
                        BlueprintError::CannotAddField,
                        BlueprintError::PathDoesNotExist(original_path.join(", ")),
                    )
                    .trace(field_name)
                };
                let path_resolver_error_handler = |resolver_name: &str,
                                                   field_type: &str,
                                                   field_name: &str,
                                                   original_path: &[String]|
                 -> Valid<Type, BlueprintError> {
                    Valid::<Type, BlueprintError>::fail_with(
                        BlueprintError::CannotAddField,
                        BlueprintError::PathContainsResolver(
                            original_path.join(", "),
                            resolver_name.to_string(),
                            field_type.to_string(),
                            field_name.to_string(),
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
            None => Valid::fail(BlueprintError::FieldNotFoundInPath(
                add_field.path[0].clone(),
                add_field.path.join(","),
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
    name: &str,
) -> Valid<FieldDefinition, BlueprintError> {
    update_args()
        .and(update_resolver(operation_type, object_name))
        .and(update_modify().trace(config::Modify::trace_name().as_str()))
        .and(fix_dangling_resolvers())
        .and(update_cache_resolvers())
        .and(update_protected(object_name).trace(Protected::trace_name().as_str()))
        .and(update_enum_alias())
        .and(update_union_resolver())
        .and(update_interface_resolver())
        .try_fold(
            &(config_module, field, type_of, name),
            FieldDefinition::default(),
        )
}

pub fn to_definitions<'a>() -> TryFold<'a, ConfigModule, Vec<Definition>, BlueprintError> {
    TryFold::<ConfigModule, Vec<Definition>, BlueprintError>::new(|config_module, _| {
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
                            } else if config_module.interfaces_types_map().contains_key(name) {
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
                    Valid::fail(BlueprintError::NoVariantsFoundForEnum)
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

fn to_directives(directives: &[config::Directive]) -> Vec<Directive> {
    directives.iter().cloned().map(Directive::from).collect()
}
