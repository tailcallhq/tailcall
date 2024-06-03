use super::pair_set::PairSet;
use crate::core::config::{Config, Type};

#[derive(Default)]
struct SimilarityStat {
    same_field_count: u32,
    total_field_count: u32,
}

impl SimilarityStat {
    pub fn as_f32(&self) -> f32 {
        if self.total_field_count == 0 {
            return 0.0;
        }
        self.same_field_count as f32 / self.total_field_count as f32
    }
}

pub struct Similarity<'a> {
    config: &'a Config,
    thresh: f32,
}

impl<'a> Similarity<'a> {
    pub fn new(config: &'a Config, thresh: f32) -> Similarity {
        Similarity { config, thresh }
    }

    pub fn similarity(&self, type_1: &Type, type_2: &Type) -> bool {
        self.similarity_inner(type_1, type_2, &mut PairSet::default())
    }

    fn similarity_inner(
        &self,
        type_1: &Type,
        type_2: &Type,
        visited_type: &mut PairSet<String>,
    ) -> bool {
        let config = &self.config;
        let mut similarity_stats = SimilarityStat::default();

        for (field_name_1, field_1) in type_1.fields.iter() {
            if let Some(field_2) = type_2.fields.get(field_name_1) {
                let field_1_type_of = field_1.type_of.to_owned();
                let field_2_type_of = field_2.type_of.to_owned();

                if field_1_type_of == field_2_type_of {
                    similarity_stats.same_field_count += 1; // 1 from field_1 + 1 from
                                                    // field_2
                } else if let Some(type_1) = config.types.get(field_1_type_of.as_str()) {
                    if let Some(type_2) = config.types.get(field_2_type_of.as_str()) {
                        if visited_type.contains(&field_1_type_of, &field_2_type_of) {
                            // it's cyclic type, return true as they're the same.
                            return true;
                        }
                        visited_type.insert(field_1_type_of, field_2_type_of);

                        let is_nested_type_similar =
                            self.similarity_inner(type_1, type_2, visited_type);

                        similarity_stats.same_field_count += if is_nested_type_similar { 1 } else { 0 };
                    }
                }
            }
        }

        let union_field_len =
            (type_1.fields.len() + type_2.fields.len()) as u32 - similarity_stats.same_field_count;
        similarity_stats.total_field_count = union_field_len;

        similarity_stats.as_f32() >= self.thresh
    }
}

#[cfg(test)]
mod test {
    use crate::core::config::{Config, Field, Type};

    use super::Similarity;

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

        let gen = Similarity::new(&cfg, 0.5);
        let is_similar = gen.similarity(&foo1, &foo2);

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

        let gen = Similarity::new(&cfg, 0.5);
        let is_similar = gen.similarity(&foo1, &foo2);

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

        let gen = Similarity::new(&cfg, 0.8);
        let is_similar = gen.similarity(&foo1, &foo2);

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

        let gen = Similarity::new(&cfg, 0.8);
        let is_similar = gen.similarity(&foo1, &foo2);

        assert!(is_similar)
    }
}
