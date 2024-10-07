use serde::{Deserialize, Serialize};
use tailcall_macros::DirectiveDefinition;

#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, Eq, schemars::JsonSchema, DirectiveDefinition,
)]
#[directive_definition(repeatable, locations = "Object")]
#[serde(deny_unknown_fields)]
/// The @addField operator simplifies data structures and queries by adding a field that inlines or flattens a nested field or node within your schema. more info [here](https://tailcall.run/docs/guides/operators/#addfield)
pub struct AddField {
    /// Name of the new field to be added
    pub name: String,
    /// Path of the data where the field should point to
    pub path: Vec<String>,
}
