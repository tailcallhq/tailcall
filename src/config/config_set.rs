use std::ops::Deref;

use prost_reflect::prost_types::FileDescriptorSet;

use crate::config::Config;

/// A wrapper on top of Config that contains all the resolved extensions.
#[derive(Clone, Debug, Default)]
pub struct ConfigSet {
    pub config: Config,
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Default)]
pub struct FileDescriptorSetWithId {
    pub id: Option<String>,
    pub file_descriptor_set: FileDescriptorSet,
}

/// Extensions are meta-information required before we can generate the blueprint.
/// Typically, this information cannot be inferred without performing an IO operation, i.e.,
/// reading a file, making an HTTP call, etc.
#[derive(Clone, Debug, Default)]
pub struct Extensions {
    pub grpc_file_descriptor: FileDescriptorSet,

    /// Contains the contents of the JS file
    pub script: Option<String>,

    /// Contains the file descriptor sets resolved from the links
    pub file_descriptor_from_links: Vec<FileDescriptorSetWithId>,
}

impl Extensions {
    pub fn merge_right(mut self, other: &Extensions) -> Self {
        self.grpc_file_descriptor
            .file
            .extend(other.grpc_file_descriptor.file.clone());
        self.script = other.script.clone().or(self.script.take());
        self.file_descriptor_from_links
            .extend(other.file_descriptor_from_links.clone());
        self
    }
}

impl ConfigSet {
    pub fn merge_right(mut self, other: &Self) -> Self {
        self.config = self.config.merge_right(&other.config);
        self.extensions = self.extensions.merge_right(&other.extensions);
        self
    }
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
