use std::env;
use std::marker::PhantomData;
use std::path::Path;

use path_clean::PathClean;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::core::config::{self};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config<Status = UnResolved> {
    pub inputs: Vec<Input<Status>>,
    pub output: Output<Status>,
    pub preset: Option<Preset>,
    pub schema: Schema,
}

#[derive(Clone, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Preset {
    merge_type: Option<f32>,
    consolidate_url: Option<f32>,
}

impl From<Preset> for config::transformer::Preset {
    fn from(val: Preset) -> Self {
        let mut preset = config::transformer::Preset::default();
        if let Some(merge_type) = val.merge_type {
            preset = preset.merge_type(merge_type);
        }

        if let Some(consolidate_url) = val.consolidate_url {
            preset = preset.consolidate_url(consolidate_url);
        }

        preset
    }
}

#[derive(Deserialize, Debug)]
#[serde(transparent)]
pub struct Location<A>(pub String, #[serde(skip)] PhantomData<A>);

impl Location<UnResolved> {
    fn into_resolved(self, parent_dir: Option<&Path>) -> Location<Resolved> {
        let path = {
            let path = self.0.as_str();
            if Url::parse(path).is_ok() || Path::new(path).is_absolute() {
                path.to_string()
            } else {
                let parent_dir = parent_dir.unwrap_or(Path::new(""));
                let joined_path = parent_dir.join(path);
                if let Ok(abs_path) = std::fs::canonicalize(&joined_path) {
                    abs_path.to_string_lossy().to_string()
                } else if let Ok(cwd) = env::current_dir() {
                    cwd.join(joined_path).clean().to_string_lossy().to_string()
                } else {
                    joined_path.clean().to_string_lossy().to_string()
                }
            }
        };
        Location(path, PhantomData::default())
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Input<Status = UnResolved> {
    #[serde(flatten)]
    pub source: Source<Status>,
    pub field_name: String,
    pub operation: Option<Operation>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Source<Status = UnResolved> {
    Curl { src: Location<Status> },
    Proto { src: Location<Status> },
    Config { src: Location<Status> },
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Output<Status = UnResolved> {
    pub path: Location<Status>,
    pub format: Option<config::Source>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Operation {
    Query,
    Mutation,
}

#[derive(Debug)]
pub enum Resolved {}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub enum UnResolved {}

#[derive(Deserialize, Debug)]
pub enum Transformer {
    TypeMerger {
        threshold: Option<f32>,
    },
    AmbiguousType {
        input: Option<Name>,
        output: Option<Name>,
    },
    ConsolidateBaseURL {
        threshold: Option<f32>,
    },
    BetterTypeName(Option<bool>),
    TreeShake(Option<bool>),
}

#[derive(Deserialize, Debug)]
pub struct Name {
    pub prefix: Option<String>,
    pub postfix: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Schema {
    pub query: Option<String>,
    pub mutation: Option<String>,
}

impl Output<UnResolved> {
    pub fn resolve(self, parent_dir: Option<&Path>) -> anyhow::Result<Output<Resolved>> {
        Ok(Output {
            format: self.format,
            path: self.path.into_resolved(parent_dir),
        })
    }
}

impl Source<UnResolved> {
    pub fn resolve(self, parent_dir: Option<&Path>) -> anyhow::Result<Source<Resolved>> {
        match self {
            Source::Curl { src } => {
                let resolved_path = src.into_resolved(parent_dir);
                Ok(Source::Curl { src: resolved_path })
            }
            Source::Proto { src, .. } => {
                let resolved_path = src.into_resolved(parent_dir);
                Ok(Source::Proto { src: resolved_path })
            }
            Source::Config { src, .. } => {
                let resolved_path = src.into_resolved(parent_dir);
                Ok(Source::Config { src: resolved_path })
            }
        }
    }
}

impl Input<UnResolved> {
    pub fn resolve(self, parent_dir: Option<&Path>) -> anyhow::Result<Input<Resolved>> {
        let resolved_source = self.source.resolve(parent_dir)?;
        Ok(Input {
            source: resolved_source,
            field_name: self.field_name,
            operation: self.operation,
        })
    }
}

impl Config {
    /// Resolves all the relative paths present inside the GeneratorConfig.
    pub fn into_resolved(self, config_path: &str) -> anyhow::Result<Config<Resolved>> {
        let parent_dir = Some(Path::new(config_path).parent().unwrap_or(Path::new("")));

        let inputs = self
            .inputs
            .into_iter()
            .map(|input| input.resolve(parent_dir))
            .collect::<anyhow::Result<Vec<Input<Resolved>>>>()?;

        let output = self.output.resolve(parent_dir)?;

        Ok(Config { inputs, output, schema: self.schema, preset: self.preset })
    }
}