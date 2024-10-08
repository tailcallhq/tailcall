use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};

use super::chunk::Chunk;
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

impl<'a> From<Chunk<Chunk<ChunkName<'a>>>> for QueryPath {
    fn from(chunk: Chunk<Chunk<ChunkName<'a>>>) -> Self {
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

        let val = query_data.iter().rfold("".to_string(), |s, query| {
            if s.is_empty() {
                query.to_string()
            } else {
                format!("{}\n{}", query, s)
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
enum ChunkName<'a> {
    Field(FieldName<'a>),
    Entity(TypeName<'a>),
}

impl Display for ChunkName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChunkName::Field(field_name) => write!(f, "{}", field_name),
            ChunkName::Entity(type_name) => write!(f, "__entity({})", type_name),
        }
    }
}

/// A module that tracks the query paths that can issue a N + 1 calls to
/// upstream.
pub struct PathTracker<'a> {
    config: &'a Config,
    cache: HashMap<(TypeName<'a>, bool), Chunk<Chunk<ChunkName<'a>>>>,
}

impl<'a> PathTracker<'a> {
    pub fn new(config: &'a Config) -> PathTracker {
        PathTracker { config, cache: Default::default() }
    }

    #[allow(clippy::too_many_arguments)]
    fn iter(
        &mut self,
        path: Chunk<ChunkName<'a>>,
        type_name: TypeName<'a>,
        is_list: bool,
        visited: HashSet<(TypeName<'a>, ChunkName<'a>)>,
    ) -> Chunk<Chunk<ChunkName<'a>>> {
        if let Some(chunks) = self.cache.get(&(type_name, is_list)) {
            return chunks.clone();
        }

        let mut chunks = Chunk::new();
        if let Some(type_of) = self.config.find_type(type_name.as_str()) {
            for (name, field) in type_of.fields.iter() {
                let chunk_name = ChunkName::Field(FieldName::new(name));
                let path = path.clone().append(chunk_name);
                if !visited.contains(&(type_name, chunk_name)) {
                    if is_list && field.has_resolver() && !field.has_batched_resolver() {
                        chunks = chunks.append(path);
                    } else {
                        let mut visited = visited.clone();

                        visited.insert((type_name, chunk_name));
                        let is_list = is_list | field.type_of.is_list();
                        chunks = chunks.concat(self.iter(
                            path,
                            TypeName::new(field.type_of.name()),
                            is_list,
                            visited,
                        ))
                    }
                }
            }
        }

        // self.cache.insert((type_name, is_list), chunks.clone());
        chunks
    }

    fn find_chunks(&mut self) -> Chunk<Chunk<ChunkName<'a>>> {
        // let mut visited = HashSet::new();

        let mut chunks = match &self.config.schema.query {
            None => Chunk::new(),
            Some(query) => self.iter(
                Chunk::new(),
                TypeName::new(query.as_str()),
                false,
                HashSet::new(),
            ),
        };

        for (type_name, type_of) in &self.config.types {
            if type_of.has_resolver() {
                let chunk_name = ChunkName::Entity(TypeName(type_name.as_str()));
                let chunk = Chunk::new().append(chunk_name);
                // entity resolver are used to fetch multiple instances at once
                // and therefore the resolver itself should be batched to avoid n + 1
                if type_of.has_batched_resolver() {
                    // if batched resolver is present traverse inner fields
                    chunks = chunks.concat(self.iter(
                        chunk,
                        TypeName::new(type_name.as_str()),
                        // entities are basically returning list of data
                        true,
                        HashSet::new(),
                    ));
                } else {
                    chunks = chunks.append(chunk);
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
    fn test_entity_resolver() {
        let config = include_config!("fixtures/entity-resolver.graphql").unwrap();

        assert_n_plus_one!(config);
    }
}
