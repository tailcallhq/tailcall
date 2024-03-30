use std::fmt::{Debug, Display};
use std::hash::Hash;

use indexmap::IndexSet;
use regex::Regex;

use super::Cause;

#[derive(Debug, PartialEq, Default, Clone, Eq)]
pub struct ValidationError<E: Eq + Hash>(IndexSet<Cause<E>>);

impl<E: Display + std::cmp::Eq + std::hash::Hash> Display for ValidationError<E> {
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

impl<E: std::cmp::Eq + std::hash::Hash> ValidationError<E> {
    pub fn as_vec(&self) -> &IndexSet<Cause<E>> {
        &self.0
    }

    pub fn combine(mut self, other: ValidationError<E>) -> ValidationError<E> {
        self.0.extend(other.0);
        self
    }

    pub fn empty() -> Self {
        ValidationError(IndexSet::new())
    }

    pub fn new(e: E) -> Self {
        let mut set = IndexSet::new();
        set.insert(Cause::new(e));
        ValidationError(set)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn trace(self, message: &str) -> Self {
        let mut errors = Vec::from_iter(self.0);
        for cause in errors.iter_mut() {
            cause.trace.insert(0, message.to_owned());
        }
        let errors = IndexSet::from_iter(errors);
        Self(errors)
    }

    pub fn append(self, error: E) -> Self {
        let mut errors = self.0;
        errors.insert(Cause::new(error));
        Self(errors)
    }

    pub fn transform<E1: std::cmp::Eq + std::hash::Hash>(
        self,
        f: &impl Fn(E) -> E1,
    ) -> ValidationError<E1> {
        ValidationError(self.0.into_iter().map(|cause| cause.transform(f)).collect())
    }
}

impl<E: Display + Debug + std::cmp::Eq + std::hash::Hash> std::error::Error for ValidationError<E> {}

impl<E: std::cmp::Eq + std::hash::Hash> From<Cause<E>> for ValidationError<E> {
    fn from(value: Cause<E>) -> Self {
        let mut set = IndexSet::new();
        set.insert(value);
        ValidationError(set)
    }
}

impl<E: std::cmp::Eq + std::hash::Hash> From<Vec<Cause<E>>> for ValidationError<E> {
    fn from(value: Vec<Cause<E>>) -> Self {
        let set = IndexSet::from_iter(value);
        ValidationError(set)
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
        let mut set = IndexSet::new();
        set.insert(Cause::new(message).trace(trace));
        ValidationError(set)
    }
}

impl From<hyper::header::InvalidHeaderValue> for ValidationError<String> {
    fn from(error: hyper::header::InvalidHeaderValue) -> Self {
        ValidationError::new(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use stripmargin::StripMargin;

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
}
