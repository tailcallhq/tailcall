use std::collections::BTreeMap;
use std::env;
use std::marker::PhantomData;
use std::path::Path;

use path_clean::PathClean;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::core::config::{self};

#[derive(Deserialize, Debug)]
pub struct Config<Status = UnResolved> {
    pub inputs: Vec<Input<Status>>,
    pub output: Output<Status>,
    pub transformers: Vec<Transformer>,
    pub schema: Schema,
    #[serde(skip)]
    _marker: PhantomData<Status>,
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

#[derive(Deserialize, Debug)]
pub enum Source<Status = UnResolved> {
    URL {
        url: String,
        headers: Option<BTreeMap<String, String>>,
        method: Option<Method>,
        body: Option<serde_json::Value>,
        #[serde(skip)]
        _marker: PhantomData<Status>,
    },
    Proto {
        path: String,
        #[serde(skip)]
        _marker: PhantomData<Status>,
    },
    Config {
        url: String,
        #[serde(skip)]
        _marker: PhantomData<Status>,
    },
}

#[derive(Deserialize, Debug)]
pub struct Output<Status = UnResolved> {
    pub path: String,
    pub format: Option<config::Source>,
    #[serde(skip)]
    _marker: PhantomData<Status>,
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
            Source::URL { url, headers, method, body, _marker } => {
                let resolved_url = resolve(url.as_str(), parent_dir)?;
                Ok(Source::URL {
                    url: resolved_url,
                    headers,
                    method,
                    body,
                    _marker: PhantomData,
                })
            }
            Source::Proto { path, .. } => {
                let resolved_path = resolve(path.as_str(), parent_dir)?;
                Ok(Source::Proto { path: resolved_path, _marker: PhantomData })
            }
            Source::Config { url, .. } => {
                let resolved_url = resolve(url.as_str(), parent_dir)?;
                Ok(Source::Config { url: resolved_url, _marker: PhantomData })
            }
        }
    }
}

#[derive(Deserialize, Debug)]
pub enum Operation {
    Query,
    Mutation,
}

#[derive(Debug)]
pub enum Resolved {}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
pub enum UnResolved {}

#[derive(Deserialize, Debug)]
pub enum Method {
    GET,
}

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
    BetterTypeName,
    TreeShake,
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

impl Config {
    /// Resolves all the relative paths present inside the GeneratorConfig.
    pub fn resolve_paths(self, config_path: &str) -> anyhow::Result<Config<Resolved>> {
        let parent_dir = Some(Path::new(config_path).parent().unwrap_or(Path::new("")));

        let resolved_inputs = self
            .inputs
            .into_iter()
            .map(|input| input.resolve(parent_dir))
            .collect::<anyhow::Result<Vec<Input<Resolved>>>>()?;

        Ok(Config {
            inputs: resolved_inputs,
            output: self.output.resolve(parent_dir)?,
            transformers: self.transformers,
            schema: self.schema,
            _marker: PhantomData,
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
