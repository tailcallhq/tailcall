use derive_setters::Setters;
use prost_reflect::prost_types::FileDescriptorSet;
use prost_reflect::DescriptorPool;
use serde_json::Value;
use url::Url;

use super::from_proto::from_proto;
use super::{FromJsonGenerator, NameGenerator, RequestSample};
use crate::core::config::{self, Config, ConfigModule, Link, LinkType};
use crate::core::merge_right::MergeRight;
use crate::core::proto_reader::ProtoMetadata;
use crate::core::transform::{Transform, TransformerOps};
use crate::core::valid::Validator;

/// Generator offers an abstraction over the actual config generators and allows
/// to generate the single config from multiple sources. i.e (Protobuf and Json)
/// TODO: add support for is_mutation.

#[derive(Setters)]
pub struct Generator {
    operation_name: String,
    inputs: Vec<Input>,
    type_name_prefix: String,
    transformers: Vec<Box<dyn Transform<Value = Config, Error = String>>>,
}

#[derive(Clone)]
pub enum Input {
    Json {
        url: Url,
        response: Value,
        field_name: String,
    },
    Proto(ProtoMetadata),
    Config {
        schema: String,
        source: config::Source,
    },
}

impl Default for Generator {
    fn default() -> Self {
        Self::new()
    }
}

impl Generator {
    pub fn new() -> Generator {
        Generator {
            operation_name: "Query".to_string(),
            inputs: Vec::new(),
            type_name_prefix: "T".to_string(),
            transformers: Default::default(),
        }
    }

    /// Generates configuration from the provided json samples.
    fn generate_from_json(
        &self,
        type_name_generator: &NameGenerator,
        json_samples: &[RequestSample],
    ) -> anyhow::Result<Config> {
        Ok(
            FromJsonGenerator::new(json_samples, type_name_generator, &self.operation_name)
                .generate()
                .to_result()?,
        )
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
    pub fn generate(&self, use_transformers: bool) -> anyhow::Result<ConfigModule> {
        let mut config: Config = Config::default();
        let type_name_generator = NameGenerator::new(&self.type_name_prefix);

        for input in self.inputs.iter() {
            match input {
                Input::Config { source, schema } => {
                    config = config.merge_right(Config::from_source(source.clone(), schema)?);
                }
                Input::Json { url, response, field_name } => {
                    let request_sample =
                        RequestSample::new(url.to_owned(), response.to_owned(), field_name);
                    config = config.merge_right(
                        self.generate_from_json(&type_name_generator, &[request_sample])?,
                    );
                }
                Input::Proto(proto_input) => {
                    config = config
                        .merge_right(self.generate_from_proto(proto_input, &self.operation_name)?);
                }
            }
        }

        if use_transformers {
            for t in &self.transformers {
                config = t.transform(config).to_result()?;
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
    use crate::core::config::transformer::Preset;
    use crate::core::generator::generator::Input;
    use crate::core::generator::NameGenerator;
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

        let cfg_module = Generator::default()
            .inputs(vec![Input::Proto(ProtoMetadata {
                descriptor_set: set,
                path: "../../../tailcall-fixtures/fixtures/protobuf/news.proto".to_string(),
            })])
            .generate(false)?;

        insta::assert_snapshot!(cfg_module.config().to_sdl());
        Ok(())
    }

    #[test]
    fn should_generate_config_from_configs() -> anyhow::Result<()> {
        let cfg_module = Generator::default()
            .inputs(vec![Input::Config {
                schema: std::fs::read_to_string(tailcall_fixtures::configs::USER_POSTS)?,
                source: crate::core::config::Source::GraphQL,
            }])
            .generate(true)?;

        insta::assert_snapshot!(cfg_module.config().to_sdl());
        Ok(())
    }

    #[test]
    fn should_generate_config_from_json() -> anyhow::Result<()> {
        let parsed_content =
            parse_json("src/core/generator/tests/fixtures/json/incompatible_properties.json");
        let cfg_module = Generator::default()
            .inputs(vec![Input::Json {
                url: parsed_content.url.parse()?,
                response: parsed_content.body,
                field_name: "f1".to_string(),
            }])
            .transformers(vec![Box::new(Preset::default())])
            .generate(true)?;
        insta::assert_snapshot!(cfg_module.config().to_sdl());
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
            field_name: "f1".to_string(),
        };

        // Combine inputs
        let cfg_module = Generator::default()
            .inputs(vec![proto_input, json_input, config_input])
            .transformers(vec![Box::new(Preset::default())])
            .generate(true)?;

        // Assert the combined output
        insta::assert_snapshot!(cfg_module.config().to_sdl());
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
        let field_name_generator = NameGenerator::new("f");
        for json_path in json_fixtures {
            let parsed_content = parse_json(json_path);
            inputs.push(Input::Json {
                url: parsed_content.url.parse()?,
                response: parsed_content.body,
                field_name: field_name_generator.next(),
            });
        }

        let cfg_module = Generator::default()
            .inputs(inputs)
            .transformers(vec![Box::new(Preset::default())])
            .generate(true)?;
        insta::assert_snapshot!(cfg_module.config().to_sdl());
        Ok(())
    }
}
