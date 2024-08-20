use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tailcall_macros::MergeRight;
use crate::core::merge_right::MergeRight;

use crate::core::config::Resolver;

/// Directive `@key` for Apollo Federation
#[derive(Serialize, Deserialize,Clone, Debug, Default, PartialEq, Eq, schemars::JsonSchema, MergeRight)]
pub struct Key {
    pub fields: String,
}

/// Resolver for `_entities` field for Apollo Federation
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, schemars::JsonSchema)]
pub struct EntityResolver {
    pub resolver_by_type: BTreeMap<String, Resolver>,
}
