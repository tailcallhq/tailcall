use super::Blueprint;
use crate::valid::Valid;

pub trait Transform {
  fn transform(blueprint: Blueprint) -> Valid<Blueprint, &'static str>;
}
