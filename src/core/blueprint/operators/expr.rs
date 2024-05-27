use async_graphql_value::ConstValue;

use crate::core::blueprint::*;
use crate::core::config;
use crate::core::config::Field;
use crate::core::ir::IR;
use crate::core::ir::IR::Dynamic;
use crate::core::try_fold::TryFold;
use crate::core::valid::{Valid, ValidationError, Validator};

fn validate_data_with_schema(
    config: &config::Config,
    field: &config::Field,
    gql_value: ConstValue,
) -> Valid<(), String> {
    match to_json_schema_for_field(field, config)
        .validate(&gql_value)
        .to_result()
    {
        Ok(_) => Valid::succeed(()),
        Err(err) => Valid::from_validation_err(err.transform(&(|a| a.to_owned()))),
    }
}

pub struct CompileExpr<'a> {
    pub config_module: &'a config::ConfigModule,
    pub field: &'a config::Field,
    pub value: &'a serde_json::Value,
    pub validate: bool,
}

pub fn compile_expr(inputs: CompileExpr) -> Valid<IR, String> {
    let config_module = inputs.config_module;
    let field = inputs.field;
    let value = inputs.value;
    let validate = inputs.validate;

    Valid::from(
        DynamicValue::try_from(&value.clone()).map_err(|e| ValidationError::new(e.to_string())),
    )
    .and_then(|value| {
        if !value.is_const() {
            // TODO: Add validation for const with Mustache here
            Valid::succeed(Dynamic(value.to_owned()))
        } else {
            let data = &value;
            match data.try_into() {
                Ok(gql) => {
                    let validation = if validate {
                        validate_data_with_schema(config_module, field, gql)
                    } else {
                        Valid::succeed(())
                    };
                    validation.map(|_| Dynamic(value.to_owned()))
                }
                Err(e) => Valid::fail(format!("invalid JSON: {}", e)),
            }
        }
    })
}

pub fn update_const_field<'a>(
) -> TryFold<'a, (&'a ConfigModule, &'a Field, &'a config::Type, &'a str), FieldDefinition, String>
{
    TryFold::<(&ConfigModule, &Field, &config::Type, &str), FieldDefinition, String>::new(
        |(config_module, field, _, _), b_field| {
            let Some(const_field) = &field.const_field else {
                return Valid::succeed(b_field);
            };

            compile_expr(CompileExpr {
                config_module,
                field,
                value: &const_field.body,
                validate: true,
            })
            .map(|resolver| b_field.resolver(Some(resolver)))
        },
    )
}
