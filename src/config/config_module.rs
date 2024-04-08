use std::collections::HashSet;
use std::ops::Deref;
use std::sync::Arc;

use derive_setters::Setters;
use jsonwebtoken::jwk::JwkSet;
use prost_reflect::prost_types::FileDescriptorSet;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};

use crate::config::Config;
use crate::merge_right::MergeRight;
use crate::rest::{EndpointSet, Unchecked};
use crate::scalar;

/// A wrapper on top of Config that contains all the resolved extensions and
/// computed values.
#[derive(Clone, Debug, Default, Setters)]
pub struct ConfigModule {
    pub config: Config,
    pub extensions: Extensions,
    pub input_types: HashSet<String>,
    pub output_types: HashSet<String>,
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
    pub grpc_file_descriptors: Vec<Content<FileDescriptorSet>>,

    /// Contains the contents of the JS file
    pub script: Option<String>,

    /// Contains the certificate used on HTTP2 with TLS
    pub cert: Vec<CertificateDer<'static>>,

    /// Contains the key used on HTTP2 with TLS
    pub keys: Arc<Vec<PrivateKeyDer<'static>>>,

    /// Contains the endpoints
    pub endpoint_set: EndpointSet<Unchecked>,

    pub htpasswd: Vec<Content<String>>,

    pub jwks: Vec<Content<JwkSet>>,
}

impl Extensions {
    pub fn get_file_descriptor_set(&self) -> Option<FileDescriptorSet> {
        if self.grpc_file_descriptors.is_empty() {
            return None;
        }

        let mut result = FileDescriptorSet::default();

        for descriptor_set in &self.grpc_file_descriptors {
            result.file.extend(descriptor_set.file.iter().cloned());
        }

        Some(result)
    }

    pub fn has_auth(&self) -> bool {
        !self.htpasswd.is_empty() || !self.jwks.is_empty()
    }
}

impl MergeRight for Extensions {
    fn merge_right(mut self, mut other: Self) -> Self {
        self.grpc_file_descriptors = self
            .grpc_file_descriptors
            .merge_right(other.grpc_file_descriptors);
        self.script = self.script.merge_right(other.script.take());
        self.cert = self.cert.merge_right(other.cert);
        self.keys = if !other.keys.is_empty() {
            other.keys
        } else {
            self.keys
        };
        self.endpoint_set = self.endpoint_set.merge_right(other.endpoint_set);
        self.htpasswd = self.htpasswd.merge_right(other.htpasswd);
        self.jwks = self.jwks.merge_right(other.jwks);
        self
    }
}

impl MergeRight for ConfigModule {
    fn merge_right(mut self, other: Self) -> Self {
        self.config = self.config.merge_right(other.config);
        self.extensions = self.extensions.merge_right(other.extensions);
        self.input_types = self.input_types.merge_right(other.input_types);
        self.output_types = self.output_types.merge_right(other.output_types);
        self
    }
}

impl Deref for ConfigModule {
    type Target = Config;
    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

fn recurse_type(config: &Config, type_of: &str, types: &mut HashSet<String>) {
    if let Some(type_) = config.find_type(type_of) {
        for (_, field) in type_.fields.iter() {
            if !types.contains(&field.type_of) {
                types.insert(field.type_of.clone());
                recurse_type(config, &field.type_of, types);
            }
        }
    }
}

fn get_input_types(config: &Config) -> HashSet<String> {
    let mut types = HashSet::new();

    for (_, type_of) in config.types.iter() {
        if !type_of.interface {
            for (_, field) in type_of.fields.iter() {
                for (_, arg) in field
                    .args
                    .iter()
                    .filter(|(_, arg)| !scalar::is_scalar(&arg.type_of))
                {
                    if let Some(t) = config.find_type(&arg.type_of) {
                        t.fields.iter().for_each(|(_, f)| {
                            types.insert(f.type_of.clone());
                            recurse_type(config, &f.type_of, &mut types)
                        })
                    }
                    types.insert(arg.type_of.clone());
                }
            }
        }
    }
    types
}

fn get_output_types(config: &Config, input_types: &HashSet<String>) -> HashSet<String> {
    let mut types = HashSet::new();

    if let Some(ref query) = &config.schema.query {
        types.insert(query.clone());
    }

    if let Some(ref mutation) = &config.schema.mutation {
        types.insert(mutation.clone());
    }

    for (type_name, type_of) in config.types.iter() {
        if (type_of.interface || !type_of.fields.is_empty()) && !input_types.contains(type_name) {
            for (_, field) in type_of.fields.iter() {
                types.insert(field.type_of.clone());
            }
        }
    }

    types
}

impl From<Config> for ConfigModule {
    fn from(config: Config) -> Self {
        let input_types = get_input_types(&config);
        let output_types = get_output_types(&config, &input_types);

        ConfigModule { config, input_types, output_types, ..Default::default() }
    }
}
