use nom::branch::alt;
use nom::bytes::complete::{tag, take_till};
use nom::character::complete::multispace0;
use nom::sequence::{delimited, tuple};
use nom::{Finish, IResult};
use serde::de::{Deserializer, Error};
use serde::Deserialize;

use crate::request_template::AnyPath;

#[derive(Debug, Clone, PartialEq)]
pub enum MustacheSegment {
  Literal(String),
  Expression(Vec<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Mustache {
  Segments(Vec<MustacheSegment>),
}

impl Mustache {
  pub fn new(str: &str) -> anyhow::Result<Mustache> {
    // Try to deserialize the string directly
    if let Ok(mustache) = serde_json::from_str(str) {
      return Ok(mustache);
    }

    // If direct deserialization failed, wrap it with quotes and try again
    let json_string = format!(r#""{}""#, str);
    Ok(serde_json::from_str(&json_string)?)
  }

  pub fn render(&self, value: &impl AnyPath) -> String {
    match self {
      Mustache::Segments(segments) => segments
        .iter()
        .map(|segment| match segment {
          MustacheSegment::Literal(text) => text.clone(),
          MustacheSegment::Expression(parts) => value.any_path(parts).map(|a| a.to_string()).unwrap_or_default(),
        })
        .collect(),
    }
  }
}

fn parse_expression(input: &str) -> IResult<&str, MustacheSegment> {
  let (input, expr) = delimited(
    tuple((tag("{{"), multispace0)),
    take_till(|c: char| c == '}'),
    tuple((multispace0, tag("}}"))),
  )(input)?;

  // Split by '.' while considering optional spaces
  let parts: Vec<String> = expr
    .split(|c| c == '.' || c == ' ')
    .filter_map(|s| {
      let trimmed = s.trim();
      if !trimmed.is_empty() {
        Some(trimmed.to_string())
      } else {
        None
      }
    })
    .collect();

  Ok((input, MustacheSegment::Expression(parts)))
}

fn parse_literal(input: &str) -> IResult<&str, MustacheSegment> {
  let (input, literal) = take_till(|c: char| c == '{')(input)?;

  if input.starts_with('{') {
    let (rest, unmatched_literal) = take_till(|c: char| c == '{' || c.is_whitespace())(input)?;
    let combined_literal = format!("{}{}", literal, unmatched_literal);
    return Ok((rest, MustacheSegment::Literal(combined_literal)));
  }

  Ok((input, MustacheSegment::Literal(literal.to_string())))
}

fn parse_parts(input: &str) -> anyhow::Result<(&str, Vec<MustacheSegment>)> {
  let mut remaining = input;
  let mut segments = Vec::new();

  while !remaining.is_empty() {
    let segment = alt((parse_expression, parse_literal))(remaining).finish();
    match segment {
      Ok((rem, part)) => {
        // Check if remaining input hasn't changed to avoid infinite loop
        if rem == remaining {
          segments.push(MustacheSegment::Literal(remaining.to_string()));
          break;
        }
        segments.push(part);
        remaining = rem;
      }
      Err(_) => {
        segments.push(MustacheSegment::Literal(remaining.to_string()));
        break;
      }
    }
  }

  Ok((remaining, segments))
}

impl<'de> Deserialize<'de> for Mustache {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;

    match parse_parts(&s) {
      Ok((_, segments)) => Ok(Mustache::Segments(segments)),
      Err(e) => Err(D::Error::custom(e.to_string())),
    }
  }
}

#[cfg(test)]
mod tests {
  use std::borrow::Cow;

  use super::*;

  #[test]
  fn test_deserialize_single_literal() {
    let s = r#""hello/world""#;
    let mustache: Mustache = serde_json::from_str(s).unwrap();
    assert_eq!(
      mustache,
      Mustache::Segments(vec![MustacheSegment::Literal("hello/world".to_string())])
    );
  }

  #[test]
  fn test_deserialize_single_template() {
    let s = r#""{{hello.world}}""#;
    let mustache: Mustache = serde_json::from_str(s).unwrap();
    assert_eq!(
      mustache,
      Mustache::Segments(vec![MustacheSegment::Expression(vec![
        "hello".to_string(),
        "world".to_string()
      ])])
    );
  }

  #[test]
  fn test_deserialize_mixed() {
    let s = r#""http://localhost:8090/{{foo.bar}}/api/{{hello.world}}/end""#;
    let mustache: Mustache = serde_json::from_str(s).unwrap();
    assert_eq!(
      mustache,
      Mustache::Segments(vec![
        MustacheSegment::Literal("http://localhost:8090/".to_string()),
        MustacheSegment::Expression(vec!["foo".to_string(), "bar".to_string()]),
        MustacheSegment::Literal("/api/".to_string()),
        MustacheSegment::Expression(vec!["hello".to_string(), "world".to_string()]),
        MustacheSegment::Literal("/end".to_string())
      ])
    );
  }

  #[test]
  fn test_deserialize_with_spaces() {
    let s = "\"{{ foo . bar }}\"";
    let mustache: Mustache = serde_json::from_str(s).unwrap();
    assert_eq!(
      mustache,
      Mustache::Segments(vec![MustacheSegment::Expression(vec![
        "foo".to_string(),
        "bar".to_string()
      ])])
    );
  }

  #[test]
  fn test_parse_expression_with_valid_input() {
    let result = parse_expression("{{ foo.bar }} extra");
    assert_eq!(
      result,
      Ok((
        " extra",
        MustacheSegment::Expression(vec!["foo".to_string(), "bar".to_string()])
      ))
    );
  }

  #[test]
  fn test_parse_expression_with_invalid_input() {
    let result = parse_expression("foo.bar }}");
    assert!(result.is_err());
  }

  #[test]
  fn test_parse_parts_mixed() {
    let result = parse_parts("prefix {{foo.bar}} middle {{baz.qux}} suffix").unwrap();
    assert_eq!(
      result,
      (
        "",
        vec![
          MustacheSegment::Literal("prefix ".to_string()),
          MustacheSegment::Expression(vec!["foo".to_string(), "bar".to_string()]),
          MustacheSegment::Literal(" middle ".to_string()),
          MustacheSegment::Expression(vec!["baz".to_string(), "qux".to_string()]),
          MustacheSegment::Literal(" suffix".to_string())
        ]
      )
    );
  }

  #[test]
  fn test_parse_parts_only_literal() {
    let result = parse_parts("just a string").unwrap();
    assert_eq!(
      result,
      ("", vec![MustacheSegment::Literal("just a string".to_string())])
    );
  }

  #[test]
  fn test_parse_parts_only_expression() {
    let result = parse_parts("{{foo.bar}}").unwrap();
    assert_eq!(
      result,
      (
        "",
        vec![MustacheSegment::Expression(vec!["foo".to_string(), "bar".to_string()])]
      )
    );
  }
  #[test]
  fn test_render_mixed() {
    struct DummyPath;

    impl AnyPath for DummyPath {
      fn any_path(&self, parts: &[String]) -> Option<Cow<'_, str>> {
        if parts == ["foo", "bar"] {
          Some(Cow::Borrowed("FOOBAR"))
        } else if parts == ["baz", "qux"] {
          Some(Cow::Borrowed("BAZQUX"))
        } else {
          None
        }
      }
    }

    let mustache = Mustache::Segments(vec![
      MustacheSegment::Literal("prefix ".to_string()),
      MustacheSegment::Expression(vec!["foo".to_string(), "bar".to_string()]),
      MustacheSegment::Literal(" middle ".to_string()),
      MustacheSegment::Expression(vec!["baz".to_string(), "qux".to_string()]),
      MustacheSegment::Literal(" suffix".to_string()),
    ]);

    assert_eq!(mustache.render(&DummyPath), "prefix FOOBAR middle BAZQUX suffix");
  }

  #[test]
  fn test_render_with_missing_path() {
    struct DummyPath;

    impl AnyPath for DummyPath {
      fn any_path(&self, _: &[String]) -> Option<Cow<'_, str>> {
        None
      }
    }

    let mustache = Mustache::Segments(vec![
      MustacheSegment::Literal("prefix ".to_string()),
      MustacheSegment::Expression(vec!["foo".to_string(), "bar".to_string()]),
      MustacheSegment::Literal(" suffix".to_string()),
    ]);

    assert_eq!(mustache.render(&DummyPath), "prefix  suffix");
  }
  #[test]
  fn test_render_preserves_spaces() {
    struct DummyPath;

    impl AnyPath for DummyPath {
      fn any_path(&self, parts: &[String]) -> Option<Cow<'_, str>> {
        if parts == ["foo"] {
          Some(Cow::Borrowed("bar"))
        } else {
          None
        }
      }
    }

