use derive_setters::Setters;

use crate::core::config::Config;
use crate::core::transform::{self, Transform, TransformerOps};

/// Defines a set of default transformers that can be applied to any
/// configuration to make it more maintainable.
#[derive(Setters, Debug, PartialEq)]
pub struct Preset {
    merge_type: f32,
    consolidate_url: f32,
    tree_shake: bool,
    use_better_names: bool,
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
            .pipe(super::TypeMerger::new(self.merge_type))
            .pipe(super::ImproveTypeNames.when(self.use_better_names))
            .pipe(super::ConsolidateURL::new(self.consolidate_url))
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
        }
    }
}
