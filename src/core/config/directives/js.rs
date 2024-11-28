use serde::{Deserialize, Serialize};
use tailcall_macros::{DirectiveDefinition, InputDefinition};

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
#[directive_definition(repeatable, locations = "FieldDefinition, Object", lowercase_name)]
pub struct JS {
    pub name: String,
}
