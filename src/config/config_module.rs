use std::collections::HashSet;
use std::ops::Deref;
use std::sync::Arc;

use derive_setters::Setters;
use jsonwebtoken::jwk::JwkSet;
use prost_reflect::prost_types::FileDescriptorSet;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};

use crate::blueprint::GrpcMethod;
use crate::config::Config;
use crate::merge_right::MergeRight;
use crate::rest::{EndpointSet, Unchecked};


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

pub struct Resolution {
    pub input: String,
    pub output: String,
}

impl ConfigModule {
    /// This function resolves the ambiguous types by renaming the input and
    /// output types. The resolver function should return a Resolution
    /// object containing the new input and output types.
    /// The function will return a new ConfigModule with the resolved types.
    pub fn resolve_ambiguous_types(mut self, resolver: impl Fn(&str) -> Resolution) -> Self {
        for key in self.input_types.intersection(&self.output_types) {
            let resolution = resolver(key);

            if resolution.input.eq(&resolution.output) {
                tracing::warn!(
                    "Unable to resolve input and output type: {}",
                    resolution.input
                );
                continue;
            }

            let og_ty = self.config.types.remove(key);

            if let Some(og_ty) = og_ty {
                self.config
                    .types
                    .insert(resolution.input.clone(), og_ty.clone());
                self.config.types.insert(resolution.output.clone(), og_ty);
            }

            for v in self.config.types.values_mut() {
                for field in v.fields.values_mut() {
                    if field.type_of.eq(key) {
                        field.type_of = resolution.output.clone();
                    }

                    for arg in field.args.values_mut() {
                        if arg.type_of.eq(key) {
                            arg.type_of = resolution.input.clone();
                        }
                    }
                }
            }
        }

        self
    }
}

impl From<Config> for ConfigModule {
    fn from(config: Config) -> Self {
        let input_types = config.input_types();
        let output_types = config.output_types();

        ConfigModule { config, input_types, output_types, ..Default::default() }
    }
}
