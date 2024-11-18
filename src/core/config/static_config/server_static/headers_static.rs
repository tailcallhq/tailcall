use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::core::config::KeyValue;
use crate::core::is_default;
use crate::core::macros::MergeRight;

use super::cors_static::CorsStatic;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, JsonSchema, MergeRight)]
pub struct HeadersStatic {
    #[serde(default, skip_serializing_if = "is_default")]
    /// `cache_control` sends `Cache-Control` headers in responses when
    /// activated. The `max-age` value is the least of the values received from
    /// upstream services. @default `false`.
    pub cache_control: Option<bool>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// `custom` are key-value pairs included in every server
    /// response. Useful for setting headers like `Access-Control-Allow-Origin`
    /// for cross-origin requests or additional headers for downstream services.
    pub custom: Vec<KeyValue>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// `experimental` allows the use of `X-*` experimental headers
    /// in the response. @default `[]`.
    pub experimental: Option<BTreeSet<String>>,

    /// `set_cookies` when enabled stores `set-cookie` headers
    /// and all the response will be sent with the headers.
    #[serde(default, skip_serializing_if = "is_default")]
    pub set_cookies: Option<bool>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// `cors` allows Cross-Origin Resource Sharing (CORS) for a server.
    pub cors: Option<CorsStatic>,
}

impl HeadersStatic {
    pub fn enable_cache_control(&self) -> bool {
        self.cache_control.unwrap_or(false)
    }
    pub fn set_cookies(&self) -> bool {
        self.set_cookies.unwrap_or_default()
    }
    pub fn get_cors(&self) -> Option<CorsStatic> {
        self.cors.clone()
    }
}
