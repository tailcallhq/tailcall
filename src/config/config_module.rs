use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::sync::Arc;

use derive_setters::Setters;
use jsonwebtoken::jwk::JwkSet;
use prost_reflect::prost_types::{FileDescriptorProto, FileDescriptorSet};
use rustls_pki_types::{CertificateDer, PrivateKeyDer};

use crate::config::Config;
use crate::macros::MergeRight;
use crate::merge_right::MergeRight;
use crate::proto_reader::ProtoMetadata;
use crate::rest::{EndpointSet, Unchecked};
use crate::scalar;

use super::TypeKind;

/// A wrapper on top of Config that contains all the resolved extensions and
/// computed values.
#[derive(Clone, Debug, Default, Setters, MergeRight)]
pub struct ConfigModule {
    pub config: Config,
    pub extensions: Extensions,
    pub input_types: HashSet<String>,
    pub output_types: HashSet<String>,
    pub interfaces: HashSet<String>,
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
#[derive(Clone, Debug, Default, MergeRight)]
pub struct Extensions {
    /// Contains the file descriptor set resolved from the links to proto files
    pub grpc_file_descriptors: HashMap<String, FileDescriptorProto>,

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
    pub fn add_proto(&mut self, metadata: ProtoMetadata) {
        for file in metadata.descriptor_set.file {
            self.grpc_file_descriptors
                .insert(file.name().to_string(), file);
        }
    }

    pub fn get_file_descriptor_set(&self) -> FileDescriptorSet {
        FileDescriptorSet { file: self.grpc_file_descriptors.values().cloned().collect() }
    }

    pub fn has_auth(&self) -> bool {
        !self.htpasswd.is_empty() || !self.jwks.is_empty()
    }
}

impl MergeRight for FileDescriptorSet {
    fn merge_right(mut self, other: Self) -> Self {
        self.file.extend(other.file);

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
        match &type_.kind {
            TypeKind::Object(obj) => {
                for (_, field) in obj.fields.iter() {
                    if !types.contains(&field.type_of) {
                        types.insert(field.type_of.clone());
                        recurse_type(config, &field.type_of, types);
                    }
                }
            }
            TypeKind::Union(_) => todo!(),
            _ => {}
        }
    }
}

fn get_input_types(config: &Config) -> HashSet<String> {
    let mut types = HashSet::new();

    for (_, type_of) in config.types.iter() {
        match &type_of.kind {
            TypeKind::Object(obj) => {
                for (_, field) in obj.fields.iter() {
                    for (_, arg) in field
                        .args
                        .iter()
                        .filter(|(_, arg)| !scalar::is_scalar(&arg.type_of))
                    {
                        recurse_type(config, &arg.type_of, &mut types);
                    }
                }
            }
            TypeKind::Union(_) => todo!(),
            _ => {}
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
        match &type_of.kind {
            TypeKind::Object(obj) => {
                if !input_types.contains(type_name) {
                    for field in obj.fields.values() {
                        types.insert(field.type_of.clone());
                    }
                }
            }
            TypeKind::Union(_) => todo!(),
            _ => {}
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
