use async_graphql_value::ConstValue;
use indexmap::IndexMap;

use crate::blueprint::*;
use crate::config;
use crate::config::Field;
use crate::lambda::Expression;
use crate::lambda::Expression::Literal;
use crate::mustache::Mustache;
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

pub struct CompileConst<'a> {
    pub config_module: &'a config::ConfigModule,
    pub field: &'a config::Field,
    pub value: &'a DynamicValue,
    pub validate: bool,
}

#[derive(Debug, Clone)]
pub enum MustacheOrValue {
    Mustache(Mustache),
    Value(serde_json::Value),
}

impl MustacheOrValue {
    pub fn is_const(&self) -> bool {
        match self {
            MustacheOrValue::Mustache(m) => m.is_const(),
            _ => true,
        }
    }
}

pub fn compile_const(inputs: CompileConst) -> Valid<Expression, String> {
    let config_module = inputs.config_module;
    let field = inputs.field;
    let value = inputs.value;
    let validate = inputs.validate;

    let data = value;
    match data {
        DynamicValue::Value(v) => match ConstValue::from_json(v.to_owned().to_owned()) {
            Ok(gql) => {
                let validation = if validate {
                    validate_data_with_schema(config_module, field, gql)
                } else {
                    Valid::succeed(())
                };
                validation.map(|_| Literal(data.to_owned()))
            }
            Err(e) => Valid::fail(format!("invalid JSON: {}", e)),
        },
        DynamicValue::MustacheObject(map) => {
            let a = map.into_iter().filter(|(_, v)| !v.is_const()).count();
            if a > 0 {
                Valid::succeed(Literal(data.to_owned()))
            } else {
                let mut out = IndexMap::new();
                for (k, v) in map {
                    match v {
                        MustacheOrValue::Mustache(_) => {
                            unimplemented!("mustache in const object")
                        }
                        MustacheOrValue::Value(v) => {
                            out.insert(k.clone(), ConstValue::from_json(v.to_owned()).unwrap());
                        }
                    }
                }
                let obj = ConstValue::Object(out);
                let validation = if validate {
                    validate_data_with_schema(config_module, field, obj)
                } else {
                    Valid::succeed(())
                };
                validation.map(|_| Literal(data.to_owned()))
            }
        }
        _ => Valid::succeed(Literal(data.to_owned())),
    }
}

pub fn update_const_field<'a>(
) -> TryFold<'a, (&'a ConfigModule, &'a Field, &'a config::Type, &'a str), FieldDefinition, String>
{
    TryFold::<(&ConfigModule, &Field, &config::Type, &str), FieldDefinition, String>::new(
        |(config_module, field, _, _), b_field| {
            let Some(const_field) = &field.const_field else {
                return Valid::succeed(b_field);
            };

            Valid::from(
                DynamicValue::try_from(&const_field.data.clone())
                    .map_err(|e| ValidationError::new(e.to_string())),
            )
            .and_then(|value| {
                compile_const(CompileConst { config_module, field, value: &value, validate: true })
                    .map(|resolver| b_field.resolver(Some(resolver)))
            })
        },
    )
}
