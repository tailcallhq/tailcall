use std::ops::Deref;
use std::sync::Arc;

use prost_reflect::prost_types::FileDescriptorSet;

use crate::config::proto_config::get_descriptor_set;
use crate::config::Config;
use crate::{FileIO, HttpIO, ProtoPathResolver};

/// A wrapper on top of Config that contains all the resolved extensions.
#[derive(Clone, Debug, Default)]
pub struct ConfigSet {
    pub config: Config,
    pub extensions: Extensions,
}

/// Extensions are meta-information required before we can generate the blueprint.
/// Typically, this information cannot be inferred without performing an IO operation, i.e.,
/// reading a file, making an HTTP call, etc.
#[derive(Clone, Debug, Default)]
pub struct Extensions {
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
    pub async fn resolve_extensions(
        self,
        file_io: Arc<dyn FileIO>,
        http_io: Arc<dyn HttpIO>,
        resolver: Arc<dyn ProtoPathResolver>,
    ) -> Self {
        let grpc_file_descriptor = get_descriptor_set(&self.config, file_io, http_io, resolver)
            .await
            .unwrap_or_default();
        let extensions = Extensions { grpc_file_descriptor, ..self.extensions };
        Self { extensions, ..self }
    }
}
