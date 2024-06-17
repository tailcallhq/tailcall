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

/// Generator offers an abstraction over the actual config generators and allows
/// to generate the single config from multiple sources. i.e (Protobuf and Json)
/// TODO: add support for is_mutation.
pub struct Generator {
    operation_name: String,
    inputs: Vec<Input>,
    type_name_prefix: String,
    field_name_prefix: String,
}

pub enum Input {
    Json {
        url: Url,
        response: Value,
    },
    Proto(ProtoMetadata),
    Config {
        schema: String,
        source: config::Source,
    },
}

pub struct GeneratorBuilder {
    operation_name: Option<String>,
    inputs: Option<Vec<Input>>,
    type_name_prefix: Option<String>,
    field_name_prefix: Option<String>,
}

impl GeneratorBuilder {
    pub fn new() -> Self {
        Self {
            operation_name: None,
            inputs: None,
            type_name_prefix: None,
            field_name_prefix: None,
        }
    }

    pub fn with_operation_name(mut self, operation_name: &str) -> Self {
        self.operation_name = Some(operation_name.to_string());
        self
    }

    pub fn with_inputs(mut self, inputs: Vec<Input>) -> Self {
        self.inputs = Some(inputs);
        self
    }

    pub fn with_type_name_prefix(mut self, type_name_prefix: &str) -> Self {
        self.type_name_prefix = Some(type_name_prefix.to_string());
        self
    }

    pub fn with_field_name_prefix(mut self, field_name_prefix: &str) -> Self {
        self.field_name_prefix = Some(field_name_prefix.to_string());
        self
    }

    pub fn generate(self) -> anyhow::Result<ConfigModule> {
        let config_generator = Generator {
            operation_name: self.operation_name.context("operation_name is required")?,
            inputs: self.inputs.context("inputs are required")?,
            type_name_prefix: self
                .type_name_prefix
                .context("type_name_prefix is required")?,
            field_name_prefix: self
                .field_name_prefix
                .context("field_name_prefix is required")?,
        };
        config_generator.generate()
    }
}

impl Generator {
    pub fn new() -> GeneratorBuilder {
        GeneratorBuilder::new()
    }

    /// Generates configuration from the provided json samples.
    fn generate_from_json(
        &self,
        operation_name: &str,
        type_name_generator: &NameGenerator,
        field_name_generator: &NameGenerator,
        json_samples: &[RequestSample],
    ) -> anyhow::Result<Config> {
        FromJsonGenerator::new(
            json_samples,
            type_name_generator,
            field_name_generator,
            operation_name,
        )
        .generate()
    }

    /// Generates the configuration from the provided protobuf.
    fn generate_from_proto(
        &self,
        metadata: &ProtoMetadata,
        operation_name: &str,
    ) -> anyhow::Result<Config> {
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
    fn generate(&self) -> anyhow::Result<ConfigModule> {
        let mut config: Config = Config::default();
        let field_name_generator = NameGenerator::new(&self.field_name_prefix);
        let type_name_generator = NameGenerator::new(&self.type_name_prefix);

        for input in self.inputs.iter() {
            match input {
                Input::Config { source, schema } => {
                    config = config.merge_right(Config::from_source(source.clone(), schema)?);
                }
                Input::Json { url, response } => {
                    let request_sample = RequestSample::new(url.to_owned(), response.to_owned());
                    config = config.merge_right(self.generate_from_json(
                        &self.operation_name,
                        &type_name_generator,
                        &field_name_generator,
                        &[request_sample],
                    )?);
                }
                Input::Proto(proto_input) => {
                    config = config
                        .merge_right(self.generate_from_proto(proto_input, &self.operation_name)?);
                }
            }
        }

        Ok(ConfigModule::from(config))
    }
}

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

#[cfg(test)]
mod test {
    use prost_reflect::prost_types::FileDescriptorSet;
    use serde::Deserialize;

    use super::Generator;
    use crate::core::generator::generator::Input;
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
            .with_inputs(vec![Input::Proto(ProtoMetadata {
                descriptor_set: set,
                path: "../../../tailcall-fixtures/fixtures/protobuf/news.proto".to_string(),
            })])
            .with_operation_name("Query")
            .with_field_name_prefix("f")
            .with_type_name_prefix("T")
            .generate()?;

