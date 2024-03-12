use serde::{Deserialize, Serialize};

use crate::is_default;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Headers {
    #[serde(default, skip_serializing_if = "is_default")]
    /// `cacheControlHeader` sends `Cache-Control` headers in responses when
    /// activated. The `max-age` value is the least of the values received from
    /// upstream services. @default `false`.
    pub cache_control: Option<bool>,
}

impl Headers {
    pub fn enable_cache_control(&self) -> bool {
        self.cache_control.unwrap_or(false)
    }
}
