use crate::core::{config::Config, valid::Valid};

use super::Transform;

#[derive(Default)]
pub struct RemoveUnUsedTypes {}

impl Transform for RemoveUnUsedTypes {
    fn transform(&mut self, mut config: Config) -> Valid<Config, String> {
        let unused_types = config.unused_types();
        config = config.remove_types(unused_types);
        Valid::succeed(config)
    }
}
