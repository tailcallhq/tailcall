use nom::{Finish, IResult};

use crate::path::{PathGraphql, PathString};

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Mustache(Vec<Segment>);

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Segment {
  Literal(String),
  Expression(Vec<String>),
}

impl From<Vec<Segment>> for Mustache {
  fn from(segments: Vec<Segment>) -> Self {
    Mustache(segments)
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

  // TODO: infallible function, no need to return Result
  pub fn parse(str: &str) -> anyhow::Result<Mustache> {
    let result = parse_mustache(str).finish();
    match result {
      Ok((_, mustache)) => Ok(mustache),
      Err(_) => Ok(Mustache::from(vec![Segment::Literal(str.to_string())])),
    }
  }

  pub fn render(&self, value: &impl PathString) -> String {
    match self {
      Mustache(segments) => segments
        .iter()
        .map(|segment| match segment {
          Segment::Literal(text) => text.clone(),
          Segment::Expression(parts) => value.path_string(parts).map(|a| a.to_string()).unwrap_or_default(),
        })
        .collect(),
    }
  }

  pub fn render_graphql(&self, value: &impl PathGraphql) -> String {
    match self {
      Mustache(segments) => segments
        .iter()
        .map(|segment| match segment {
          Segment::Literal(text) => text.to_string(),
          Segment::Expression(parts) => value.path_graphql(parts).unwrap_or_default(),
        })
        .collect(),
    }
  }

  pub fn expression_segments(&self) -> Vec<&Vec<String>> {
    match self {
      Mustache(segments) => segments
        .iter()
        .filter_map(|seg| match seg {
          Segment::Expression(parts) => Some(parts),
          _ => None,
        })
        .collect(),
    }
  }
}

fn parse_name(input: &str) -> IResult<&str, String> {
  nom::combinator::map(
    nom::sequence::tuple((
      nom::character::complete::multispace0,
      nom::character::complete::alpha1,
      nom::multi::many0(nom::branch::alt((
        nom::character::complete::alphanumeric1,
        nom::bytes::complete::tag("_"),
      ))),
      nom::character::complete::multispace0,
    )),
    |(_, a, b, _)| {
      let b: String = b.into_iter().collect();

      format!("{}{}", a, b)
    },
  )(input)
}

fn parse_expression(input: &str) -> IResult<&str, Vec<String>> {
  nom::combinator::map(
    nom::sequence::tuple((
      nom::bytes::complete::tag("{{"),
      nom::multi::separated_list1(nom::character::complete::char('.'), parse_name),
      nom::bytes::complete::tag("}}"),
    )),
    |(_, vec, _)| vec,
  )(input)
}

fn parse_segment(input: &str) -> IResult<&str, Segment> {
  let expression = nom::combinator::map(parse_expression, Segment::Expression);
  let literal = nom::combinator::map(nom::bytes::complete::take_while1(|c| c != '{'), |r: &str| {
    Segment::Literal(r.to_string())
  });

  nom::branch::alt((expression, literal))(input)
}

fn parse_mustache(input: &str) -> IResult<&str, Mustache> {
  nom::combinator::map(nom::multi::many1(parse_segment), Mustache)(input)
}

#[cfg(test)]
mod tests {
  mod parse {
    use pretty_assertions::assert_eq;

    use crate::mustache::{Mustache, Segment};

    #[test]
    fn test_single_literal() {
      let s = r"hello/world";
      let mustache: Mustache = Mustache::parse(s).unwrap();
      assert_eq!(
        mustache,
        Mustache::from(vec![Segment::Literal("hello/world".to_string())])
      );
    }

    #[test]
    fn test_single_template() {
      let s = r"{{hello.world}}";
      let mustache: Mustache = Mustache::parse(s).unwrap();
      assert_eq!(
        mustache,
        Mustache::from(vec![Segment::Expression(vec![
          "hello".to_string(),
          "world".to_string()
        ])])
      );
    }

    #[test]
    fn test_mixed() {
      let s = r"http://localhost:8090/{{foo.bar}}/api/{{hello.world}}/end";
      let mustache: Mustache = Mustache::parse(s).unwrap();
      assert_eq!(
        mustache,
        Mustache::from(vec![
          Segment::Literal("http://localhost:8090/".to_string()),
          Segment::Expression(vec!["foo".to_string(), "bar".to_string()]),
          Segment::Literal("/api/".to_string()),
          Segment::Expression(vec!["hello".to_string(), "world".to_string()]),
          Segment::Literal("/end".to_string())
        ])
      );
    }

    #[test]
    fn test_with_spaces() {
      let s = "{{ foo . bar }}";
      let mustache: Mustache = Mustache::parse(s).unwrap();
      assert_eq!(
        mustache,
        Mustache::from(vec![Segment::Expression(vec!["foo".to_string(), "bar".to_string()])])
      );
    }

