use std::fmt::Display;

use nom::branch::alt;
use nom::bytes::complete::{tag, take_until};
use nom::character::complete::char;
use nom::combinator::map;
use nom::multi::many0;
use nom::sequence::delimited;
use nom::{Finish, IResult};

use crate::path::{PathGraphql, PathString};

#[derive(Debug, Clone)]
pub struct Mustache {
    segments: Vec<Segment>,
    jacques: jaq_interpret::Filter,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Segment {
    Literal(String),
    Expression(Vec<String>),
}

impl From<Vec<Segment>> for Mustache {
    fn from(segments: Vec<Segment>) -> Self {
        Mustache { segments, jacques: jaq_interpret::Filter::default() }
    }
}

impl Mustache {
    pub fn is_const(&self) -> bool {
        for s in &self.segments {
            if let Segment::Expression(_) = s {
                return false;
            }
        }
        true
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
        let val: String = self
            .segments
            .iter()
            .map(|segment| match segment {
                Segment::Literal(text) => text.to_string(),
                Segment::Expression(parts) => value
                    .path_string(parts)
                    .map(|a| a.to_string())
                    .unwrap_or_default(),
            })
            .collect();

        if val.is_empty() {
            self.evaluate(value)
        } else {
            val
        }
    }

    fn evaluate(&self, value: &impl PathString) -> String {
        value
            .evaluate(&self.jacques)
            .unwrap_or_default()
            .to_string()
    }

    pub fn render_graphql(&self, value: &impl PathGraphql) -> String {
        self.segments
            .iter()
            .map(|segment| match segment {
                Segment::Literal(text) => text.to_string(),
                Segment::Expression(parts) => value.path_graphql(parts).unwrap_or_default(),
            })
            .collect()
    }

    pub fn get_segments(&self) -> Vec<&Segment> {
        self.segments.iter().collect()
    }

