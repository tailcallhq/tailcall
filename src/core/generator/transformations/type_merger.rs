use std::collections::{HashMap, HashSet};

use crate::core::{config::{Config, Type}, generator::json::ConfigTransformer};

pub struct TypeMerger {
    /// thresh required for the merging process.
    thresh: f32,
    /// used to generate merge type unique names.
    merge_counter: u32,
}

impl TypeMerger {
    pub fn new(thresh: f32) -> Self {
        let mut validated_thresh = thresh;
        if thresh <= 0.0 || thresh > 1.0 {
            validated_thresh = 1.0;
            tracing::warn!(
                "Invalid threshold value ({:.2}), reverting to default threshold ({:.2}). allowed range is [0.1 - 1.0] inclusive",
                thresh,
                validated_thresh
            );
        }
        Self { thresh: validated_thresh, merge_counter: 1 }
    }
}

impl Default for TypeMerger {
    fn default() -> Self {
        Self { thresh: 1.0, merge_counter: 1 }
    }
}

fn calculate_similarity(
    config: &Config,
    type_info_1: &Type,
    type_info_2: &Type,
    thresh: f32,
) -> bool {
    let matching_fields_count = count_matching_fields(config, type_info_1, type_info_2, thresh);
    let total_fields_count =
        (type_info_1.fields.len() + type_info_2.fields.len()) - matching_fields_count;

    total_fields_count > 0 && (matching_fields_count as f32 / total_fields_count as f32) >= thresh
}

fn is_type_comparable(type_: &str) -> bool {
    matches!(
        type_.to_lowercase().as_str(),
        "int" | "id" | "boolean" | "float" | "string" | "any" | "empty"
    )
}

fn count_matching_fields(
    config: &Config,
    type_info_1: &Type,
    type_info_2: &Type,
    thresh: f32,
) -> usize {
    type_info_1
        .fields
        .iter()
        .filter(|(field_name, field_info_1)| {
            type_info_2
                .fields
                .get(*field_name)
                .map_or(false, |field_info_2| {
                    // if they are primitive types then easy to compare else drill down on the inner
                    // types to check if they satisfies the thresh criteria.
                    let field1_primitive = is_type_comparable(field_info_1.type_of.as_str());
                    let field2_primitive = is_type_comparable(field_info_2.type_of.as_str());

                    if field1_primitive && field2_primitive {
                        field_info_1.type_of == field_info_2.type_of
                    } else if !field1_primitive && !field2_primitive {
                        calculate_similarity(
                            config,
                            config.types.get(field_info_1.type_of.as_str()).unwrap(),
                            config.types.get(field_info_2.type_of.as_str()).unwrap(),
                            thresh,
                        )
                    } else {
                        false
                    }
                })
        })
        .count()
}

fn check_for_conflicts(grouped_types: &HashSet<String>, type_name: &str, config: &Config) -> bool {
    if grouped_types.iter().any(|grp_type_name| {
        config.types.get(grp_type_name).map_or(false, |type_| {
            type_
                .fields
                .values()
                .any(|field| field.type_of == type_name)
        })
    }) {
        return true;
    }

    config.types.get(type_name).map_or(false, |type_| {
        type_
            .fields
            .values()
            .any(|field| grouped_types.contains(&field.type_of))
    })
}

