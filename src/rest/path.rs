use async_graphql::{Name, Variables};

use super::typed_variables::TypedVariable;
use crate::rest::type_map::TypeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Segment {
    Literal(String),
    Param(TypedVariable),
}

impl Segment {
    pub fn lit(s: &str) -> Self {
        Self::Literal(s.to_string())
    }

    pub fn param(t: TypedVariable) -> Self {
        Self::Param(t)
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Path {
    pattern: String,
    pub(super) segments: Vec<Segment>,
}

impl Path {
    pub fn as_str(&self) -> &str {
        self.pattern.as_str()
    }

    pub fn parse(q: &TypeMap, input: &str) -> anyhow::Result<Self> {
        let variables = q;

        let mut segments = Vec::new();
        for s in input.split('/').filter(|s| !s.is_empty()) {
            if let Some(key) = s.strip_prefix('$') {
                let value = variables.get(key).ok_or(anyhow::anyhow!(
                    "undefined param: {} in {}",
                    s,
                    input
                ))?;
                let t = TypedVariable::try_from(value, key)?;
                segments.push(Segment::param(t));
            } else {
                segments.push(Segment::lit(s));
            }
        }
        Ok(Self { segments, pattern: input.to_string() })
    }

    pub fn matches(&self, path: &str) -> Option<Variables> {
        let mut variables = Variables::default();
        let mut req_segments = path.split('/').filter(|s| !s.is_empty());
        for (segment, req_segment) in self.segments.iter().zip(&mut req_segments) {
            match segment {
                Segment::Literal(segment) => {
                    if segment != req_segment {
                        return None;
                    }
                }
                Segment::Param(t_var) => {
                    let tpe = t_var.to_value(req_segment).ok()?;
                    variables.insert(Name::new(t_var.name()), tpe);
                }
            }
        }

        // If there is still some segments in incoming request it should not match
        if req_segments.next().is_some() {
            return None;
        }

        Some(variables)
    }
}