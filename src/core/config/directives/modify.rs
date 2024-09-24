use serde::{Deserialize, Serialize};
use tailcall_macros::{DirectiveDefinition, InputDefinition};

use crate::core::is_default;

#[derive(
    Serialize,
    Deserialize,
    Clone,
    Debug,
    PartialEq,
    Eq,
    schemars::JsonSchema,
    DirectiveDefinition,
    InputDefinition,
)]
#[directive_definition(locations = "FieldDefinition")]
#[serde(deny_unknown_fields)]
pub struct Modify {
    #[serde(default, skip_serializing_if = "is_default")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub omit: Option<bool>,
}
