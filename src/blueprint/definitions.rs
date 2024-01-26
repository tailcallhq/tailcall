use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeSet;
use std::hash::Hash;

use regex::Regex;

use crate::blueprint::Type::ListType;
use crate::blueprint::*;
use crate::config;
use crate::config::{Config, Field, GraphQLOperationType, Union};
use crate::directive::DirectiveCodec;
use crate::lambda::{Expression, Lambda};
use crate::try_fold::TryFold;
use crate::valid::Valid;

pub fn to_scalar_type_definition(name: &str) -> Valid<Definition, String> {
    Valid::succeed(Definition::ScalarTypeDefinition(ScalarTypeDefinition {
        name: name.to_string(),
        directive: Vec::new(),
        description: None,
    }))
}

pub fn to_union_type_definition((name, u): (&String, &Union)) -> Definition {
    Definition::UnionTypeDefinition(UnionTypeDefinition {
        name: name.to_owned(),
        description: u.doc.clone(),
        directives: Vec::new(),
        types: u.types.clone(),
    })
}

pub fn to_input_object_type_definition(
    definition: ObjectTypeDefinition,
) -> Valid<Definition, String> {
    Valid::succeed(Definition::InputObjectTypeDefinition(
        InputObjectTypeDefinition {
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
        },
    ))
}

pub fn to_interface_type_definition(definition: ObjectTypeDefinition) -> Valid<Definition, String> {
    Valid::succeed(Definition::InterfaceTypeDefinition(
        InterfaceTypeDefinition {
            name: definition.name,
            fields: definition.fields,
            description: definition.description,
        },
    ))
}

type InvalidPathHandler = dyn Fn(&str, &[String], &[String]) -> Valid<Type, String>;
type PathResolverErrorHandler = dyn Fn(&str, &str, &str, &[String]) -> Valid<Type, String>;
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
            let next_dir_http = next_field
                .http
                .as_ref()
                .map(|_| config::Http::directive_name());
            let next_dir_const = next_field
                .const_field
                .as_ref()
                .map(|_| config::Const::directive_name());
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

fn to_enum_type_definition(
    name: &str,
    type_: &config::Type,
    variants: &BTreeSet<String>,
) -> Valid<Definition, String> {
    let enum_type_definition = Definition::EnumTypeDefinition(EnumTypeDefinition {
        name: name.to_string(),
        directives: Vec::new(),
        description: type_.doc.clone(),
        enum_values: variants
            .iter()
            .map(|variant| EnumValueDefinition {
                description: None,
                name: variant.clone(),
                directives: Vec::new(),
            })
            .collect(),
    });
    Valid::succeed(enum_type_definition)
}

fn to_object_type_definition(
    name: &str,
    type_of: &config::Type,
    config: &Config,
) -> Valid<Definition, String> {
    to_fields(name, type_of, config).map(|fields| {
        Definition::ObjectTypeDefinition(ObjectTypeDefinition {
            name: name.to_string(),
            description: type_of.doc.clone(),
            fields,
            implements: type_of.implements.clone(),
        })
    })
}

fn update_args<'a>(
    hasher: DefaultHasher,
) -> TryFold<'a, (&'a Config, &'a Field, &'a config::Type, &'a str), FieldDefinition, String> {
    TryFold::<(&Config, &Field, &config::Type, &str), FieldDefinition, String>::new(
        move |(_, field, _, name), _| {
            let mut hasher = hasher.clone();
            name.hash(&mut hasher);
            let cache = field
                .cache
                .as_ref()
                .map(|config::Cache { max_age }| Cache { max_age: *max_age, hasher });

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
                cache,
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
        let resolver = Lambda::context_path(context.path.to_owned());
        if has_index {
            updated_base_field.of_type =
                Type::NamedType { name: of_type.name().to_string(), non_null: false }
        } else {
            updated_base_field.of_type = of_type;
        }

        updated_base_field = updated_base_field
            .resolver_or_default(resolver, |r| r.to_input_path(context.path.to_owned()));
        Valid::succeed(updated_base_field)
    })
}

/// Sets empty resolver to fields that has
/// nested resolvers for its fields.
/// To solve the problem that by default such fields will be resolved to null value
/// and nested resolvers won't be called
pub fn update_nested_resolvers<'a>(
) -> TryFold<'a, (&'a Config, &'a Field, &'a config::Type, &'a str), FieldDefinition, String> {
    TryFold::<(&Config, &Field, &config::Type, &str), FieldDefinition, String>::new(
        move |(config, field, _, name), mut b_field| {
            if !field.has_resolver()
                && validate_field_has_resolver(name, field, &config.types).is_succeed()
            {
                b_field = b_field.resolver(Some(Expression::Literal(serde_json::Value::Object(
                    Default::default(),
                ))));
            }

            Valid::succeed(b_field)
        },
    )
}

fn validate_field_type_exist(config: &Config, field: &Field) -> Valid<(), String> {
    let field_type = &field.type_of;
    if !is_scalar(field_type) && !config.contains(field_type) {
        Valid::fail(format!("Undeclared type '{field_type}' was found"))
    } else {
        Valid::succeed(())
    }
}

fn to_fields(
    object_name: &str,
    type_of: &config::Type,
    config: &Config,
) -> Valid<Vec<FieldDefinition>, String> {
    let operation_type = if config.schema.mutation.as_deref().eq(&Some(object_name)) {
        GraphQLOperationType::Mutation
    } else {
        GraphQLOperationType::Query
    };

    let to_field = move |name: &String, field: &Field| {
        let directives = field.resolvable_directives();

        if directives.len() > 1 {
            return Valid::fail(format!(
                "Multiple resolvers detected [{}]",
                directives.join(", ")
            ));
        }

        let mut hasher = DefaultHasher::new();
        object_name.hash(&mut hasher);

        update_args(hasher)
            .and(update_http().trace(config::Http::trace_name().as_str()))
            .and(update_grpc(&operation_type).trace(config::Grpc::trace_name().as_str()))
            .and(update_js().trace(config::JS::trace_name().as_str()))
            .and(update_const_field().trace(config::Const::trace_name().as_str()))
            .and(update_graphql(&operation_type).trace(config::GraphQL::trace_name().as_str()))
            .and(update_expr(&operation_type).trace(config::Expr::trace_name().as_str()))
            .and(update_modify().trace(config::Modify::trace_name().as_str()))
            .and(update_nested_resolvers())
            .try_fold(&(config, field, type_of, name), FieldDefinition::default())
    };

    // Process fields that are not marked as `omit`
    let fields = Valid::from_iter(
        type_of
            .fields
            .iter()
            .filter(|(_, field)| !field.is_omitted()),
        |(name, field)| {
            validate_field_type_exist(config, field)
                .and(to_field(name, field))
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
            Some((_, source_field)) => to_field(&add_field.name, source_field)
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
                            config,
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

pub fn to_definitions<'a>() -> TryFold<'a, Config, Vec<Definition>, String> {
    TryFold::<Config, Vec<Definition>, String>::new(|config, _| {
        let output_types = config.output_types();
        let input_types = config.input_types();
        Valid::from_iter(config.types.iter(), |(name, type_)| {
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
            types.extend(config.unions.iter().map(to_union_type_definition));
            types
        })
    })
}