impl ConfigTransformer for TypeMerger {
    fn apply(&mut self, mut config: Config) -> Config {
        let mut type_to_merge_type_mapping = HashMap::new();
        let mut similar_type_group_list: Vec<HashSet<String>> = vec![];
        let mut visited_types = HashSet::new();
        let mut i = 0;

        // step 1: identify all the types that satisfies the thresh criteria and group them.
        for (type_name_1, type_info_1) in config.types.iter() {
            if visited_types.contains(type_name_1) || type_name_1 == "Query" {
                continue;
            }

            let mut type_1_sim = HashSet::new();
            type_1_sim.insert(type_name_1.to_string());

            for (type_name_2, type_info_2) in config.types.iter().skip(i + 1) {
                if visited_types.contains(type_name_2) {
                    continue;
                }
                let is_similar =
                    calculate_similarity(&config, type_info_1, type_info_2, self.thresh);
                if is_similar {
                    let has_conflicts = check_for_conflicts(&type_1_sim, type_name_2, &config);
                    if !has_conflicts {
                        visited_types.insert(type_name_2.clone());
                        type_1_sim.insert(type_name_2.clone());
                    }
                }
            }
            if type_1_sim.len() > 1 {
                similar_type_group_list.push(type_1_sim);
            }

            i += 1;
        }

        if similar_type_group_list.is_empty() {
            return config;
        }

        // step 2: merge similar types into single merged type.
        for same_types in similar_type_group_list {
            let mut merged_into = Type::default();
            let merged_type_name = format!("M{}", self.merge_counter);
            self.merge_counter += 1;

            for type_name in same_types {
                type_to_merge_type_mapping.insert(type_name.clone(), merged_type_name.clone());
                let type_ = config.types.get(type_name.as_str()).unwrap();
                merged_into = merge_type(type_, merged_into);
            }

            config.types.insert(merged_type_name, merged_into);
        }

        // step 3: replace typeof of fields with newly merged types.
        for (type_name, type_info) in config.types.iter_mut() {
            for actual_field in type_info.fields.values_mut() {
                if let Some(merged_into_type_name) =
                    type_to_merge_type_mapping.get(actual_field.type_of.as_str())
                {
                    if merged_into_type_name == type_name {
                        tracing::info!("Found cyclic type, reverting merging.");
                        continue;
                    }
                    actual_field.type_of = merged_into_type_name.to_string();
                }
            }
        }

        // step 4: remove all merged types.
        let unused_types: HashSet<_> = type_to_merge_type_mapping.keys().cloned().collect();
        let repeat_merging = unused_types.len() > 0;
        config = config.remove_types(unused_types);

        if repeat_merging {
            return self.apply(config);
        }
        config
    }
}

fn merge_type(type_: &Type, mut merge_into: Type) -> Type {
    merge_into.fields.extend(type_.fields.clone());
    merge_into
        .added_fields
        .extend(type_.added_fields.iter().cloned());
    merge_into
        .implements
        .extend(type_.implements.iter().cloned());

    merge_into
}

#[cfg(test)]
mod test {
    use super::TypeMerger;
    use crate::core::config::{Config, Field, Type};
    use crate::core::generator::transformations::type_merger::is_type_comparable;
    use crate::core::generator::json::ConfigTransformer;
    
    #[test]
    fn test_validate_thresh() {
        let ty_merger = TypeMerger::new(0.0);
        assert_eq!(ty_merger.thresh, 1.0);

        let ty_merger = TypeMerger::new(1.2);
        assert_eq!(ty_merger.thresh, 1.0);

        let ty_merger = TypeMerger::new(-0.5);
        assert_eq!(ty_merger.thresh, 1.0);

        let ty_merger = TypeMerger::new(0.5);
        assert_eq!(ty_merger.thresh, 0.5);
    }

