use tailcall_valid::{Valid, Validator};

use super::pair_map::PairMap;
use super::pair_set::PairSet;
use crate::core::config::{Config, Type};
use crate::core::scalar::Scalar;

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
    pub fn new(config: &'a Config) -> Similarity<'a> {
        Similarity { config, type_similarity_cache: PairMap::default() }
    }

    pub fn similarity(
        &mut self,
        (type_1_name, type_1): (&str, &Type),
        (type_2_name, type_2): (&str, &Type),
        threshold: f32,
    ) -> Valid<bool, String> {
        let type_info = SimilarityTypeInfo { type_1_name, type_1, type_2, type_2_name };

        self.similarity_inner(type_info, &mut PairSet::default(), threshold)
    }

    fn similarity_inner(
        &mut self,
        type_info: SimilarityTypeInfo,
        visited_type: &mut PairSet<String>,
        threshold: f32,
    ) -> Valid<bool, String> {
        let type_1_name = type_info.type_1_name;
        let type_2_name = type_info.type_2_name;
        let type_1 = type_info.type_1;
        let type_2 = type_info.type_2;

        if let Some(type_similarity_result) = self
            .type_similarity_cache
            .get(&type_1_name.to_string(), &type_2_name.to_string())
        {
            Valid::succeed(*type_similarity_result)
        } else {
            let config = self.config;
            let mut same_field_count = 0;

            for (field_name_1, field_1) in type_1.fields.iter() {
                if let Some(field_2) = type_2.fields.get(field_name_1) {
                    let field_1_type_of = field_1.type_of.name();
                    let field_2_type_of = field_2.type_of.name();

                    if config.is_scalar(field_1_type_of) && config.is_scalar(field_2_type_of) {
                        // if field type_of is scalar and they don't match then we can't merge
                        // types.
                        let json_scalar = &Scalar::JSON.to_string();
                        if field_1_type_of == field_2_type_of
                            || field_1_type_of == json_scalar
                            || field_2_type_of == json_scalar
                        {
                            if field_1.type_of.is_list() == field_2.type_of.is_list() {
                                same_field_count += 1;
                            } else {
                                return Valid::fail("Type merge failed: The fields have different list types and cannot be merged.".to_string());
                            }
                        } else {
                            return Valid::fail(
                                "Type merge failed: same field names but different scalar types."
                                    .to_string(),
                            );
                        }
                    } else if field_1_type_of == field_2_type_of {
                        // in order to consider the fields to be exactly same.
                        // it's output type must match (we can ignore the required bounds).
                        if field_1.type_of.is_list() == field_2.type_of.is_list() {
                            // if they're of both of list type then they're probably of same type.
                            same_field_count += 1;
                        } else {
                            // If the list properties don't match, we cannot merge these types.
                            return Valid::fail("Type merge failed: The fields have different list types and cannot be merged.".to_string());
                        }
                    } else if let Some(type_1) = config.types.get(field_1_type_of) {
                        if let Some(type_2) = config.types.get(field_2_type_of) {
                            if visited_type.contains(field_1_type_of, field_2_type_of) {
                                // it's cyclic type, return true as they're the same.
                                return Valid::succeed(true);
                            }
                            visited_type
                                .insert(field_1_type_of.to_owned(), field_2_type_of.to_owned());

                            let type_info = SimilarityTypeInfo {
                                type_1,
                                type_2,
                                type_1_name: field_1_type_of,
                                type_2_name: field_2_type_of,
                            };

                            let is_nested_type_similar =
                                self.similarity_inner(type_info, visited_type, threshold);

                            if let Ok(result) = is_nested_type_similar.clone().to_result() {
                                same_field_count += if result { 1 } else { 0 };
                            } else {
                                return is_nested_type_similar;
                            }
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

            Valid::succeed(is_similar)
        }
    }
}

#[cfg(test)]
mod test {
    use tailcall_valid::Validator;

    use super::Similarity;
    use crate::core::config::{config, Config, Field};
    use crate::core::Type;

    #[test]
    fn should_return_error_when_same_field_has_different_scalar_type() {
        let mut foo1 = config::Type::default();
        foo1.fields.insert(
            "a".to_owned(),
            Field { type_of: "Int".to_owned().into(), ..Default::default() },
        );
        foo1.fields.insert(
            "b".to_owned(),
            Field { type_of: "String".to_owned().into(), ..Default::default() },
        );
        foo1.fields.insert(
            "c".to_owned(),
            Field { type_of: "Bar1".to_owned().into(), ..Default::default() },
        );

        let mut foo2 = config::Type::default();
        foo2.fields.insert(
            "a".to_owned(),
            Field { type_of: "Int".to_owned().into(), ..Default::default() },
        );
        foo2.fields.insert(
            "b".to_owned(),
            Field { type_of: "Float".to_owned().into(), ..Default::default() },
        );
        foo2.fields.insert(
            "c".to_owned(),
            Field { type_of: "Bar2".to_owned().into(), ..Default::default() },
        );

        let mut bar1 = config::Type::default();
        bar1.fields.insert(
            "a".to_owned(),
            Field { type_of: "Int".to_owned().into(), ..Default::default() },
        );
        bar1.fields.insert(
            "c".to_owned(),
            Field { type_of: "Float".to_owned().into(), ..Default::default() },
        );

        let mut bar2 = config::Type::default();
        bar2.fields.insert(
            "a".to_owned(),
            Field { type_of: "Int".to_owned().into(), ..Default::default() },
        );
        bar2.fields.insert(
            "c".to_owned(),
            Field { type_of: "String".to_owned().into(), ..Default::default() },
        );

        let mut cfg: Config = Config::default();
        cfg.types.insert("Foo1".to_owned(), foo1.to_owned());
        cfg.types.insert("Foo2".to_owned(), foo2.to_owned());
        cfg.types.insert("Bar1".to_owned(), bar1);
        cfg.types.insert("Bar2".to_owned(), bar2);

        let mut gen = Similarity::new(&cfg);
        let is_similar = gen
            .similarity(("Foo1", &foo1), ("Foo2", &foo2), 0.5)
            .to_result();

        assert!(is_similar.is_err())
    }

    #[test]
    fn test_cyclic_type() {
        let mut foo1 = config::Type::default();
        foo1.fields.insert(
            "a".to_owned(),
            Field { type_of: "Bar1".to_owned().into(), ..Default::default() },
        );

        let mut foo2 = config::Type::default();
        foo2.fields.insert(
            "a".to_owned(),
            Field { type_of: "Bar2".to_owned().into(), ..Default::default() },
        );

        let mut bar1 = config::Type::default();
        bar1.fields.insert(
            "a".to_owned(),
            Field { type_of: "Foo1".to_owned().into(), ..Default::default() },
        );

        let mut bar2 = config::Type::default();
        bar2.fields.insert(
            "a".to_owned(),
            Field { type_of: "Foo2".to_owned().into(), ..Default::default() },
        );

        let mut cfg: Config = Config::default();
        cfg.types.insert("Foo1".to_owned(), foo1.to_owned());
        cfg.types.insert("Foo2".to_owned(), foo2.to_owned());
        cfg.types.insert("Bar1".to_owned(), bar1);
        cfg.types.insert("Bar2".to_owned(), bar2);

        let mut gen = Similarity::new(&cfg);
        let is_similar = gen
            .similarity(("Foo1", &foo1), ("Foo2", &foo2), 0.8)
            .to_result()
            .unwrap();

        assert!(is_similar)
    }

    #[test]
    fn test_nested_types() {
        let mut foo1 = config::Type::default();
        foo1.fields.insert(
            "a".to_owned(),
            Field { type_of: "Bar1".to_owned().into(), ..Default::default() },
        );

        let mut foo2 = config::Type::default();
        foo2.fields.insert(
            "a".to_owned(),
            Field { type_of: "Bar2".to_owned().into(), ..Default::default() },
        );

        let mut bar1 = config::Type::default();
        bar1.fields.insert(
            "a".to_owned(),
            Field { type_of: "Far1".to_owned().into(), ..Default::default() },
        );

        let mut bar2 = config::Type::default();
        bar2.fields.insert(
            "a".to_owned(),
            Field { type_of: "Far2".to_owned().into(), ..Default::default() },
        );

        let mut far1 = config::Type::default();
        far1.fields.insert(
            "a".to_owned(),
            Field { type_of: "Int".to_owned().into(), ..Default::default() },
        );
        let mut far2 = config::Type::default();
        far2.fields.insert(
            "a".to_owned(),
            Field { type_of: "Int".to_owned().into(), ..Default::default() },
        );

        let mut cfg: Config = Config::default();
        cfg.types.insert("Foo1".to_owned(), foo1.to_owned());
        cfg.types.insert("Foo2".to_owned(), foo2.to_owned());
        cfg.types.insert("Bar1".to_owned(), bar1);
        cfg.types.insert("Bar2".to_owned(), bar2);
        cfg.types.insert("Far1".to_owned(), far1);
        cfg.types.insert("Far2".to_owned(), far2);

        let mut gen = Similarity::new(&cfg);
        let is_similar = gen
            .similarity(("Foo1", &foo1), ("Foo2", &foo2), 0.8)
            .to_result()
            .unwrap();

        assert!(is_similar)
    }

    #[test]
    fn test_required_and_optional_fields() {
        let required_int_field = Field {
            type_of: Type::from("Int".to_owned()).into_required(),
            ..Default::default()
        };

        let optional_int_field = Field { type_of: "Int".to_owned().into(), ..Default::default() };

        let mut ty1 = config::Type::default();
        ty1.fields
            .insert("a".to_string(), required_int_field.clone());
        ty1.fields
            .insert("b".to_string(), required_int_field.clone());
        ty1.fields
            .insert("c".to_string(), required_int_field.clone());

        let mut ty2 = config::Type::default();
        ty2.fields
            .insert("a".to_string(), optional_int_field.clone());
        ty2.fields
            .insert("b".to_string(), optional_int_field.clone());
        ty2.fields
            .insert("c".to_string(), optional_int_field.clone());

        let mut config = Config::default();
        config.types.insert("Foo".to_string(), ty1.clone());
        config.types.insert("Bar".to_string(), ty2.clone());

        let types_equal = Similarity::new(&config)
            .similarity(("Foo", &ty1), ("Bar", &ty2), 1.0)
            .to_result()
            .unwrap();
        assert!(types_equal)
    }

    #[test]
    fn test_required_list_of_optional_int_vs_optional_list() {
        let required_int_field = Field {
            type_of: Type::from("Int".to_owned()).into_list().into_required(),
            ..Default::default()
        };

        let optional_int_field = Field {
            type_of: Type::from("Int".to_owned()).into_list(),
            ..Default::default()
        };

        let mut ty1 = config::Type::default();
        ty1.fields
            .insert("a".to_string(), required_int_field.clone());

        let mut ty2 = config::Type::default();
        ty2.fields
            .insert("a".to_string(), optional_int_field.clone());

        let mut config = Config::default();
        config.types.insert("Foo".to_string(), ty1.clone());
        config.types.insert("Bar".to_string(), ty2.clone());

        let types_equal = Similarity::new(&config)
            .similarity(("Foo", &ty1), ("Bar", &ty2), 1.0)
            .to_result()
            .unwrap();
        assert!(types_equal)
    }

    #[test]
    fn test_list_of_required_int_vs_required_list() {
        let required_int_field = Field {
            type_of: Type::from("Int".to_owned()).into_required().into_list(),
            ..Default::default()
        };

        let optional_int_field = Field {
            type_of: Type::from("Int".to_owned()).into_required().into_list(),
            ..Default::default()
        };

        let mut ty1 = config::Type::default();
        ty1.fields
            .insert("a".to_string(), required_int_field.clone());

        let mut ty2 = config::Type::default();
        ty2.fields
            .insert("a".to_string(), optional_int_field.clone());

        let mut config = Config::default();
        config.types.insert("Foo".to_string(), ty1.clone());
        config.types.insert("Bar".to_string(), ty2.clone());

        let types_equal = Similarity::new(&config)
            .similarity(("Foo", &ty1), ("Bar", &ty2), 1.0)
            .to_result()
            .unwrap();
        assert!(types_equal)
    }

    #[test]
    fn test_list_of_required_int_vs_list_of_required_int() {
        let required_int_field = Field {
            type_of: Type::from("Int".to_owned()).into_required().into_list(),
            ..Default::default()
        };

        let mut ty1 = config::Type::default();
        ty1.fields
            .insert("a".to_string(), required_int_field.clone());

        let mut ty2 = config::Type::default();
        ty2.fields
            .insert("a".to_string(), required_int_field.clone());

        let mut config = Config::default();
        config.types.insert("Foo".to_string(), ty1.clone());
        config.types.insert("Bar".to_string(), ty2.clone());

        let types_equal = Similarity::new(&config)
            .similarity(("Foo", &ty1), ("Bar", &ty2), 1.0)
            .to_result()
            .unwrap();
        assert!(types_equal)
    }

    #[test]
    fn test_required_list_vs_required_list() {
        let required_int_field = Field {
            type_of: Type::from("Int".to_owned()).into_list().into_required(),
            ..Default::default()
        };

        let mut ty1 = config::Type::default();
        ty1.fields
            .insert("a".to_string(), required_int_field.clone());

        let mut ty2 = config::Type::default();
        ty2.fields
            .insert("a".to_string(), required_int_field.clone());

        let mut config = Config::default();
        config.types.insert("Foo".to_string(), ty1.clone());
        config.types.insert("Bar".to_string(), ty2.clone());

        let types_equal = Similarity::new(&config)
            .similarity(("Foo", &ty1), ("Bar", &ty2), 1.0)
            .to_result()
            .unwrap();
        assert!(types_equal)
    }

    #[test]
    fn test_required_list_of_required_int_vs_required_list_of_required_int() {
        let required_int_field = Field {
            type_of: Type::from("Int".to_owned())
                .into_required()
                .into_list()
                .into_required(),
            ..Default::default()
        };

        let mut ty1 = config::Type::default();
        ty1.fields
            .insert("a".to_string(), required_int_field.clone());

        let mut ty2 = config::Type::default();
        ty2.fields
            .insert("a".to_string(), required_int_field.clone());

        let mut config = Config::default();
        config.types.insert("Foo".to_string(), ty1.clone());
        config.types.insert("Bar".to_string(), ty2.clone());

        let types_equal = Similarity::new(&config)
            .similarity(("Foo", &ty1), ("Bar", &ty2), 1.0)
            .to_result()
            .unwrap();
        assert!(types_equal)
    }

    #[test]
    fn test_merge_incompatible_list_and_non_list_fields() {
        // Define fields
        let int_field = Field { type_of: "Int".to_owned().into(), ..Default::default() };
        let list_int_field = Field {
            type_of: Type::from("Int".to_owned()).into_list(),
            ..Default::default()
        };

        // Define types Foo and Bar
        let mut foo = config::Type::default();
        foo.fields.insert("a".to_string(), int_field.clone());
        foo.fields.insert("b".to_string(), int_field.clone());
        foo.fields.insert("c".to_string(), list_int_field.clone());

        let mut bar = config::Type::default();
        bar.fields.insert("a".to_string(), int_field.clone());
        bar.fields.insert("b".to_string(), int_field.clone());
        bar.fields.insert("c".to_string(), int_field.clone());

        // Create configuration with Foo and Bar types
        let mut config = Config::default();
        config.types.insert("Foo".to_owned(), foo.clone());
        config.types.insert("Bar".to_owned(), bar.clone());

        // Calculate similarity between Foo and Bar
        let result = Similarity::new(&config)
            .similarity(("Foo", &foo), ("Bar", &bar), 0.5)
            .to_result();

        // Assert that merging incompatible list and non-list fields fails
        assert!(result.is_err())
    }

    #[test]
    fn test_unknown_types_similarity() {
        let sdl = r#"
            type A {
                primarySubcategoryId: String
            }
            type B {
                primarySubcategoryId: JSON
            }
        "#;
        let config = Config::from_sdl(sdl).to_result().unwrap();

        let mut similarity = Similarity::new(&config);

        let result = similarity
            .similarity(
                ("B", config.types.get("B").unwrap()),
                ("A", config.types.get("A").unwrap()),
                0.9,
            )
            .to_result()
            .unwrap();
        assert!(result);
    }
}
