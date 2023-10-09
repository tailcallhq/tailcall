use crate::blueprint::transform::Transform;
use crate::blueprint::transformers::Valid;
use crate::blueprint::{Blueprint, Definition, ScalarTypeDefinition};
use crate::config::Config;
use crate::valid::ValidExtensions;

pub struct ScalarTransform {
  pub name: String,
}

impl From<ScalarTransform> for Transform<Config, Blueprint, String> {
  fn from(value: ScalarTransform) -> Self {
    let name = value.name.clone();
    Transform::new(move |config, blueprint| value.transform(config, blueprint).trace(name.as_str()))
  }
}

impl ScalarTransform {
  fn transform(self, _: &Config, mut blueprint: Blueprint) -> Valid<Blueprint> {
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
