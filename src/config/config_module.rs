use std::collections::HashSet;
use std::ops::Deref;
use std::sync::Arc;

use derive_setters::Setters;
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
#[derive(Clone, Debug, Default, Setters)]
pub struct ConfigModule {
    pub config: Config,
    pub extensions: Extensions,
}

impl ConfigModule {
    pub fn output_types(&self) -> HashSet<&String> {
        let mut types = HashSet::new();
        let input_types = self.input_types();

        if let Some(ref query) = &self.schema.query {
            types.insert(query);
        }

        if let Some(ref mutation) = &self.schema.mutation {
            types.insert(mutation);
        }
        for (type_name, type_of) in self.types.iter() {
            if (type_of.interface || !type_of.fields.is_empty())
                && !input_types.contains(&type_name)
            {
                for (_, field) in type_of.fields.iter() {
                    types.insert(&field.type_of);
                }
            }
        }
        types
    }

    pub fn input_types(&self) -> HashSet<&String> {
        let mut types = HashSet::new();
        for (_, type_of) in self.types.iter() {
            if !type_of.interface {
                for (_, field) in type_of.fields.iter() {
                    for (_, arg) in field
                        .args
                        .iter()
                        .filter(|(_, arg)| !scalar::is_scalar(&arg.type_of))
                    {
                        if let Some(t) = self.find_type(&arg.type_of) {
                            t.fields.iter().for_each(|(_, f)| {
                                types.insert(&f.type_of);
                                self.recurse_type(&f.type_of, &mut types)
                            })
                        }
                        types.insert(&arg.type_of);
                    }
                }
            }
        }
        types
    }

    pub fn find_type(&self, name: &str) -> Option<&Type> {
        self.types.get(name)
    }

    pub fn find_union(&self, name: &str) -> Option<&Union> {
        self.unions.get(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.types.contains_key(name) || self.unions.contains_key(name)
    }
}

impl From<Config> for ConfigModule {
    fn from(config: Config) -> Self {
        ConfigModule { config, ..Default::default() }
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
