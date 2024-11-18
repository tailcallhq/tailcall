use std::collections::BTreeMap;

use derive_setters::Setters;
use serde::{Deserialize, Serialize};
use tailcall_macros::MergeRight;

use super::{Enum, RootSchema, Type, Union};
use crate::core::is_default;

#[derive(
    Serialize,
    Deserialize,
    Clone,
    Debug,
    Default,
    Setters,
    PartialEq,
    Eq,
    schemars::JsonSchema,
    MergeRight,
)]
pub struct BlueprintBuilder {
    ///
    /// Specifies the entry points for query and mutation in the generated
    /// GraphQL schema.
    pub schema: RootSchema,

    ///
    /// A map of all the types in the schema.
    #[serde(default)]
    #[setters(skip)]
    pub types: BTreeMap<String, Type>,

    ///
    /// A map of all the union types in the schema.
    #[serde(default, skip_serializing_if = "is_default")]
    pub unions: BTreeMap<String, Union>,

    ///
    /// A map of all the enum types in the schema
    #[serde(default, skip_serializing_if = "is_default")]
    pub enums: BTreeMap<String, Enum>,
}
