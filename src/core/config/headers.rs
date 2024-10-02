use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::core::config::cors::Cors;
use crate::core::config::KeyValue;
use crate::core::is_default;
use crate::core::macros::MergeRight;

#[derive(
    Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, schemars::JsonSchema, MergeRight,
)]
#[serde(rename_all = "camelCase")]
pub struct Headers {
    #[serde(default, skip_serializing_if = "is_default")]
    /// `cacheControl` sends `Cache-Control` headers in responses when
    /// activated. The `max-age` value is the least of the values received from
    /// upstream services. @default `false`.
    pub cache_control: Option<bool>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// `headers` are key-value pairs included in every server
    /// response. Useful for setting headers like `Access-Control-Allow-Origin`
    /// for cross-origin requests or additional headers for downstream services.
    pub custom: Vec<KeyValue>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// `experimental` allows the use of `X-*` experimental headers
    /// in the response. @default `[]`.
    pub experimental: Option<BTreeSet<String>>,

    /// `setCookies` when enabled stores `set-cookie` headers
    /// and all the response will be sent with the headers.
    #[serde(default, skip_serializing_if = "is_default")]
    pub set_cookies: Option<bool>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// `cors` allows Cross-Origin Resource Sharing (CORS) for a server.
    pub cors: Option<Cors>,
}

impl Headers {
    pub fn enable_cache_control(&self) -> bool {
        self.cache_control.unwrap_or(false)
    }
    pub fn set_cookies(&self) -> bool {
        self.set_cookies.unwrap_or_default()
    }
    pub fn get_cors(&self) -> Option<Cors> {
        self.cors.clone()
    }
}
