use std::fs;
use std::path::Path;

use inquire::Confirm;
use pathdiff::diff_paths;

use super::config::{Config, InputSource, Resolved};
use crate::core::config::{self, ConfigModule};
use crate::core::generator::source::{ConfigSource, ImportSource};
use crate::core::generator::{ConfigInput, Generator as ConfigGenerator, JsonInput, ProtoInput};
use crate::core::proto_reader::ProtoReader;
use crate::core::resource_reader::ResourceReader;
use crate::core::runtime::TargetRuntime;

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

/// it reads the the config file and generates the required tailcall
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

        // While reading resolve the internal paths of generalized config.
        config.resolve_paths(config_path)
    }

    /// performs all the i/o's required in the config file and generates
    /// concrete vec containing data for generator.
    async fn resolve_io(
        &self,
        config: Config<Resolved>,
    ) -> anyhow::Result<(Vec<JsonInput>, Vec<ConfigInput>, Vec<ProtoInput>)> {
        let mut json_samples = vec![];
        let mut proto_samples = vec![];
        let mut config_samples = vec![];

        let reader = ResourceReader::cached(self.runtime.clone());
        let proto_reader = ProtoReader::init(reader.clone(), self.runtime.clone());
        let output_dir = Path::new(&config.output.file)
            .parent()
            .unwrap_or(Path::new(""));

        for input in config.input {
            match input.source {
                InputSource::Import { src, .. } => {
                    let source = ImportSource::detect(&src)?;
                    match source {
                        ImportSource::Url => {
                            let contents = reader.read_file(&src).await?.content;
                            json_samples.push(JsonInput {
                                url: src.parse()?,
                                data: serde_json::from_str(&contents)?,
                            });
                        }
                        ImportSource::Proto => {
                            let mut metadata = proto_reader.read(&src).await?;
                            if let Some(relative_path_to_proto) = to_relative_path(output_dir, &src)
                            {
                                metadata.path = relative_path_to_proto;
                            }
                            proto_samples.push(ProtoInput { data: metadata });
                        }
                    }
                }
                InputSource::Config { src, .. } => {
                    let source = config::Source::detect(&src)?;
                    let schema = reader.read_file(&src).await?.content;
                    config_samples.push(ConfigInput { schema, source });
                }
            }
        }

        Ok((json_samples, config_samples, proto_samples))
    }

    /// generates the final configuration.
    pub async fn generate(self) -> anyhow::Result<ConfigModule> {
        let config = self.read().await?;
        let path = config.output.file.to_owned();
        let (json_samples, config_samples, proto_samples) = self.resolve_io(config).await?;

        let config = ConfigGenerator::new()
            .with_json_samples(json_samples)
            .with_proto_samples(proto_samples)
            .with_config_samples(config_samples)
            .with_operation_name("Query")
            .generate()?;

        self.write(&config, &path).await?;
        Ok(config)
    }
}
