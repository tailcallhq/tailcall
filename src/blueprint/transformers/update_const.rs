use async_graphql_value::ConstValue;

use crate::blueprint::from_config::to_json_schema_for_field;
use crate::blueprint::transform::Transform;
use crate::blueprint::transformers::Valid;
use crate::blueprint::FieldDefinition;
use crate::config;
use crate::config::Config;
use crate::lambda::Expression::Literal;
use crate::valid::ValidExtensions;

pub struct UpdateConstTransform {
  pub field: config::Field,
}

impl From<UpdateConstTransform> for Transform<Config, FieldDefinition, String> {
  fn from(value: UpdateConstTransform) -> Self {
    Transform::new(move |config, field_definition| value.transform(config, field_definition).trace("@const"))
  }
}

impl UpdateConstTransform {
  fn transform(self, config: &Config, mut field_def: FieldDefinition) -> Valid<FieldDefinition> {
    match self.field.const_field.as_ref() {
      Some(const_field) => {
        let data = const_field.data.to_owned();
        match ConstValue::from_json(data.to_owned()) {
          Ok(gql_value) => match to_json_schema_for_field(&self.field, config).validate(&gql_value) {
            Ok(_) => {
              field_def.resolver = Some(Literal(data));
              Valid::Ok(field_def)
            }
            Err(err) => err.into(),
          },
          Err(e) => Valid::fail(format!("invalid JSON: {}", e)),
        }
      }
      None => Valid::Ok(field_def),
    }
  }
}
