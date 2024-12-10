use derive_setters::Setters;

use crate::core::config::Config;
use crate::core::transform::{self, Transform, TransformerOps};

/// Defines a set of default transformers that can be applied to any
/// configuration to make it more maintainable and readable.
#[derive(Setters, Debug, PartialEq)]
pub struct Preset {
    pub merge_type: f32,
    pub tree_shake: bool,
    pub infer_type_names: bool,
    pub unwrap_single_field_types: bool,
}

impl Preset {
    pub fn new() -> Self {
        Self {
            merge_type: 0.0,
            tree_shake: false,
            infer_type_names: true,
            unwrap_single_field_types: true,
        }
    }
}

impl Transform for Preset {
    type Value = Config;
    type Error = String;

    fn transform(&self, config: Self::Value) -> tailcall_valid::Valid<Self::Value, Self::Error> {
        transform::default()
            .pipe(super::Required)
            .pipe(super::TreeShake.when(self.tree_shake))
            .pipe(
                super::TypeMerger::new(self.merge_type)
                    .when(super::TypeMerger::is_enabled(self.merge_type)),
            )
            .pipe(super::FlattenSingleField.when(self.unwrap_single_field_types))
            .pipe(super::ImproveTypeNames.when(self.infer_type_names))
            .transform(config)
    }
}

impl Default for Preset {
    fn default() -> Self {
        Self {
            merge_type: 1.0,
            infer_type_names: true,
            tree_shake: true,
            unwrap_single_field_types: false,
        }
    }
}
