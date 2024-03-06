use std::collections::HashMap;

use derive_setters::Setters;

use crate::http::Method;

#[derive(Debug, PartialEq)]
pub enum Segment {
    Literal(String),
    Param(String),
}

impl Segment {
    pub fn literal(s: &str) -> Self {
        Self::Literal(s.to_string())
    }

    pub fn param(s: &str) -> Self {
        Self::Param(s.to_string())
    }
}

#[derive(Debug, PartialEq, Default)]
pub struct Path {
    segments: Vec<Segment>,
}

impl Path {
    pub fn parse(s: &str) -> anyhow::Result<Self> {
        let segments = s
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| {
                if s.starts_with('$') {
                    Segment::param(&s[1..])
                } else {
                    Segment::literal(s)
                }
            })
            .collect();
        Ok(Self { segments })
    }

    pub fn new(segments: Vec<Segment>) -> Self {
        Self { segments }
    }
}

#[derive(Debug, PartialEq, Default)]
pub struct Query {
    params: Vec<(String, Segment)>,
}

impl Query {
    fn from_map(map: HashMap<String, String>) -> Self {
        let params = map
            .into_iter()
            .map(|(k, v)| {
                if k.starts_with('$') {
                    (k, Segment::param(&v))
                } else {
                    (k, Segment::literal(&v))
                }
            })
            .collect();
        Self { params }
    }
}

#[derive(Debug, PartialEq, Setters, Default)]
pub struct Router {
    method: Method,
    path: Path,
    query: Query,
    body: Option<String>,
}

impl Router {
    pub fn new(method: Method, path: Path, query: Query) -> Self {
        Self { method, path, query, body: None }
    }

    pub fn from_path(route_string: &str) -> anyhow::Result<Router> {
        let path = Path::parse(route_string)?;
        Ok(Self::default().path(path))
    }

    pub fn with_query_params(mut self, query_params: HashMap<String, String>) -> Self {
        self.query = Query::from_map(query_params);
        self
    }

    pub fn with_path_str(mut self, path: &str) -> anyhow::Result<Self> {
        self.path = Path::parse(path)?;
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parse_path() {
        let inputs = vec![
            ("/users", vec![Segment::literal("users")]),
            (
                "/users/$id",
                vec![Segment::literal("users"), Segment::param("id")],
            ),
            (
                "/users/$id/posts",
                vec![
                    Segment::literal("users"),
                    Segment::param("id"),
                    Segment::literal("posts"),
                ],
            ),
        ];

        for (input, expected) in inputs {
            let path = Path::parse(input).unwrap();
            assert_eq!(path, Path::new(expected));
        }
    }

    #[test]
    fn test_from_query() {
        let inputs = vec![
            (vec![], Query { params: vec![] }),
            (
                vec![("id".to_string(), "1".to_string())],
                Query { params: vec![("id".to_string(), Segment::literal("1"))] },
            ),
            (
                vec![("id".to_string(), "$id".to_string())],
                Query { params: vec![("id".to_string(), Segment::param("id"))] },
            ),
        ];

        for (input, expected) in inputs {
            let query = Query::from_map(input.into_iter().collect());
            assert_eq!(query, expected);
        }
    }
}
