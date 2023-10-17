use async_graphql_value::ConstValue;

use crate::blueprint::from_config::to_json_schema_for_field;
use crate::blueprint::FieldDefinition;
use crate::config;
use crate::config::Config;
use crate::lambda::Expression::Literal;
use crate::try_fold::TryFolding;
use crate::valid::{Valid, ValidExtensions};

/// Update const
pub struct ConstFold {
  pub field: config::Field,
}

impl TryFolding for ConstFold {
  type Input = Config;
  type Value = FieldDefinition;
  type Error = String;

  fn try_fold(self, cfg: &Self::Input, mut field_definition: Self::Value) -> Valid<Self::Value, Self::Error> {
    match self.field.const_field.as_ref() {
      Some(const_field) => {
        let data = const_field.data.to_owned();
        match ConstValue::from_json(data.to_owned()) {
          Ok(gql_value) => match to_json_schema_for_field(&self.field, cfg).validate(&gql_value) {
            Ok(_) => {
              field_definition.resolver = Some(Literal(data));
              Ok(field_definition)
            }
            Err(err) => err.into(),
          },
          Err(e) => Valid::fail(format!("invalid JSON: {}", e)),
        }
      }
      None => Ok(field_definition),
    }
    .trace("@const")
  }
}