    pub fn expression_segments(&self) -> Vec<&Vec<String>> {
        self.segments
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
            .segments
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
            nom::sequence::tuple((
                nom::combinator::opt(char('.')), // Optional leading dot
                nom::multi::separated_list1(char('.'), parse_name),
            )),
            |(_, expr_parts)| Segment::Expression(expr_parts),
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
    let result = map(parse_segment, |segments| {
        segments
            .into_iter()
            .filter(|seg| match seg {
                Segment::Literal(s) => (!s.is_empty()) && s != "\"",
                _ => true,
            })
            .collect::<Vec<Segment>>()
    })(input);
    let (_res, segments) = result?;
    let (res, jacques) = parse_jq(input).unwrap_or((input, jaq_interpret::Filter::default()));

    Ok((res, Mustache { segments, jacques }))
}

fn parse_jq(input: &str) -> IResult<&str, jaq_interpret::Filter> {
    let (input, _) = tag("{{")(input)?;
    let (input, filter) = take_until("}}")(input)?;
    let (input, _) = tag("}}")(input)?;
    let filter = filter.trim();
    let mut defs = jaq_interpret::ParseCtx::new(vec![]);
    defs.insert_natives(jaq_core::core());
    defs.insert_defs(jaq_std::std());

    let (filter, errs) = jaq_parse::parse(filter, jaq_parse::main());
    if !errs.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            "failed to parse filter",
            nom::error::ErrorKind::Tag,
        )));
    }
    let filter = filter;
    if filter.is_none() {
        return Err(nom::Err::Error(nom::error::Error::new(
            "failed to parse filter",
            nom::error::ErrorKind::Tag,
        )));
    }
    let filter = defs.compile(filter.unwrap());
    Ok((input, filter))
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
                mustache.segments,
                Mustache::from(vec![Segment::Literal("hello/world".to_string())]).segments
            );
        }

        #[test]
        fn test_single_template() {
            let s = r"{{hello.world}}";
            let mustache: Mustache = Mustache::parse(s).unwrap();
            assert_eq!(
                mustache.segments,
                Mustache::from(vec![Segment::Expression(vec![
                    "hello".to_string(),
                    "world".to_string(),
                ])])
                .segments
            );
        }

        #[test]
        fn test_mixed() {
            let s = r"http://localhost:8090/{{foo.bar}}/api/{{hello.world}}/end";
            let mustache: Mustache = Mustache::parse(s).unwrap();
            assert_eq!(
                mustache.segments,
                Mustache::from(vec![
                    Segment::Literal("http://localhost:8090/".to_string()),
                    Segment::Expression(vec!["foo".to_string(), "bar".to_string()]),
                    Segment::Literal("/api/".to_string()),
                    Segment::Expression(vec!["hello".to_string(), "world".to_string()]),
                    Segment::Literal("/end".to_string()),
                ])
                .segments
            );
        }

        #[test]
        fn test_with_spaces() {
            let s = "{{ foo . bar }}";
            let mustache: Mustache = Mustache::parse(s).unwrap();
            assert_eq!(
                mustache.segments,
                Mustache::from(vec![Segment::Expression(vec![
                    "foo".to_string(),
                    "bar".to_string(),
                ])])
                .segments
            );
        }

        #[test]
        fn test_parse_expression_with_valid_input() {
            let result = Mustache::parse("{{ foo.bar }} extra").unwrap();
            let expected = Mustache::from(vec![
                Segment::Expression(vec!["foo".to_string(), "bar".to_string()]),
                Segment::Literal(" extra".to_string()),
            ]);
            assert_eq!(result.segments, expected.segments);
        }

        #[test]
        fn test_parse_expression_with_invalid_input() {
            let result = Mustache::parse("foo.bar }}").unwrap();
            let expected = Mustache::from(vec![Segment::Literal("foo.bar }}".to_string())]);
            assert_eq!(result.segments, expected.segments);
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
            assert_eq!(result.segments, expected.segments);
        }

        #[test]
        fn test_parse_segments_only_literal() {
            let result = Mustache::parse("just a string").unwrap();
            let expected = Mustache {
                segments: vec![Segment::Literal("just a string".to_string())],
                jacques: jaq_interpret::Filter::default(),
            };
            assert_eq!(result.segments, expected.segments);
        }

        #[test]
        fn test_parse_segments_only_expression() {
            let result = Mustache::parse("{{foo.bar}}").unwrap();
            let expected = Mustache {
                segments: vec![Segment::Expression(vec![
                    "foo".to_string(),
                    "bar".to_string(),
                ])],
                jacques: jaq_interpret::Filter::default(),
            };
            assert_eq!(result.segments, expected.segments);
        }

        #[test]
        fn test_unfinished_expression() {
            let s = r"{{hello.world";
            let mustache: Mustache = Mustache::parse(s).unwrap();
            assert_eq!(
                mustache.segments,
                Mustache::from(vec![Segment::Literal("{{hello.world".to_string())]).segments
            );
        }

        #[test]
        fn test_new_number() {
            let mustache = Mustache::parse("123").unwrap();
            assert_eq!(
                mustache.segments,
                Mustache::from(vec![Segment::Literal("123".to_string())]).segments
            );
        }

        #[test]
        fn parse_env_name() {
            let result = Mustache::parse("{{env.FOO}}").unwrap();
            assert_eq!(
                result.segments,
                Mustache::from(vec![Segment::Expression(vec![
                    "env".to_string(),
                    "FOO".to_string(),
                ])])
                .segments
            );
        }

        #[test]
        fn parse_env_with_underscores() {
            let result = Mustache::parse("{{env.FOO_BAR}}").unwrap();
            assert_eq!(
                result.segments,
                Mustache::from(vec![Segment::Expression(vec![
                    "env".to_string(),
                    "FOO_BAR".to_string(),
                ])])
                .segments
            );
        }

        #[test]
        fn single_curly_brackets() {
            let result = Mustache::parse("test:{SHA}string").unwrap();
            assert_eq!(
                result.segments,
                Mustache::from(vec![Segment::Literal("test:{SHA}string".to_string())]).segments
            );
        }

        #[test]
        fn test_optional_dot_expression() {
            let s = r"{{.foo.bar}}";
            let mustache: Mustache = Mustache::parse(s).unwrap();
            assert_eq!(
                mustache.segments,
                Mustache::from(vec![Segment::Expression(vec![
                    "foo".to_string(),
                    "bar".to_string(),
                ])])
                .segments
            );
        }
    }

    mod render {
        use std::borrow::Cow;

        use jaq_interpret::Filter;
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

                fn evaluate(&self, _filter: &Filter) -> Option<async_graphql::Value> {
                    None // TODO
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
                fn path_string<T: AsRef<str>>(&self, _: &[T]) -> Option<Cow<'_, str>> {
                    None
                }

                fn evaluate(&self, _filter: &Filter) -> Option<async_graphql::Value> {
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
                fn path_string<T: AsRef<str>>(&self, parts: &[T]) -> Option<Cow<'_, str>> {
                    let parts: Vec<&str> = parts.iter().map(AsRef::as_ref).collect();

                    if parts == ["foo"] {
                        Some(Cow::Borrowed("bar"))
                    } else {
                        None
                    }
                }

                fn evaluate(&self, _filter: &Filter) -> Option<async_graphql::Value> {
                    todo!()
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
