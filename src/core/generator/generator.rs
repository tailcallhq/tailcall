use anyhow::Context;
use prost_reflect::prost_types::FileDescriptorSet;
use prost_reflect::DescriptorPool;
use serde_json::Value;
use url::Url;

use super::from_proto::from_proto;
use super::{FromJsonGenerator, Generate, NameGenerator, RequestSample};
use crate::core::config::{self, Config, ConfigModule, Link, LinkType};
use crate::core::merge_right::MergeRight;
use crate::core::proto_reader::ProtoMetadata;
use crate::core::valid::Validator;

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

pub struct ConfigInput {
    pub schema: String,
    pub source: config::Source,
}

pub struct JsonInput {
    pub url: Url,
    pub data: Value,
}

pub struct ProtoInput {
    pub data: ProtoMetadata,
}

/// Generator offers an abstraction over the actual config generators and allows
/// to generate the single config from multiple sources. i.e (Protobuf and Json)
pub struct Generator {
    operation_name: Option<String>,
    proto_samples: Option<Vec<ProtoInput>>,
    json_samples: Option<Vec<JsonInput>>,
    config_samples: Option<Vec<ConfigInput>>,
    is_mutation: Option<bool>,
    type_name_prefix: Option<String>,
    field_name_prefix: Option<String>,
}

impl Default for Generator {
    fn default() -> Self {
        Self::new()
    }
}

impl Generator {
    pub fn new() -> Self {
        Self {
            operation_name: None,
            proto_samples: None,
            json_samples: None,
            config_samples: None,
            is_mutation: None,
            type_name_prefix: None,
            field_name_prefix: None,
        }
    }

    pub fn with_proto_samples(mut self, samples: Vec<ProtoInput>) -> Self {
        self.proto_samples = Some(samples);
        self
    }

    pub fn with_json_samples(mut self, samples: Vec<JsonInput>) -> Self {
        self.json_samples = Some(samples);
        self
    }

    pub fn with_config_samples(mut self, samples: Vec<ConfigInput>) -> Self {
        self.config_samples = Some(samples);
        self
    }

    pub fn with_operation_name(mut self, query: &str) -> Self {
        self.operation_name = Some(query.to_owned());
        self
    }

    pub fn with_is_mutation(mut self, mutation: bool) -> Self {
        self.is_mutation = Some(mutation);
        self
    }

    /// type name prefix will be used in generation of type names.
    pub fn with_type_name_prefix(mut self, prefix: &str) -> Self {
        self.type_name_prefix = Some(prefix.to_owned());
        self
    }

    /// field name prefix will be used in generation of field names.
    pub fn with_field_name_prefix(mut self, prefix: &str) -> Self {
        self.field_name_prefix = Some(prefix.to_owned());
        self
    }

    /// Generates configuration from the provided json samples.
    fn generate_from_json(
        &self,
        operation_name: &str,
        json_samples: &[JsonInput],
    ) -> anyhow::Result<Config> {
        let type_name_prefix = self.type_name_prefix.clone().unwrap_or("T".to_string());
        let field_name_prefix = self.field_name_prefix.clone().unwrap_or("f".to_string());

        let request_samples = json_samples
            .iter()
            .map(|sample| RequestSample::new(sample.url.to_owned(), sample.data.to_owned()))
            .collect();

        Ok(FromJsonGenerator::new(
            request_samples,
            &NameGenerator::new(&type_name_prefix),
            &NameGenerator::new(&field_name_prefix),
            operation_name,
        )
        .generate()
        .to_result()?)
    }

    /// Generates the configuration from the provided protobuf.
    fn generate_from_proto(
        &self,
        proto_input: &ProtoInput,
        operation_name: &str,
    ) -> anyhow::Result<Config> {
        let metadata = &proto_input.data;
        let descriptor_set = resolve_file_descriptor_set(metadata.descriptor_set.clone())?;
        let mut config = from_proto(&[descriptor_set], operation_name)?;
        config.links.push(Link {
            id: None,
            src: metadata.path.to_owned(),
            type_of: LinkType::Protobuf,
        });
        Ok(config)
    }

    /// Generated the actual configuratio from provided samples.
    pub fn generate(&self) -> anyhow::Result<ConfigModule> {
        let operation_name = self
            .operation_name
            .clone()
            .context("Operation name is required to generate the configuration.")?;

        let mut config = Config::default();

        if let Some(json_samples) = &self.json_samples {
            let json_config = self.generate_from_json(&operation_name, json_samples)?;
            config = config.merge_right(json_config);
        }

        if let Some(proto_samples) = &self.proto_samples {
            for proto_input in proto_samples {
                let proto_config = self.generate_from_proto(proto_input, &operation_name)?;

                config = config.merge_right(proto_config);
            }
        }

        if let Some(config_samples) = &self.config_samples {
            for config_input in config_samples {
                config = config.merge_right(Config::from_source(
                    config_input.source.clone(),
                    &config_input.schema,
                )?);
            }
        }

        Ok(ConfigModule::from(config))
    }
}

