use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tailcall_macros::MergeRight;

use crate::core::config::Resolver;

// from the Apollo spec https://specs.apollo.dev/#sec-federation-v2-9
pub static FEDERATION_DIRECTIVES: &[&str] = &[
    "authenticated",
    "context",
    "cost",
    "fromContext",
    "link",
    "key",
    "tag",
    "shareable",
    "inaccessible",
    "override",
    "extends",
    "external",
    "provides",
    "requires",
    "requiresScope",
    "composeDirective",
    "interfaceObject",
    "listSize",
    "policy",
];

/// Directive `@key` for Apollo Federation
#[derive(
    Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, schemars::JsonSchema, MergeRight,
)]
pub struct Key {
    pub fields: String,
}

/// Resolver for `_entities` field for Apollo Federation
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EntityResolver {
    pub resolver_by_type: BTreeMap<String, Resolver>,
}
