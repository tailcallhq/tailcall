use std::ops::Deref;

use prost_reflect::prost_types::FileDescriptorSet;

use crate::config::Config;

/// A wrapper on top of Config that contains all the resolved extensions.
#[derive(Clone, Debug, Default)]
struct ConfigSet {
    pub config: Config,
    pub extensions: Extensions,
}

/// Extensions is meta information that is required before we can generate the blueprint.
/// Typically this information can not be inferred without performing an IO operation, ie.
/// reading a file, making an HTTP call etc.
#[derive(Clone, Debug, Default)]
struct Extensions {
    pub grpc_file_descriptor: FileDescriptorSet,

    /// Contains the contents of the JS file
    pub script: Option<String>,
}

impl Deref for ConfigSet {
    type Target = Config;
    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl From<Config> for ConfigSet {
    fn from(config: Config) -> Self {
        ConfigSet { config, ..Default::default() }
    }
}

impl ConfigSet {
    async fn resolve_extensions(self) -> Self {
        todo!()
    }
}
