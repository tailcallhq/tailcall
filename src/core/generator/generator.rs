use std::collections::BTreeMap;

use derive_setters::Setters;
use prost_reflect::prost_types::FileDescriptorSet;
use prost_reflect::DescriptorPool;
use serde_json::Value;
use url::Url;

use super::from_proto::from_proto;
use super::{FromJsonGenerator, NameGenerator, RequestSample};
use crate::core::config::{self, Config, ConfigModule, Link, LinkType};
use crate::core::http::Method;
use crate::core::merge_right::MergeRight;
use crate::core::mustache::TemplateString;
use crate::core::proto_reader::ProtoMetadata;
use crate::core::transform::{Transform, TransformerOps};
use crate::core::valid::Validator;

/// Generator offers an abstraction over the actual config generators and allows
/// to generate the single config from multiple sources. i.e (Protobuf and Json)

#[derive(Setters)]
pub struct Generator {
    query: String,
    mutation: Option<String>,
    inputs: Vec<Input>,
    type_name_prefix: String,
    transformers: Vec<Box<dyn Transform<Value = Config, Error = String>>>,
}

#[derive(Clone)]
pub enum Input {
    Json {
        url: Url,
        method: Method,
        req_body: Value,
        res_body: Value,
        field_name: String,
        is_mutation: bool,
        headers: Option<BTreeMap<String, TemplateString>>,
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
            query: "Query".into(),
            mutation: None,
            inputs: Vec::new(),
            type_name_prefix: "T".into(),
            transformers: Default::default(),
        }
    }

    /// Generates configuration from the provided json samples.
    fn generate_from_json(
        &self,
        type_name_generator: &NameGenerator,
        json_samples: &[RequestSample],
    ) -> anyhow::Result<Config> {
        Ok(FromJsonGenerator::new(
            json_samples,
            type_name_generator,
            &self.query,
            &self.mutation,
        )
        .generate()
        .to_result()?)
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
                Input::Json {
                    url,
                    method,
                    req_body,
                    res_body,
                    field_name,
                    is_mutation,
                    headers,
                } => {
                    let req_sample = RequestSample::new(
                        url.to_owned(),
                        res_body.to_owned(),
                        field_name.to_owned(),
                    )
                    .with_method(method.to_owned())
                    .with_headers(headers.to_owned())
                    .with_is_mutation(is_mutation.to_owned())
                    .with_req_body(req_body.to_owned());

                    config = config
                        .merge_right(self.generate_from_json(&type_name_generator, &[req_sample])?);
                }
                Input::Proto(proto_input) => {
                    config =
                        config.merge_right(self.generate_from_proto(proto_input, &self.query)?);
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
pub mod test {
    use std::collections::BTreeMap;

    use prost_reflect::prost_types::FileDescriptorSet;
    use serde::{Deserialize, Deserializer};
    use serde_json::Value;
    use url::Url;

    use super::Generator;
    use crate::core::config::transformer::Preset;
    use crate::core::generator::generator::Input;
    use crate::core::http::Method;
    use crate::core::mustache::TemplateString;
    use crate::core::proto_reader::ProtoMetadata;

    fn compile_protobuf(files: &[&str]) -> anyhow::Result<FileDescriptorSet> {
        Ok(protox::compile(files, [tailcall_fixtures::protobuf::SELF])?)
    }

    #[derive(Deserialize)]
    pub struct Request {
        pub url: Url,
        pub method: Method,
        pub body: Option<Value>,
        pub headers: Option<BTreeMap<String, TemplateString>>,
    }

    pub struct JsonFixture {
        pub request: Request,
        pub response: Value,
        pub is_mutation: bool,
        pub field_name: String,
    }

    impl JsonFixture {
        pub async fn read(path: &str) -> anyhow::Result<JsonFixture> {
            let content = tokio::fs::read_to_string(path).await?;
            let result: JsonFixture = serde_json::from_str(&content)?;
            Ok(result)
        }
    }

    impl<'de> Deserialize<'de> for JsonFixture {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let json_content: Value = Value::deserialize(deserializer)?;

            let req_value = json_content
                .get("request")
                .ok_or_else(|| serde::de::Error::missing_field("request"))?;

            let request = serde_json::from_value(req_value.to_owned()).unwrap();

            let response = json_content
                .get("response")
                .and_then(|resp| resp.get("body"))
                .cloned()
                .ok_or_else(|| serde::de::Error::missing_field("response.body"))?;

            let is_mutation = json_content
                .get("is_mutation")
                .ok_or_else(|| serde::de::Error::missing_field("isMutation"))?
                .as_bool()
                .unwrap_or_default();

            let field_name = json_content
                .get("fieldName")
                .ok_or_else(|| serde::de::Error::missing_field("fieldName"))?
                .as_str()
                .unwrap_or_default();

            Ok(JsonFixture {
                request,
                response,
                is_mutation,
                field_name: field_name.to_owned(),
            })
        }
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
                schema: std::fs::read_to_string(tailcall_fixtures::configs::JSONPLACEHOLDER)?,
                source: crate::core::config::Source::GraphQL,
            }])
            .generate(true)?;

        insta::assert_snapshot!(cfg_module.config().to_sdl());
        Ok(())
    }

    #[tokio::test]
    async fn should_generate_config_from_json() -> anyhow::Result<()> {
        let JsonFixture { request, response, field_name, is_mutation } = JsonFixture::read(
            "src/core/generator/tests/fixtures/json/incompatible_properties.json",
        )
        .await?;
        let cfg_module = Generator::default()
            .inputs(vec![Input::Json {
                url: request.url,
                method: request.method,
                req_body: request.body.unwrap_or_default(),
                res_body: response,
                field_name,
                is_mutation,
                headers: request.headers,
            }])
            .transformers(vec![Box::new(Preset::default())])
            .generate(true)?;
        insta::assert_snapshot!(cfg_module.config().to_sdl());
        Ok(())
    }

    #[tokio::test]
    async fn should_generate_combined_config() -> anyhow::Result<()> {
        // Proto input
        let news_proto = tailcall_fixtures::protobuf::NEWS;
        let proto_set = compile_protobuf(&[news_proto])?;
        let proto_input = Input::Proto(ProtoMetadata {
            descriptor_set: proto_set,
            path: "../../../tailcall-fixtures/fixtures/protobuf/news.proto".to_string(),
        });

        // Config input
        let config_input = Input::Config {
            schema: std::fs::read_to_string(tailcall_fixtures::configs::JSONPLACEHOLDER)?,
            source: crate::core::config::Source::GraphQL,
        };

        // Json Input
        let JsonFixture { request, response, field_name, is_mutation } = JsonFixture::read(
            "src/core/generator/tests/fixtures/json/incompatible_properties.json",
        )
        .await?;
        let json_input = Input::Json {
            url: request.url,
            method: request.method,
            req_body: request.body.unwrap_or_default(),
            res_body: response,
            field_name,
            is_mutation,
            headers: request.headers,
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

    #[tokio::test]
    async fn generate_from_config_from_multiple_jsons() -> anyhow::Result<()> {
        let mut inputs = vec![];
        let json_fixtures = [
            "src/core/generator/tests/fixtures/json/incompatible_properties.json",
            "src/core/generator/tests/fixtures/json/list_incompatible_object.json",
            "src/core/generator/tests/fixtures/json/list.json",
        ];
        for json_path in json_fixtures {
            let JsonFixture { request, response, field_name, is_mutation } =
                JsonFixture::read(json_path).await?;
            inputs.push(Input::Json {
                url: request.url,
                method: request.method,
                req_body: request.body.unwrap_or_default(),
                res_body: response,
                field_name,
                is_mutation,
                headers: request.headers,
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
