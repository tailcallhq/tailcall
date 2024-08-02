use std::collections::HashSet;

use convert_case::{Case, Casing};
use indexmap::IndexMap;

use crate::core::config::Config;
use crate::core::transform::Transform;
use crate::core::valid::Valid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CandidateStats {
    frequency: u32,
    priority: Priority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Priority {
    High,
    Medium,
}

impl Priority {
    fn as_u8(&self) -> u8 {
        match self {
            Priority::High => 3,
            Priority::Medium => 2,
        }
    }
}

impl std::cmp::Ord for Priority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_u8().cmp(&other.as_u8())
    }
}

impl std::cmp::PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

struct CandidateConvergence<'a> {
    /// maintains the generated candidates in the form of
    /// {TypeName: {{candidate_name: {frequency: 1, priority: 2}}}}
    candidates: IndexMap<String, IndexMap<String, CandidateStats>>,
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
    fn converge(self) -> IndexMap<String, String> {
        let mut finalized_candidates = IndexMap::new();
        let mut converged_candidate_set = HashSet::new();

        for (type_name, candidate_list) in self.candidates.iter() {
            // Filter out candidates that have already been converged or are already present
            // in types
            let candidates_to_consider = candidate_list.iter().filter(|(candidate_name, _)| {
                let candidate_type_name = candidate_name.to_case(Case::Pascal);
                !converged_candidate_set.contains(&candidate_type_name)
                    && !self.config.types.contains_key(&candidate_type_name)
            });

            // Find the candidate with the highest frequency and priority
            if let Some((candidate_name, _)) = candidates_to_consider
                .max_by_key(|(key, value)| (value.priority, value.frequency, *key))
            {
                let singularized_candidate_name = candidate_name.to_case(Case::Pascal);
                finalized_candidates
                    .insert(type_name.to_owned(), singularized_candidate_name.clone());
                converged_candidate_set.insert(singularized_candidate_name);
            }
        }

        finalized_candidates
    }
}

struct CandidateGeneration<'a> {
    /// maintains the generated candidates in the form of
    /// {TypeName: {{candidate_name: {frequency: 1, priority: 2}}}}
    candidates: IndexMap<String, IndexMap<String, CandidateStats>>,
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
        let mut types_processing_order = [
            &self.config.schema.query,
            &self.config.schema.mutation,
            &self.config.schema.subscription,
        ]
        .iter()
        .flat_map(|t| t.as_ref())
        .chain(self.config.types.keys())
        .collect::<Vec<_>>();

        // we want to process the operation types first so we add it manually and then
        // use the all types in config, so inorder to process each type only once we
        // remove the duplicate operation types which got added via
        // `self.config.types.keys()`.
        types_processing_order.dedup();

        for type_name in self.config.types.keys() {
            if let Some(type_info) = self.config.types.get(type_name) {
                for (field_name, field_info) in type_info.fields.iter() {
                    if self.config.is_scalar(&field_info.type_of) {
                        // if output type is scalar then skip it.
                        continue;
                    }

                    let inner_map = self
                        .candidates
                        .entry(field_info.type_of.to_owned())
                        .or_default();

                    let singularized_candidate = pluralizer::pluralize(field_name, 1, false);
                    let priority = if self.config.is_root_operation_type(type_name) {
                        Priority::High // user suggested name has the highest
                                       // priority over
                                       // auto inferred names.
                    } else {
                        Priority::Medium
                    };

                    if let Some(val) = inner_map.get_mut(&singularized_candidate) {
                        val.frequency += 1;
                        val.priority = std::cmp::max(val.priority, priority);
                    } else {
                        // generate the backup name:
                        inner_map.insert(
                            singularized_candidate,
                            CandidateStats { frequency: 1, priority },
                        );
                    }
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
    fn transform(&self, config: Config) -> Valid<Self::Value, Self::Error> {
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

    #[test]
    fn test_prioritize_user_given_names() {
        let sdl = r#"
            schema { query: Query }
            type Post {
                id: Int
            }
            type T2 {
                userPosts: [Posts]
            }
            type Query {
                userInfo: T2
            }
        "#;

        let config = Config::from_sdl(sdl).to_result().unwrap();

        let config = ImproveTypeNames.transform(config).to_result().unwrap();

        insta::assert_snapshot!(config.to_sdl());
    }
}
