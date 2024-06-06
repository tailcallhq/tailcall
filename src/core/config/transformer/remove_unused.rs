use super::Transform;
use crate::core::config::Config;
use crate::core::valid::Valid;

/// `RemoveUnused` is responsible for removing unused types from a
/// configuration.
///
/// It scans the configuration and identifies types that are not referenced
/// elsewhere, effectively cleaning up unused clutter from the configuration.
pub struct RemoveUnused;

impl Transform for RemoveUnused {
    fn transform(&self, mut config: Config) -> Valid<Config, String> {
        let unused_types = config.unused_types();
        config = config.remove_types(unused_types);
        Valid::succeed(config)
    }
}
