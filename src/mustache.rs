use nom::branch::alt;
use nom::bytes::complete::{tag, take_until};
use nom::character::complete::char;
use nom::combinator::map;
use nom::multi::many0;
use nom::sequence::delimited;
use nom::{Finish, IResult};

use crate::path_resolver::PathResolver;

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

fn value_to_json_string(value: Option<async_graphql::Value>) -> Option<String> {
    value.and_then(|value| match value {
        async_graphql::Value::String(s) => Some(s),
        async_graphql::Value::Object(v) => serde_json::to_value(v).map(|v| v.to_string()).ok(),
        async_graphql::Value::List(v) => serde_json::to_value(v).map(|v| v.to_string()).ok(),
        _ => Some(value.to_string()),
    })
}

fn value_to_graphql_string(value: Option<async_graphql::Value>) -> Option<String> {
    value.map(|value| match value {
        async_graphql::Value::String(s) => format!(r#""{s}""#),
        _ => value.to_string(),
    })
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

    pub fn render(&self, value: &impl PathResolver) -> Option<async_graphql::Value> {
        let segments = &self.0;

        if let [Segment::Expression(path)] = &segments[..] {
            value.get_path_value(path)
        } else {
            let s: String = segments
                .iter()
                .map(|segment| match segment {
                    Segment::Literal(text) => text.clone(),
                    Segment::Expression(path) => {
                        value_to_json_string(value.get_path_value(path)).unwrap_or_default()
                    }
                })
                .collect();

            Some(async_graphql::Value::String(s))
        }
    }

    pub fn render_string(&self, value: &impl PathResolver) -> Option<String> {
        let value = self.render(value);

        value_to_json_string(value)
    }

    pub fn render_graphql(&self, value: &impl PathResolver) -> Option<String> {
        let value = self.render(value);

        value_to_graphql_string(value)
    }

    pub fn get_segments(&self) -> Vec<&Segment> {
        match self {
            Mustache(segments) => segments.iter().collect(),
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

impl ToString for Mustache {
    fn to_string(&self) -> String {
        match self {
            Mustache(segments) => segments
                .iter()
                .map(|segment| match segment {
                    Segment::Literal(text) => text.clone(),
                    Segment::Expression(parts) => format!("{{{{{}}}}}", parts.join(".")),
                })
                .collect::<Vec<String>>()
                .join(""),
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
                    Segment::Literal(s) => (!s.is_empty()) && s != "\"",
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
        fn test_to_string() {
            let expectations = vec![
                r"/users/{{value.id}}/todos",
                r"http://localhost:8090/{{foo.bar}}/api/{{hello.world}}/end",
                r"http://localhost:{{args.port}}",
                r"/users/{{value.userId}}",
                r"/bar?id={{args.id}}&flag={{args.flag}}",
                r"/foo?id={{value.id}}",
                r"{{value.d}}",
                r"/posts/{{args.id}}",
                r"http://localhost:8000",
            ];

            for expected in expectations {
                let mustache = Mustache::parse(expected).unwrap();

                assert_eq!(expected, mustache.to_string());
            }
        }

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
                    "world".to_string(),
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
                    Segment::Literal("/end".to_string()),
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
                    "bar".to_string(),
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
                    "FOO".to_string(),
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
                    "FOO_BAR".to_string(),
                ])])
            );
        }

        #[test]
        fn single_curly_brackets() {
            let result = Mustache::parse("test:{SHA}string").unwrap();
            assert_eq!(
                result,
                Mustache::from(vec![Segment::Literal("test:{SHA}string".to_string())])
            );
        }
    }

    mod render {
        use serde_json::json;

        use crate::mustache::{Mustache, Segment};
        use crate::path_resolver::PathResolver;

        #[test]
        fn test_query_params_template() {
            let s = r"/v1/templates?project-id={{value.projectId}}";
            let mustache: Mustache = Mustache::parse(s).unwrap();
            let ctx = json!({"value": {"projectId": "123"}});
            let result = mustache.render_string(&ctx);
            assert_eq!(result.unwrap(), "/v1/templates?project-id=123");
        }

        #[test]
        fn test_render_mixed() {
            struct DummyPath;

            impl PathResolver for DummyPath {
                fn get_path_value<Path>(&self, path: &[Path]) -> Option<async_graphql::Value>
                where
                    Path: AsRef<str>,
                {
                    let parts: Vec<&str> = path.iter().map(AsRef::as_ref).collect();

                    if parts == ["foo", "bar"] {
                        Some("FOOBAR".into())
                    } else if parts == ["baz", "qux"] {
                        Some("BAZQUX".into())
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
                mustache.render_string(&DummyPath).unwrap(),
                "prefix FOOBAR middle BAZQUX suffix"
            );
        }

        #[test]
        fn test_render_with_missing_path() {
            struct DummyPath;

            impl PathResolver for DummyPath {
                fn get_path_value<Path>(&self, _path: &[Path]) -> Option<async_graphql::Value>
                where
                    Path: AsRef<str>,
                {
                    None
                }
            }

            let mustache = Mustache::from(vec![
                Segment::Literal("prefix ".to_string()),
                Segment::Expression(vec!["foo".to_string(), "bar".to_string()]),
                Segment::Literal(" suffix".to_string()),
            ]);

            assert_eq!(
                mustache.render_string(&DummyPath).unwrap(),
                "prefix  suffix"
            );
        }

        #[test]
        fn test_json_like() {
            let mustache =
                Mustache::parse(r#"{registered: "{{foo}}", display: "{{bar}}"}"#).unwrap();
            let ctx = json!({"foo": "baz", "bar": "qux"});
            let result = mustache.render_string(&ctx);
            assert_eq!(result.unwrap(), r#"{registered: "baz", display: "qux"}"#);
        }

        #[test]
        fn test_json_like_static() {
            let mustache = Mustache::parse(r#"{registered: "foo", display: "bar"}"#).unwrap();
            let ctx = json!({}); // Context is not used in this case
            let result = mustache.render_string(&ctx);
            assert_eq!(result.unwrap(), r#"{registered: "foo", display: "bar"}"#);
        }

        #[test]
        fn test_render_preserves_spaces() {
            struct DummyPath;

            impl PathResolver for DummyPath {
                fn get_path_value<Path>(&self, path: &[Path]) -> Option<async_graphql::Value>
                where
                    Path: AsRef<str>,
                {
                    let parts: Vec<&str> = path.iter().map(AsRef::as_ref).collect();

                    if parts == ["foo"] {
                        Some("bar".into())
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

            assert_eq!(mustache.render_string(&DummyPath).unwrap(), "    bar    ");
        }
    }

    mod render_graphql {
        use crate::mustache::{Mustache, Segment};
        use crate::path_resolver::PathResolver;

        #[test]
        fn test_render_mixed() {
            struct DummyPath;

            impl PathResolver for DummyPath {
                fn get_path_value<Path>(&self, path: &[Path]) -> Option<async_graphql::Value>
                where
                    Path: AsRef<str>,
                {
                    let parts: Vec<&str> = path.iter().map(AsRef::as_ref).collect();

                    if parts == ["foo", "bar"] {
                        Some("FOOBAR".into())
                    } else if parts == ["baz", "qux"] {
                        Some("BAZQUX".into())
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
                mustache.render_graphql(&DummyPath).unwrap(),
                "\"prefix FOOBAR middle BAZQUX suffix\""
            );
        }

        #[test]
        fn test_render_with_missing_path() {
            struct DummyPath;

            impl PathResolver for DummyPath {
                fn get_path_value<Path>(&self, _path: &[Path]) -> Option<async_graphql::Value>
                where
                    Path: AsRef<str>,
                {
                    None
                }
            }

            let mustache = Mustache::from(vec![
                Segment::Literal("prefix ".to_string()),
                Segment::Expression(vec!["foo".to_string(), "bar".to_string()]),
                Segment::Literal(" suffix".to_string()),
            ]);

            assert_eq!(
                mustache.render_graphql(&DummyPath).unwrap(),
                "\"prefix  suffix\""
            );
        }
    }
}
