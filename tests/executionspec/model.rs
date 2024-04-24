use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
};

use derive_setters::Setters;
use serde::{Deserialize, Serialize};
use tailcall::config::Source;

use super::runtime::{APIRequest, Mock};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Annotation {
    Skip,
    Only,
}

#[derive(Clone, Setters)]
pub struct ExecutionSpec {
    pub path: PathBuf,
    pub name: String,
    pub safe_name: String,

    pub server: Vec<(Source, String)>,
    pub mock: Option<Vec<Mock>>,
    pub env: Option<HashMap<String, String>>,
    pub test: Option<Vec<APIRequest>>,
    pub files: BTreeMap<String, String>,

    // Annotations for the runner
    pub runner: Option<Annotation>,

    pub check_identity: bool,
    pub sdl_error: bool,
}
