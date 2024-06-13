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

impl Default for Generator {
    fn default() -> Self {
        Self {
            field_name_gen: NameGenerator::new("f"),
            type_name_gen: NameGenerator::new("T"),
        }
    }
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
        // TODO: add more transformers here and fix the bug present in AmbiguousType.
        // let config = ConfigModule::from(config)
        //     .transform(AmbiguousType::default())
        //     .to_result()?;

        Ok(ConfigModule::from(config))
    }
}

#[cfg(test)]
mod test {
    use prost_reflect::prost_types::FileDescriptorSet;

    use super::{Generator, GeneratorInput};
    use crate::core::proto_reader::ProtoMetadata;

    fn compile_protobuf(files: &[&str]) -> anyhow::Result<FileDescriptorSet> {
        Ok(protox::compile(files, [tailcall_fixtures::protobuf::SELF])?)
    }

    #[test]
    fn should_generate_config_from_proto() -> anyhow::Result<()> {
        let news_proto = tailcall_fixtures::protobuf::NEWS;
        let set = compile_protobuf(&[news_proto])?;

        let gen = Generator::default();
        let cfg_module = gen.run(
            "Query",
            &[GeneratorInput::Proto {
                metadata: ProtoMetadata {
                    descriptor_set: set,
                    path: "../../../tailcall-fixtures/fixtures/protobuf/news.proto".to_string(),
                },
            }],
        )?;
        insta::assert_snapshot!(cfg_module.config.to_sdl());
        Ok(())
    }

    #[test]
    fn should_generate_config_from_configs() -> anyhow::Result<()> {
        let gen = Generator::default();
        let cfg_module = gen.run(
            "Query",
            &[GeneratorInput::Config {
                schema: std::fs::read_to_string(tailcall_fixtures::configs::USER_POSTS)?,
                source: crate::core::config::Source::GraphQL,
            }],
        )?;
        insta::assert_snapshot!(cfg_module.config.to_sdl());
        Ok(())
    }

    fn parse_json(path: &str) -> serde_json::Value {
        let content = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&content).unwrap()
    }

    #[test]
    fn should_generate_config_from_json() -> anyhow::Result<()> {
        let gen = Generator::default();
        let cfg_module = gen.run(
            "Query",
            &[GeneratorInput::Json {
                url: "https://example.com".parse()?,
                data: parse_json(
                    "src/core/generator/tests/fixtures/json/incompatible_properties.json",
                ),
            }],
        )?;
        insta::assert_snapshot!(cfg_module.config.to_sdl());
        Ok(())
    }
}
