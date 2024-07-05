use std::collections::{BTreeMap, BTreeSet, HashSet};

use super::mergeable_types::MergeableTypes;
use super::similarity::Similarity;
use crate::core::config::{Config, Type};
use crate::core::merge_right::MergeRight;
use crate::core::transform::Transform;
use crate::core::valid::{Valid, Validator};

pub struct TypeMerger {
    /// threshold required for the merging process.
    threshold: f32,
}

impl TypeMerger {
    pub fn new(threshold: f32) -> Self {
        let mut validated_thresh = threshold;
        if !(0.0..=1.0).contains(&threshold) {
            validated_thresh = 1.0;
            tracing::warn!(
                "Invalid threshold value ({:.2}), reverting to default threshold ({:.2}). allowed range is [0.0 - 1.0] inclusive",
                threshold,
                validated_thresh
            );
        }
        Self { threshold: validated_thresh }
    }
}

impl Default for TypeMerger {
    fn default() -> Self {
        Self { threshold: 1.0 }
    }
}

impl TypeMerger {
    fn merger(&self, mut merge_counter: u32, mut config: Config) -> Config {
        let mut type_to_merge_type_mapping = BTreeMap::new();
        let mut similar_type_group_list: Vec<BTreeSet<String>> = vec![];
        let mut visited_types = HashSet::new();
        let mut i = 0;
        let mut stat_gen = Similarity::new(&config);
        let mergeable_types = MergeableTypes::new(&config, self.threshold);

        // step 1: identify all the types that satisfies the thresh criteria and group
        // them.
        for type_name_1 in mergeable_types.iter() {
            if let Some(type_info_1) = config.types.get(type_name_1) {
                if visited_types.contains(type_name_1) {
                    continue;
                }

                let mut similar_type_set = BTreeSet::new();
                similar_type_set.insert(type_name_1.to_string());

                for type_name_2 in mergeable_types.iter().skip(i + 1) {
                    if visited_types.contains(type_name_2)
                        || !mergeable_types.mergeable(type_name_1, type_name_2)
                    {
                        continue;
                    }

                    if let Some(type_info_2) = config.types.get(type_name_2) {
                        let threshold = mergeable_types.get_threshold(type_name_1, type_name_2);

                        visited_types.insert(type_name_1.clone());
                        let is_similar = stat_gen
                            .similarity(
                                (type_name_1, type_info_1),
                                (type_name_2, type_info_2),
                                threshold,
                            )
                            .to_result();
                        if let Ok(similar) = is_similar {
                            if similar {
                                visited_types.insert(type_name_2.clone());
                                similar_type_set.insert(type_name_2.to_owned());
                            }
                        }
                    }
                }
                if similar_type_set.len() > 1 {
                    similar_type_group_list.push(similar_type_set);
                }
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
            let mut did_we_merge = false;
            for type_name in same_types {
                if let Some(type_) = config.types.get(type_name.as_str()) {
                    type_to_merge_type_mapping.insert(type_name.clone(), merged_type_name.clone());
                    merged_into = merge_type(type_, merged_into);
                    did_we_merge = true;
                }
            }

            if did_we_merge {
                config.types.insert(merged_type_name, merged_into);
                merge_counter += 1;
            }
        }

        if type_to_merge_type_mapping.is_empty() {
            return config;
        }

        // step 3: replace typeof of fields with newly merged types.
        for type_info in config.types.values_mut() {
            for actual_field in type_info.fields.values_mut() {
                if let Some(merged_into_type_name) =
                    type_to_merge_type_mapping.get(actual_field.type_of.as_str())
                {
                    actual_field.type_of = merged_into_type_name.to_string();
                }

                // make the changes in the input arguments as well.
                for arg_ in actual_field.args.values_mut() {
                    if let Some(merge_into_type_name) =
                        type_to_merge_type_mapping.get(arg_.type_of.as_str())
                    {
                        arg_.type_of = merge_into_type_name.to_string();
                    }
                }
            }
            // replace the merged type names in interface.
            type_info.implements = type_info
                .implements
                .iter()
                .filter_map(|interface_type_name| {
                    type_to_merge_type_mapping
                        .get(interface_type_name)
                        .cloned()
                        .or(Some(interface_type_name.clone()))
                })
                .collect();
        }

        // replace the merged types in union as well.
        for union_type_ in config.unions.values_mut() {
            // Collect changes to be made
            let mut types_to_remove = HashSet::new();
            let mut types_to_add = HashSet::new();

            for type_name in union_type_.types.iter() {
                if let Some(merge_into_type_name) = type_to_merge_type_mapping.get(type_name) {
                    types_to_remove.insert(type_name.clone());
                    types_to_add.insert(merge_into_type_name.clone());
                }
            }
            // Apply changes
            for type_name in types_to_remove {
                union_type_.types.remove(&type_name);
            }

            for type_name in types_to_add {
                union_type_.types.insert(type_name);
            }
        }

        // replace the merged types in union as well.
        for union_type_ in config.unions.values_mut() {
            union_type_.types = union_type_
                .types
                .iter()
                .filter_map(|type_name| {
                    type_to_merge_type_mapping
                        .get(type_name)
                        .cloned()
                        .or(Some(type_name.clone()))
                })
                .collect();
        }

        // step 4: remove all merged types.
        let unused_types: HashSet<_> = type_to_merge_type_mapping.keys().cloned().collect();
        let repeat_merging = !unused_types.is_empty();
        config = config.remove_types(unused_types);

        if repeat_merging {
            return self.merger(merge_counter, config);
        }
        config
    }
}

fn merge_type(type_: &Type, merge_into: Type) -> Type {
    merge_into.merge_right(type_.clone())
}

impl Transform for TypeMerger {
    type Value = Config;
    type Error = String;
    fn transform(&self, config: Config) -> Valid<Config, String> {
        let config = self.merger(1, config);
        Valid::succeed(config)
    }
}

#[cfg(test)]
mod test {
    use tailcall_fixtures;

