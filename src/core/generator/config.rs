use std::path::Path;

use path_clean::PathClean;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::core::config;

use super::source::ImportSource;

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum InputSource {
    Config { src: String },
    Import { src: String },
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Input {
    #[serde(flatten)]
    pub source: InputSource,
}

#[derive(Serialize, Deserialize, Default, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    /// Controls the output format (graphql, json, yaml)
    pub format: config::Source,
    /// Specifies the output file name
    pub file: String,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    #[serde(default = "defaults::schema::query")]
    pub query: String,
    #[serde(default = "defaults::schema::mutation")]
    pub mutation: String,
}

impl Default for Schema {
    fn default() -> Self {
        Self {
            query: defaults::schema::query(),
            mutation: defaults::schema::mutation(),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GenerateOptions {
    #[serde(default)]
    pub schema: Schema,
}

#[derive(Serialize, Deserialize, Default, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Transform {
    // TODO: change types
    pub ambiguous_name_resolver: Option<serde_json::Value>,
    pub merge_types: Option<serde_json::Value>,
    pub js: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Default, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GeneratorConfig {
    pub input: Vec<Input>,
    #[serde(default)]
    pub output: Output,
    #[serde(default)]
    pub generate: GenerateOptions,
    #[serde(default)]
    pub transform: Transform,
}

impl GeneratorConfig {
    /// Resolves all the paths present inside the GeneratorConfig.
    pub fn resolve_paths(mut self, file_path: &str) -> anyhow::Result<Self> {
        let config_path = Path::new(file_path).parent().unwrap_or(Path::new("."));

        for input in self.input.iter_mut() {
            if let InputSource::Import { src } = &mut input.source {
                match ImportSource::detect(&src)? {
                    ImportSource::Proto => {
                        let src_path = Path::new(&src);
                        if !src_path.is_relative() {
                            continue;
                        }

                        let cleaned_path = config_path.join(src_path).clean();
                        *src = cleaned_path.to_string_lossy().into_owned();
                    }
                    _ => {}
                }
            }
        }

        let output_config_path = Path::new(&self.output.file);
        if output_config_path.is_relative() {
            let cleaned_path = config_path.join(output_config_path).clean();
            self.output.file = cleaned_path.to_string_lossy().into_owned();
        }

        Ok(self)
    }
}

mod defaults {
    pub mod schema {
        pub fn query() -> String {
            "Query".to_string()
        }

        pub fn mutation() -> String {
            "Mutation".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;

    use super::GeneratorConfig;

    #[test]
    fn test_from_json() {
        let content = std::fs::read_to_string(tailcall_fixtures::generator::SIMPLE_JSON).unwrap();
        let config: GeneratorConfig = serde_json::from_str(&content).unwrap();

        assert_debug_snapshot!(&config);
    }
}
