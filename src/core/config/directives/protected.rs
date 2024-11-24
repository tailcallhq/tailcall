use serde::{Deserialize, Serialize};
use tailcall_macros::{DirectiveDefinition, MergeRight};

/// Specifies the authentication requirements for accessing a field or type.
///
/// This allows you to control access by listing the IDs of authentication
/// providers.
/// - If `id` is not provided, all available providers must authorize the
///   request.
/// - If multiple provider IDs are listed, the request must be authorized by all
///   of them.
///
/// Example: If you want only specific providers to allow access, include their
/// IDs in the list. Otherwise, leave it empty to require authorization from all
/// available providers.

#[derive(
    Clone,
    Debug,
    Deserialize,
    Serialize,
    PartialEq,
    Eq,
    Default,
    schemars::JsonSchema,
    MergeRight,
    DirectiveDefinition,
)]
#[directive_definition(locations = "Object,FieldDefinition")]
pub struct Protected {
    /// List of authentication provider IDs that can access this field or type.
    /// - Leave empty to require authorization from all providers.
    /// - Include multiple IDs to require authorization from each one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Vec<String>>,
}
