use serde::{Deserialize, Serialize};
use tailcall_macros::{DirectiveDefinition, MergeRight};

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
#[directive_definition(locations = "Object,FieldDefinition", repeatable)]
pub struct Protected {
    pub providers: Option<Vec<String>>,
}
