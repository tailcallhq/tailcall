use async_graphql_value::ConstValue;
use tailcall_valid::{Valid, ValidationError, Validator};

use crate::core::blueprint::*;
use crate::core::config;
use crate::core::config::Expr;
use crate::core::ir::model::IR;
use crate::core::ir::model::IR::Dynamic;

fn validate_data_with_schema(
    config: &config::Config,
    field: &config::Field,
    gql_value: ConstValue,
) -> Valid<(), String> {
    match to_json_schema(&field.type_of, config)
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
    pub expr: &'a Expr,
    pub validate: bool,
}

pub fn compile_expr(inputs: CompileExpr) -> Valid<IR, String> {
    let config_module = inputs.config_module;
    let field = inputs.field;
    let value = &inputs.expr.body;
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
