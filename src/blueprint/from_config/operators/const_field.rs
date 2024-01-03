use async_graphql_value::ConstValue;

use crate::blueprint::*;
use crate::config;
use crate::config::{Config, Field};
use crate::lambda::Expression;
use crate::lambda::Expression::Literal;
use crate::try_fold::TryFold;
use crate::valid::Valid;

pub fn compile_const(
  config: &config::Config,
  field: &config::Field,
  const_field: &config::Const,
) -> Valid<Expression, String> {
  let data = const_field.data.to_owned();
  match ConstValue::from_json(data.to_owned()) {
    Ok(gql_value) => match to_json_schema_for_field(field, config).validate(&gql_value).to_result() {
      Ok(_) => Valid::succeed(Literal(data)),
      Err(err) => Valid::from_validation_err(err.transform(&|a| a.to_owned())),
    },
    Err(e) => Valid::fail(format!("invalid JSON: {}", e)),
  }
}

pub fn update_const_field<'a>(
) -> TryFold<'a, (&'a Config, &'a Field, &'a config::Type, &'a str), FieldDefinition, String> {
  TryFold::<(&Config, &Field, &config::Type, &str), FieldDefinition, String>::new(|(config, field, _, _), b_field| {
    let Some(const_field) = &field.const_field else {
      return Valid::succeed(b_field);
    };

    compile_const(config, field, const_field).map(|resolver| b_field.resolver(Some(resolver)))
  })
}
