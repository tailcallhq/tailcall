use std::fs;

use inquire::Confirm;

use super::config::GeneratorConfig;
use super::source::ConfigSource;
use super::Generator;
use crate::core::config::{self, ConfigModule};
use crate::core::runtime::TargetRuntime;

/// Checks if file or folder already exists or not.
fn is_exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}

/// FIXME: move this to CLI
pub struct ConfigConsoleGenerator {
    config_path: String,
    runtime: TargetRuntime,
    generator: Generator,
}

impl ConfigConsoleGenerator {
    pub fn new(config_path: &str, runtime: TargetRuntime) -> Self {
        Self {
            generator: Generator::new(runtime.clone()),
            config_path: config_path.to_string(),
            runtime,
        }
    }

    /// Writes the configuration to the output file if allowed.
    async fn write(self, graphql_config: ConfigModule, output_path: &str) -> anyhow::Result<()> {
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

    async fn read(&self) -> anyhow::Result<GeneratorConfig> {
        let config_path = &self.config_path;
        let source = ConfigSource::detect(config_path)?;
        let config_content = self.runtime.file.read(config_path).await?;

        let config: GeneratorConfig = match source {
            ConfigSource::Json => serde_json::from_str(&config_content)?,
            ConfigSource::Yml => serde_yaml::from_str(&config_content)?,
        };

        // While reading resolve the internal paths of generalized config.
        Ok(config.resolve_paths(config_path))
    }

    /// Reads the configuration from the specified path.
    pub async fn generate(self) -> anyhow::Result<()> {
        let config = self.read().await?;
        let path = config.output.file.to_owned();
        let config = self.generator.run(config.clone()).await?;

        self.write(config, &path).await?;
        Ok(())
    }
}
