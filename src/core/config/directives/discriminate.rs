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
/// applied on a field `@discriminate(field: "object_type")` and the given value
/// `{"foo": "bar", "object_type": "Buzz"}` the resolved type of the object will
/// be `Buzz`. If `field` is not applied it defaults to "type". The `field` does
/// not have to be part of the GraphQL Schema, but it is required to be part of
/// the JSON response. In case this field is missing from the response an
/// appropriate error message will be displayed.
pub struct Discriminate {
    #[serde(default, skip_serializing_if = "is_default")]
    pub field: Option<String>,
}

impl Discriminate {
    pub fn get_field(&self) -> String {
        self.field.clone().unwrap_or("type".to_string())
    }
}
