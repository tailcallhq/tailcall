mod type_merger;

pub use type_merger::TypeMerger;

use crate::core::config::Config;

pub trait Transform {
    fn apply(&mut self, config: Config) -> Config;
}
