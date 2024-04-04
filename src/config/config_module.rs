use std::collections::HashSet;
use std::ops::{Deref, Not};
use std::sync::Arc;

use jsonwebtoken::jwk::JwkSet;
use prost_reflect::prost_types::FileDescriptorSet;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};

use super::{Type, Union};
use crate::blueprint::GrpcMethod;
use crate::config::Config;
use crate::merge_right::MergeRight;
use crate::rest::{EndpointSet, Unchecked};
use crate::scalar;

/// A wrapper on top of Config that contains all the resolved extensions.
#[derive(Clone, Debug, Default)]
pub struct ConfigModule {
    pub config: Config,
    pub extensions: Extensions,
    pub input_types: HashSet<String>,
    pub output_types: HashSet<String>,
}

impl ConfigModule {
    pub fn find_type(&self, name: &str) -> Option<&Type> {
        self.types.get(name)
    }

    pub fn find_union(&self, name: &str) -> Option<&Union> {
        self.unions.get(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.types.contains_key(name) || self.unions.contains_key(name)
    }

    pub fn input_types(&self) -> &HashSet<String> {
        &self.input_types
    }

    pub fn output_types(&self) -> &HashSet<String> {
        &self.output_types
    }
}

fn extract_input_type(config: &Config) -> HashSet<String> {
    config
        .types
        .iter()
        .filter_map(|cfg| cfg.1.interface.not().then_some(&cfg.1.fields))
        .fold(HashSet::new(), |mut set, fields| {
            fields
                .iter()
                .flat_map(|field| {
                    field
                        .1
                        .args
                        .iter()
                        .map(|field| field.1)
                        .filter(|arg| !scalar::is_scalar(&arg.type_of))
                })
                .for_each(|arg| {
                    if let Some(t) = config.find_type(&arg.type_of) {
                        t.fields.iter().for_each(|(_, f)| {
                            set.insert(f.type_of.clone());
                            config.recurse_type(&f.type_of, &mut set)
                        })
                    }
                    set.insert(arg.type_of.clone());
                });
            set
        })
}

fn extract_output_type(config: &Config, input_types: &HashSet<String>) -> HashSet<String> {
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
        let input_types = extract_input_type(&config);
        let output_types = extract_output_type(&config, &input_types);

        ConfigModule { config, input_types, output_types, ..Default::default() }
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
    pub fn get_file_descriptor_set(&self, grpc: &GrpcMethod) -> Option<&FileDescriptorSet> {
        self.grpc_file_descriptors
            .iter()
            .find(|content| {
                content
                    .file
                    .iter()
                    .any(|file| file.package == Some(grpc.package.to_owned()))
            })
            .map(|a| &a.content)
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