        insta::assert_snapshot!(cfg_module.config.to_sdl());
        Ok(())
    }

    #[test]
    fn should_generate_config_from_configs() -> anyhow::Result<()> {
        let cfg_module = Generator::new()
            .with_inputs(vec![Input::Config {
                schema: std::fs::read_to_string(tailcall_fixtures::configs::USER_POSTS)?,
                source: crate::core::config::Source::GraphQL,
            }])
            .with_operation_name("Query")
            .with_field_name_prefix("f")
            .with_type_name_prefix("T")
            .generate()?;

        insta::assert_snapshot!(cfg_module.config.to_sdl());
        Ok(())
    }

    #[test]
    fn should_generate_config_from_json() -> anyhow::Result<()> {
        let parsed_content =
            parse_json("src/core/generator/tests/fixtures/json/incompatible_properties.json");
        let cfg_module = Generator::new()
            .with_inputs(vec![Input::Json {
                url: parsed_content.url.parse()?,
                response: parsed_content.body,
            }])
            .with_operation_name("Query")
            .with_field_name_prefix("f")
            .with_type_name_prefix("T")
            .generate()?;
        insta::assert_snapshot!(cfg_module.config.to_sdl());
        Ok(())
    }

    #[test]
    fn should_generate_combined_config() -> anyhow::Result<()> {
        // Proto input
        let news_proto = tailcall_fixtures::protobuf::NEWS;
        let proto_set = compile_protobuf(&[news_proto])?;
        let proto_input = Input::Proto(ProtoMetadata {
            descriptor_set: proto_set,
            path: "../../../tailcall-fixtures/fixtures/protobuf/news.proto".to_string(),
        });

        // Config input
        let config_input = Input::Config {
            schema: std::fs::read_to_string(tailcall_fixtures::configs::USER_POSTS)?,
            source: crate::core::config::Source::GraphQL,
        };

        // Json Input
        let parsed_content =
            parse_json("src/core/generator/tests/fixtures/json/incompatible_properties.json");
        let json_input = Input::Json {
            url: parsed_content.url.parse()?,
            response: parsed_content.body,
        };

        // Combine inputs
        let cfg_module = Generator::new()
            .with_inputs(vec![proto_input, json_input, config_input])
            .with_operation_name("Query")
            .with_field_name_prefix("f")
            .with_type_name_prefix("T")
            .generate()?;

        // Assert the combined output
        insta::assert_snapshot!(cfg_module.config.to_sdl());
        Ok(())
    }

    #[test]
    fn generate_from_config_from_multiple_jsons() -> anyhow::Result<()> {
        let mut inputs = vec![];
        let json_fixtures = [
            "src/core/generator/tests/fixtures/json/incompatible_properties.json",
            "src/core/generator/tests/fixtures/json/list_incompatible_object.json",
            "src/core/generator/tests/fixtures/json/list.json",
        ];

        for json_path in json_fixtures {
            let parsed_content = parse_json(json_path);
            inputs.push(Input::Json {
                url: parsed_content.url.parse()?,
                response: parsed_content.body,
            });
        }

        let cfg_module = Generator::new()
            .with_inputs(inputs)
            .with_operation_name("Query")
            .with_field_name_prefix("f")
            .with_type_name_prefix("T")
            .generate()?;
        insta::assert_snapshot!(cfg_module.config.to_sdl());
        Ok(())
    }

    #[test]
    fn should_generate_error_if_operation_name_not_provided() -> anyhow::Result<()> {
        let parsed_content =
            parse_json("src/core/generator/tests/fixtures/json/incompatible_properties.json");
        let cfg_module = Generator::new()
            .with_inputs(vec![Input::Json {
                url: parsed_content.url.parse()?,
                response: parsed_content.body,
            }])
            .with_field_name_prefix("f")
            .with_type_name_prefix("T")
            .generate();

        assert!(cfg_module.is_err());
        assert_eq!(
            cfg_module.unwrap_err().to_string(),
            "operation_name is required"
        );
        Ok(())
    }
}
