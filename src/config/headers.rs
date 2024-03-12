use serde::{Deserialize, Serialize};

use crate::is_default;

use super::KeyValue;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Headers {
    #[serde(default, skip_serializing_if = "is_default")]
    /// `cacheControl` sends `Cache-Control` headers in responses when
    /// activated. The `max-age` value is the least of the values received from
    /// upstream services. @default `false`.
    pub cache_control: Option<bool>,
    /// The `headers` are key-value pairs included in every server
    /// response. Useful for setting headers like `Access-Control-Allow-Origin`
    /// for cross-origin requests or additional headers for downstream services.
    pub custom: Option<Vec<KeyValue>>,
}

impl Headers {
    pub fn enable_cache_control(&self) -> bool {
        self.cache_control.unwrap_or(false)
    }
}
