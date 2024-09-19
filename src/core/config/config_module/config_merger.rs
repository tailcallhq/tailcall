use crate::core::{config::Config, merge_right::MergeRight, valid::Valid};

use super::{Cache, ConfigModule};

pub(super) struct ConfigMerger;

impl ConfigMerger {
    pub fn merge(&self, left: ConfigModule, right: ConfigModule) -> Valid<ConfigModule, String> {
        let mut types = left.cache.config.types;

        for (name, mut rty) in right.cache.config.types {
            if let Some(lty) = types.remove(&name) {
                rty = lty.merge_right(rty);
            }

            types.insert(name, rty);
        }

        let config = Config { types, ..left.cache.config };

        let cache = Cache {
            config,
            input_types: left.cache.input_types.merge_right(right.cache.input_types),
            output_types: left
                .cache
                .output_types
                .merge_right(right.cache.output_types),
            interface_types: left
                .cache
                .interface_types
                .merge_right(right.cache.interface_types),
        };

        Valid::succeed(ConfigModule { extensions: left.extensions, cache })
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::core::{
        config::{Config, ConfigModule},
        valid::Validator,
    };
    use insta::assert_snapshot;
    use tailcall_fixtures::configs::federation;

    use super::ConfigMerger;

    #[test]
    fn test_federation_merge() -> anyhow::Result<()> {
        let config = Config::from_sdl(&fs::read_to_string(federation::ROUTER)?).to_result()?;
        let router = ConfigModule::from(config);

        let config =
            Config::from_sdl(&fs::read_to_string(federation::SUBGRAPH_USERS)?).to_result()?;
        let subgraph_users = ConfigModule::from(config);

        let config =
            Config::from_sdl(&fs::read_to_string(federation::SUBGRAPH_POSTS)?).to_result()?;
        let subgraph_posts = ConfigModule::from(config);

        let merger = ConfigMerger;
        let merged = router;
        let merged = merger.merge(merged, subgraph_users).to_result()?;
        let merged = merger.merge(merged, subgraph_posts).to_result()?;

        assert_snapshot!(merged.to_sdl());

        Ok(())
    }
}
