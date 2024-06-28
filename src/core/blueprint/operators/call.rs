use serde_json::Value;

use crate::core::blueprint::*;
use crate::core::config;
use crate::core::config::{Field, GraphQLOperationType};
use crate::core::ir::model::IR;
use crate::core::try_fold::TryFold;
use crate::core::valid::{Valid, ValidationError, Validator};

pub fn update_call<'a>(
    operation_type: &'a GraphQLOperationType,
    object_name: &'a str,
) -> TryFold<'a, (&'a ConfigModule, &'a Field, &'a config::Type, &'a str), FieldDefinition, String>
{
    TryFold::<(&ConfigModule, &Field, &config::Type, &str), FieldDefinition, String>::new(
        move |(config, field, _, name), b_field| {
            let Some(ref calls) = field.call else {
                return Valid::succeed(b_field);
            };

            compile_call(config, calls, operation_type, object_name).map(|field| {
                let args = field
                    .args
                    .into_iter()
                    .map(|mut arg_field| {
                        arg_field.of_type = match &arg_field.of_type {
                            Type::NamedType { name, .. } => {
                                Type::NamedType { name: name.to_owned(), non_null: false }
                            }
                            Type::ListType { of_type, .. } => {
                                Type::ListType { of_type: of_type.to_owned(), non_null: false }
                            }
                        };

                        arg_field
                    })
                    .collect();

                b_field
                    .args(args)
                    .resolver(field.resolver)
                    .name(name.to_string())
            })
        },
    )
}

fn compile_call(
    config_module: &ConfigModule,
    call: &config::Call,
    operation_type: &GraphQLOperationType,
    object_name: &str,
) -> Valid<FieldDefinition, String> {
    Valid::from_iter(call.steps.iter(), |step| {
        get_field_and_field_name(step, config_module).and_then(|(field, field_name, type_of)| {
            let args = step.args.iter();

            let empties: Vec<&String> = field
                .args
                .iter()
                .filter_map(|(k, arg)| {
                    if arg.required && !args.clone().any(|(k1, _)| k1.eq(k)) {
                        Some(k)
                    } else {
                        None
                    }
                })
                .collect();

            if empties.len().gt(&0) {
                return Valid::fail(format!(
                    "no argument {} found",
                    empties
                        .into_iter()
                        .map(|k| format!("'{}'", k))
                        .collect::<Vec<String>>()
                        .join(", ")
                ))
                .trace(field_name.as_str());
            }

            to_field_definition(
                field,
                operation_type,
                object_name,
                config_module,
                type_of,
                &field.type_of,
            )
            .and_then(|b_field| {
                if b_field.resolver.is_none() {
                    Valid::fail(format!("{} field has no resolver", field_name))
                } else {
                    Valid::succeed(b_field)
                }
            })
            .fuse(
                Valid::from(
                    DynamicValue::try_from(&Value::Object(step.args.clone().into_iter().collect()))
                        .map_err(|e| ValidationError::new(e.to_string())),
                )
                .map(IR::Dynamic),
            )
            .map(|(mut b_field, args_expr)| {
                if !step.args.is_empty() {
                    b_field.map_expr(|expr| args_expr.clone().pipe(expr));
                }

                b_field
            })
        })
    })
    .and_then(|b_fields| {
        Valid::from_option(
            b_fields.into_iter().reduce(|mut b_field, b_field_next| {
                b_field.name = b_field_next.name;
                b_field.args.extend(b_field_next.args);
                b_field.of_type = b_field_next.of_type;
                b_field.map_expr(|expr| {
                    b_field_next
                        .resolver
                        .as_ref()
                        .map(|other_expr| expr.clone().pipe(other_expr.clone()))
                        .unwrap_or(expr)
                });

                b_field
            }),
            "Steps can't be empty".to_string(),
        )
    })
}

fn get_type_and_field(call: &config::Step) -> Option<(String, String)> {
    if let Some(query) = &call.query {
        Some(("Query".to_string(), query.clone()))
    } else {
        call.mutation
            .as_ref()
            .map(|mutation| ("Mutation".to_string(), mutation.clone()))
    }
}

fn get_field_and_field_name<'a>(
    call: &'a config::Step,
    config_module: &'a ConfigModule,
) -> Valid<(&'a Field, String, &'a config::Type), String> {
    Valid::from_option(
        get_type_and_field(call),
        "call must have query or mutation".to_string(),
    )
    .and_then(|(type_name, field_name)| {
        Valid::from_option(
            config_module.config().find_type(&type_name),
            format!("{} type not found on config", type_name),
        )
        .and_then(|query_type| {
            Valid::from_option(
                query_type.fields.get(&field_name),
                format!("{} field not found", field_name),
            )
            .fuse(Valid::succeed(field_name))
            .fuse(Valid::succeed(query_type))
            .into()
        })
    })
}
