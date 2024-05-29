pub mod query_generator;
pub mod schema_generator;
pub mod types_generator;
pub mod upstream_generator;

use crate::core::{config::Config, merge_right::MergeRight};

pub trait ConfigGenerator {
    fn apply(&mut self, config: Config) -> Config;
}

pub struct StepConfigGenerator {
    config: Config,
}

impl Default for StepConfigGenerator {
    fn default() -> Self {
        Self { config: Default::default() }
    }
}

impl StepConfigGenerator {
    pub fn pipe(mut self, mut other: impl ConfigGenerator) -> Self {
        let update_config = other.apply(self.config.clone());
        self.config = self.config.merge_right(update_config);
        self
    }

    pub fn generate(self) -> Config {
        self.config
    }
}
