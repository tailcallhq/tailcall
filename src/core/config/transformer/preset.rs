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
    pub infer_type_names: bool,
    pub unwrap_single_field_types: bool,
}

pub struct ImproveTypeNames {
    message_sent: bool,
}

impl ImproveTypeNames {
    pub fn new() -> Self {
        Self { message_sent: false }
    }

    // Method to send the system message only once
    pub fn send_message_once(&mut self) {
        if !self.message_sent {
            println!("System message: Inferring type names...");
            self.message_sent = true;
        }
    }
}

impl Preset {
    pub fn new() -> Self {
        Self {
            merge_type: 0.0,
            consolidate_url: 0.0,
            tree_shake: false,
            infer_type_names: false,
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
        // Instantiate ImproveTypeNames with state to ensure the system message about
        // inferring type names is sent only once during the transformation process.
        // This prevents redundant messages from being logged multiple times.
        let mut type_name_improver = super::ImproveTypeNames::new();

        if self.infer_type_names {
            type_name_improver.send_message_once();
        }
        transform::default()
            .pipe(super::Required)
            .pipe(super::TreeShake.when(self.tree_shake))
            .pipe(
                super::TypeMerger::new(self.merge_type)
                    .when(super::TypeMerger::is_enabled(self.merge_type)),
            )
            .pipe(super::FlattenSingleField.when(self.unwrap_single_field_types))
            .pipe(type_name_improver.when(self.infer_type_names))
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
            infer_type_names: true,
            tree_shake: true,
            unwrap_single_field_types: false,
        }
    }
}
