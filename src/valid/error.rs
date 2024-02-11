use std::fmt::{Debug, Display};

use async_graphql::ServerError;
use regex::Regex;

use super::Cause;
use crate::async_graphql_hyper::GraphQLResponse;

#[derive(Debug, PartialEq, Default, Clone)]
pub struct ValidationError<E>(Vec<Cause<E>>);

impl<E: Display> Display for ValidationError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Validation Error\n")?;
        let errors = self.as_vec();
        for error in errors {
            f.write_str(format!("{} {}", '\u{2022}', error.message).as_str())?;
            if !error.trace.is_empty() {
                f.write_str(
                    &(format!(
                        " [{}]",
                        error
                            .trace
                            .iter()
                            .cloned()
                            .collect::<Vec<String>>()
                            .join(", ")
                    )),
                )?;
            }
            f.write_str("\n")?;
        }

        Ok(())
    }
}

impl<E> ValidationError<E> {
    pub fn as_vec(&self) -> &Vec<Cause<E>> {
        &self.0
    }

    pub fn into_vec(self) -> Vec<Cause<E>> {
        self.0
    }

    pub fn combine(mut self, mut other: ValidationError<E>) -> ValidationError<E> {
        self.0.append(&mut other.0);
        self
    }

    pub fn empty() -> Self {
        ValidationError(Vec::new())
    }

    pub fn new(e: E) -> Self {
        ValidationError(vec![Cause::new(e)])
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn trace(self, message: &str) -> Self {
        let mut errors = self.0;
        for cause in errors.iter_mut() {
            cause.trace.insert(0, message.to_owned());
        }
        Self(errors)
    }

    pub fn append(self, error: E) -> Self {
        let mut errors = self.0;
        errors.push(Cause::new(error));
        Self(errors)
    }

    pub fn transform<E1>(self, f: &impl Fn(E) -> E1) -> ValidationError<E1> {
        ValidationError(self.0.into_iter().map(|cause| cause.transform(f)).collect())
    }
}

impl<E: Display + Debug> std::error::Error for ValidationError<E> {}

impl<E> From<Cause<E>> for ValidationError<E> {
    fn from(value: Cause<E>) -> Self {
        ValidationError(vec![value])
    }
}

impl<E> From<Vec<Cause<E>>> for ValidationError<E> {
    fn from(value: Vec<Cause<E>>) -> Self {
        ValidationError(value)
    }
}

impl From<serde_path_to_error::Error<serde_json::Error>> for ValidationError<String> {
    fn from(error: serde_path_to_error::Error<serde_json::Error>) -> Self {
        let mut trace = Vec::new();
        let segments = error.path().iter();
        let len = segments.len();
        for (i, segment) in segments.enumerate() {
            match segment {
                serde_path_to_error::Segment::Seq { index } => {
                    trace.push(format!("[{}]", index));
                }
                serde_path_to_error::Segment::Map { key } => {
                    trace.push(key.to_string());
                }
                serde_path_to_error::Segment::Enum { variant } => {
                    trace.push(variant.to_string());
                }
                serde_path_to_error::Segment::Unknown => {
                    trace.push("?".to_owned());
                }
            }
            if i < len - 1 {
                trace.push(".".to_owned());
            }
        }

        let re = Regex::new(r" at line \d+ column \d+$").unwrap();
        let message = re
            .replace(
                format!("Parsing failed because of {}", error.inner()).as_str(),
                "",
            )
            .into_owned();

        ValidationError(vec![Cause::new(message).trace(trace)])
    }
}

impl From<ValidationError<String>> for GraphQLResponse {
    fn from(value: ValidationError<String>) -> Self {
        let mut response = async_graphql::Response::default();

        response.errors = value
            .into_vec()
            .into_iter()
            .map(ServerError::from)
            .collect();

        GraphQLResponse::from(response)
    }
}

#[cfg(test)]
mod tests {
    use async_graphql::{BatchResponse, ServerError};
    use pretty_assertions::assert_eq;
    use stripmargin::StripMargin;

    use crate::async_graphql_hyper::GraphQLResponse;
    use crate::valid::{Cause, ValidationError};

    #[derive(Debug, PartialEq, serde::Deserialize)]
    struct Foo {
        a: i32,
    }

    #[test]
    fn test_error_display_formatting() {
        let error = ValidationError::from(vec![
            Cause::new("1").trace(vec!["a", "b"]),
            Cause::new("2"),
            Cause::new("3"),
        ]);
        let expected_output = "\
        |Validation Error
        |• 1 [a, b]
        |• 2
        |• 3
        |"
        .strip_margin();
        assert_eq!(format!("{}", error), expected_output);
    }

    #[test]
    fn test_from_serde_error() {
        let foo = &mut serde_json::Deserializer::from_str("{ \"a\": true }");
        let actual =
            ValidationError::from(serde_path_to_error::deserialize::<_, Foo>(foo).unwrap_err());
        let expected = ValidationError::new(
            "Parsing failed because of invalid type: boolean `true`, expected i32".to_string(),
        )
        .trace("a");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_to_graphql_response() {
        let err = ValidationError::from(
            Cause::new("1".to_string()).trace(vec!["a".to_string(), "b".to_string()]),
        );
        let res = GraphQLResponse::from(err.clone());
        let res = match res.0 {
            BatchResponse::Single(x) => x,
            BatchResponse::Batch(x) => x.into_iter().next().unwrap(),
        };

        assert_eq!(res.errors.len(), 1);
        assert_eq!(ServerError::from(err.as_vec()[0].to_owned()), res.errors[0]);
    }
}
