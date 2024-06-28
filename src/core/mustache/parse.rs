use nom::branch::alt;
use nom::bytes::complete::{tag, take_until};
use nom::character::complete::char;
use nom::combinator::map;
use nom::multi::many0;
use nom::sequence::delimited;
use nom::{Finish, IResult};

use super::*;

impl Mustache {
    // TODO: infallible function, no need to return Result
    pub fn parse(str: &str) -> anyhow::Result<Mustache> {
        let result = parse_mustache(str).finish();
        match result {
            Ok((_, mustache)) => Ok(mustache),
            Err(_) => Ok(Mustache::from(vec![Segment::Literal(str.to_string())])),
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
    map(parse_segment, |segments| {
        Mustache::from(segments.into_iter().filter(|seg| match seg {
            Segment::Literal(s) => (!s.is_empty()) && s != "\"",
            _ => true,
        }))
    })(input)
}

#[cfg(test)]
mod tests {

    use pretty_assertions::assert_eq;

    use crate::core::mustache::{Mustache, Segment};

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
        let expected = Mustache::from(vec![Segment::Literal("just a string".to_string())]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_segments_only_expression() {
        let result = Mustache::parse("{{foo.bar}}").unwrap();
        let expected = Mustache::from(vec![Segment::Expression(vec![
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

    #[test]
    fn test_optional_dot_expression() {
        let s = r"{{.foo.bar}}";
        let mustache: Mustache = Mustache::parse(s).unwrap();
        assert_eq!(
            mustache,
            Mustache::from(vec![Segment::Expression(vec![
                "foo".to_string(),
                "bar".to_string(),
            ])])
        );
    }
}
