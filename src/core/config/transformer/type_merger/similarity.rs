use super::pair_map::PairMap;
use super::pair_set::PairSet;
use crate::core::config::{Config, Type};

/// Given Two types,it tells similarity between two types based on a specified
/// threshold.
pub struct Similarity<'a> {
    config: &'a Config,
    type_similarity_cache: PairMap<String, bool>,
}

/// holds the necessary information for comparing the similarity between two
/// types.
struct SimilarityTypeInfo<'a> {
    type_1_name: &'a str,
    type_1: &'a Type,
    type_2_name: &'a str,
    type_2: &'a Type,
}

impl<'a> Similarity<'a> {
    pub fn new(config: &'a Config) -> Similarity {
        Similarity { config, type_similarity_cache: PairMap::default() }
    }

    pub fn similarity(
        &mut self,
        (type_1_name, type_1): (&str, &Type),
        (type_2_name, type_2): (&str, &Type),
        threshold: f32,
    ) -> bool {
        let type_info = SimilarityTypeInfo { type_1_name, type_1, type_2, type_2_name };

        self.similarity_inner(type_info, &mut PairSet::default(), threshold)
    }

    fn similarity_inner(
        &mut self,
        type_info: SimilarityTypeInfo,
        visited_type: &mut PairSet<String>,
        threshold: f32,
    ) -> bool {
        let type_1_name = type_info.type_1_name;
        let type_2_name = type_info.type_2_name;
        let type_1 = type_info.type_1;
        let type_2 = type_info.type_2;

        if let Some(type_similarity_result) = self
            .type_similarity_cache
            .get(&type_1_name.to_string(), &type_2_name.to_string())
        {
            *type_similarity_result
        } else {
            let config = self.config;
            let mut same_field_count = 0;

            for (field_name_1, field_1) in type_1.fields.iter() {
                if let Some(field_2) = type_2.fields.get(field_name_1) {
                    let field_1_type_of = field_1.type_of.to_owned();
                    let field_2_type_of = field_2.type_of.to_owned();

                    if field_1_type_of == field_2_type_of {
                        // in order to consider the fields to be exactly same.
                        // it's output type must match (required bounds should match too).
                        if field_1.list == field_2.list
                            && field_1.required == field_2.required
                            && field_1.list_type_required == field_2.list_type_required
                        {
                            same_field_count += 1;
                        }
                    } else if let Some(type_1) = config.types.get(field_1_type_of.as_str()) {
                        if let Some(type_2) = config.types.get(field_2_type_of.as_str()) {
                            if visited_type.contains(&field_1_type_of, &field_2_type_of) {
                                // it's cyclic type, return true as they're the same.
                                return true;
                            }
                            visited_type
                                .insert(field_1_type_of.to_owned(), field_2_type_of.to_owned());

                            let type_info = SimilarityTypeInfo {
                                type_1,
                                type_2,
                                type_1_name: field_1_type_of.as_str(),
                                type_2_name: field_2_type_of.as_str(),
                            };

                            let is_nested_type_similar =
                                self.similarity_inner(type_info, visited_type, threshold);

                            same_field_count += if is_nested_type_similar { 1 } else { 0 };
                        }
                    }
                }
            }

            let total_field_count =
                (type_1.fields.len() + type_2.fields.len()) as u32 - same_field_count;

            let is_similar = (same_field_count as f32 / total_field_count as f32) >= threshold;

            self.type_similarity_cache.add(
                type_1_name.to_owned(),
                type_2_name.to_owned(),
                is_similar,
            );

            is_similar
        }
    }
}

#[cfg(test)]
mod test {
    use super::Similarity;
    use crate::core::config::{Config, Field, Type};

