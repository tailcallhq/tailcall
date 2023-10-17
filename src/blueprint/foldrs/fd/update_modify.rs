use crate::blueprint::transform::Transform;
use crate::blueprint::FieldDefinition;
use crate::config;
use crate::config::{Config, ModifyField};
use crate::lambda::Lambda;
use crate::try_fold::TryFolding;
use crate::valid::{Valid, ValidExtensions};

pub struct ModifyFold {
  pub modify: Option<ModifyField>,
  pub type_info: config::Type,
}

impl TryFolding for ModifyFold {
  type Input = Config;
  type Value = FieldDefinition;
  type Error = String;

  fn try_fold(self, cfg: &Self::Input, mut field_definition: Self::Value) -> Valid<Self::Value, Self::Error> {
    match self.modify.as_ref() {
      Some(modify) => {
        if let Some(new_name) = &modify.name {
          for name in self.type_info.implements.iter() {
            let interface = cfg.find_type(name);
            if let Some(interface) = interface {
              if interface.fields.iter().any(|(name, _)| name == new_name) {
                return Valid::fail("Field is already implemented from interface".to_string());
              }
            }
          }

          let lambda = Lambda::context_field(field_definition.name.clone());
          field_definition = field_definition.resolver_or_default(lambda, |r| r);
          field_definition = field_definition.name(new_name.clone());
          Ok(field_definition)
        } else {
          Ok(field_definition)
        }
      }
      None => Valid::Ok(field_definition),
    }
    .trace("@modify")
  }
}
