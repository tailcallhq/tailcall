use hyper::header;
use hyper::http::HeaderValue;
use serde::{Deserialize, Serialize};
use serde_unit_struct::{Deserialize_unit_struct, Serialize_unit_struct};

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CorsParams {
    #[serde(default)]
    pub allow_credentials: bool,
    #[serde(default)]
    pub allow_headers: ConstOrMirror,
    #[serde(default)]
    pub allow_methods: ConstOrMirror,
    #[serde(default)]
    pub allow_origin: StringOrSequence,
    #[serde(default)]
    pub allow_private_network: bool,
    #[serde(default)]
    pub expose_headers: StringOrSequence,
    #[serde(default)]
    pub max_age: Option<usize>,
    #[serde(default = "preflight_request_headers")]
    pub vary: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, schemars::JsonSchema)]
#[serde(untagged)]
pub enum ConstOrMirror {
    MirrorRequest(MirrorRequest),
    Const(Option<String>),
}

#[derive(
    Serialize_unit_struct,
    Deserialize_unit_struct,
    Clone,
    Debug,
    PartialEq,
    Eq,
    schemars::JsonSchema,
)]
pub struct MirrorRequest;

impl Default for ConstOrMirror {
    fn default() -> Self {
        Self::Const(None)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, schemars::JsonSchema)]
#[serde(untagged)]
pub enum StringOrSequence {
    String(String),
    Sequence(Vec<String>),
}

impl Default for StringOrSequence {
    fn default() -> Self {
        Self::Sequence(vec![])
    }
}

impl TryFrom<StringOrSequence> for Vec<HeaderValue> {
    type Error = anyhow::Error;

    fn try_from(value: StringOrSequence) -> anyhow::Result<Self> {
        match value {
            StringOrSequence::String(string) => Ok(vec![string.parse()?]),
            StringOrSequence::Sequence(sequence) => sequence
                .into_iter()
                .map(|val| Ok(val.parse()?))
                .collect::<anyhow::Result<Vec<_>>>(),
        }
    }
}

impl TryFrom<StringOrSequence> for HeaderValue {
    type Error = anyhow::Error;

    fn try_from(value: StringOrSequence) -> anyhow::Result<Self> {
        Ok(match value {
            StringOrSequence::String(string) => string.parse()?,
            StringOrSequence::Sequence(sequence) => sequence[..].join(", ").parse()?,
        })
    }
}

pub fn preflight_request_headers() -> Vec<String> {
    vec![
        header::ORIGIN.to_string(),
        header::ACCESS_CONTROL_REQUEST_METHOD.to_string(),
        header::ACCESS_CONTROL_REQUEST_HEADERS.to_string(),
    ]
}
