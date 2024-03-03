use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CorsParams {
    pub allow_credentials: Option<bool>,
    pub allow_headers: ConstOrMirror,
    pub allow_methods: ConstOrMirror,
    pub allow_origin: Vec<String>,
    pub allow_private_network: bool,
    pub expose_headers: Option<String>,
    pub max_age: Option<usize>,
    pub vary: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, schemars::JsonSchema)]
#[serde(untagged)]
pub enum ConstOrMirror {
    Const(Option<String>),
    #[default]
    MirrorRequest,
}