    use super::TypeMerger;
    use crate::core::config::{Config, Field, Type};
    use crate::core::transform::Transform;
    use crate::core::valid::Validator;

    #[test]
    fn test_validate_thresh() {
        let ty_merger = TypeMerger::default();
        assert_eq!(ty_merger.threshold, 1.0);

        let ty_merger = TypeMerger::new(0.0);
        assert_eq!(ty_merger.threshold, 0.0);

        let ty_merger = TypeMerger::new(1.2);
        assert_eq!(ty_merger.threshold, 1.0);

        let ty_merger = TypeMerger::new(-0.5);
        assert_eq!(ty_merger.threshold, 1.0);

        let ty_merger = TypeMerger::new(0.5);
        assert_eq!(ty_merger.threshold, 0.5);
    }

    #[test]
    fn test_cyclic_merge_case() -> anyhow::Result<()> {
        let str_field = Field { type_of: "String".to_owned(), ..Default::default() };
        let int_field = Field { type_of: "Int".to_owned(), ..Default::default() };
        let bool_field = Field { type_of: "Boolean".to_owned(), ..Default::default() };

        let mut ty1 = Type::default();
        ty1.fields.insert("body".to_string(), str_field.clone());
        ty1.fields.insert("id".to_string(), int_field.clone());
        ty1.fields
            .insert("is_verified".to_string(), bool_field.clone());
        ty1.fields.insert("userId".to_string(), int_field.clone());

        let mut ty2 = Type::default();
        ty2.fields.insert(
            "t1".to_string(),
            Field { type_of: "T1".to_string(), ..Default::default() },
        );
        ty2.fields
            .insert("is_verified".to_string(), bool_field.clone());
        ty2.fields.insert("userId".to_string(), int_field.clone());
        ty2.fields.insert("body".to_string(), str_field.clone());

        let mut config = Config::default();

        config.types.insert("T1".to_string(), ty1);
        config.types.insert("T2".to_string(), ty2);

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
    fn test_input_types() {
        let sdl = std::fs::read_to_string(tailcall_fixtures::configs::INPUT_TYPE_CONFIG).unwrap();
        let config = Config::from_sdl(&sdl).to_result().unwrap();
        let config = TypeMerger::default().transform(config).to_result().unwrap();
        insta::assert_snapshot!(config.to_sdl());
    }

    #[test]
    fn test_union_types() {
        let sdl = std::fs::read_to_string(tailcall_fixtures::configs::UNION_CONFIG).unwrap();
        let config = Config::from_sdl(&sdl).to_result().unwrap();
        let config = TypeMerger::default().transform(config).to_result().unwrap();
        insta::assert_snapshot!(config.to_sdl());
    }

    #[test]
    fn test_list_field_types() {
        let sdl = std::fs::read_to_string(tailcall_fixtures::configs::USER_LIST).unwrap();
        let config = Config::from_sdl(&sdl).to_result().unwrap();
        let config = TypeMerger::default().transform(config).to_result().unwrap();
        insta::assert_snapshot!(config.to_sdl());
    }

    #[test]
    fn test_fail_when_scalar_field_not_match() {
        let str_field = Field { type_of: "String".to_owned(), ..Default::default() };
        let int_field = Field { type_of: "Int".to_owned(), ..Default::default() };

        let mut ty1 = Type::default();
        ty1.fields.insert("a".to_string(), int_field.clone());
        ty1.fields.insert("b".to_string(), int_field.clone());
        ty1.fields.insert("c".to_string(), int_field.clone());

        let mut ty2 = Type::default();
        ty2.fields.insert("a".to_string(), int_field.clone());
        ty2.fields.insert("b".to_string(), int_field.clone());
        ty2.fields.insert("c".to_string(), str_field.clone());

        let mut config = Config::default();
        config.types.insert("T1".to_string(), ty1);
        config.types.insert("T2".to_string(), ty2);

        let config = TypeMerger::new(0.5).transform(config).to_result().unwrap();
        insta::assert_snapshot!(config.to_sdl());
    }

    #[test]
    fn test_interface_types() {
        let int_field = Field { type_of: "Int".to_owned(), ..Default::default() };

        let mut ty1 = Type::default();
        ty1.fields.insert("a".to_string(), int_field.clone());

        let mut ty2 = Type::default();
        ty2.fields.insert("a".to_string(), int_field.clone());

        let mut ty3 = Type::default();
        ty3.fields.insert("a".to_string(), int_field.clone());

        ty3.implements.insert("A".to_string());
        ty3.implements.insert("B".to_string());

        let mut config = Config::default();
        config.types.insert("A".to_string(), ty1);
        config.types.insert("B".to_string(), ty2);
        config.types.insert("C".to_string(), ty3);

        let config = TypeMerger::default().transform(config).to_result().unwrap();
        insta::assert_snapshot!(config.to_sdl());
    }
}
