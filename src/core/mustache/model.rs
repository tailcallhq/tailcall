use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Mustache(Vec<Segment>);

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Segment {
    Literal(String),
    Expression(Vec<String>),
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

    pub fn expression_segments(&self) -> Vec<&Vec<String>> {
        self.segments()
            .iter()
            .filter_map(|seg| match seg {
                Segment::Expression(parts) => Some(parts),
                _ => None,
            })
            .collect()
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
