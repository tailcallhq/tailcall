use std::collections::{BTreeMap, HashSet};

use inflector::Inflector;

use crate::core::config::Config;
use crate::core::transform::Transform;
use crate::core::valid::Valid;

#[derive(Debug, Default)]
struct CandidateStats {
    frequency: u32,
    priority: u8,
}

struct CandidateConvergence<'a> {
    /// maintains the generated candidates in the form of
    /// {TypeName: {{candidate_name: {frequency: 1, priority: 0}}}}
    candidates: BTreeMap<String, BTreeMap<String, CandidateStats>>,
    config: &'a Config,
}

impl<'a> CandidateConvergence<'a> {
    fn new(candate_gen: CandidateGeneration<'a>) -> Self {
        Self {
            candidates: candate_gen.candidates,
            config: candate_gen.config,
        }
    }

    /// Converges on the most frequent candidate name for each type.
    /// This method selects the most frequent candidate name for each type,
    /// ensuring uniqueness.
    fn converge(self) -> Vec<(String, String)> {
        let mut finalized_candidates = BTreeMap::new();
        let mut converged_candidate_set = HashSet::new();

        for (type_name, candidate_list) in self.candidates.iter() {
            // Find the most frequent candidate that hasn't been converged yet and it's not
            // already present in types.

            // M5 = Product, Product
            // M1 = Dimension,
            // M2 = Meta
            // M4 = Review
            // M3 = (productSearch,1), (Product)
            if let Some((candidate_name, _)) = candidate_list
                .iter()
                .filter(|(candidate_name, _)| {
                    let singularized_candidate_name = candidate_name.to_singular().to_pascal_case();
                    !converged_candidate_set.contains(&singularized_candidate_name)
                        && !self.config.types.contains_key(&singularized_candidate_name)
                })
                .max_by(|a, b| {
                    a.1.frequency
                        .cmp(&b.1.frequency)
                        .then_with(|| {
                            // If frequencies are equal, compare candidate names lexicographically
                            a.0.cmp(b.0)
                        })
                        .then_with(|| a.1.priority.cmp(&b.1.priority))
                })
            {
                let singularized_candidate_name = candidate_name.to_singular().to_pascal_case();
                finalized_candidates
                    .insert(type_name.to_owned(), singularized_candidate_name.clone());
                converged_candidate_set.insert(singularized_candidate_name);
            }
        }

        // Convert BTreeMap to Vec and sort by key to normalize the data.
        let mut normalize_candidates: Vec<(String, String)> =
            finalized_candidates.into_iter().collect();
        normalize_candidates.sort_by(|a, b| a.0.cmp(&b.0));

        normalize_candidates
    }
}

struct CandidateGeneration<'a> {
    /// maintains the generated candidates in the form of
    /// {TypeName: {{candidate_name: {frequency: 1, priority: 0}}}}
    candidates: BTreeMap<String, BTreeMap<String, CandidateStats>>,
    config: &'a Config,
}

impl<'a> CandidateGeneration<'a> {
    fn new(config: &'a Config) -> Self {
        Self { candidates: Default::default(), config }
    }

    /// Generates candidate type names based on the provided configuration.
    /// This method iterates over the configuration and collects candidate type
    /// names for each type.
    fn generate(mut self) -> CandidateConvergence<'a> {
        for (type_name, type_info) in self.config.types.iter() {
            for (field_name, field_info) in type_info.fields.iter() {
                if self.config.is_scalar(&field_info.type_of) {
                    // If field type is scalar then ignore type name inference.
                    continue;
                }

                let inner_map = self
                    .candidates
                    .entry(field_info.type_of.to_owned())
                    .or_default();

                if let Some(key_val) = inner_map.get_mut(field_name) {
                    key_val.frequency += 1
                } else {
                    // in order to infer the types correctly, always prioritize the non-operation
                    // types but final selection will still depend upon the
                    // frequency.
                    let priority = match self.config.is_root_operation_type(type_name) {
                        true => 0,
                        false => 1,
                    };

                    inner_map.insert(
                        field_name.to_owned(),
                        CandidateStats { frequency: 1, priority },
                    );
                }
            }
        }
        CandidateConvergence::new(self)
    }
}

#[derive(Default)]
pub struct ImproveTypeNames;

impl ImproveTypeNames {
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

impl Transform for ImproveTypeNames {
    type Value = Config;
    type Error = String;
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

    use super::ImproveTypeNames;
    use crate::core::config::Config;
    use crate::core::transform::Transform;
    use crate::core::valid::Validator;

    fn read_fixture(path: &str) -> String {
        fs::read_to_string(path).unwrap()
    }

    #[test]
    fn test_type_name_generator_transform() {
        let config = Config::from_sdl(read_fixture(configs::AUTO_GENERATE_CONFIG).as_str())
            .to_result()
            .unwrap();

        let transformed_config = ImproveTypeNames.transform(config).to_result().unwrap();
        insta::assert_snapshot!(transformed_config.to_sdl());
    }

    #[test]
    fn test_type_name_generator_with_cyclic_types() -> anyhow::Result<()> {
        let config = Config::from_sdl(read_fixture(configs::CYCLIC_CONFIG).as_str())
            .to_result()
            .unwrap();

        let transformed_config = ImproveTypeNames.transform(config).to_result().unwrap();
        insta::assert_snapshot!(transformed_config.to_sdl());

        Ok(())
    }

    #[test]
    fn test_type_name_generator() -> anyhow::Result<()> {
        let config = Config::from_sdl(read_fixture(configs::NAME_GENERATION).as_str())
            .to_result()
            .unwrap();

        let transformed_config = ImproveTypeNames.transform(config).to_result().unwrap();
        insta::assert_snapshot!(transformed_config.to_sdl());

        Ok(())
    }
}
