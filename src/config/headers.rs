use serde::{Deserialize, Serialize};

use crate::config::KeyValue;
use crate::is_default;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, schemars::JsonSchema)]
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
}

impl Headers {
    pub fn enable_cache_control(&self) -> bool {
        self.cache_control.unwrap_or(false)
    }
}

pub fn merge_headers(current: Option<Headers>, other: Option<Headers>) -> Option<Headers> {
    let mut headers = current.clone();

    if let Some(other_headers) = other {
        if let Some(mut self_headers) = current.clone() {
            self_headers.cache_control = other_headers.cache_control.or(self_headers.cache_control);
            self_headers.custom.extend(other_headers.custom);

            headers = Some(self_headers);
        } else {
            headers = Some(other_headers);
        }
    }

    headers
}