    #[test]
    fn should_return_false_when_thresh_is_not_met() {
        let mut foo1 = Type::default();
        foo1.fields.insert(
            "a".to_owned(),
            Field { type_of: "Int".to_owned(), ..Default::default() },
        );
        foo1.fields.insert(
            "b".to_owned(),
            Field { type_of: "String".to_owned(), ..Default::default() },
        );
        foo1.fields.insert(
            "c".to_owned(),
            Field { type_of: "Bar1".to_owned(), ..Default::default() },
        );

        let mut foo2 = Type::default();
        foo2.fields.insert(
            "a".to_owned(),
            Field { type_of: "Int".to_owned(), ..Default::default() },
        );
        foo2.fields.insert(
            "b".to_owned(),
            Field { type_of: "Float".to_owned(), ..Default::default() },
        );
        foo2.fields.insert(
            "c".to_owned(),
            Field { type_of: "Bar2".to_owned(), ..Default::default() },
        );

        let mut bar1 = Type::default();
        bar1.fields.insert(
            "a".to_owned(),
            Field { type_of: "Int".to_owned(), ..Default::default() },
        );
        bar1.fields.insert(
            "c".to_owned(),
            Field { type_of: "Float".to_owned(), ..Default::default() },
        );

        let mut bar2 = Type::default();
        bar2.fields.insert(
            "a".to_owned(),
            Field { type_of: "Int".to_owned(), ..Default::default() },
        );
        bar2.fields.insert(
            "c".to_owned(),
            Field { type_of: "String".to_owned(), ..Default::default() },
        );

        let mut cfg: Config = Config::default();
        cfg.types.insert("Foo1".to_owned(), foo1.to_owned());
        cfg.types.insert("Foo2".to_owned(), foo2.to_owned());
        cfg.types.insert("Bar1".to_owned(), bar1);
        cfg.types.insert("Bar2".to_owned(), bar2);

        let mut gen = Similarity::new(&cfg);
        let is_similar = gen.similarity(("Foo1", &foo1), ("Foo2", &foo2), 0.5);

        assert!(!is_similar)
    }

    #[test]
    fn should_return_true_when_thresh_is_met() {
        let mut foo1 = Type::default();
        foo1.fields.insert(
            "a".to_owned(),
            Field { type_of: "Int".to_owned(), ..Default::default() },
        );
        foo1.fields.insert(
            "b".to_owned(),
            Field { type_of: "String".to_owned(), ..Default::default() },
        );
        foo1.fields.insert(
            "c".to_owned(),
            Field { type_of: "Bar1".to_owned(), ..Default::default() },
        );

        let mut foo2 = Type::default();
        foo2.fields.insert(
            "a".to_owned(),
            Field { type_of: "Int".to_owned(), ..Default::default() },
        );
        foo2.fields.insert(
            "b".to_owned(),
            Field { type_of: "Float".to_owned(), ..Default::default() },
        );
        foo2.fields.insert(
            "c".to_owned(),
            Field { type_of: "Bar2".to_owned(), ..Default::default() },
        );

        let mut bar1 = Type::default();
        bar1.fields.insert(
            "a".to_owned(),
            Field { type_of: "Int".to_owned(), ..Default::default() },
        );
        bar1.fields.insert(
            "c".to_owned(),
            Field { type_of: "Float".to_owned(), ..Default::default() },
        );

        let mut bar2 = Type::default();
        bar2.fields.insert(
            "a".to_owned(),
            Field { type_of: "Int".to_owned(), ..Default::default() },
        );
        bar2.fields.insert(
            "c".to_owned(),
            Field { type_of: "Float".to_owned(), ..Default::default() },
        );
        bar2.fields.insert(
            "k".to_owned(),
            Field { type_of: "Int".to_owned(), ..Default::default() },
        );

        let mut cfg: Config = Config::default();
        cfg.types.insert("Foo1".to_owned(), foo1.to_owned());
        cfg.types.insert("Foo2".to_owned(), foo2.to_owned());
        cfg.types.insert("Bar1".to_owned(), bar1);
        cfg.types.insert("Bar2".to_owned(), bar2);

        let mut gen = Similarity::new(&cfg);
        let is_similar = gen.similarity(("Foo1", &foo1), ("Foo2", &foo2), 0.5);

        assert!(is_similar)
    }

