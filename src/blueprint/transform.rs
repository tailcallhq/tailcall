use super::Blueprint;
use crate::{
  config::Config,
  valid::{Valid, ValidExtensions},
};

pub struct Transform {
  pub transform: Box<dyn Fn(&Config, Blueprint) -> Valid<Blueprint, &'static str>>,
}

impl Transform {
  fn new(transform: impl Fn(&Config, Blueprint) -> Valid<Blueprint, &'static str>) -> Self {
    Self { transform: Box::new(transform) }
  }

  fn and_then(self, other: Self) -> Self {
    Self::new(move |config, blueprint| {
      (self.transform)(config, blueprint).fold(
        |blueprint| (other.transform)(config, blueprint),
        other.transform(config, blueprint),
      )
    })
  }
}
