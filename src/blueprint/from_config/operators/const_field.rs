use async_graphql_value::ConstValue;

use crate::blueprint::*;
use crate::config;
use crate::config::{Config, Field};
use crate::lambda::Expression;
use crate::lambda::Expression::Literal;
use crate::try_fold::TryFold;
use crate::valid::Valid;

fn validate_data_with_schema(
  config: &config::Config,
  field: &config::Field,
  gql_value: ConstValue,
) -> Valid<(), String> {
  match to_json_schema_for_field(field, config).validate(&gql_value).to_result() {
    Ok(_) => Valid::succeed(()),
    Err(err) => Valid::from_validation_err(err.transform(&(|a| a.to_owned()))),
  }
}

pub struct CompileConst<'a> {
  pub config: &'a config::Config,
  pub field: &'a config::Field,
  pub value: &'a serde_json::Value,
  pub validate: bool,
}

pub fn compile_const(inputs: CompileConst) -> Valid<Expression, String> {
  let config = inputs.config;
  let field = inputs.field;
  let value = inputs.value;
  let validate = inputs.validate;

  let data = value.to_owned();
  match ConstValue::from_json(data.to_owned()) {
    Ok(gql) => {
      let validation = if validate {
        validate_data_with_schema(config, field, gql)
      } else {
        Valid::succeed(())
      };
      validation.map(|_| Literal(data))
    }
    Err(e) => Valid::fail(format!("invalid JSON: {}", e)),
  }
}

pub fn update_const_field<'a>(
) -> TryFold<'a, (&'a Config, &'a Field, &'a config::Type, &'a str), FieldDefinition, String> {
  TryFold::<(&Config, &Field, &config::Type, &str), FieldDefinition, String>::new(|(config, field, _, _), b_field| {
    let Some(const_field) = &field.const_field else {
      return Valid::succeed(b_field);
    };

    compile_const(CompileConst { config, field, value: &const_field.data, validate: true })
      .map(|resolver| b_field.resolver(Some(resolver)))
  })
}
