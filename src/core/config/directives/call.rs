use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tailcall_macros::DirectiveDefinition;

use crate::core::is_default;

///
/// Provides the ability to refer to a field defined in the root Query or
/// Mutation.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, schemars::JsonSchema)]
pub struct Step {
    #[serde(default, skip_serializing_if = "is_default")]
    /// The name of the field on the `Query` type that you want to call.
    pub query: Option<String>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// The name of the field on the `Mutation` type that you want to call.
    pub mutation: Option<String>,

    /// The arguments that will override the actual arguments of the field.
    #[serde(default, skip_serializing_if = "is_default")]
    pub args: BTreeMap<String, Value>,
}

///
/// Provides the ability to refer to multiple fields in the Query or
/// Mutation root.
#[derive(
    Serialize,
    Deserialize,
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    schemars::JsonSchema,
    DirectiveDefinition,
)]
#[directive_definition(repeatable, locations = "FieldDefinition, Object")]
pub struct Call {
    /// Steps are composed together to form a call.
    /// If you have multiple steps, the output of the previous step is passed as
    /// input to the next step.
    pub steps: Vec<Step>,
    #[serde(default, skip_serializing_if = "is_default")]
    /// Enables deduplication of IO operations to enhance performance.
    ///
    /// This flag prevents duplicate IO requests from being executed
    /// concurrently, reducing resource load. Caution: May lead to issues
    /// with APIs that expect unique results for identical inputs, such as
    /// nonce-based APIs.
    pub dedupe: Option<bool>,
}
