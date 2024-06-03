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
}

impl Similarity<'_> {
    pub fn new(config: &Config) -> Similarity {
        Similarity { config }
    }

    pub fn similarity(&self, type_1: &Type, type_2: &Type) -> f32 {
        self.similarity_inner(type_1, type_2, &mut PairSet::default())
            .as_f32()
    }

    /// calculate_distance returns pair of u32 ints -> (count of similar fields,
    /// total count of fields)
    /// TODO: optimize this recursive function.
    fn similarity_inner(
        &self,
        type_1: &Type,
        type_2: &Type,
        visited_type: &mut PairSet<String>,
    ) -> SimilarityStat {
        let config = &self.config;
        let mut distance = SimilarityStat::default();

        for (field_name_1, field_1) in type_1.fields.iter() {
            if let Some(field_2) = type_2.fields.get(field_name_1) {
                let field_1_type_of = field_1.type_of.to_owned();
                let field_2_type_of = field_2.type_of.to_owned();

                if field_1_type_of == field_2_type_of {
                    distance.same_field_count += 2; // 1 from field_1 + 1 from
                                                    // field_2
                } else if let Some(type_1) = config.types.get(field_1_type_of.as_str()) {
                    if let Some(type_2) = config.types.get(field_2_type_of.as_str()) {
                        if visited_type.contains(&field_1_type_of, &field_2_type_of) {
                            distance.same_field_count += 2;
                            continue;
                        }
                        visited_type.insert(field_1_type_of, field_2_type_of);

                        let type_similarity_metric =
                            self.similarity_inner(type_1, type_2, visited_type);

                        distance.total_field_count -= 2; // don't count the non-comparable field, it'll get counted by recursive
                                                         // call.

                        distance.same_field_count += type_similarity_metric.same_field_count;
                        distance.total_field_count += type_similarity_metric.total_field_count;
                    }
                }
            }
        }

        distance.total_field_count += (type_1.fields.len() + type_2.fields.len()) as u32;

        distance
    }
}
