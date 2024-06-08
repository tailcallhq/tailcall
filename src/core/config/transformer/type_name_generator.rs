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

struct CandidateGeneration {
    candidates: BTreeMap<String, BTreeMap<String, u32>>,
}

impl CandidateGeneration {
    fn new() -> Self {
        Self { candidates: Default::default() }
    }

    // step 1: generate the required candidates.
    // i.e { Type : [{candidate_name : count}] }
    fn generate_candidate_type_names(&mut self, config: &Config) {
        for type_info in config.types.values() {
            for (field_name, field_info) in type_info.fields.iter() {
                if config.is_scalar(&field_info.type_of) || is_auto_generated_field_name(field_name)
                {
                    // If field type is scalar or field name is auto-generated, ignore type name inference.
                    continue;
                }

                let inner_map = self
                    .candidates
                    .entry(field_info.type_of.to_owned())
                    .or_default();

                *inner_map.entry(field_name.to_owned()).or_insert(0) += 1;
            }
        }
    }

    // step 2: converge on the candidate name. i.e { Type : Candidate_Name }
    fn finalize_candidates(&self) -> BTreeMap<String, String> {
        let mut finalized_candidates = BTreeMap::new();
        let mut converged_candidate_set = HashSet::new();

        for (type_name, candidate_list) in self.candidates.iter() {
            // Find the most frequent candidate that hasn't been converged yet.
            if let Some((candidate_name, _)) = candidate_list
                .into_iter()
                .max_by_key(|&(_, count)| count)
                .filter(|(candidate_name, _)| !converged_candidate_set.contains(candidate_name))
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
    /// Generates type names based on inferred candidates from the provided configuration.
    fn generate_type_name(&self, mut config: Config) -> Config {
        let mut candidate_gen = CandidateGeneration::new();
        candidate_gen.generate_candidate_type_names(&config);
        let finalized_candidates = candidate_gen.finalize_candidates();

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
        let config = self.generate_type_name(config);

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
