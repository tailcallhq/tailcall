use indexmap::IndexMap;

use crate::core::config::Config;

/// A mapping of type names to their referenced fields and the number of times
/// each type is referenced.
///
/// Structure maintains a map where each key is a type name, and the
/// corresponding value is another map that tracks the field names and the count
/// of how many times the type name was present in the configuration.
pub struct TypeUsageIndex<'a> {
    type_refs: IndexMap<&'a str, IndexMap<&'a str, u32>>,
}

impl<'a> TypeUsageIndex<'a> {
    pub fn new(config: &'a Config) -> Self {
        let type_refs = config
            .types
            .keys()
            .map(|type_name| {
                let type_references = config
                    .types
                    .values()
                    .flat_map(|t_| &t_.fields)
                    .filter(|(_, field_)| field_.type_of.as_str() == type_name.as_str())
                    .fold(IndexMap::new(), |mut acc, (field_name, _)| {
                        *acc.entry(field_name.as_str()).or_insert(0) += 1;
                        acc
                    });

                (type_name.as_str(), type_references)
            })
            .collect();

        Self { type_refs }
    }

    /// Given a type name, returns a map of field names and their reference
    /// counts, or an empty map if no references are found.
    pub fn usage_map(&self, type_name: &str) -> IndexMap<&str, u32> {
        self.type_refs
            .get(type_name)
            .cloned()
            .unwrap_or_else(IndexMap::new)
    }
}

#[cfg(test)]
mod tests {
    use super::TypeUsageIndex;
    use crate::core::config::Config;
    use crate::core::valid::Validator;

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
        let ty_index = TypeUsageIndex::new(&config);
        let ty_refs = ty_index.usage_map("T1");
        assert_eq!(ty_refs.len(), 1);
        assert_eq!(ty_refs.get("user").unwrap(), &2u32);
    }
}
