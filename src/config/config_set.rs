use std::ops::Deref;

use derive_setters::Setters;
use prost_reflect::prost_types::FileDescriptorSet;

use crate::config::Config;

/// A wrapper on top of Config that contains all the resolved extensions.
#[derive(Clone, Debug, Default, Setters)]
pub struct ConfigSet {
    pub config: Config,
    // Contains the file path of the file from which the config was read
    // In case multiple files are read, the last file is stored
    pub path: Option<String>,
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Default)]
pub struct Content<A> {
    pub id: Option<String>,
    pub content: A,
}

impl<A> Deref for Content<A> {
    type Target = A;
    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

/// Extensions are meta-information required before we can generate the blueprint.
/// Typically, this information cannot be inferred without performing an IO operation, i.e.,
/// reading a file, making an HTTP call, etc.
#[derive(Clone, Debug, Default)]
pub struct Extensions {
    pub grpc_file_descriptor: FileDescriptorSet,

    /// Contains the file descriptor sets resolved from the links
    pub file_descriptor_from_links: Vec<Content<FileDescriptorSet>>,
}

impl Extensions {
    pub fn merge_right(mut self, other: &Extensions) -> Self {
        self.grpc_file_descriptor
            .file
            .extend(other.grpc_file_descriptor.file.clone());
        self.file_descriptor_from_links
            .extend(other.file_descriptor_from_links.clone());
        self
    }
}

impl ConfigSet {
    pub fn merge_right(mut self, other: &Self) -> Self {
        self.config = self.config.merge_right(&other.config);
        self.extensions = self.extensions.merge_right(&other.extensions);
        self.path = other.path.clone().or(self.path);
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
