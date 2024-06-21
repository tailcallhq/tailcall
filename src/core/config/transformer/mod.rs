mod ambiguous_type;
mod consolidate_url;
mod improve_type_names;
mod merge_types;
mod preset;
mod tree_shake;

pub use ambiguous_type::{AmbiguousType, Resolution};
pub use consolidate_url::ConsolidateURL;
pub use improve_type_names::ImproveTypeNames;
pub use merge_types::TypeMerger;
pub use preset::Preset;
pub use tree_shake::TreeShake;