    let mustache = Mustache::Segments(vec![
      MustacheSegment::Literal("    ".to_string()),
      MustacheSegment::Expression(vec!["foo".to_string()]),
      MustacheSegment::Literal("    ".to_string()),
    ]);

    assert_eq!(mustache.render(&DummyPath).as_str(), "    bar    ");
  }
  #[test]
  fn test_deserialize_unfinished_expression() {
    let s = r#""{{hello.world""#;
    let mustache: Mustache = serde_json::from_str(s).unwrap();
    assert_eq!(
      mustache,
      Mustache::Segments(vec![MustacheSegment::Literal("{{hello.world".to_string())])
    );
  }
  #[test]
  fn test_deserialize_invalid_json() {
    let s = "hello.world}}";
    let mustache: Result<Mustache, _> = serde_json::from_str(s);
    assert!(mustache.is_err());
  }
  #[test]
  fn test_deserialize_nested_expression() {
    let s = r#""{{hello.{{world}}.foo}}"#;
    let mustache: Result<Mustache, _> = serde_json::from_str(s);
    assert!(mustache.is_err());
  }
  #[test]
  fn test_new_number() {
    let mustache = Mustache::new("123").unwrap();
    assert_eq!(
      mustache,
      Mustache::Segments(vec![MustacheSegment::Literal("123".to_string())])
    );
  }
}
