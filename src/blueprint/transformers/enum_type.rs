use crate::blueprint::transform::Transform;
use crate::blueprint::transformers::Valid;
use crate::blueprint::{Blueprint, Definition, EnumTypeDefinition, EnumValueDefinition};
use crate::config;
use crate::config::Config;
use crate::valid::ValidExtensions;

pub struct EnumTransform {
  pub name: String,
  pub type_of: config::Type,
}

impl From<EnumTransform> for Transform<Config, Blueprint, String> {
  fn from(value: EnumTransform) -> Self {
    let name = value.name.clone();
    Transform::new(move |config, blueprint| value.transform(config, blueprint).trace(name.as_str()))
  }
}

impl EnumTransform {
  fn transform(self, _config: &Config, mut blueprint: Blueprint) -> Valid<Blueprint> {
    let Some(variants) = self.type_of.variants else {
      return Valid::fail(format!("No variants in {}", self.name));
    };
    let enum_type_definition = Definition::EnumTypeDefinition(EnumTypeDefinition {
      name: self.name.clone(),
      directives: Vec::new(),
      description: self.type_of.doc.clone(),
      enum_values: variants
        .iter()
        .map(|variant| EnumValueDefinition { description: None, name: variant.clone(), directives: Vec::new() })
        .collect(),
    });

    blueprint.definitions.push(enum_type_definition);

    Ok(blueprint)
  }
}
