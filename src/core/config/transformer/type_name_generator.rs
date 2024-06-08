use std::collections::{BTreeMap, HashSet};

use inflector::Inflector;
use regex::Regex;

use crate::core::config::transformer::Transform;
use crate::core::config::Config;
use crate::core::valid::Valid;

fn is_auto_generated_field_name(field_name: &str) -> bool {
    lazy_static::lazy_static! {
        static ref RE: Regex = Regex::new(r"^f\d+$|^fn$").unwrap();
    }
    RE.is_match(field_name)
}

struct CandidateGeneration<'a> {
    /// maintains the generated candidates in the form of {TypeName: [{candidate_name: frequency}]}
    candidates: BTreeMap<String, BTreeMap<String, u32>>,
    config: &'a Config,
}

impl<'a> CandidateGeneration<'a> {
    fn new(config: &'a Config) -> Self {
        Self { candidates: Default::default(), config }
    }

    /// Generates candidate type names based on the provided configuration.
    /// This method iterates over the configuration and collects candidate type names for each type.
    fn generate(mut self) -> Self {
        for type_info in self.config.types.values() {
            for (field_name, field_info) in type_info.fields.iter() {
                if self.config.is_scalar(&field_info.type_of)
                    || is_auto_generated_field_name(field_name)
                {
                    // If field type is scalar or field name is auto-generated, ignore type name
                    // inference.
                    continue;
                }

                let inner_map = self
                    .candidates
                    .entry(field_info.type_of.to_owned())
                    .or_default();

                *inner_map.entry(field_name.to_owned()).or_insert(0) += 1;
            }
        }
        self
    }

    /// Converges on the most frequent candidate name for each type.
    /// This method selects the most frequent candidate name for each type, ensuring uniqueness.
    fn converge(self) -> BTreeMap<String, String> {
        let mut finalized_candidates = BTreeMap::new();
        let mut converged_candidate_set = HashSet::new();

        for (type_name, candidate_list) in self.candidates.iter() {
            // Find the most frequent candidate that hasn't been converged yet and it's not already present in types.
            if let Some((candidate_name, _)) = candidate_list
                .iter()
                .max_by_key(|&(_, count)| count)
                .filter(|(candidate_name, _)| {
                    !converged_candidate_set.contains(candidate_name)
                        && !self.config.types.contains_key(*candidate_name)
                })
            {
                let singularized_candidate_name = candidate_name.to_singular().to_pascal_case();
                finalized_candidates.insert(type_name.to_owned(), singularized_candidate_name);
                converged_candidate_set.insert(candidate_name);
            }
        }

        finalized_candidates
    }
}

pub struct TypeNameGenerator;

impl TypeNameGenerator {
    /// Generates type names based on inferred candidates from the provided
    /// configuration.
    fn generate_type_names(&self, mut config: Config) -> Config {
        let finalized_candidates = CandidateGeneration::new(&config).generate().converge();

        for (old_type_name, new_type_name) in finalized_candidates {
            if let Some(type_) = config.types.remove(old_type_name.as_str()) {
                // Add newly generated type.
                config.types.insert(new_type_name.to_owned(), type_);

                // Replace all the instances of old name in config.
                for actual_type in config.types.values_mut() {
                    for actual_field in actual_type.fields.values_mut() {
                        if actual_field.type_of == old_type_name {
                            // Update the field's type with the new name
                            actual_field.type_of.clone_from(&new_type_name);
                        }
                    }
                }
            }
        }
        config
    }
}

impl Transform for TypeNameGenerator {
    fn transform(&self, config: Config) -> Valid<Config, String> {
        let config = self.generate_type_names(config);

        Valid::succeed(config)
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use anyhow::Ok;
    use tailcall_fixtures::configs;

    use super::TypeNameGenerator;
    use crate::core::config::transformer::Transform;
    use crate::core::config::Config;
    use crate::core::valid::Validator;

    fn read_fixture(path: &str) -> String {
        fs::read_to_string(path).unwrap()
    }

    #[test]
    fn test_type_name_generator_transform() {
        let config = Config::from_sdl(read_fixture(configs::AUTO_GENERATE_CONFIG).as_str())
            .to_result()
            .unwrap();

        let transformed_config = TypeNameGenerator.transform(config).to_result().unwrap();
        insta::assert_snapshot!(transformed_config.to_sdl());
    }

    #[test]
    fn test_type_name_generator_with_cyclic_types() -> anyhow::Result<()> {
        let config = Config::from_sdl(read_fixture(configs::CYCLIC_CONFIG).as_str())
            .to_result()
            .unwrap();

        let transformed_config = TypeNameGenerator.transform(config).to_result().unwrap();
        insta::assert_snapshot!(transformed_config.to_sdl());

        Ok(())
    }
}
