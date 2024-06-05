use std::collections::HashMap;

use inflector::{string::singularize::to_singular, Inflector};

use crate::core::{
    config::{transformer::Transform, Config},
    valid::Valid,
};

pub struct TypeNameGenerator;

impl TypeNameGenerator {
    fn generate_candidate_names(&self, config: &Config) -> HashMap<String, HashMap<String, u32>> {
        // HashMap to store mappings between types and their fields
        let mut type_field_mapping: HashMap<String, HashMap<String, u32>> = Default::default();

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
                    .or_insert_with(HashMap::new);

                // Increment the count of the field in the inner map
                *inner_map.entry(field_name.clone()).or_insert(0) += 1;
            }
        }

        type_field_mapping
    }

    fn finalized_candidates(
        &self,
        candidate_mappings: HashMap<String, HashMap<String, u32>>,
    ) -> HashMap<String, String> {
        candidate_mappings
            .iter()
            .flat_map(|(outer_key, inner_map)| {
                // Find the entry with the highest count
                let max_entry = inner_map.iter().max_by_key(|(_, &count)| count);
                // Check if max_entry exists and convert that key into singular type and return it
                if let Some((inner_key, _)) = max_entry {
                    Some((outer_key.clone(), to_singular(inner_key).to_pascal_case()))
                } else {
                    None
                }
            })
            .collect::<HashMap<String, String>>()
    }

    fn generate_type_name(
        &self,
        finalized_candidates: HashMap<String, String>,
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
                            actual_field.type_of = new_name.clone();
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
        // step 1: generate the required mapping.
        let candidate_mappings = self.generate_candidate_names(&config);

        // step 2: find out the most suitable name for type
        let finalized_candidates = self.finalized_candidates(candidate_mappings);

        // step 3: replace its every occurance.
        let config = self.generate_type_name(finalized_candidates, config);

        // step 4: return the config.
        Valid::succeed(config)
    }
}
