use std::collections::{HashMap, HashSet};
use std::fmt::Display;

use super::chunk::Chunk;
use super::Queries;
use crate::core::config::Config;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct TypeName<'a>(&'a str);
impl<'a> TypeName<'a> {
    pub fn new(name: &'a str) -> Self {
        Self(name)
    }
    pub fn as_str(self) -> &'a str {
        self.0
    }
}
impl Display for TypeName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct FieldName<'a>(&'a str);
impl<'a> FieldName<'a> {
    pub fn new(name: &'a str) -> Self {
        Self(name)
    }
    pub fn as_str(self) -> &'a str {
        self.0
    }
}
impl Display for FieldName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct Identifier<'a> {
    config: &'a Config,
    cache: HashMap<(TypeName<'a>, bool), Chunk<Chunk<FieldName<'a>>>>,
}

impl<'a> Identifier<'a> {
    pub fn new(config: &'a Config) -> Identifier {
        Identifier { config, cache: Default::default() }
    }

    #[allow(clippy::too_many_arguments)]
    fn iter(
        &mut self,
        path: Chunk<FieldName<'a>>,
        type_name: TypeName<'a>,
        is_list: bool,
        visited: HashSet<(TypeName<'a>, FieldName<'a>)>,
    ) -> Chunk<Chunk<FieldName<'a>>> {
        if let Some(chunks) = self.cache.get(&(type_name, is_list)) {
            return chunks.clone();
        }

        let mut chunks = Chunk::new();
        if let Some(type_of) = self.config.find_type(type_name.as_str()) {
            for (name, field) in type_of.fields.iter() {
                let field_name = FieldName::new(name);
                let path = path.clone().append(field_name);
                if !visited.contains(&(type_name, field_name)) {
                    let is_batch = field.has_batched_resolver();
                    if field.has_resolver() && !is_batch && is_list {
                        chunks = chunks.append(path.clone());
                    } else {
                        let mut visited = visited.clone();
                        visited.insert((type_name, field_name));
                        let is_list = is_list | field.list;
                        chunks = chunks.concat(self.iter(
                            path,
                            TypeName::new(field.type_of.as_str()),
                            is_list,
                            visited,
                        ))
                    }
                }
            }
        }

        self.cache.insert((type_name, is_list), chunks.clone());
        chunks
    }

    fn find_chunks(&mut self) -> Chunk<Chunk<FieldName<'a>>> {
        match &self.config.schema.query {
            None => Chunk::new(),
            Some(query) => self.iter(
                Chunk::new(),
                TypeName::new(query.as_str()),
                false,
                HashSet::new(),
            ),
        }
    }

    pub fn identify(mut self) -> Queries<'a> {
        Queries::from_chunk(self.find_chunks())
    }
}

#[cfg(test)]
mod tests {
    use crate::include_config;

    #[test]
    fn test_nplusone_resolvers() {
        let config = include_config!("fixtures/simple-resolvers.graphql").unwrap();
        let actual = config.n_plus_one();
        let formatted = actual.to_string();
        let mut formatted = formatted.split('\n').collect::<Vec<_>>();
        formatted.sort();

        insta::assert_snapshot!(formatted.join("\n"));
    }

    #[test]
    fn test_nplusone_batched_resolvers() {
        let config = include_config!("fixtures/simple-batch-resolver.graphql").unwrap();
        let actual = config.n_plus_one();
        let formatted = actual.to_string();
        let mut formatted = formatted.split('\n').collect::<Vec<_>>();
        formatted.sort();

        insta::assert_snapshot!(formatted.join("\n"));
    }

    #[test]
    fn test_nplusone_nested_resolvers() {
        let config = include_config!("fixtures/nested-resolvers.graphql").unwrap();
        let actual = config.n_plus_one();
        let formatted = actual.to_string();
        let mut formatted = formatted.split('\n').collect::<Vec<_>>();
        formatted.sort();

        insta::assert_snapshot!(formatted.join("\n"));
    }

    #[test]
    fn test_nplusone_nested_resolvers_non_list_resolvers() {
        let config = include_config!("fixtures/non-list-resolvers.graphql").unwrap();
        let actual = config.n_plus_one();
        let formatted = actual.to_string();
        let mut formatted = formatted.split('\n').collect::<Vec<_>>();
        formatted.sort();

        insta::assert_snapshot!(formatted.join("\n"));
    }

    #[test]
    fn test_nplusone_nested_resolvers_without_resolvers() {
        let config = include_config!("fixtures/nested-without-resolvers.graphql").unwrap();
        let actual = config.n_plus_one();
        let formatted = actual.to_string();
        let mut formatted = formatted.split('\n').collect::<Vec<_>>();
        formatted.sort();

        insta::assert_snapshot!(formatted.join("\n"));
    }

    #[test]
    fn test_nplusone_cycles() {
        let config = include_config!("fixtures/cycles.graphql").unwrap();
        let actual = config.n_plus_one();
        let formatted = actual.to_string();
        let mut formatted = formatted.split('\n').collect::<Vec<_>>();
        formatted.sort();

        insta::assert_snapshot!(formatted.join("\n"));
    }

    #[test]
    fn test_nplusone_cycles_with_resolvers() {
        let config = include_config!("fixtures/cyclic-resolvers.graphql").unwrap();
        let actual = config.n_plus_one();
        let formatted = actual.to_string();
        let mut formatted = formatted.split('\n').collect::<Vec<_>>();
        formatted.sort();

        insta::assert_snapshot!(formatted.join("\n"));
    }
    #[test]
    fn test_nplusone_nested_non_list() {
        let config = include_config!("fixtures/nested-non-list.graphql").unwrap();
        let actual = config.n_plus_one();
        let formatted = actual.to_string();
        let mut formatted = formatted.split('\n').collect::<Vec<_>>();
        formatted.sort();

        insta::assert_snapshot!(formatted.join("\n"));
    }
}