    #[test]
    fn test_cyclic_type() {
        let mut foo1 = Type::default();
        foo1.fields.insert(
            "a".to_owned(),
            Field { type_of: "Bar1".to_owned(), ..Default::default() },
        );

        let mut foo2 = Type::default();
        foo2.fields.insert(
            "a".to_owned(),
            Field { type_of: "Bar2".to_owned(), ..Default::default() },
        );

        let mut bar1 = Type::default();
        bar1.fields.insert(
            "a".to_owned(),
            Field { type_of: "Foo1".to_owned(), ..Default::default() },
        );

        let mut bar2 = Type::default();
        bar2.fields.insert(
            "a".to_owned(),
            Field { type_of: "Foo2".to_owned(), ..Default::default() },
        );

        let mut cfg: Config = Config::default();
        cfg.types.insert("Foo1".to_owned(), foo1.to_owned());
        cfg.types.insert("Foo2".to_owned(), foo2.to_owned());
        cfg.types.insert("Bar1".to_owned(), bar1);
        cfg.types.insert("Bar2".to_owned(), bar2);

        let mut gen = Similarity::new(&cfg);
        let is_similar = gen.similarity(("Foo1", &foo1), ("Foo2", &foo2), 0.8);

        assert!(is_similar)
    }

    #[test]
    fn test_nested_types() {
        let mut foo1 = Type::default();
        foo1.fields.insert(
            "a".to_owned(),
            Field { type_of: "Bar1".to_owned(), ..Default::default() },
        );

        let mut foo2 = Type::default();
        foo2.fields.insert(
            "a".to_owned(),
            Field { type_of: "Bar2".to_owned(), ..Default::default() },
        );

        let mut bar1 = Type::default();
        bar1.fields.insert(
            "a".to_owned(),
            Field { type_of: "Far1".to_owned(), ..Default::default() },
        );

        let mut bar2 = Type::default();
        bar2.fields.insert(
            "a".to_owned(),
            Field { type_of: "Far2".to_owned(), ..Default::default() },
        );

        let mut far1 = Type::default();
        far1.fields.insert(
            "a".to_owned(),
            Field { type_of: "Int".to_owned(), ..Default::default() },
        );
        let mut far2 = Type::default();
        far2.fields.insert(
            "a".to_owned(),
            Field { type_of: "Int".to_owned(), ..Default::default() },
        );

        let mut cfg: Config = Config::default();
        cfg.types.insert("Foo1".to_owned(), foo1.to_owned());
        cfg.types.insert("Foo2".to_owned(), foo2.to_owned());
        cfg.types.insert("Bar1".to_owned(), bar1);
        cfg.types.insert("Bar2".to_owned(), bar2);
        cfg.types.insert("Far1".to_owned(), far1);
        cfg.types.insert("Far2".to_owned(), far2);

        let mut gen = Similarity::new(&cfg);
        let is_similar = gen.similarity(("Foo1", &foo1), ("Foo2", &foo2), 0.8);

        assert!(is_similar)
    }

    #[test]
    fn test_required_and_optional_fields() {
        let required_int_field = Field {
            type_of: "Int".to_owned(),
            required: true,
            ..Default::default()
        };

        let optional_int_field = Field { type_of: "Int".to_owned(), ..Default::default() };

        let mut ty1 = Type::default();
        ty1.fields
            .insert("a".to_string(), required_int_field.clone());
        ty1.fields
            .insert("b".to_string(), required_int_field.clone());
        ty1.fields
            .insert("c".to_string(), required_int_field.clone());

        let mut ty2 = Type::default();
        ty2.fields
            .insert("a".to_string(), optional_int_field.clone());
        ty2.fields
            .insert("b".to_string(), optional_int_field.clone());
        ty2.fields
            .insert("c".to_string(), optional_int_field.clone());

        let mut config = Config::default();
        config.types.insert("Foo".to_string(), ty1.clone());
        config.types.insert("Bar".to_string(), ty2.clone());

        let types_equal = Similarity::new(&config).similarity(("Foo", &ty1), ("Bar", &ty2), 1.0);
        assert!(!types_equal)
    }