    #[test]
    fn test_parse_expression_with_valid_input() {
      let result = Mustache::parse("{{ foo.bar }} extra").unwrap();
      let expected = Mustache::from(vec![
        Segment::Expression(vec!["foo".to_string(), "bar".to_string()]),
        Segment::Literal(" extra".to_string()),
      ]);
      assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_expression_with_invalid_input() {
      let result = Mustache::parse("foo.bar }}").unwrap();
      let expected = Mustache::from(vec![Segment::Literal("foo.bar }}".to_string())]);
      assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_segments_mixed() {
      let result = Mustache::parse("prefix {{foo.bar}} middle {{baz.qux}} suffix").unwrap();
      let expected = Mustache::from(vec![
        Segment::Literal("prefix ".to_string()),
        Segment::Expression(vec!["foo".to_string(), "bar".to_string()]),
        Segment::Literal(" middle ".to_string()),
        Segment::Expression(vec!["baz".to_string(), "qux".to_string()]),
        Segment::Literal(" suffix".to_string()),
      ]);
      assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_segments_only_literal() {
      let result = Mustache::parse("just a string").unwrap();
      let expected = Mustache(vec![Segment::Literal("just a string".to_string())]);
      assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_segments_only_expression() {
      let result = Mustache::parse("{{foo.bar}}").unwrap();
      let expected = Mustache(vec![Segment::Expression(vec!["foo".to_string(), "bar".to_string()])]);
      assert_eq!(result, expected);
    }

    #[test]
    fn test_unfinished_expression() {
      let s = r"{{hello.world";
      let mustache: Mustache = Mustache::parse(s).unwrap();
      assert_eq!(
        mustache,
        Mustache::from(vec![Segment::Literal("{{hello.world".to_string())])
      );
    }

    #[test]
    fn test_new_number() {
      let mustache = Mustache::parse("123").unwrap();
      assert_eq!(mustache, Mustache::from(vec![Segment::Literal("123".to_string())]));
    }
  }
  mod render {
    use std::borrow::Cow;

    use serde_json::json;

    use crate::mustache::{Mustache, Segment};
    use crate::path::PathString;

    #[test]
    fn test_query_params_template() {
      let s = r"/v1/templates?project-id={{value.projectId}}";
      let mustache: Mustache = Mustache::parse(s).unwrap();
      let ctx = json!(json!({"value": {"projectId": "123"}}));
      let result = mustache.render(&ctx);
      assert_eq!(result, "/v1/templates?project-id=123");
    }

    #[test]
    fn test_render_mixed() {
      struct DummyPath;

      impl PathString for DummyPath {
        fn path_string<T: AsRef<str>>(&self, parts: &[T]) -> Option<Cow<'_, str>> {
          let parts: Vec<&str> = parts.iter().map(AsRef::as_ref).collect();

          if parts == ["foo", "bar"] {
            Some(Cow::Borrowed("FOOBAR"))
          } else if parts == ["baz", "qux"] {
            Some(Cow::Borrowed("BAZQUX"))
          } else {
            None
          }
        }
      }

      let mustache = Mustache::from(vec![
        Segment::Literal("prefix ".to_string()),
        Segment::Expression(vec!["foo".to_string(), "bar".to_string()]),
        Segment::Literal(" middle ".to_string()),
        Segment::Expression(vec!["baz".to_string(), "qux".to_string()]),
        Segment::Literal(" suffix".to_string()),
      ]);

      assert_eq!(mustache.render(&DummyPath), "prefix FOOBAR middle BAZQUX suffix");
    }

    #[test]
    fn test_render_with_missing_path() {
      struct DummyPath;

      impl PathString for DummyPath {
        fn path_string<T: AsRef<str>>(&self, _: &[T]) -> Option<Cow<'_, str>> {
          None
        }
      }

      let mustache = Mustache::from(vec![
        Segment::Literal("prefix ".to_string()),
        Segment::Expression(vec!["foo".to_string(), "bar".to_string()]),
        Segment::Literal(" suffix".to_string()),
      ]);

      assert_eq!(mustache.render(&DummyPath), "prefix  suffix");
    }

    #[test]
    fn test_render_preserves_spaces() {
      struct DummyPath;

      impl PathString for DummyPath {
        fn path_string<T: AsRef<str>>(&self, parts: &[T]) -> Option<Cow<'_, str>> {
          let parts: Vec<&str> = parts.iter().map(AsRef::as_ref).collect();

          if parts == ["foo"] {
            Some(Cow::Borrowed("bar"))
          } else {
            None
          }
        }
      }

      let mustache = Mustache::from(vec![
        Segment::Literal("    ".to_string()),
        Segment::Expression(vec!["foo".to_string()]),
        Segment::Literal("    ".to_string()),
      ]);

      assert_eq!(mustache.render(&DummyPath).as_str(), "    bar    ");
    }
  }

  mod render_graphql {
    use crate::mustache::{Mustache, Segment};
    use crate::path::PathGraphql;

    #[test]
    fn test_render_mixed() {
      struct DummyPath;

      impl PathGraphql for DummyPath {
        fn path_graphql<T: AsRef<str>>(&self, parts: &[T]) -> Option<String> {
          let parts: Vec<&str> = parts.iter().map(AsRef::as_ref).collect();

          if parts == ["foo", "bar"] {
            Some("FOOBAR".to_owned())
          } else if parts == ["baz", "qux"] {
            Some("BAZQUX".to_owned())
          } else {
            None
          }
        }
      }

      let mustache = Mustache::from(vec![
        Segment::Literal("prefix ".to_string()),
        Segment::Expression(vec!["foo".to_string(), "bar".to_string()]),
        Segment::Literal(" middle ".to_string()),
        Segment::Expression(vec!["baz".to_string(), "qux".to_string()]),
        Segment::Literal(" suffix".to_string()),
      ]);

      assert_eq!(
        mustache.render_graphql(&DummyPath),
        "prefix FOOBAR middle BAZQUX suffix"
      );
    }

    #[test]
    fn test_render_with_missing_path() {
      struct DummyPath;

      impl PathGraphql for DummyPath {
        fn path_graphql<T: AsRef<str>>(&self, _: &[T]) -> Option<String> {
          None
        }
      }

      let mustache = Mustache::from(vec![
        Segment::Literal("prefix ".to_string()),
        Segment::Expression(vec!["foo".to_string(), "bar".to_string()]),
        Segment::Literal(" suffix".to_string()),
      ]);

      assert_eq!(mustache.render_graphql(&DummyPath), "prefix  suffix");
    }
  }
}
