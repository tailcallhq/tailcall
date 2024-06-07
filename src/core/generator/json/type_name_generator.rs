use std::collections::{BTreeMap, HashSet};

use inflector::Inflector;

use crate::core::config::transformer::Transform;
use crate::core::config::Config;
use crate::core::valid::Valid;

pub struct TypeNameGenerator;

impl TypeNameGenerator {
    fn generate_candidate_names(&self, config: &Config) -> BTreeMap<String, BTreeMap<String, u32>> {
        let mut type_to_candidate_field_mapping: BTreeMap<String, BTreeMap<String, u32>> =
            Default::default();

        let ingore_type_list = config.get_operation_type_names();

        for (type_name, type_info) in config.types.iter() {
            if ingore_type_list.contains(type_name) {
                // ignore operation type fields as it's fields are auto-generated and doesn't
                // help in us in type name generation.
                continue;
            }

            for (field_name, field_info) in type_info.fields.iter() {
                if config.is_scalar(&field_info.type_of) {
                    continue;
                }

                let inner_map = type_to_candidate_field_mapping
                    .entry(field_info.type_of.to_owned())
                    .or_default();

                *inner_map.entry(field_name.to_owned()).or_insert(0) += 1;
            }
        }

        type_to_candidate_field_mapping
    }

    fn finalize_candidates(
        &self,
        candidate_mappings: BTreeMap<String, BTreeMap<String, u32>>,
    ) -> BTreeMap<String, String> {
        let mut finalized_candidates = BTreeMap::new();
        let mut converged_candidate_set = HashSet::new();

        for (type_name, candidate_list) in candidate_mappings {
            // Find the most frequent candidate that hasn't been converged yet.
            if let Some((candidate_name, _)) = candidate_list
                .into_iter()
                .max_by_key(|&(_, count)| count)
                .filter(|(candidate_name, _)| !converged_candidate_set.contains(candidate_name))
            {
                let singularized_candidate_name = candidate_name.to_singular().to_pascal_case();
                finalized_candidates.insert(type_name, singularized_candidate_name);
                converged_candidate_set.insert(candidate_name);
            }
        }

        finalized_candidates
    }

    fn generate_type_name(
        &self,
        finalized_candidates: BTreeMap<String, String>,
        mut config: Config,
    ) -> Config {
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
        // step 1: generate the required candidate mappings. i.e { Type :
        // [{candidate_name : count}] }
        let candidate_mappings = self.generate_candidate_names(&config);

        // step 2: converge on the candidate name. i.e { Type : Candidate_Name }
        let finalized_candidates = self.finalize_candidates(candidate_mappings);

        // step 3: replace its every occurance.
        let config = self.generate_type_name(finalized_candidates, config);

        // step 4: return the config.
        Valid::succeed(config)
    }
}

#[cfg(test)]
mod test {
    use anyhow::Ok;

    use super::TypeNameGenerator;
    use crate::core::config::transformer::Transform;
    use crate::core::config::Config;
    use crate::core::valid::Validator;

    #[test]
    fn test_type_name_generator_transform() -> anyhow::Result<()> {
        let config = Config::from_sdl(
            r#"schema @server @upstream {
            query: Query
          }
          
          type Query {
            f1: [T31] @http(baseURL: "https://jsonplaceholder.typicode.com", path: "/users")
          }
          
          type T1 {
            lat: String
            lng: String
          }
          
          type T2 {
            city: String
            geo: T1
            street: String
            suite: String
            zipcode: String
          }
          
          type T3 {
            bs: String
            catchPhrase: String
            name: String
          }
          
          type T31 {
            address: T2
            company: T3
            email: String
            id: Int
            name: String
            phone: String
            username: String
            website: String
          }
          "#,
        )
        .to_result()?;

        let transformed_config = TypeNameGenerator.transform(config).to_result()?;
        insta::assert_snapshot!(transformed_config.to_sdl());

        Ok(())
    }

    #[test]
    fn test_type_name_generator_with_cyclic_types() -> anyhow::Result<()> {
        let config = Config::from_sdl(
            r#"schema @server @upstream {
            query: Query
          }
          
          type Query {
            f1: [T31] @http(baseURL: "https://jsonplaceholder.typicode.com", path: "/users")
          }
          
          type T31 {
            id: ID!
            name: String!
            posts: [T32]!
          }

          type T32 {
            id: ID!
            title: String!
            content: String!
            author: T31!
            cycle: T33
          }

          type T33 {
            id: ID!
            cycle: T33
          }
          "#,
        )
        .to_result()?;

        let transformed_config = TypeNameGenerator.transform(config).to_result()?;
        insta::assert_snapshot!(transformed_config.to_sdl());

        Ok(())
    }
}