    #[test]
    fn test_required_list_of_optional_int_vs_optional_list() {
        let required_int_field = Field {
            type_of: "Int".to_owned(),
            list: true,
            required: true,
            ..Default::default()
        };

        let optional_int_field =
            Field { type_of: "Int".to_owned(), list: true, ..Default::default() };

        let mut ty1 = Type::default();
        ty1.fields
            .insert("a".to_string(), required_int_field.clone());

        let mut ty2 = Type::default();
        ty2.fields
            .insert("a".to_string(), optional_int_field.clone());

        let mut config = Config::default();
        config.types.insert("Foo".to_string(), ty1.clone());
        config.types.insert("Bar".to_string(), ty2.clone());

        let types_equal = Similarity::new(&config).similarity(("Foo", &ty1), ("Bar", &ty2), 1.0);
        assert!(!types_equal)
    }

    #[test]
    fn test_list_of_required_int_vs_required_list() {
        let required_int_field = Field {
            type_of: "Int".to_owned(),
            list: true,
            list_type_required: true,
            ..Default::default()
        };

        let optional_int_field = Field {
            type_of: "Int".to_owned(),
            list: true,
            required: true,
            ..Default::default()
        };

        let mut ty1 = Type::default();
        ty1.fields
            .insert("a".to_string(), required_int_field.clone());

        let mut ty2 = Type::default();
        ty2.fields
            .insert("a".to_string(), optional_int_field.clone());

        let mut config = Config::default();
        config.types.insert("Foo".to_string(), ty1.clone());
        config.types.insert("Bar".to_string(), ty2.clone());

        let types_equal = Similarity::new(&config).similarity(("Foo", &ty1), ("Bar", &ty2), 1.0);
        assert!(!types_equal)
    }

    #[test]
    fn test_list_of_required_int_vs_list_of_required_int() {
        let required_int_field = Field {
            type_of: "Int".to_owned(),
            list: true,
            list_type_required: true,
            ..Default::default()
        };

        let optional_int_field = Field {
            type_of: "Int".to_owned(),
            list: true,
            list_type_required: true,
            ..Default::default()
        };

        let mut ty1 = Type::default();
        ty1.fields
            .insert("a".to_string(), required_int_field.clone());

        let mut ty2 = Type::default();
        ty2.fields
            .insert("a".to_string(), optional_int_field.clone());

        let mut config = Config::default();
        config.types.insert("Foo".to_string(), ty1.clone());
        config.types.insert("Bar".to_string(), ty2.clone());

        let types_equal = Similarity::new(&config).similarity(("Foo", &ty1), ("Bar", &ty2), 1.0);
        assert!(types_equal)
    }

    #[test]
    fn test_required_list_vs_required_list() {
        let required_int_field = Field {
            type_of: "Int".to_owned(),
            list: true,
            required: true,
            ..Default::default()
        };

        let optional_int_field = Field {
            type_of: "Int".to_owned(),
            list: true,
            required: true,
            ..Default::default()
        };

        let mut ty1 = Type::default();
        ty1.fields
            .insert("a".to_string(), required_int_field.clone());

        let mut ty2 = Type::default();
        ty2.fields
            .insert("a".to_string(), optional_int_field.clone());

        let mut config = Config::default();
        config.types.insert("Foo".to_string(), ty1.clone());
        config.types.insert("Bar".to_string(), ty2.clone());

        let types_equal = Similarity::new(&config).similarity(("Foo", &ty1), ("Bar", &ty2), 1.0);
        assert!(types_equal)
    }

    #[test]
    fn test_required_list_of_required_int_vs_required_list_of_required_int() {
        let required_int_field = Field {
            type_of: "Int".to_owned(),
            list: true,
            required: true,
            list_type_required: true,
            ..Default::default()
        };

        let optional_int_field = Field {
            type_of: "Int".to_owned(),
            list: true,
            required: true,
            list_type_required: true,
            ..Default::default()
        };

        let mut ty1 = Type::default();
        ty1.fields
            .insert("a".to_string(), required_int_field.clone());

        let mut ty2 = Type::default();
        ty2.fields
            .insert("a".to_string(), optional_int_field.clone());

        let mut config = Config::default();
        config.types.insert("Foo".to_string(), ty1.clone());
        config.types.insert("Bar".to_string(), ty2.clone());

        let types_equal = Similarity::new(&config).similarity(("Foo", &ty1), ("Bar", &ty2), 1.0);
        assert!(types_equal)
    }
}
