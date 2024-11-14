use std::collections::BTreeSet;

use derive_getters::Getters;
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
    Getters,
    PartialEq,
    Eq,
    JsonSchema,
    MergeRight,
)]
pub struct BatchConfig {
    pub delay: usize,
    pub headers: BTreeSet<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub max_size: Option<usize>,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_size: Some(DEFAULT_MAX_SIZE),
            delay: 0,
            headers: BTreeSet::new(),
        }
    }
}
