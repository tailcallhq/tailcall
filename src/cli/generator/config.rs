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
    #[serde(skip)]
    _marker: PhantomData<Status>,
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
#[serde(rename_all = "camelCase")]
pub struct Input<Status = UnResolved> {
    #[serde(flatten)]
    pub source: Source<Status>,
    pub field_name: String,
    pub operation: Option<Operation>,
    #[serde(skip)]
    _marker: PhantomData<Status>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Source<Status = UnResolved> {
    Curl {
        src: String,
        #[serde(skip)]
        _marker: PhantomData<Status>,
    },
    Proto {
        src: String,
        #[serde(skip)]
        _marker: PhantomData<Status>,
    },
    Config {
        src: String,
        #[serde(skip)]
        _marker: PhantomData<Status>,
    },
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Output<Status = UnResolved> {
    pub path: String,
    pub format: Option<config::Source>,
    #[serde(skip)]
    _marker: PhantomData<Status>,
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
            path: resolve(&self.path, parent_dir)?,
            _marker: PhantomData,
        })
    }
}

impl Source<UnResolved> {
    pub fn resolve(self, parent_dir: Option<&Path>) -> anyhow::Result<Source<Resolved>> {
        match self {
            Source::Curl { src: url, _marker } => {
                let resolved_url = resolve(url.as_str(), parent_dir)?;
                Ok(Source::Curl { src: resolved_url, _marker: PhantomData })
            }
            Source::Proto { src: path, .. } => {
                let resolved_path = resolve(path.as_str(), parent_dir)?;
                Ok(Source::Proto { src: resolved_path, _marker: PhantomData })
            }
            Source::Config { src: url, .. } => {
                let resolved_url = resolve(url.as_str(), parent_dir)?;
                Ok(Source::Config { src: resolved_url, _marker: PhantomData })
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
            _marker: PhantomData,
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

        Ok(Config {
            inputs,
            output,
            schema: self.schema,
            _marker: PhantomData,
            preset: self.preset,
        })
    }
}

// TODO: In our codebase we've similar functions like below, create a separate
// module for helpers functions like these.
fn resolve(path: &str, parent_dir: Option<&Path>) -> anyhow::Result<String> {
    if Url::parse(path).is_ok() || Path::new(path).is_absolute() {
        return Ok(path.to_string());
    }

    let parent_dir = parent_dir.unwrap_or(Path::new(""));
    let joined_path = parent_dir.join(path);
    if let Ok(abs_path) = std::fs::canonicalize(&joined_path) {
        return Ok(abs_path.to_string_lossy().to_string());
    }
    if let Ok(cwd) = env::current_dir() {
        return Ok(cwd.join(joined_path).clean().to_string_lossy().to_string());
    }

    Ok(joined_path.clean().to_string_lossy().to_string())
}
