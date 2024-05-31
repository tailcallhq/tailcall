use std::collections::{HashMap, HashSet};

use crate::core::{
    config::{Config, ConfigModule, Type},
    valid::Valid,
};

use super::Transform;

pub struct TypeMerger {
    /// thresh required for the merging process.
    thresh: f32,
}

impl TypeMerger {
    pub fn new(thresh: f32) -> Self {
        let mut validated_thresh = thresh;
        if thresh < 0.0 || thresh > 1.0 {
            validated_thresh = 1.0;
            tracing::warn!(
                "Invalid threshold value ({:.2}), reverting to default threshold ({:.2}). allowed range is [0.0 - 1.0] inclusive",
                thresh,
                validated_thresh
            );
        }
        Self { thresh: validated_thresh }
    }
}

impl Default for TypeMerger {
    fn default() -> Self {
        Self { thresh: 1.0 }
    }
}

fn is_type_comparable(type_: &str) -> bool {
    matches!(
        type_.to_lowercase().as_str(),
        "int" | "id" | "boolean" | "float" | "string" | "any" | "empty"
    )
}

/// calculate_distance returns pair of u32 ints -> (count of non-similar fields, total count of fields)
fn calculate_distance(
    config: &Config,
    type_1: &Type,
    type_2: &Type,
    visited_type: &mut HashSet<(String, String)>,
) -> (u32, u32) {
    let mut same_field_cnt = 0;
    let mut not_same_field_count = 0;
    let mut total_field_count = 0;

    for (field_name_1, field_1) in type_1.fields.iter() {
        if let Some(field_2) = type_2.fields.get(field_name_1) {
            let field_1_type_of = field_1.type_of.to_string();
            let field_2_type_of = field_2.type_of.to_string();

            let is_field_1_comparable = is_type_comparable(field_1_type_of.as_str());
            let is_field_2_comparable = is_type_comparable(field_2_type_of.as_str());

            if is_field_1_comparable && is_field_2_comparable {
                same_field_cnt += 2;    // 1 from field_1 + 1 from field_2
            } else if !is_field_1_comparable && !is_field_2_comparable {
                if visited_type.contains(&(field_1_type_of.clone(), field_2_type_of.clone())) {
                    same_field_cnt += 2;
                    continue;
                } else if visited_type.contains(&(field_2_type_of.clone(), field_1_type_of.clone()))
                {
                    same_field_cnt += 2;
                    continue;
                }

                let type_a = config.types.get(field_1_type_of.as_str()).unwrap();
                let type_b = config.types.get(field_2_type_of.as_str()).unwrap();

                visited_type.insert((field_1_type_of, field_2_type_of));

                let pair = calculate_distance(config, type_a, type_b, visited_type);

                not_same_field_count += pair.0;
                total_field_count += pair.1;
            }
        }
    }

    not_same_field_count += (type_1.fields.len() + type_2.fields.len()) as u32 - same_field_cnt;
    total_field_count += (type_1.fields.len() + type_2.fields.len()) as u32;

    (not_same_field_count, total_field_count)
}

impl TypeMerger {
    fn merger(&self, mut merge_counter: u32, mut config: Config) -> Config {
        let mut type_to_merge_type_mapping = HashMap::new();
        let mut similar_type_group_list: Vec<HashSet<String>> = vec![];
        let mut visited_types = HashSet::new();
        let mut i = 0;

        // step 1: identify all the types that satisfies the thresh criteria and group them.
        let query_name = config.schema.query.clone().unwrap_or_default();
        for (type_name_1, type_info_1) in config.types.iter() {
            if visited_types.contains(type_name_1) || type_name_1 == query_name.as_str() {
                continue;
            }

            let mut type_1_sim = HashSet::new();
            type_1_sim.insert(type_name_1.to_string());

            for (type_name_2, type_info_2) in config.types.iter().skip(i + 1) {
                if visited_types.contains(type_name_2)
                    || type_name_1 == type_name_2
                    || type_name_2 == query_name.as_str()
                {
                    continue;
                }
                let distance_pair =
                    calculate_distance(&config, type_info_1, type_info_2, &mut HashSet::new());
                let distance = distance_pair.0 as f32 / distance_pair.1 as f32;
                if 1.0 - distance >= self.thresh {
                    visited_types.insert(type_name_2.clone());
                    type_1_sim.insert(type_name_2.clone());
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
            let merged_type_name = format!("M{}", merge_counter);
            merge_counter += 1;

            for type_name in same_types {
                type_to_merge_type_mapping.insert(type_name.clone(), merged_type_name.clone());
                let type_ = config.types.get(type_name.as_str()).unwrap();
                merged_into = merge_type(type_, merged_into);
            }

            config.types.insert(merged_type_name, merged_into);
        }

        // step 3: replace typeof of fields with newly merged types.
        for  type_info in config.types.values_mut() {
            for actual_field in type_info.fields.values_mut() {
                if let Some(merged_into_type_name) =
                    type_to_merge_type_mapping.get(actual_field.type_of.as_str())
                {
                    actual_field.type_of = merged_into_type_name.to_string();
                }
            }
        }

        // step 4: remove all merged types.
        let unused_types: HashSet<_> = type_to_merge_type_mapping.keys().cloned().collect();
        let repeat_merging = unused_types.len() > 0;
        config = config.remove_types(unused_types);

        if repeat_merging {
            return self.merger(merge_counter, config);
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

impl Transform for TypeMerger {
    fn transform(&self, config: Config) -> Valid<Config, String> {
        let config = self.merger(1, config);
        Valid::succeed(config)
    }
}

#[cfg(test)]
mod test {
    use super::TypeMerger;
    use crate::core::{config::{transformer::{type_merger::is_type_comparable, Transform}, Config, ConfigModule, Field, Type}, valid::Validator};
    
    #[test]
    fn test_validate_thresh() {
        let ty_merger = TypeMerger::new(0.0);
        assert_eq!(ty_merger.thresh, 0.0);

        let ty_merger = TypeMerger::new(1.2);
        assert_eq!(ty_merger.thresh, 1.0);

        let ty_merger = TypeMerger::new(-0.5);
        assert_eq!(ty_merger.thresh, 1.0);

        let ty_merger = TypeMerger::new(0.5);
        assert_eq!(ty_merger.thresh, 0.5);
    }

    #[test]
    fn test_cyclic_merge_case() -> anyhow::Result<()> {
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
        
        config = TypeMerger::new(0.5).transform(config).to_result()?;

        insta::assert_snapshot!(config.to_sdl());

        Ok(())
    }

    #[test]
    fn test_type_merger() -> anyhow::Result<()> {
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

        config = TypeMerger::new(1.0).transform(config).to_result()?;

        assert_eq!(config.types.len(), 2);
        insta::assert_snapshot!(config.to_sdl());
        Ok(())
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
