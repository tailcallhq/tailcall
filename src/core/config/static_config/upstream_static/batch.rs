use std::collections::BTreeSet;

use derive_setters::Setters;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tailcall_macros::MergeRight;

use crate::core::is_default;

pub const DEFAULT_MAX_SIZE: usize = 100;

#[derive(
    Serialize,
    Deserialize,
    Clone,
    Debug,
    Setters,
    PartialEq,
    Eq,
    JsonSchema,
    MergeRight,
)]
pub struct Batch {
    pub delay: usize,
    pub headers: BTreeSet<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub max_size: Option<usize>,
}

impl Default for Batch {
    fn default() -> Self {
        Self {
            max_size: Some(DEFAULT_MAX_SIZE),
            delay: 0,
            headers: BTreeSet::new(),
        }
    }
}
