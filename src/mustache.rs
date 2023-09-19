use serde::de::Deserializer;
use serde::{Deserialize, Serialize};

use crate::path;

#[derive(Debug, Clone, PartialEq)]
pub enum Mustache {
  Literal(String),
  Expression(Vec<String>),
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
        return Ok(Mustache::Expression(parts));
      }
    }

    Ok(Mustache::Literal(s))
  }
}

impl Serialize for Mustache {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    match self {
      Mustache::Literal(s) => serializer.serialize_str(s),
      Mustache::Expression(parts) => {
        let combined = format!("{{{{{}}}}}", parts.join("."));
        serializer.serialize_str(&combined)
      }
    }
  }
}

impl std::fmt::Display for Mustache {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Mustache::Literal(text) => write!(f, "{}", text),
      Mustache::Expression(expressions) => {
        let mut expression = String::new();
        for e in expressions {
          expression.push_str(e);
        }
        write!(f, "{{{}}}", expression)
      }
    }
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn test_deserialize_simple() {
    let s = r#""hello""#;
    let mustache: super::Mustache = serde_json::from_str(s).unwrap();
    assert_eq!(mustache, super::Mustache::Literal("hello".to_string()));
  }

  #[test]
  fn test_deserialize_template() {
    let s = r#""{{hello}}""#;
    let mustache: super::Mustache = serde_json::from_str(s).unwrap();
    assert_eq!(mustache, super::Mustache::Expression(vec!["hello".to_string()]));
  }
}
