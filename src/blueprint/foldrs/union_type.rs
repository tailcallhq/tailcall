use crate::blueprint::{Blueprint, Definition, UnionTypeDefinition};
use crate::config;
use crate::config::Config;
use crate::try_fold::TryFolding;
use crate::valid::Valid;

pub struct UnionTransFold {
  pub name: String,
  pub union: config::Union,
}

impl TryFolding for UnionTransFold {
  type Input = Config;
  type Value = Blueprint;
  type Error = String;

  fn try_fold(self, _cfg: &Self::Input, mut blueprint: Self::Value) -> Valid<Self::Value, Self::Error> {
    blueprint
      .definitions
      .push(Definition::UnionTypeDefinition(UnionTypeDefinition {
        name: self.name.to_owned(),
        description: self.union.doc.clone(),
        directives: Vec::new(),
        types: self.union.types.clone(),
      }));
    Ok(blueprint)
  }
}
