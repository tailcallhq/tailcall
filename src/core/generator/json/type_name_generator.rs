use std::collections::{BTreeMap, HashSet};

use inflector::Inflector;

use super::NameGenerator;
use crate::core::config::transformer::Transform;
use crate::core::config::Config;
use crate::core::valid::Valid;

pub struct TypeNameGenerator<'a> {
    root_name: &'a str,
    name_generator: &'a NameGenerator,
}

impl<'a> TypeNameGenerator<'a> {
    pub fn new(root_name: &'a str, name_generator: &'a NameGenerator) -> Self {
        Self { root_name, name_generator }
    }

    fn generate_candidate_names(&self, config: &Config) -> BTreeMap<String, BTreeMap<String, u32>> {
        // HashMap to store mappings between types and their fields
        let mut type_field_mapping: BTreeMap<String, BTreeMap<String, u32>> = Default::default();

        // Iterate through each type in the configuration
        let query_name = config.schema.query.clone().unwrap_or_default().to_string();
        let mutation_name = config
            .schema
            .mutation
            .clone()
            .unwrap_or_default()
            .to_string();
        let subscription_name = config
            .schema
            .subscription
            .clone()
            .unwrap_or_default()
            .to_string();
        let ingore_type_list = [query_name, mutation_name, subscription_name];

        for (type_name, type_info) in config.types.iter() {
            if ingore_type_list.contains(type_name) {
                continue;
            }

            // Iterate through each field in the type
            for (field_name, field_info) in type_info.fields.iter() {
                if config.is_scalar(&field_info.type_of) {
                    continue;
                }

                // Access or create the inner HashMap for the field type
                let inner_map = type_field_mapping
                    .entry(field_info.type_of.clone())
                    .or_default();

                // Increment the count of the field in the inner map
                *inner_map.entry(field_name.clone()).or_insert(0) += 1;
            }
        }

        type_field_mapping
    }

    fn finalize_candidates(
        &self,
        candidate_mappings: BTreeMap<String, BTreeMap<String, u32>>,
    ) -> BTreeMap<String, String> {
        let mut finalized_candidates = BTreeMap::new();
        let mut converged_candidate_set = HashSet::new();

        for (type_name, candidate_list) in candidate_mappings {
            // Find the most frequent candidate that hasn't been converged yet
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
        for (old_name, new_name) in finalized_candidates {
            if let Some(type_) = config.types.remove(old_name.as_str()) {
                // Add newly generated type.
                config.types.insert(new_name.clone(), type_);

                // Replace all the instances of old name in config.
                for (_, actual_type) in config.types.iter_mut() {
                    for (_, actual_field) in actual_type.fields.iter_mut() {
                        if actual_field.type_of == old_name {
                            // Update the field's type with the new name
                            actual_field.type_of.clone_from(&new_name);
                        }
                    }
                }
            }
        }
        config
    }

    fn generate_root_type_name(&self, mut config: Config) -> Config {
        // First, iterate and replace all the field types
        for type_ in config.types.values_mut() {
            for field_ in type_.fields.values_mut() {
                if field_.type_of == self.name_generator.get_name() {
                    self.root_name.clone_into(&mut field_.type_of)
                }
            }
        }

        // Now, handle the renaming of the type itself
        if let Some(type_) = config.types.remove(&self.name_generator.get_name()) {
            config.types.insert(self.root_name.to_owned(), type_);
        }

        config
    }
}

impl Transform for TypeNameGenerator<'_> {
    fn transform(&self, config: Config) -> Valid<Config, String> {
        // step 1: generate the required candidate mappings. i.e { Type :
        // [{candidate_name : count}] }
        let candidate_mappings = self.generate_candidate_names(&config);

        // step 2: converge on the candidate name. i.e { Type : Candidate_Name }
        let finalized_candidates = self.finalize_candidates(candidate_mappings);

        // step 3: replace its every occurance.
        let config = self.generate_type_name(finalized_candidates, config);

        // step 4: replace the generated type name with user provided name if it's
        // possible. i.e if we are able to generate the correct name for root type then
        // user provided name isn't used.
        let config = self.generate_root_type_name(config);

        // step 5: return the config.
        Valid::succeed(config)
    }
}

#[cfg(test)]
mod test {
    use std::cell::RefCell;

    use anyhow::Ok;

    use super::TypeNameGenerator;
    use crate::core::config::transformer::Transform;
    use crate::core::config::Config;
    use crate::core::counter::Counter;
    use crate::core::generator::json::NameGenerator;
    use crate::core::valid::Validator;

    impl NameGenerator {
        pub fn init(start: usize, prefix: &str) -> Self {
            Self {
                counter: Counter::new(start),
                prefix: prefix.to_owned(),
                current_name: RefCell::new(format!("{}{}", prefix, start)),
            }
        }
    }

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

        let name_generator = &NameGenerator::init(31, "T");
        let transformed_config = TypeNameGenerator::new("RootType", name_generator)
            .transform(config)
            .to_result()?;
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

        let name_generator = &NameGenerator::init(31, "T");
        let transformed_config = TypeNameGenerator::new("RootType", name_generator)
            .transform(config)
            .to_result()?;
        insta::assert_snapshot!(transformed_config.to_sdl());

        Ok(())
    }
}
