use std::collections::{HashMap, HashSet};
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

fn insert_resolution(
    mut map: HashMap<String, Resolution>,
    current_name: &str,
    resolution: Resolution,
) -> HashMap<String, Resolution> {
    if resolution.input.eq(&resolution.output) {
        tracing::warn!(
            "Unable to resolve input and output type: {}",
            resolution.input
        );
    }

    if !map.contains_key(current_name) {
        map.entry(current_name.to_string()).or_insert(resolution);
    }

    map
}

impl ConfigModule {
    /// This function resolves the ambiguous types by renaming the input and
    /// output types. The resolver function should return a Resolution
    /// object containing the new input and output types.
    /// The function will return a new ConfigModule with the resolved types.
    pub fn resolve_ambiguous_types(mut self, resolver: impl Fn(&str) -> Resolution) -> Self {
        let mut resolution_map = HashMap::new();

        for current_name in self.input_types.intersection(&self.output_types) {
            let resolution = resolver(current_name);

            resolution_map = insert_resolution(resolution_map, current_name, resolution);

            if let Some(ty) = self.config.types.get(current_name) {
                for field in ty.fields.values() {
                    for args in field.args.values() {
                        if self.output_types.contains(&args.type_of) {
                            let resolution = resolver(args.type_of.as_str());
                            resolution_map = insert_resolution(
                                resolution_map,
                                args.type_of.as_str(),
                                resolution,
                            );
                        }
                    }
                }
            }
        }

        for (current_name, resolution) in &resolution_map {
            let input_name = &resolution.input;
            let output_name = &resolution.output;

            if let Some(og_ty) = self.config.types.get(current_name).cloned() {
                self.config.types.insert(input_name.clone(), og_ty.clone());
                self.input_types.insert(input_name.clone());

                self.config.types.insert(output_name.clone(), og_ty.clone());
                self.output_types.insert(output_name.clone());
            }
        }

        let keys = self.config.types.keys().cloned().collect::<Vec<String>>();

        for k in keys {
            if let Some(ty) = self.config.types.get_mut(&k) {
                for field in ty.fields.values_mut() {
                    if let Some(resolution) = resolution_map.get(&field.type_of) {
                        if self.output_types.contains(&k) {
                            field.type_of = resolution.output.clone();
                        } else if self.input_types.contains(&k) {
                            field.type_of = resolution.input.clone();
                        }
                    }
                    for arg in field.args.values_mut() {
                        if let Some(resolution) = resolution_map.get(&arg.type_of) {
                            arg.type_of = resolution.input.clone();
                        }
                    }
                }
            }
        }

        for current_name in resolution_map.keys() {
            self.config.types.remove(current_name);
            self.input_types.remove(current_name);
            self.output_types.remove(current_name);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Type;

    fn build_qry(mut config: Config) -> Config {
        let mut type1 = Type::default();
        let mut field1 =
            crate::config::Field { type_of: "Type1".to_string(), ..Default::default() };

        let arg1 = crate::config::Arg { type_of: "Type1".to_string(), ..Default::default() };

        field1.args.insert("arg1".to_string(), arg1);

        let arg2 = crate::config::Arg { type_of: "Type2".to_string(), ..Default::default() };

        let _field3 = crate::config::Field { type_of: "Type3".to_string(), ..Default::default() };
        let arg3 = crate::config::Arg { type_of: "Type3".to_string(), ..Default::default() };

        field1.args.insert("arg2".to_string(), arg2);
        field1.args.insert("arg3".to_string(), arg3);

        let mut field2 = field1.clone();
        field2.type_of = "Type2".to_string();

        type1.fields.insert("field1".to_string(), field1);
        type1.fields.insert("field2".to_string(), field2);

        config.types.insert("Query".to_string(), type1);
        config = config.query("Query");

        config
    }

    #[test]
    fn test_resolve_ambiguous_types() {
        // Create a ConfigModule instance with ambiguous types
        let mut config = Config::default();

        let mut type1 = Type::default();
        let mut type2 = Type::default();
        let mut type3 = Type::default();

        type1.fields.insert(
            "name".to_string(),
            crate::config::Field::default().type_of("String".to_string()),
        );

        type2.fields.insert(
            "ty1".to_string(),
            crate::config::Field::default().type_of("Type1".to_string()),
        );
        type2.fields.insert(
            "ty3".to_string(),
            crate::config::Field::default().type_of("Type3".to_string()),
        );

        type3.fields.insert(
            "ty1".to_string(),
            crate::config::Field::default().type_of("Type1".to_string()),
        );
        type3.fields.insert(
            "ty2".to_string(),
            crate::config::Field::default().type_of("Type2".to_string()),
        );

        config.types.insert("Type1".to_string(), type1);
        config.types.insert("Type2".to_string(), type2);
        config.types.insert("Type3".to_string(), type3);

        config = build_qry(config);

        let mut config_module = ConfigModule::from(config);

        let resolver = |type_name: &str| -> Resolution {
            Resolution {
                input: format!("{}Input", type_name),
                output: format!("{}Output", type_name),
            }
        };

        config_module = config_module.resolve_ambiguous_types(resolver);

        assert!(config_module.config.types.contains_key("Type1Input"));
        assert!(config_module.config.types.contains_key("Type1Output"));
        assert!(config_module.config.types.contains_key("Type2Output"));
        assert!(config_module.config.types.contains_key("Type2Input"));
        assert!(config_module.config.types.contains_key("Type3"));
    }
}
