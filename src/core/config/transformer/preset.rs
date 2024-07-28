use std::collections::HashSet;

use derive_setters::Setters;

use crate::core::config::Config;
use crate::core::transform::{self, Transform, TransformerOps};

/// Defines a set of default transformers that can be applied to any
/// configuration to make it more maintainable and readable.
#[derive(Setters, Debug, PartialEq)]
pub struct Preset {
    pub merge_type: f32,
    pub consolidate_url: f32,
    pub tree_shake: bool,
    pub use_better_names: bool,
    unwrap_single_field_types: bool,
    suggested_names: HashSet<String>,
}

impl Preset {
    pub fn new() -> Self {
        Self {
            merge_type: 0.0,
            consolidate_url: 0.0,
            tree_shake: false,
            use_better_names: false,
            unwrap_single_field_types: true,
            suggested_names: HashSet::new(),
        }
    }
}

impl Transform for Preset {
    type Value = Config;
    type Error = String;

    fn transform(
        &self,
        config: Self::Value,
    ) -> crate::core::valid::Valid<Self::Value, Self::Error> {
        transform::default()
            .pipe(super::Required)
            .pipe(super::TreeShake.when(self.tree_shake))
            .pipe(
                super::TypeMerger::new(self.merge_type)
                    .when(super::TypeMerger::is_enabled(self.merge_type)),
            )
            .pipe(super::FlattenSingleField.when(self.unwrap_single_field_types))
            .pipe(
                super::SuggestNames::new(self.suggested_names.clone())
                    .when(!self.suggested_names.is_empty()),
            )
            .pipe(super::ImproveTypeNames.when(self.use_better_names))
            .pipe(
                super::ConsolidateURL::new(self.consolidate_url)
                    .when(super::ConsolidateURL::is_enabled(self.consolidate_url)),
            )
            .transform(config)
    }
}

impl Default for Preset {
    fn default() -> Self {
        Self {
            merge_type: 1.0,
            consolidate_url: 0.5,
            use_better_names: true,
            tree_shake: true,
            unwrap_single_field_types: true,
            suggested_names: HashSet::new(),
        }
    }
}
