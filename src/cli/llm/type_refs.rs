use indexmap::IndexMap;

use crate::core::config::Config;

/// A mapping of type names to their referenced fields and the number of times each type is referenced.
///
/// Structure maintains a map where each key is a type name, and the corresponding value is
/// another map that tracks the field names and the count of how many times the type name was present in the configuration.
pub struct TypeUsageIndex {
    type_refs: IndexMap<String, IndexMap<String, u32>>,
}

impl TypeUsageIndex {
    pub fn new(config: &Config) -> Self {
        let precomputed_refs = config
            .types
            .keys()
            .map(|t_name| {
                // Collect field names and counts where the field's type matches the current type_name
                let type_refs = config
                    .types
                    .values()
                    .flat_map(|type_inner| &type_inner.fields)
                    .filter_map(|(field_name, field_)| {
                        if field_.type_of == **t_name {
                            Some(field_name)
                        } else {
                            None
                        }
                    })
                    .fold(IndexMap::new(), |mut acc, field_name| {
                        *acc.entry(field_name.to_owned()).or_insert(0) += 1;
                        acc
                    });

                (t_name.to_owned(), type_refs)
            })
            .collect::<IndexMap<_, _>>();

        Self { type_refs: precomputed_refs }
    }
    pub fn get(&self, type_name: &str) -> Option<&IndexMap<String, u32>> {
        self.type_refs.get(type_name)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{config::Config, valid::Validator};

    use super::TypeUsageIndex;

    #[test]
    fn test_type_index() {
        let sdl = r#"
            type T1 {
                name: String
                age: Int
            }
            type T2 {
                id: ID
                title: String
                user: T1
            }
            type Query {
                user: T1
            }
        
        "#;

        let config = Config::from_sdl(sdl).to_result().unwrap();
        let ty_tango = TypeUsageIndex::new(&config);
        let ty_refs = ty_tango.get("T1").unwrap();
        assert_eq!(ty_refs.len(), 1);
        assert_eq!(ty_refs.get("user").unwrap(), &2u32);
    }
}
