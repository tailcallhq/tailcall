use std::collections::HashMap;

use async_graphql_value::ConstValue;

use crate::blueprint::*;
use crate::config;
use crate::config::{Config, Field};
use crate::json::JsonSchema;
use crate::lambda::Expression;
use crate::lambda::Expression::Dynamic;
use crate::mustache::{Mustache, Segment};
use crate::try_fold::TryFold;
use crate::valid::{Valid, ValidationError, Validator};

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

pub fn compile_expr(inputs: CompileExpr) -> Valid<Expression, String> {
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
            json_schema_for_dynamic_value(&value, inputs)
                .and_then(|_| Valid::succeed(Dynamic(value.to_owned())))
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

fn path_schema_for_parts(parts: &[String], config: &Config, field: &Field) -> JsonSchema {
    let args_schema = to_json_schema_for_args(&field.args, config);
    let out_schema = to_json_schema_for_field(field, config);
    if let Some((head, tail)) = parts.split_first() {
        if head == "args" {
            args_schema.path(tail).unwrap().clone()
        } else if head == "value" {
            out_schema.path(tail).unwrap().clone()
        } else {
            JsonSchema::Str
        }
    } else {
        JsonSchema::Str
    }
}
fn to_json_schema_for_mustache(mustache: &Mustache, config: &Config, field: &Field) -> JsonSchema {
    match mustache {
        Mustache(segments) => {
            if segments.len() == 1 {
                match segments.iter().as_ref().first().unwrap() {
                    Segment::Literal(_text) => JsonSchema::Str,
                    Segment::Expression(parts) => {
                        path_schema_for_parts(parts.as_slice(), config, field)
                    }
                }
            } else {
                JsonSchema::Str
            }
        }
    }
}

fn to_json_schema(value: &DynamicValue, config: &config::Config, field: &Field) -> JsonSchema {
    let s = match value {
        DynamicValue::Value(v) => JsonSchema::from(v.clone()),
        DynamicValue::Mustache(m) => to_json_schema_for_mustache(m, config, field),
        DynamicValue::Object(obj) => {
            let mut schema_fields = HashMap::new();
            for (name, value) in obj.iter() {
                schema_fields.insert(
                    name.as_str().to_string(),
                    to_json_schema(value, config, field),
                );
            }
            JsonSchema::Obj(schema_fields)
        }
        DynamicValue::Array(arr) => JsonSchema::Arr(Box::new(to_json_schema(
            arr.first().unwrap(),
            config,
            field,
        ))),
    };
    if field.list {
        JsonSchema::Arr(Box::new(s))
    } else if !field.non_null() && !s.is_optional() {
        JsonSchema::Opt(Box::new(s))
    } else {
        s
    }
}
fn json_schema_for_dynamic_value(
    value: &DynamicValue,
    compile_expr: CompileExpr,
) -> Valid<(), String> {
    let out_schema =
        to_json_schema_for_field(compile_expr.field, &compile_expr.config_module.config);

    let json_schema = to_json_schema(
        value,
        &compile_expr.config_module.config,
        compile_expr.field,
    );
    println!("json_schema: {:?}", json_schema);
    println!("out_schema: {:?}", out_schema);
    out_schema.compare(&json_schema, compile_expr.field.name())
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
