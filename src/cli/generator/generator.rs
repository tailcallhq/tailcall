use std::fs;
use std::path::Path;

use hyper::header::{HeaderName, HeaderValue};
use hyper::HeaderMap;
use inquire::Confirm;
use pathdiff::diff_paths;

use super::config::{Config, Resolved, Source};
use super::source::ConfigSource;
use crate::core::config::{self, ConfigModule, ConfigReaderContext};
use crate::core::generator::{Generator as ConfigGenerator, Input};
use crate::core::proto_reader::ProtoReader;
use crate::core::resource_reader::{Resource, ResourceReader};
use crate::core::runtime::TargetRuntime;

/// CLI that reads the the config file and generates the required tailcall
/// configuration.
pub struct Generator {
    /// path of config file.
    config_path: String,
    runtime: TargetRuntime,
}

impl Generator {
    pub fn new(config_path: &str, runtime: TargetRuntime) -> Self {
        Self { config_path: config_path.to_string(), runtime }
    }

    /// Writes the configuration to the output file if allowed.
    async fn write(self, graphql_config: &ConfigModule, output_path: &str) -> anyhow::Result<()> {
        let output_source = config::Source::detect(output_path)?;
        let config = match output_source {
            config::Source::Json => graphql_config.to_json(true)?,
            config::Source::Yml => graphql_config.to_yaml()?,
            config::Source::GraphQL => graphql_config.to_sdl(),
        };

        if self.should_overwrite(output_path)? {
            self.runtime
                .file
                .write(output_path, config.as_bytes())
                .await?;

            tracing::info!("Config successfully generated at {output_path}");
        }

        Ok(())
    }

    /// Checks if the output file already exists and prompts for overwrite
    /// confirmation.
    fn should_overwrite(&self, output_path: &str) -> anyhow::Result<bool> {
        if is_exists(output_path) {
            let should_overwrite = Confirm::new(
                format!(
                    "The output file '{}' already exists. Do you want to overwrite it?",
                    output_path
                )
                .as_str(),
            )
            .with_default(false)
            .prompt()?;
            if !should_overwrite {
                return Ok(false);
            }
        }
        Ok(true)
    }

    async fn read(&self) -> anyhow::Result<Config<Resolved>> {
        let config_path = &self.config_path;
        let source = ConfigSource::detect(config_path)?;
        let config_content = self.runtime.file.read(config_path).await?;

        let config: Config = match source {
            ConfigSource::Json => serde_json::from_str(&config_content)?,
            ConfigSource::Yml => serde_yaml::from_str(&config_content)?,
        };

        // While reading resolve the internal paths and mustache headers of generalized
        // config.
        let reader_context = ConfigReaderContext {
            runtime: &self.runtime,
            vars: &Default::default(),
            headers: Default::default(),
        };
        config.into_resolved(config_path, reader_context)
    }

    /// performs all the i/o's required in the config file and generates
    /// concrete vec containing data for generator.
    async fn resolve_io(&self, config: Config<Resolved>) -> anyhow::Result<Vec<Input>> {
        let mut input_samples = vec![];

        let reader = ResourceReader::cached(self.runtime.clone());
        let proto_reader = ProtoReader::init(reader.clone(), self.runtime.clone());
        let output_dir = Path::new(&config.output.path.0)
            .parent()
            .unwrap_or(Path::new(""));

        for input in config.inputs {
            match input.source {
                Source::Curl { src, field_name, headers: resolved_headers } => {
                    let url = src.0;
                    let mut request = reqwest::Request::new(reqwest::Method::GET, url.parse()?);
                    if let Some(headers_inner) = resolved_headers.headers() {
                        let mut header_map = HeaderMap::new();
                        for (key, value) in headers_inner {
                            let header_name = HeaderName::try_from(key)?;
                            let header_value = HeaderValue::try_from(value)?;
                            header_map.insert(header_name, header_value);
                        }
                        *request.headers_mut() = header_map;
                    }
                    let resource: Resource = request.into();
                    let response = reader.read_file(resource).await?;
                    input_samples.push(Input::Json {
                        url: url.parse()?,
                        response: serde_json::from_str(&response.content)?,
                        field_name,
                    });
                }
                Source::Proto { src } => {
                    let path = src.0;
                    let mut metadata = proto_reader.read(&path).await?;
                    if let Some(relative_path_to_proto) = to_relative_path(output_dir, &path) {
                        metadata.path = relative_path_to_proto;
                    }
                    input_samples.push(Input::Proto(metadata));
                }
                Source::Config { src } => {
                    let path = src.0;
                    let source = config::Source::detect(&path)?;
                    let schema = reader.read_file(path).await?.content;
                    input_samples.push(Input::Config { schema, source });
                }
            }
        }

        Ok(input_samples)
    }

    /// generates the final configuration.
    pub async fn generate(self) -> anyhow::Result<ConfigModule> {
        let config = self.read().await?;
        let path = config.output.path.0.to_owned();
        let query_type = config.schema.query.clone();
        let preset: config::transformer::Preset = config.preset.clone().unwrap_or_default().into();
        let input_samples = self.resolve_io(config).await?;

        let mut config_gen = ConfigGenerator::default()
            .inputs(input_samples)
            .transformers(vec![Box::new(preset)]);
        if let Some(query_type_name) = query_type {
            // presently only query opeartion is supported.
            config_gen = config_gen.operation_name(query_type_name);
        }

        let config = config_gen.generate(true)?;

        self.write(&config, &path).await?;
        Ok(config)
    }
}

/// Checks if file or folder already exists or not.
fn is_exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}

/// Expects both paths to be absolute and returns a relative path from `from` to
/// `to`. expects `from`` to be directory.
fn to_relative_path(from: &Path, to: &str) -> Option<String> {
    let from_path = Path::new(from).to_path_buf();
    let to_path = Path::new(to).to_path_buf();

    // Calculate the relative path from `from_path` to `to_path`
    diff_paths(to_path, from_path).map(|p| p.to_string_lossy().to_string())
}
