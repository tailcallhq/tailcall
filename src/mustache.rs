use serde::de::Deserializer;
use serde::{Deserialize, Serialize};

use crate::path;

#[derive(Debug, Clone, PartialEq)]
pub enum Mustache {
    Simple(String),
    Template(Vec<String>),
}

impl<'de> Deserialize<'de> for Mustache {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        if let Some(captures) = path::RE.captures(&s) {
            if let Some(matched) = captures.get(1) {
                let parts: Vec<String> = matched.as_str().split('.').map(String::from).collect();
                return Ok(Mustache::Template(parts));
            }
        }

        Ok(Mustache::Simple(s))
    }
}

impl Serialize for Mustache {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Mustache::Simple(s) => serializer.serialize_str(s),
            Mustache::Template(parts) => {
                let combined = format!("{{{{{}}}}}", parts.join("."));
                serializer.serialize_str(&combined)
            }
        }
    }
}

impl std::fmt::Display for Mustache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mustache::Simple(text) => write!(f, "{}", text),
            Mustache::Template(expressions) => {
                let mut expression = String::new();
                for e in expressions {
                    expression.push_str(e);
                }
                write!(f, "{{{}}}", expression)
            }
        }
    }
}
