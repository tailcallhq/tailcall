use std::fmt::{Display, Formatter};
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Hash, Default)]
pub struct Mustache(Vec<Segment>);

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Expression(Vec<String>);

impl Deref for Expression {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Expression {
    pub fn new(expr: Vec<String>) -> Self {
        Self(expr)
    }
    pub fn insert(&mut self, index: usize, value: String) {
        self.0.insert(index, value);
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{{{.{}}}}}", self.0.join("."))
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Segment {
    Literal(String),
    Expression(Expression),
}

impl<A: IntoIterator<Item = Segment>> From<A> for Mustache {
    fn from(value: A) -> Self {
        Mustache(value.into_iter().collect())
    }
}

impl Mustache {
    pub fn is_const(&self) -> bool {
        match self {
            Mustache(segments) => {
                for s in segments {
                    if let Segment::Expression(_) = s {
                        return false;
                    }
                }
                true
            }
        }
    }

    pub fn segments(&self) -> &Vec<Segment> {
        &self.0
    }

    pub fn segments_mut(&mut self) -> &mut Vec<Segment> {
        &mut self.0
    }

    pub fn expression_segments(&self) -> Vec<&Vec<String>> {
        self.segments()
            .iter()
            .filter_map(|seg| match seg {
                Segment::Expression(parts) => Some(parts.deref()),
                _ => None,
            })
            .collect()
    }

    /// Checks if the mustache template contains the given expression
    pub fn expression_contains(&self, expression: &str) -> bool {
        self.segments()
            .iter()
            .any(|seg| matches!(seg, Segment::Expression(parts) if parts.iter().any(|part| part.as_str() == expression)))
    }
}

impl Display for Mustache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = self
            .segments()
            .iter()
            .map(|segment| match segment {
                Segment::Literal(text) => text.clone(),
                Segment::Expression(parts) => parts.to_string(),
            })
            .collect::<Vec<String>>()
            .join("");

        write!(f, "{}", str)
    }
}
