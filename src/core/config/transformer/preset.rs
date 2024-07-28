use derive_setters::Setters;

use crate::core::config::Config;
use crate::core::transform::{self, Transform, TransformerOps};

/// Defines a set of default transformers that can be applied to any
/// configuration to make it more maintainable and readable.
#[derive(Setters, Debug, PartialEq)]
pub struct Preset {
    pub merge_type: PresetMergeTypeOption,
    pub consolidate_url: f32,
    pub tree_shake: bool,
    pub use_better_names: bool,
    unwrap_single_field_types: bool,
}

#[derive(Debug, PartialEq)]
pub struct PresetMergeTypeOption {
    pub threshold: f32,
    pub merge_unknown_types: bool,
}

impl PresetMergeTypeOption {
    pub fn new(threshold: f32, merge_unknown_types: bool) -> Self {
        Self { threshold, merge_unknown_types }
    }
}

impl Default for PresetMergeTypeOption {
    fn default() -> Self {
        Self { threshold: 1.0, merge_unknown_types: true }
    }
}

impl Preset {
    pub fn new() -> Self {
        Self {
            merge_type: PresetMergeTypeOption { threshold: 0.0, merge_unknown_types: false },
            consolidate_url: 0.0,
            tree_shake: false,
            use_better_names: false,
            unwrap_single_field_types: true,
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
                super::TypeMerger::new(
                    self.merge_type.threshold,
                    self.merge_type.merge_unknown_types,
                )
                .when(super::TypeMerger::is_enabled(self.merge_type.threshold)),
            )
            .pipe(super::FlattenSingleField.when(self.unwrap_single_field_types))
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
            merge_type: PresetMergeTypeOption::default(),
            consolidate_url: 0.5,
            use_better_names: true,
            tree_shake: true,
            unwrap_single_field_types: false,
        }
    }
}
