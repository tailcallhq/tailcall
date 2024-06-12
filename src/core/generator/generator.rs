use prost_reflect::prost_types::FileDescriptorSet;
use prost_reflect::DescriptorPool;
use serde_json::Value;
use url::Url;

use super::from_proto::from_proto;
use super::{from_json, ConfigGenerationRequest, NameGenerator};
use crate::core::config::{Config, ConfigModule, Link, LinkType, Source};
use crate::core::merge_right::MergeRight;
use crate::core::proto_reader::ProtoMetadata;

// this function resolves all the names to fully-qualified syntax in descriptors
// that is important for generation to work
// TODO: probably we can drop this in case the config_reader will use
// protox::compile instead of more low-level protox_parse::parse
fn resolve_file_descriptor_set(
    descriptor_set: FileDescriptorSet,
) -> anyhow::Result<FileDescriptorSet> {
    let descriptor_set = DescriptorPool::from_file_descriptor_set(descriptor_set)?;
    let descriptor_set = FileDescriptorSet {
        file: descriptor_set
            .files()
            .map(|file| file.file_descriptor_proto().clone())
            .collect(),
    };

    Ok(descriptor_set)
}

pub enum GeneratorInput {
    Config { schema: String, source: Source },
    Proto { metadata: ProtoMetadata },
    Json { url: Url, data: Value },
}

pub struct Generator {
    field_name_gen: NameGenerator,
    type_name_gen: NameGenerator,
}

impl Generator {
    pub fn new(field_prefix: &str, type_prefix: &str) -> Self {
        Self {
            field_name_gen: NameGenerator::new(field_prefix),
            type_name_gen: NameGenerator::new(type_prefix),
        }
    }
    pub fn run(
        &self,
        query: &str,
        config_generation_req: &[GeneratorInput],
    ) -> anyhow::Result<ConfigModule> {
        let mut config = Config::default();

        for req in config_generation_req {
            match req {
                GeneratorInput::Config { schema, source } => {
                    config = config.merge_right(Config::from_source(source.to_owned(), schema)?)
                }
                GeneratorInput::Json { url, data } => {
                    let req = ConfigGenerationRequest::new(url.to_owned(), data.to_owned());
                    config = config.merge_right(from_json(
                        &[req],
                        query,
                        &self.field_name_gen,
                        &self.type_name_gen,
                    )?);
                }
                GeneratorInput::Proto { metadata } => {
                    let descriptor_set =
                        resolve_file_descriptor_set(metadata.descriptor_set.to_owned())?;
                    config = config.merge_right(from_proto(&[descriptor_set], query)?);

                    config.links.push(Link {
                        id: None,
                        src: metadata.path.to_owned(),
                        type_of: LinkType::Protobuf,
                    });
                }
            }
        }
        Ok(ConfigModule::from(config))
    }
}
