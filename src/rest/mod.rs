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

#[derive(Debug, PartialEq)]
pub struct Path {
    segments: Vec<Segment>,
}

impl Path {
    pub fn parse(s: &str) -> anyhow::Result<Self> {
        let segments = s
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| {
                if s.starts_with(':') {
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

pub struct Query {
    params: Vec<(String, String)>,
}

pub struct Router {
    method: Method,
    path: Path,
    query: Query,
}

impl Router {
    pub fn new(method: Method, path: Path, query: Query) -> Self {
        Self { method, path, query }
    }

    pub fn parse(route_string: &str) -> anyhow::Result<Router> {
        todo!()
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
                "/users/:id",
                vec![Segment::literal("users"), Segment::param("id")],
            ),
            (
                "/users/:id/posts",
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
}
