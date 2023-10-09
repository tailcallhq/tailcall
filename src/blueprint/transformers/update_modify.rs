use crate::blueprint::transform::Transform;
use crate::blueprint::transformers::Valid;
use crate::blueprint::FieldDefinition;
use crate::config;
use crate::config::{Config, ModifyField};
use crate::lambda::Lambda;
use crate::valid::ValidExtensions;

pub struct ModifyTransform {
  pub modify: Option<ModifyField>,
  pub type_info: config::Type,
}

impl From<ModifyTransform> for Transform<Config, FieldDefinition, String> {
  fn from(value: ModifyTransform) -> Self {
    Transform::new(move |config, field_definition| value.transform(config, field_definition).trace("@modify"))
  }
}

impl ModifyTransform {
  fn transform(self, config: &Config, mut field_def: FieldDefinition) -> Valid<FieldDefinition> {
    match self.modify.as_ref() {
      Some(modify) => {
        if let Some(new_name) = &modify.name {
          for name in self.type_info.implements.iter() {
            let interface = config.find_type(name);
            if let Some(interface) = interface {
              if interface.fields.iter().any(|(name, _)| name == new_name) {
                return Valid::fail("Field is already implemented from interface".to_string());
              }
            }
          }

          let lambda = Lambda::context_field(field_def.name.clone());
          field_def = field_def.resolver_or_default(lambda, |r| r);
          field_def = field_def.name(new_name.clone());
          Valid::Ok(field_def)
        } else {
          Valid::Ok(field_def)
        }
      }
      None => Valid::Ok(field_def),
    }
  }
}
