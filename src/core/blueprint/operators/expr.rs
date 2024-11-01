use async_graphql_value::ConstValue;

use crate::core::blueprint::*;
use crate::core::config;
use crate::core::config::{Expr, Field, Resolver};
use crate::core::ir::model::IR;
use crate::core::ir::model::IR::Dynamic;
use crate::core::try_fold::TryFold;
use crate::core::valid::{Valid, Validator};

fn validate_data_with_schema(
    config: &config::Config,
    field: &config::Field,
    gql_value: ConstValue,
) -> Valid<(), miette::MietteDiagnostic> {
    to_json_schema(&field.type_of, config).validate(&gql_value)
}

pub struct CompileExpr<'a> {
    pub config_module: &'a config::ConfigModule,
    pub field: &'a config::Field,
    pub expr: &'a Expr,
    pub validate: bool,
}

pub fn compile_expr(inputs: CompileExpr) -> Valid<IR, miette::MietteDiagnostic> {
    let config_module = inputs.config_module;
    let field = inputs.field;
    let value = &inputs.expr.body;
    let validate = inputs.validate;

    Valid::from(DynamicValue::try_from(&value.clone()).map_err(|e| miette::diagnostic!("{}", e)))
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
                    Err(e) => Valid::fail(miette::diagnostic!("invalid JSON: {}", e)),
                }
            }
        })
}

pub fn update_const_field<'a>() -> TryFold<
    'a,
    (&'a ConfigModule, &'a Field, &'a config::Type, &'a str),
    FieldDefinition,
    miette::MietteDiagnostic,
> {
    TryFold::<(&ConfigModule, &Field, &config::Type, &str), FieldDefinition, miette::MietteDiagnostic>::new(
        |(config_module, field, _, _), b_field| {
            let Some(Resolver::Expr(expr)) = &field.resolver else {
                return Valid::succeed(b_field);
            };

            compile_expr(CompileExpr { config_module, field, expr, validate: true })
                .map(|resolver| b_field.resolver(Some(resolver)))
        },
    )
}