#[cfg(test)]
mod test {
    use prost_reflect::prost_types::FileDescriptorSet;
    use serde::Deserialize;

    use super::Generator;
    use crate::core::generator::{ConfigInput, JsonInput, ProtoInput};
    use crate::core::proto_reader::ProtoMetadata;

    fn compile_protobuf(files: &[&str]) -> anyhow::Result<FileDescriptorSet> {
        Ok(protox::compile(files, [tailcall_fixtures::protobuf::SELF])?)
    }

    #[derive(Deserialize)]
    struct JsonFixture {
        url: String,
        body: serde_json::Value,
    }

    fn parse_json(path: &str) -> JsonFixture {
        let content = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&content).unwrap()
    }

    #[test]
    fn should_generate_config_from_proto() -> anyhow::Result<()> {
        let news_proto = tailcall_fixtures::protobuf::NEWS;
        let set = compile_protobuf(&[news_proto])?;

        let cfg_module = Generator::new()
            .with_proto_samples(vec![ProtoInput {
                data: ProtoMetadata {
                    descriptor_set: set,
                    path: "../../../tailcall-fixtures/fixtures/protobuf/news.proto".to_string(),
                },
            }])
            .with_operation_name("Query")
            .generate()?;

        insta::assert_snapshot!(cfg_module.config.to_sdl());
        Ok(())
    }

    #[test]
    fn should_generate_config_from_configs() -> anyhow::Result<()> {
        let cfg_module = Generator::new()
            .with_config_samples(vec![ConfigInput {
                schema: std::fs::read_to_string(tailcall_fixtures::configs::USER_POSTS)?,
                source: crate::core::config::Source::GraphQL,
            }])
            .with_operation_name("Query")
            .generate()?;

        insta::assert_snapshot!(cfg_module.config.to_sdl());
        Ok(())
    }

    #[test]
    fn should_generate_config_from_json() -> anyhow::Result<()> {
        let parsed_content =
            parse_json("src/core/generator/tests/fixtures/json/incompatible_properties.json");
        let cfg_module = Generator::new()
            .with_json_samples(vec![JsonInput {
                url: parsed_content.url.parse()?,
                data: parsed_content.body,
            }])
            .with_operation_name("Query")
            .generate()?;
        insta::assert_snapshot!(cfg_module.config.to_sdl());
        Ok(())
    }

    #[test]
    fn should_generate_combined_config() -> anyhow::Result<()> {
        // Proto input
        let news_proto = tailcall_fixtures::protobuf::NEWS;
        let proto_set = compile_protobuf(&[news_proto])?;
        let proto_input = ProtoInput {
            data: ProtoMetadata {
                descriptor_set: proto_set,
                path: "../../../tailcall-fixtures/fixtures/protobuf/news.proto".to_string(),
            },
        };

        // Config input
        let config_input = ConfigInput {
            schema: std::fs::read_to_string(tailcall_fixtures::configs::USER_POSTS)?,
            source: crate::core::config::Source::GraphQL,
        };

        // Json Input
        let parsed_content =
            parse_json("src/core/generator/tests/fixtures/json/incompatible_properties.json");
        let json_input = JsonInput { url: parsed_content.url.parse()?, data: parsed_content.body };

        // Combine inputs
        let cfg_module = Generator::new()
            .with_proto_samples(vec![proto_input])
            .with_config_samples(vec![config_input])
            .with_json_samples(vec![json_input])
            .with_operation_name("Query")
            .generate()?;

        // Assert the combined output
        insta::assert_snapshot!(cfg_module.config.to_sdl());
        Ok(())
    }

    #[test]
    fn should_generate_error_if_operation_name_not_provided() -> anyhow::Result<()> {
        let parsed_content =
            parse_json("src/core/generator/tests/fixtures/json/incompatible_properties.json");
        let cfg_module = Generator::new()
            .with_json_samples(vec![JsonInput {
                url: parsed_content.url.parse()?,
                data: parsed_content.body,
            }])
            .generate();

        assert!(cfg_module.is_err());
        assert_eq!(
            cfg_module.unwrap_err().to_string(),
            "Operation name is required to generate the configuration."
        );
        Ok(())
    }
}
