use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::core::config::{self, reader::ConfigReader};

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum InputSource {
    /// src: maintains the same src written in config.
    /// resolved_src: holds the correctly resolved src with respect to config.
    Config {
        src: String,
        resolved_src: Option<String>,
    },
    Import {
        src: String,
        resolved_src: Option<String>,
    },
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
    pub fn resolve_paths(mut self, file_path: &str) -> Self {
        let config_path = Path::new(file_path).parent().unwrap_or(Path::new(""));

        for input in self.input.iter_mut() {
            match &mut input.source {
                InputSource::Config { src, resolved_src }
                | InputSource::Import { src, resolved_src } => {
                    *resolved_src = Some(ConfigReader::resolve_path(src, Some(config_path)));
                }
            }
        }

        self.output.file = ConfigReader::resolve_path(&self.output.file, Some(config_path));

        self
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

    #[test]
    fn test_resolve_paths() {
        let file_path = "../../../tailcall-fixtures/fixtures/generator/simple-json.json";
        let content = std::fs::read_to_string(tailcall_fixtures::generator::SIMPLE_JSON).unwrap();
        let config: GeneratorConfig = serde_json::from_str(&content).unwrap();
        let config = config.resolve_paths(file_path);
        assert_debug_snapshot!(&config);
    }
}
