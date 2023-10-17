use crate::blueprint::{from_config, FieldDefinition};
use crate::config;
use crate::config::Config;
use crate::lambda::Lambda;
use crate::try_fold::TryFolding;
use crate::valid::Valid;

pub struct UnsafeFold {
  pub field: config::Field,
}

impl TryFolding for UnsafeFold {
  type Input = Config;
  type Value = FieldDefinition;
  type Error = String;

  fn try_fold(self, _cfg: &Self::Input, mut field_definition: Self::Value) -> Valid<Self::Value, Self::Error> {
    if let Some(op) = self.field.unsafe_operation {
      field_definition = field_definition.resolver_or_default(Lambda::context().to_unsafe_js(op.script.clone()), |r| {
        r.to_unsafe_js(op.script.clone())
      });
    }
    Ok(field_definition)
  }
}
