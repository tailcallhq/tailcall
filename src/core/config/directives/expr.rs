use serde::{Deserialize, Serialize};
use serde_json::Value;
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
#[directive_definition(repeatable, locations = "FieldDefinition, Object")]
#[serde(deny_unknown_fields)]
/// The `@expr` operators allows you to specify an expression that can evaluate
/// to a value. The expression can be a static value or built form a Mustache
/// template. schema.
pub struct Expr {
    pub body: Value,
}
