pub mod query_generator;
pub mod schema_generator;
pub mod types_generator;
pub mod upstream_generator;

use crate::core::config::Config;
use crate::core::merge_right::MergeRight;

pub trait ConfigGenerator {
    fn apply(&mut self, config: Config) -> Config;
}

#[derive(Default)]
pub struct StepConfigGenerator {
    config: Config,
}

impl StepConfigGenerator {
    pub fn pipe(mut self, mut other: impl ConfigGenerator) -> Self {
        let update_config = other.apply(self.config.clone());
        self.config = self.config.merge_right(update_config);
        self
    }

    pub fn get(self) -> Config {
        self.config
    }
}
