use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, schemars::JsonSchema)]
pub struct Mustache(Vec<Segment>);

#[derive(Debug, Clone, PartialEq, Eq, Hash, schemars::JsonSchema)]
pub enum Segment {
    Literal(String),
    Expression(Vec<String>),
}

impl<'de> Deserialize<'de> for Mustache {
    fn deserialize<D>(deserializer: D) -> Result<Mustache, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = serde_json::Value::deserialize(deserializer)?;
        Ok(Mustache::parse(
            s.as_str()
                .ok_or(serde::de::Error::custom("expected string"))?,
        ))
    }
}

impl Serialize for Mustache {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
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
                Segment::Expression(parts) => format!("{{{{{}}}}}", parts.join(".")),
            })
            .collect::<Vec<String>>()
            .join("");

        write!(f, "{}", str)
    }
}
