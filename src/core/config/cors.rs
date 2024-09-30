use http::header;
use serde::{Deserialize, Serialize};

use crate::core::http::Method;
use crate::core::is_default;
use crate::core::macros::MergeRight;

/// Type to configure Cross-Origin Resource Sharing (CORS) for a server.
#[derive(
    Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, schemars::JsonSchema, MergeRight,
)]
#[serde(rename_all = "camelCase")]
pub struct Cors {
    /// Indicates whether the server allows credentials (e.g., cookies,
    /// authorization headers) to be sent in cross-origin requests.
    #[serde(default, skip_serializing_if = "is_default")]
    pub allow_credentials: Option<bool>,

    /// A list of allowed headers in cross-origin requests.
    /// This can be used to specify custom headers that are allowed to be
    /// included in cross-origin requests.
    #[serde(default, skip_serializing_if = "is_default")]
    pub allow_headers: Vec<String>,

    /// A list of allowed HTTP methods in cross-origin requests.
    /// These methods specify the actions that are permitted in cross-origin
    /// requests.
    #[serde(default, skip_serializing_if = "is_default")]
    pub allow_methods: Vec<Method>,

    /// A list of origins that are allowed to access the server's resources in
    /// cross-origin requests. An origin can be a domain, a subdomain, or
    /// even 'null' for local file schemes.
    #[serde(default, skip_serializing_if = "is_default")]
    pub allow_origins: Vec<String>,

    /// Indicates whether requests from private network addresses are allowed in
    /// cross-origin requests. Private network addresses typically include
    /// IP addresses reserved for internal networks.
    #[serde(default, skip_serializing_if = "is_default")]
    pub allow_private_network: Option<bool>,

    /// A list of headers that the server exposes to the browser in cross-origin
    /// responses. Exposing certain headers allows the client-side code to
    /// access them in the response.
    #[serde(default, skip_serializing_if = "is_default")]
    pub expose_headers: Vec<String>,

    /// The maximum time (in seconds) that the client should cache preflight
    /// OPTIONS requests in order to avoid sending excessive requests to the
    /// server.
    #[serde(default, skip_serializing_if = "is_default")]
    pub max_age: Option<usize>,

    /// A list of header names that indicate the values of which might cause the
    /// server's response to vary, potentially affecting caching.
    #[serde(
        default = "preflight_request_headers",
        skip_serializing_if = "is_default"
    )]
    pub vary: Vec<String>,
}

fn preflight_request_headers() -> Vec<String> {
    vec![
        header::ORIGIN.to_string(),
        header::ACCESS_CONTROL_REQUEST_METHOD.to_string(),
        header::ACCESS_CONTROL_REQUEST_HEADERS.to_string(),
    ]
}
