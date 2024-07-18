use derive_setters::Setters;

use crate::core::config::Config;
use crate::core::transform::{self, Transform, TransformerOps};

/// Defines a set of default transformers that can be applied to any
/// configuration to make it more maintainable.
#[derive(Default, Setters, Debug, PartialEq)]
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
            .pipe(
                super::TypeMerger::new(self.merge_type)
                    .when(super::TypeMerger::is_enabled(self.merge_type)),
            )
            .pipe(super::ImproveTypeNames.when(self.use_better_names))
            .pipe(
                super::ConsolidateURL::new(self.consolidate_url)
                    .when(super::ConsolidateURL::is_enabled(self.consolidate_url)),
            )
            .transform(config)
    }
}
