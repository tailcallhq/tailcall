use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use tailcall_chunk::Chunk;

use crate::core::config::Config;

///
/// Represents a list of query paths that can issue a N + 1 query
#[derive(Default, Debug, PartialEq)]
pub struct QueryPath(Vec<Vec<String>>);

impl QueryPath {
    pub fn size(&self) -> usize {
        self.0.len()
    }
}

impl<'a> From<Chunk<Chunk<Name<'a>>>> for QueryPath {
    fn from(chunk: Chunk<Chunk<Name<'a>>>) -> Self {
        QueryPath(
            chunk
                .as_vec()
                .iter()
                .map(|chunk| {
                    chunk
                        .as_vec()
                        .iter()
                        .map(|chunk_name| chunk_name.to_string())
                        .collect()
                })
                .collect(),
        )
    }
}

impl Display for QueryPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let query_data: Vec<String> = self
            .0
            .iter()
            .map(|query_path| {
                let mut path = "query { ".to_string();
                path.push_str(
                    query_path
                        .iter()
                        .rfold("".to_string(), |s, field_name| {
                            if s.is_empty() {
                                field_name.to_string()
                            } else {
                                format!("{} {{ {} }}", field_name, s)
                            }
                        })
                        .as_str(),
                );
                path.push_str(" }");
                path
            })
            .collect();

        let val = query_data.iter().fold("".to_string(), |s, query| {
            if s.is_empty() {
                query.to_string()
            } else {
                format!("{}\n{}", s, query)
            }
        });

        f.write_str(&val)
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct TypeName<'a>(&'a str);
impl<'a> TypeName<'a> {
    fn new(name: &'a str) -> Self {
        Self(name)
    }
    fn as_str(self) -> &'a str {
        self.0
    }
}
impl Display for TypeName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct FieldName<'a>(&'a str);
impl<'a> FieldName<'a> {
    fn new(name: &'a str) -> Self {
        Self(name)
    }
}
impl Display for FieldName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum Name<'a> {
    Field(FieldName<'a>),
    Entity(TypeName<'a>),
}

impl Display for Name<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Name::Field(field_name) => write!(f, "{}", field_name),
            Name::Entity(type_name) => write!(
                f,
                "__entities(representations: [{{ __typename: \"{}\"}}])",
                type_name
            ),
        }
    }
}

/// A module that tracks the query paths that can issue a N + 1 calls to
/// upstream.
pub struct PathTracker<'a> {
    config: &'a Config,
    // Caches resolved chunks for the specific type
    // with is_list info since the result is different depending on this flag
    cache: HashMap<(TypeName<'a>, bool), Chunk<Chunk<Name<'a>>>>,
}

impl<'a> PathTracker<'a> {
    pub fn new(config: &'a Config) -> PathTracker<'a> {
        PathTracker { config, cache: Default::default() }
    }

    fn iter(
        &mut self,
        parent_name: Option<Name<'a>>,
        type_name: TypeName<'a>,
        is_list: bool,
    ) -> Chunk<Chunk<Name<'a>>> {
        let chunks = if let Some(chunks) = self.cache.get(&(type_name, is_list)) {
            chunks.clone()
        } else {
            // set empty value in the cache to prevent infinity recursion
            self.cache.insert((type_name, is_list), Chunk::default());

            let mut chunks = Chunk::default();
            if let Some(type_of) = self.config.find_type(type_name.as_str()) {
                for (name, field) in type_of.fields.iter() {
                    let field_name = Name::Field(FieldName::new(name));

                    if is_list && field.has_resolver() && !field.has_batched_resolver() {
                        chunks = chunks.append(Chunk::new(field_name));
                    } else {
                        let is_list = is_list | field.type_of.is_list();
                        chunks = chunks.concat(self.iter(
                            Some(field_name),
                            TypeName::new(field.type_of.name()),
                            is_list,
                        ))
                    }
                }
            }

            self.cache.insert((type_name, is_list), chunks.clone());

            chunks
        };

        // chunks contains only paths from the current type.
        // Prepend every subpath with parent path
        if let Some(path) = parent_name {
            let vec = chunks.as_vec();

            Chunk::from_iter(vec.into_iter().map(|chunk| chunk.prepend(path)))
        } else {
            chunks
        }
    }

