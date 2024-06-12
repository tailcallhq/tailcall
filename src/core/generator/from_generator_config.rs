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

pub struct Writer {
    pub config: ConfigModule,
    runtime: TargetRuntime,
    output_path: String,
}

impl Writer {
    pub fn new(output_path: String, config: ConfigModule, runtime: TargetRuntime) -> Self {
        Self { config, runtime, output_path }
    }

    /// Checks if the output file already exists and prompts for overwrite
    /// confirmation.
    pub fn should_overwrite(&self, output_path: &str) -> anyhow::Result<bool> {
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

    /// Writes the configuration to the output file if allowed.
    pub async fn write(self) -> anyhow::Result<()> {
        let output_path = &self.output_path;

        let output_source = config::Source::detect(output_path)?;
        let config = match output_source {
            config::Source::Json => self.config.to_json(true)?,
            config::Source::Yml => self.config.to_yaml()?,
            config::Source::GraphQL => self.config.to_sdl(),
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
}

pub struct ConfigGenerator {
    config: GeneratorConfig,
    runtime: TargetRuntime,
    generator: Generator,
}

impl ConfigGenerator {
    pub fn new(config: GeneratorConfig, runtime: TargetRuntime) -> Self {
        Self { generator: Generator::new(runtime.clone()), config, runtime }
    }
    /// generates the actual configuration from generator config.
    pub async fn generate(self) -> anyhow::Result<Writer> {
        let output_path = self.config.output.file.to_owned();
        let output_config = self.generator.run(self.config).await?;
        Ok(Writer::new(output_path, output_config, self.runtime))
    }
}

pub struct Reader {
    runtime: TargetRuntime,
    config_path: String,
}

impl Reader {
    pub fn new(runtime: TargetRuntime, config_path: &str) -> Self {
        Self { runtime, config_path: config_path.to_string() }
    }

    /// Reads the configuration from the specified path.
    pub async fn generate(self) -> anyhow::Result<()> {
        let config_path = &self.config_path;
        let source = ConfigSource::detect(config_path)?;
        let config_content = self.runtime.file.read(config_path).await?;

        let config: GeneratorConfig = match source {
            ConfigSource::Json => serde_json::from_str(&config_content)?,
            ConfigSource::Yml => serde_yaml::from_str(&config_content)?,
        };

        // while reading resolve the internal paths of generalized config.
        let config = config.resolve_paths(config_path);

        ConfigGenerator::new(config, self.runtime)
            .generate()
            .await?
            .write()
            .await?;
        Ok(())
    }
}
