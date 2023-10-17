use crate::blueprint::{Blueprint, Definition, ScalarTypeDefinition};
use crate::config::Config;
use crate::try_fold::TryFolding;
use crate::valid::Valid;

pub struct ScalarFold {
  pub name: String,
}

impl TryFolding for ScalarFold {
  type Input = Config;
  type Value = Blueprint;
  type Error = String;

  fn try_fold(self, _cfg: &Self::Input, mut blueprint: Self::Value) -> Valid<Self::Value, Self::Error> {
    blueprint
      .definitions
      .push(Definition::ScalarTypeDefinition(ScalarTypeDefinition {
        name: self.name,
        directive: Vec::new(),
        description: None,
      }));
    Ok(blueprint)
  }
}
