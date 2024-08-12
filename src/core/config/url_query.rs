use serde::{Deserialize, Serialize};

use crate::core::is_default;

#[derive(Serialize, Deserialize, Clone, Debug, Default, Eq, PartialEq, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct URLQuery {
    pub key: String,
    pub value: String,
    #[serde(default, skip_serializing_if = "is_default")]
    /// Determines whether to ignore query parameters with empty values when
    /// forming URLs.
    ///
    /// When set to true, query parameters without values are completely ignored
    /// during URL formation. When false (default), parameters without values
    /// are included in the URL.
    pub skip_empty: Option<bool>,
}
