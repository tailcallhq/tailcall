use std::fmt::Display;

use crate::blueprint::Blueprint;
use crate::config::Config;
use crate::valid::ValidationError;

pub mod javascript_plugin;
trait Plugin<'a, E: Display> {
  fn run(config: &'a Config, blueprint: Blueprint) -> Result<Blueprint, ValidationError<E>>;
}