    #[test]
    fn test_cyclic_merge_case() {
        let str_field = Field { type_of: "String".to_owned(), ..Default::default() };
        let int_field = Field { type_of: "Int".to_owned(), ..Default::default() };
        let bool_field = Field { type_of: "Boolean".to_owned(), ..Default::default() };
        let float_field = Field { type_of: "Float".to_owned(), ..Default::default() };

        let mut ty = Type::default();
        ty.fields.insert("body".to_string(), str_field.clone());
        ty.fields.insert("id".to_string(), int_field.clone());
        ty.fields.insert("title".to_string(), bool_field.clone());
        ty.fields.insert("userId".to_string(), float_field.clone());

        let mut ty1 = Type::default();
        ty1.fields.insert(
            "t1".to_string(),
            Field { type_of: "T1".to_string(), ..Default::default() },
        );
        ty1.fields.insert("lat".to_string(), str_field.clone());
        ty1.fields.insert("lng".to_string(), int_field.clone());
        ty1.fields.insert("title".to_string(), bool_field.clone());
        ty1.fields.insert("userId".to_string(), float_field.clone());

        let mut config = Config::default();

        config.types.insert("T1".to_string(), ty);
        config.types.insert("T2".to_string(), ty1);

        let mut q_type = Type::default();
        q_type.fields.insert(
            "q1".to_string(),
            Field { type_of: "T1".to_string(), ..Default::default() },
        );
        q_type.fields.insert(
            "q2".to_string(),
            Field { type_of: "T2".to_string(), ..Default::default() },
        );

        config.types.insert("Query".to_owned(), q_type);
        config = config.query("Query");

        insta::assert_snapshot!(TypeMerger::new(0.5).apply(config).to_sdl());
    }

    #[test]
    fn test_type_merger() {
        let str_field = Field { type_of: "String".to_owned(), ..Default::default() };
        let int_field = Field { type_of: "Int".to_owned(), ..Default::default() };
        let bool_field = Field { type_of: "Boolean".to_owned(), ..Default::default() };
        let float_field = Field { type_of: "Float".to_owned(), ..Default::default() };
        let id_field = Field { type_of: "ID".to_owned(), ..Default::default() };

        let mut ty = Type::default();
        ty.fields.insert("f1".to_string(), str_field.clone());
        ty.fields.insert("f2".to_string(), int_field.clone());
        ty.fields.insert("f3".to_string(), bool_field.clone());
        ty.fields.insert("f4".to_string(), float_field.clone());
        ty.fields.insert("f5".to_string(), id_field.clone());

        let mut config = Config::default();
        config.types.insert("T1".to_string(), ty.clone());
        config.types.insert("T2".to_string(), ty.clone());
        config.types.insert("T3".to_string(), ty.clone());
        config.types.insert("T4".to_string(), ty.clone());

        let mut q_type = Type::default();
        q_type.fields.insert(
            "q1".to_string(),
            Field { type_of: "T1".to_string(), ..Default::default() },
        );
        q_type.fields.insert(
            "q2".to_string(),
            Field { type_of: "T2".to_string(), ..Default::default() },
        );
        q_type.fields.insert(
            "q3".to_string(),
            Field { type_of: "T3".to_string(), ..Default::default() },
        );
        q_type.fields.insert(
            "q4".to_string(),
            Field { type_of: "T4".to_string(), ..Default::default() },
        );

        config.types.insert("Query".to_owned(), q_type);
        config = config.query("Query");

        assert_eq!(config.types.len(), 5);

        config = TypeMerger::new(1.0).apply(config);

        assert_eq!(config.types.len(), 2);
        insta::assert_snapshot!(config.to_sdl());
    }

    #[test]
    fn test_is_type_comparable() {
        assert!(is_type_comparable("int"));
        assert!(is_type_comparable("Int"));
        assert!(is_type_comparable("INT"));

        assert!(is_type_comparable("id"));
        assert!(is_type_comparable("ID"));
        assert!(is_type_comparable("Id"));

        assert!(is_type_comparable("boolean"));
        assert!(is_type_comparable("Boolean"));

        assert!(is_type_comparable("float"));
        assert!(is_type_comparable("Float"));

        assert!(is_type_comparable("string"));
        assert!(is_type_comparable("String"));

        assert!(is_type_comparable("empty"));
        assert!(is_type_comparable("Empty"));
        assert!(is_type_comparable("Any"));
        assert!(is_type_comparable("any"));

        assert!(!is_type_comparable("T1"));
        assert!(!is_type_comparable("t1"));
        assert!(!is_type_comparable("M1"));
        assert!(!is_type_comparable("m1"));
    }
}
