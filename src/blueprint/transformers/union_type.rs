use crate::blueprint::transform::Transform;
use crate::blueprint::transformers::Valid;
use crate::blueprint::{Blueprint, Definition, UnionTypeDefinition};
use crate::config;
use crate::config::Config;

pub struct UnionTransform {
  pub name: String,
  pub union: config::Union,
}

impl From<UnionTransform> for Transform<Config, Blueprint, String> {
  fn from(value: UnionTransform) -> Self {
    Transform::new(move |config, blueprint| value.transform(config, blueprint))
  }
}

impl UnionTransform {
  fn transform(self, _: &Config, mut blueprint: Blueprint) -> Valid<Blueprint> {
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
