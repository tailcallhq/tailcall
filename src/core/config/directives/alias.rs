use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use tailcall_macros::{DirectiveDefinition, MergeRight};

/// The @alias directive indicates that aliases of one enum value.
#[derive(
    Default,
    Serialize,
    Deserialize,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    schemars::JsonSchema,
    MergeRight,
    DirectiveDefinition,
)]
#[directive_definition(locations = "EnumValue")]
pub struct Alias {
    pub options: BTreeSet<String>,
}
