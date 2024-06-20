use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::sync::Arc;

use derive_setters::Setters;
use jsonwebtoken::jwk::JwkSet;
use prost_reflect::prost_types::{FileDescriptorProto, FileDescriptorSet};
use rustls_pki_types::{CertificateDer, PrivateKeyDer};

use crate::core::config::{Arg, Config};
use crate::core::macros::MergeRight;
use crate::core::merge_right::MergeRight;
use crate::core::proto_reader::ProtoMetadata;
use crate::core::rest::{EndpointSet, Unchecked};
use crate::core::scalar;
use crate::core::transform::Transform;
use crate::core::valid::{Valid, Validator};

/// A wrapper on top of Config that contains all the resolved extensions and
/// computed values.
#[derive(Clone, Debug, Default, Setters, MergeRight)]
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

impl From<Config> for ConfigModule {
    fn from(config: Config) -> Self {
        ConfigModule { config, ..Default::default() }
    }
}

impl ConfigModule {
    pub fn transform<T: Transform<Value = Config, Error = String>>(
        self,
        transformer: T,
    ) -> Valid<Self, String> {
        transformer.transform(self.config).map(ConfigModule::from)
    }

    pub fn input_types(&self) -> HashSet<String> {
        self.arguments()
            .iter()
            .filter(|(_, arg)| !self.is_scalar(&arg.type_of))
            .map(|(_, arg)| arg.type_of.as_str())
            .fold(HashSet::new(), |types, type_of| {
                self.find_connections(type_of, types)
            })
    }

    ///
    /// Given a starting type, this function searches for all the unique types
    /// that this type can be connected to via it's fields
    fn find_connections(&self, type_of: &str, mut types: HashSet<String>) -> HashSet<String> {
        if let Some(type_) = self.find_type(type_of) {
            types.insert(type_of.into());
            for (_, field) in type_.fields.iter() {
                if !types.contains(&field.type_of) && !self.is_scalar(&field.type_of) {
                    types.insert(field.type_of.clone());
                    types = self.find_connections(&field.type_of, types);
                }
            }
        }
        types
    }

    ///
    /// Checks if a type is a scalar or not.
    pub fn is_scalar(&self, type_name: &str) -> bool {
        self.types
            .get(type_name)
            .map_or(scalar::is_predefined_scalar(type_name), |ty| ty.scalar())
    }

    /// Returns a list of all the types that are used as output types
    pub fn output_types(&self) -> HashSet<String> {
        let mut types = HashSet::new();

        if let Some(ref query) = &self.schema.query {
            types = self.find_connections(query, types);
        }

        if let Some(ref mutation) = &self.schema.mutation {
            types = self.find_connections(mutation, types);
        }

        types
    }

    /// Returns a list of all the types that are used as interface
    pub fn interface_types(&self) -> HashSet<String> {
        let mut types = HashSet::new();

        for ty in self.types.values() {
            for interface in ty.implements.iter() {
                types.insert(interface.clone());
            }
        }

        types
    }

    /// Returns a list of all the arguments in the configuration
    fn arguments(&self) -> Vec<(&String, &Arg)> {
        self.types
            .iter()
            .flat_map(|(_, type_of)| type_of.fields.iter())
            .flat_map(|(_, field)| field.args.iter())
            .collect::<Vec<_>>()
    }
}
