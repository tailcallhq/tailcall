use serde_json::Value;
use tailcall_valid::{Valid, Validator};

use crate::core::blueprint::*;
use crate::core::config;
use crate::core::config::{Field, GraphQLOperationType};
use crate::core::ir::model::IR;

pub fn compile_call(
    config_module: &ConfigModule,
    call: &config::Call,
    operation_type: &GraphQLOperationType,
    object_name: &str,
) -> Valid<IR, BlueprintError> {
    Valid::from_iter(call.steps.iter(), |step| {
        get_field_and_field_name(step, config_module).and_then(|(field, field_name, type_of)| {
            let args = step.args.iter();

            let empties: Vec<&String> = field
                .args
                .iter()
                .filter_map(|(k, arg)| {
                    if !arg.type_of.is_nullable() && !args.clone().any(|(k1, _)| k1.eq(k)) {
                        Some(k)
                    } else {
                        None
                    }
                })
                .collect();

            if empties.len().gt(&0) {
                return Valid::fail(BlueprintError::ArgumentNotFound(
                    empties
                        .into_iter()
                        .map(|k| format!("'{}'", k))
                        .collect::<Vec<String>>()
                        .join(", "),
                ))
                .trace(field_name.as_str());
            }

            to_field_definition(
                field,
                operation_type,
                object_name,
                config_module,
                type_of,
                field.type_of.name(),
            )
            .and_then(|b_field| {
                if b_field.resolver.is_none() {
                    Valid::fail(BlueprintError::FieldHasNoResolver(field_name.clone()))
                } else {
                    Valid::succeed(b_field)
                }
            })
            .fuse(
                match DynamicValue::try_from(&Value::Object(
                    step.args.clone().into_iter().collect(),
                )) {
                    Ok(value) => Valid::succeed(value),
                    Err(e) => Valid::fail(BlueprintError::Error(e)),
                }
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
                b_field.map_expr(|expr| {
                    b_field_next
                        .resolver
                        .as_ref()
                        .map(|other_expr| expr.clone().pipe(other_expr.clone()))
                        .unwrap_or(expr)
                });

                b_field
            }),
            BlueprintError::StepsCanNotBeEmpty,
        )
    })
    .and_then(|field| {
        Valid::from_option(field.resolver, BlueprintError::ResultResolverCanNotBeEmpty)
    })
}

fn get_type_and_field(call: &config::Step) -> Option<(String, String)> {
    // TODO: type names for query and mutations should be inferred from the
    // config_module and should not be static values
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
) -> Valid<(&'a Field, String, &'a config::Type), BlueprintError> {
    Valid::from_option(
        get_type_and_field(call),
        BlueprintError::CallMustHaveQueryOrMutation,
    )
    .and_then(|(type_name, field_name)| {
        Valid::from_option(
            config_module.config().find_type(&type_name),
            BlueprintError::TypeNotFoundInConfig(type_name.clone()),
        )
        .and_then(|query_type| {
            Valid::from_option(
                query_type.fields.get(&field_name),
                BlueprintError::FieldNotFoundInType(field_name.clone()),
            )
            .fuse(Valid::succeed(field_name))
            .fuse(Valid::succeed(query_type))
            .into()
        })
    })
}
