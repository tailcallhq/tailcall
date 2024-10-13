use serde::{Deserialize, Serialize};
use tailcall_macros::{DirectiveDefinition, InputDefinition, MergeRight};

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
    MergeRight,
)]
#[directive_definition(locations = "FieldDefinition")]
#[serde(deny_unknown_fields)]
///
/// The `@discriminate` directive is used to drive Tailcall discriminator to use
/// a field of an object to resolve the type. For example with the directive
/// applied on a field `@discriminate(field: "type")` and the given value
/// `{"foo": "bar", "type": "Buzz"}` the resolved type of the object will be
/// `Buzz`.
pub struct Discriminate {
    #[serde(default, skip_serializing_if = "is_default")]
    pub field: String,
}
