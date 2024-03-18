use std::collections::BTreeMap;
use std::ops::Deref;
use std::sync::Arc;

use derive_setters::Setters;
use prost_reflect::prost_types::FileDescriptorSet;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};

use crate::blueprint::Scope;
use crate::config::Config;
use crate::merge_right::MergeRight;
use crate::rest::{EndpointSet, Unchecked};

/// A wrapper on top of Config that contains all the resolved extensions.
#[derive(Clone, Debug, Default, Setters)]
pub struct ConfigModule {
    pub config: Config,
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

/// Extensions are meta-information required before we can generate the
/// blueprint. Typically, this information cannot be inferred without performing
/// an IO operation, i.e., reading a file, making an HTTP call, etc.
#[derive(Clone, Debug, Default)]
pub struct Extensions {
    /// Contains the file descriptor sets resolved from the links
    pub grpc_file_descriptors_map: BTreeMap<Scope, Content<FileDescriptorSet>>,

    /// Contains the contents of the JS file
    pub script: Option<String>,

    /// Contains the certificate used on HTTP2 with TLS
    pub cert: Vec<CertificateDer<'static>>,

    /// Contains the key used on HTTP2 with TLS
    pub keys: Arc<Vec<PrivateKeyDer<'static>>>,

    /// Contains the endpoints
    pub endpoint_set: EndpointSet<Unchecked>,
}

impl Extensions {
    pub fn get_file_descriptor_set(&self, scope: &Scope) -> Option<&FileDescriptorSet> {
        self.grpc_file_descriptors_map
            .get(scope)
            .map(|a| &a.content)
    }
}

impl MergeRight for Extensions {
    fn merge_right(mut self, mut other: Self) -> Self {
        self.grpc_file_descriptors_map = self
            .grpc_file_descriptors_map
            .merge_right(other.grpc_file_descriptors_map);
        self.script = self.script.merge_right(other.script.take());
        self.cert = self.cert.merge_right(other.cert);
        self.keys = if !other.keys.is_empty() {
            other.keys
        } else {
            self.keys
        };
        self.endpoint_set = self.endpoint_set.merge_right(other.endpoint_set);
        self
    }
}

impl MergeRight for ConfigModule {
    fn merge_right(mut self, other: Self) -> Self {
        self.config = self.config.merge_right(other.config);
        self.extensions = self.extensions.merge_right(other.extensions);
        self
    }
}

impl Deref for ConfigModule {
    type Target = Config;
    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl From<Config> for ConfigModule {
    fn from(config: Config) -> Self {
        ConfigModule { config, ..Default::default() }
    }
}
