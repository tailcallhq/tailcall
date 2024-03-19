use hyper::header;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CorsParams {
    #[serde(default)]
    pub allow_credentials: bool,
    #[serde(default)]
    pub allow_headers: Option<Vec<String>>,
    #[serde(default)]
    pub allow_methods: Option<Vec<String>>,
    #[serde(default)]
    pub allow_origin: Vec<String>,
    #[serde(default)]
    pub allow_private_network: bool,
    #[serde(default)]
    pub expose_headers: Vec<String>,
    #[serde(default)]
    pub max_age: Option<usize>,
    #[serde(default = "preflight_request_headers")]
    pub vary: Vec<String>,
}

fn preflight_request_headers() -> Vec<String> {
    vec![
        header::ORIGIN.to_string(),
        header::ACCESS_CONTROL_REQUEST_METHOD.to_string(),
        header::ACCESS_CONTROL_REQUEST_HEADERS.to_string(),
    ]
}
