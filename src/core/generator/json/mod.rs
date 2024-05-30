mod query_generator;
mod schema_generator;
mod types_generator;
mod upstream_generator;

pub use query_generator::QueryGenerator;
pub use schema_generator::SchemaGenerator;
pub use types_generator::TypesGenerator;

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
