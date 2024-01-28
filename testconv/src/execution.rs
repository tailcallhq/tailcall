use std::collections::HashMap;
use std::path::PathBuf;

use derive_setters::Setters;
use serde::{Deserialize, Serialize};
use tailcall::config::Config;

use crate::common::{Annotation, DownstreamRequest, Mock};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DownstreamAssertion {
    pub request: DownstreamRequest,
}

#[derive(Serialize, Deserialize, Clone, Setters, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AssertSpec {
    #[serde(default)]
    pub mock: Vec<Mock>,

    pub assert: Vec<DownstreamAssertion>,

    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Clone, Setters, Debug)]
pub struct ExecutionSpec {
    pub path: PathBuf,
    pub name: String,
    pub safe_name: String,

    pub server: Vec<Config>,
    pub assert: Option<AssertSpec>,

    // Annotations for the runner
    pub runner: Option<Annotation>,
}
