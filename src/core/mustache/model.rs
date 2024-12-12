use std::fmt::Display;

use super::JqTransform;

#[derive(Debug, Clone, PartialEq, Hash, Default)]
pub struct Mustache(Vec<Segment>);

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Segment {
    Literal(String),
    Expression(Vec<String>),
    JqTransform(JqTransform),
}

impl<A: IntoIterator<Item = Segment>> From<A> for Mustache {
    fn from(value: A) -> Self {
        Mustache(value.into_iter().collect())
    }
}

impl Mustache {
    /// Used to check if the returned expression resolves to a constant value
    /// always
    pub fn is_const(&self) -> bool {
        self.0.iter().all(|v| match v {
            Segment::Literal(_) => true,
            Segment::Expression(_) => false,
            Segment::JqTransform(jq_transform) => jq_transform.is_const(),
        })
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
                Segment::Expression(parts) => Some(parts),
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
                Segment::Expression(parts) => format!("{{{{.{}}}}}", parts.join(".")),
                Segment::JqTransform(jq) => format!("{{{{{}}}}}", jq.template()),
            })
            .collect::<Vec<String>>()
            .join("");

        write!(f, "{}", str)
    }
}
