use serde::{Deserialize, Serialize};

use crate::core::is_default;

#[derive(Serialize, Deserialize, Clone, Debug, Default, Eq, PartialEq, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
/// The URLQuery input type represents a query parameter to be included in a
/// URL.
pub struct URLQuery {
    /// The key or name of the query parameter.
    pub key: String,
    /// The actual value or a mustache template to resolve the value dynamically
    /// for the query parameter.
    pub value: String,
    #[serde(default, skip_serializing_if = "is_default")]
    /// Determines whether to ignore query parameters with empty values.
    pub skip_empty: Option<bool>,
}
