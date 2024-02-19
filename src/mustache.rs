use std::borrow::Cow;

use nom::branch::alt;
use nom::bytes::complete::{tag, take_until};
use nom::character::complete::char;
use nom::combinator::map;
use nom::multi::many0;
use nom::sequence::delimited;
use nom::{Finish, IResult};
use serde_json::{json, Value};

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

    fn convert_value(value: &async_graphql::Value) -> Option<Cow<'_, str>> {
        match value {
            async_graphql::Value::String(s) => Some(Cow::Borrowed(s.as_str())),
            async_graphql::Value::Number(n) => Some(Cow::Owned(n.to_string())),
            async_graphql::Value::Boolean(b) => Some(Cow::Owned(b.to_string())),
            async_graphql::Value::Object(map) => Some(json!(map).to_string().into()),
            async_graphql::Value::List(list) => Some(json!(list).to_string().into()),
            _ => None,
        }
    }
    pub fn render(&self, value: &impl PathString) -> String {
        match self {
            Mustache(segments) => segments
                .iter()
                .map(|segment| match segment {
                    Segment::Literal(text) => text.clone(),
                    Segment::Expression(parts) => value
                        .path_string(parts)
                        .and_then(|a| Mustache::convert_value(a.as_ref()).map(|s| s.to_string()))
                        .unwrap_or_default(),
                })
                .collect(),
        }
    }

    pub fn render_value(&self, value: &impl PathString) -> anyhow::Result<async_graphql::Value> {
        let p = self.render(value);
        let json = serde_json::from_str::<Value>(&p)?;
        match json {
            Value::String(s) => {
                let out = serde_json::from_str::<Value>(s.as_str());
                match out {
                    Ok(v) => async_graphql::Value::from_json(v).map_err(|e| anyhow::anyhow!(e)),
                    Err(_) => Ok(async_graphql::Value::String(s)),
                }
            }
            _ => async_graphql::Value::from_json(json).map_err(|e| anyhow::anyhow!(e)),
        }
    }

    pub fn render_graphql(&self, value: &impl PathGraphql) -> String {
        match self {
            Mustache(segments) => segments
                .iter()
                .map(|segment| match segment {
                    Segment::Literal(text) => text.to_string(),
                    Segment::Expression(parts) => value
                        .path_graphql(parts)
                        .and_then(|a| Mustache::convert_value(a.as_ref()).map(|s| s.to_string()))
                        .unwrap_or_default(),
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
    let spaces = nom::character::complete::multispace0;
    let alpha = nom::character::complete::alpha1;
    let alphanumeric_or_underscore = nom::multi::many0(nom::branch::alt((
        nom::character::complete::alphanumeric1,
        nom::bytes::complete::tag("_"),
    )));

    let parser = nom::sequence::tuple((spaces, alpha, alphanumeric_or_underscore, spaces));

    nom::combinator::map(parser, |(_, a, b, _)| {
        let b: String = b.into_iter().collect();
        format!("{}{}", a, b)
    })(input)
}

fn parse_expression(input: &str) -> IResult<&str, Segment> {
    delimited(
        tag("{{"),
        map(
            nom::multi::separated_list1(char('.'), parse_name),
            Segment::Expression,
        ),
        tag("}}"),
    )(input)
}

fn parse_segment(input: &str) -> IResult<&str, Vec<Segment>> {
    let expression_result = many0(alt((
        parse_expression,
        map(take_until("{{"), |txt: &str| {
            Segment::Literal(txt.to_string())
        }),
    )))(input);

    if let Ok((remaining, segments)) = expression_result {
        if remaining.is_empty() {
            Ok((remaining, segments))
        } else {
            let mut segments = segments;
            segments.push(Segment::Literal(remaining.to_string()));
            Ok(("", segments))
        }
    } else {
        Ok(("", vec![Segment::Literal(input.to_string())]))
    }
}

fn parse_mustache(input: &str) -> IResult<&str, Mustache> {
    map(parse_segment, |segments| {
        Mustache(
            segments
                .into_iter()
                .filter(|seg| match seg {
                    Segment::Literal(s) => !s.is_empty(),
                    _ => true,
                })
                .collect(),
        )
    })(input)
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
                Mustache::from(vec![Segment::Expression(vec![
                    "foo".to_string(),
                    "bar".to_string()
                ])])
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
            let expected = Mustache(vec![Segment::Expression(vec![
                "foo".to_string(),
                "bar".to_string(),
            ])]);
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
            assert_eq!(
                mustache,
                Mustache::from(vec![Segment::Literal("123".to_string())])
            );
        }

        #[test]
        fn parse_env_name() {
            let result = Mustache::parse("{{env.FOO}}").unwrap();
            assert_eq!(
                result,
                Mustache::from(vec![Segment::Expression(vec![
                    "env".to_string(),
                    "FOO".to_string()
                ])])
            );
        }

        #[test]
        fn parse_env_with_underscores() {
            let result = Mustache::parse("{{env.FOO_BAR}}").unwrap();
            assert_eq!(
                result,
                Mustache::from(vec![Segment::Expression(vec![
                    "env".to_string(),
                    "FOO_BAR".to_string()
                ])])
            );
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
                fn path_string<T: AsRef<str>>(
                    &self,
                    parts: &[T],
                ) -> Option<Cow<'_, async_graphql::Value>> {
                    let parts: Vec<&str> = parts.iter().map(AsRef::as_ref).collect();

                    if parts == ["foo", "bar"] {
                        Some(Cow::Owned(async_graphql::Value::String(
                            "FOOBAR".to_string(),
                        )))
                    } else if parts == ["baz", "qux"] {
                        Some(Cow::Owned(async_graphql::Value::String(
                            "BAZQUX".to_string(),
                        )))
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
                mustache.render(&DummyPath),
                "prefix FOOBAR middle BAZQUX suffix"
            );
        }

        #[test]
        fn test_render_with_missing_path() {
            struct DummyPath;

            impl PathString for DummyPath {
                fn path_string<T: AsRef<str>>(
                    &self,
                    _: &[T],
                ) -> Option<Cow<'_, async_graphql::Value>> {
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
        fn test_json_like() {
            let mustache =
                Mustache::parse(r#"{registered: "{{foo}}", display: "{{bar}}"}"#).unwrap();
            let ctx = json!({"foo": "baz", "bar": "qux"});
            let result = mustache.render(&ctx);
            assert_eq!(result, r#"{registered: "baz", display: "qux"}"#);
        }

        #[test]
        fn test_json_like_static() {
            let mustache = Mustache::parse(r#"{registered: "foo", display: "bar"}"#).unwrap();
            let ctx = json!({}); // Context is not used in this case
            let result = mustache.render(&ctx);
            assert_eq!(result, r#"{registered: "foo", display: "bar"}"#);
        }

        #[test]
        fn test_render_preserves_spaces() {
            struct DummyPath;

            impl PathString for DummyPath {
                fn path_string<T: AsRef<str>>(
                    &self,
                    parts: &[T],
                ) -> Option<Cow<'_, async_graphql::Value>> {
                    let parts: Vec<&str> = parts.iter().map(AsRef::as_ref).collect();

                    if parts == ["foo"] {
                        Some(Cow::Owned(async_graphql::Value::String("bar".to_string())))
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
        use std::borrow::Cow;

        use crate::mustache::{Mustache, Segment};
        use crate::path::PathGraphql;

        #[test]
        fn test_render_mixed() {
            struct DummyPath;

            impl PathGraphql for DummyPath {
                fn path_graphql<T: AsRef<str>>(
                    &self,
                    parts: &[T],
                ) -> Option<Cow<'_, async_graphql::Value>> {
                    let parts: Vec<&str> = parts.iter().map(AsRef::as_ref).collect();

                    if parts == ["foo", "bar"] {
                        Some(Cow::Owned(async_graphql::Value::String(
                            "FOOBAR".to_string(),
                        )))
                    } else if parts == ["baz", "qux"] {
                        Some(Cow::Owned(async_graphql::Value::String(
                            "BAZQUX".to_string(),
                        )))
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
                fn path_graphql<T: AsRef<str>>(
                    &self,
                    _: &[T],
                ) -> Option<Cow<'_, async_graphql::Value>> {
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
