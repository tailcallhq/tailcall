use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};

use tailcall_chunk::Chunk;

use crate::core::config::Config;

///
/// Represents a list of query paths that can issue a N + 1 query
#[derive(Default, Debug, PartialEq)]
pub struct QueryPath<'a>(Vec<Vec<&'a str>>);

impl QueryPath<'_> {
    pub fn size(&self) -> usize {
        self.0.len()
    }
}

impl<'a> From<Chunk<Chunk<FieldName<'a>>>> for QueryPath<'a> {
    fn from(chunk: Chunk<Chunk<FieldName<'a>>>) -> Self {
        QueryPath(
            chunk
                .as_vec()
                .iter()
                .map(|chunk| {
                    chunk
                        .as_vec()
                        .iter()
                        .map(|field_name| field_name.as_str())
                        .collect()
                })
                .collect(),
        )
    }
}

impl Display for QueryPath<'_> {
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
    fn as_str(self) -> &'a str {
        self.0
    }
}
impl Display for FieldName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Highly optimized structure to hold a key.
pub struct KeyHolder {
    buffer: String,
    separator: Vec<usize>,
}

impl KeyHolder {
    pub fn new() -> Self {
        KeyHolder { buffer: String::default(), separator: Vec::new() }
    }

    /// Appends a path to the key.
    pub fn append(&mut self, path: &str) {
        if !self.buffer.is_empty() {
            // Append the separator only if the buffer isn't empty.
            self.separator.push(self.buffer.len());
            self.buffer.push('.');
        }
        self.buffer.push_str(path);
    }

    /// Removes the last segment of the key.
    pub fn pop(&mut self) {
        if let Some(index) = self.separator.pop() {
            self.buffer.truncate(index);
        } else {
            self.clear();
        }
    }

    /// Returns the current key as a `&str`.
    pub fn key(&self) -> &str {
        &self.buffer
    }

    /// Clears the key entirely.
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.separator.clear();
    }
}

/// A module that tracks the query paths that can issue a N + 1 calls to
/// upstream.
pub struct PathTracker<'a> {
    config: &'a Config,
    cache: HashMap<String, Chunk<Chunk<FieldName<'a>>>>,
}

impl<'a> PathTracker<'a> {
    pub fn new(config: &'a Config) -> PathTracker<'a> {
        PathTracker { config, cache: Default::default() }
    }

    #[allow(clippy::too_many_arguments)]
    fn iter_inner(
        &mut self,
        path: Chunk<FieldName<'a>>,
        type_name: TypeName<'a>,
        is_list: bool,
        visited: HashSet<(TypeName<'a>, FieldName<'a>)>,
        key_holder: &mut KeyHolder,
    ) -> Chunk<Chunk<FieldName<'a>>> {
        if let Some(chunks) = self.cache.get(key_holder.key()) {
            return chunks.clone();
        }

        let mut chunks = Chunk::default();
        if let Some(type_of) = self.config.find_type(type_name.as_str()) {
            for (name, field) in type_of.fields.iter() {
                let field_name = FieldName::new(name);
                let path = path.clone().append(field_name);
                key_holder.append(name);
                if !visited.contains(&(type_name, field_name)) {
                    if is_list && field.has_resolver() && !field.has_batched_resolver() {
                        chunks = chunks.append(path.clone());
                    } else {
                        let mut visited = visited.clone();
                        visited.insert((type_name, field_name));
                        let is_list = is_list | field.type_of.is_list();
                        chunks = chunks.concat(self.iter_inner(
                            path,
                            TypeName::new(field.type_of.name()),
                            is_list,
                            visited,
                            key_holder,
                        ))
                    }
                }
                key_holder.pop();
            }
        }

        self.cache
            .insert(key_holder.key().to_owned(), chunks.clone());
        chunks
    }

    fn iter(
        &mut self,
        path: Chunk<FieldName<'a>>,
        type_name: TypeName<'a>,
    ) -> Chunk<Chunk<FieldName<'a>>> {
        self.iter_inner(
            path,
            type_name,
            false,
            HashSet::new(),
            &mut KeyHolder::new(),
        )
    }

    fn find_chunks(&mut self) -> Chunk<Chunk<FieldName<'a>>> {
        match &self.config.schema.query {
            None => Chunk::default(),
            Some(query) => self.iter(Chunk::default(), TypeName::new(query.as_str())),
        }
    }

    pub fn find(mut self) -> QueryPath<'a> {
        QueryPath::from(self.find_chunks())
    }
}

#[cfg(test)]
mod tests {
    use super::KeyHolder;
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
        let actual = config.n_plus_one();

        insta::assert_snapshot!(actual);
    }

    #[test]
    fn test_key_holder() {
        let mut holder = KeyHolder::new();
        holder.append("query");
        holder.append("user");

        assert_eq!(holder.key(), "query.user");
        holder.pop();
        assert_eq!(holder.key(), "query");
        holder.pop();
        assert_eq!(holder.key(), "");
        holder.pop();
        assert_eq!(holder.key(), "");
    }
}
