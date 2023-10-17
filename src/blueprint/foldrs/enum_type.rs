use crate::blueprint::transform::Transform;
use crate::blueprint::{Blueprint, Definition, EnumTypeDefinition, EnumValueDefinition};
use crate::config;
use crate::config::Config;
use crate::try_fold::TryFolding;
use crate::valid::{Valid, ValidExtensions};

pub struct EnumFold {
  pub name: String,
  pub type_of: config::Type,
}

impl From<EnumFold> for Transform<Config, Blueprint, String> {
  fn from(value: EnumFold) -> Self {
    let name = value.name.clone();
    Transform::new(move |config, blueprint| value.transform(config, blueprint).trace(name.as_str()))
  }
}

impl TryFolding for EnumFold {
  type Input = Config;
  type Value = Blueprint;
  type Error = String;

  fn try_fold(self, _cfg: &Self::Input, mut blueprint: Self::Value) -> Valid<Self::Value, Self::Error> {
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