    fn find_chunks(&mut self) -> Chunk<Chunk<Name<'a>>> {
        let mut chunks = match &self.config.schema.query {
            None => Chunk::default(),
            Some(query) => self.iter(None, TypeName::new(query.as_str()), false),
        };

        for (type_name, type_of) in &self.config.types {
            if type_of.has_resolver() {
                let parent_path = Name::Entity(TypeName(type_name.as_str()));
                // entity resolver are used to fetch multiple instances at once
                // and therefore the resolver itself should be batched to avoid n + 1
                if type_of.has_batched_resolver() {
                    // if batched resolver is present traverse inner fields
                    chunks = chunks.concat(self.iter(
                        Some(parent_path),
                        TypeName::new(type_name.as_str()),
                        // entities are basically returning list of data
                        true,
                    ));
                } else {
                    chunks = chunks.append(Chunk::new(parent_path));
                }
            }
        }

        chunks
    }

    pub fn find(mut self) -> QueryPath {
        QueryPath::from(self.find_chunks())
    }
}

#[cfg(test)]
mod tests {
    use crate::include_config;

    #[macro_export]
    macro_rules! assert_n_plus_one {
        ($cfg:expr) => {{
            let actual = $cfg.n_plus_one();
            insta::assert_snapshot!(actual);
        }};
    }

    #[test]
    fn test_resolvers() {
        let config = include_config!("fixtures/simple-resolvers.graphql").unwrap();
        assert_n_plus_one!(config);
    }

    #[test]
    fn test_batched_resolvers() {
        let config = include_config!("fixtures/simple-batch-resolver.graphql").unwrap();
        assert_n_plus_one!(config);
    }

    #[test]
    fn test_nested_resolvers() {
        let config = include_config!("fixtures/nested-resolvers.graphql").unwrap();
        assert_n_plus_one!(config);
    }

    #[test]
    fn test_nested_resolvers_non_list_resolvers() {
        let config = include_config!("fixtures/non-list-resolvers.graphql").unwrap();
        assert_n_plus_one!(config);
    }

    #[test]
    fn test_nested_resolvers_without_resolvers() {
        let config = include_config!("fixtures/nested-without-resolvers.graphql").unwrap();
        assert_n_plus_one!(config);
    }

    #[test]
    fn test_cycles() {
        let config = include_config!("fixtures/cycles.graphql").unwrap();
        assert_n_plus_one!(config);
    }

    #[test]
    fn test_cycles_with_resolvers() {
        let config = include_config!("fixtures/cyclic-resolvers.graphql").unwrap();
        assert_n_plus_one!(config);
    }

    #[test]
    fn test_cycles_with_resolver() {
        let config = include_config!("fixtures/cyclic-resolver.graphql").unwrap();
        let actual = config.n_plus_one();

        insta::assert_snapshot!(actual);
    }

    #[test]
    fn test_nested_non_list() {
        let config = include_config!("fixtures/nested-non-list.graphql").unwrap();
        assert_n_plus_one!(config);
    }

    #[test]
    fn test_multiple_keys() {
        let config = include_config!("fixtures/multiple-keys.graphql").unwrap();

        assert_n_plus_one!(config);
    }

    #[test]
    fn test_multiple_type_usage() {
        let config = include_config!("fixtures/multiple-type-usage.graphql").unwrap();

        assert_n_plus_one!(config);
    }

    #[test]
    fn test_entity_resolver() {
        let config = include_config!("fixtures/entity-resolver.graphql").unwrap();

        assert_n_plus_one!(config);
    }

    #[test]
    fn test_nested_config() {
        let config = include_config!("fixtures/nested.graphql").unwrap();

        assert_n_plus_one!(config);
    }

    #[test]
    fn test_multiple_deeply_nested() {
        let config = include_config!("fixtures/multiple-deeply-nested.graphql").unwrap();

        assert_n_plus_one!(config);
    }
}
