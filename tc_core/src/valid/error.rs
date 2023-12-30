use std::fmt::{Debug, Display};

use regex::Regex;

use super::Cause;

#[derive(Debug, PartialEq, Default, Clone)]
pub struct ValidationError<E>(Vec<Cause<E>>);

impl<E: Display> Display for ValidationError<E> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for _error in self.as_vec() {
      f.write_str("Validation Error\n")?;
      let errors = self.as_vec();
      for error in errors {
        f.write_str(format!("{} {}", '\u{2022}', error.message).as_str())?;
        f.write_str(&(format!(" [{}]", error.trace.iter().cloned().collect::<Vec<String>>().join(", "))))?;
        f.write_str("\n")?;
      }
    }

    Ok(())
  }
}

impl<E> ValidationError<E> {
  pub fn as_vec(&self) -> &Vec<Cause<E>> {
    &self.0
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
      .replace(format!("Parsing failed because of {}", error.inner()).as_str(), "")
      .into_owned();

    ValidationError(vec![Cause::new(message).trace(trace.into())])
  }
}

#[cfg(test)]
mod tests {
  use pretty_assertions::assert_eq;

  use crate::valid::ValidationError;

  #[derive(Debug, PartialEq, serde::Deserialize)]
  struct Foo {
    a: i32,
  }

  #[test]
  fn test_from_serde_error() {
    let foo = &mut serde_json::Deserializer::from_str("{ \"a\": true }");
    let actual = ValidationError::from(serde_path_to_error::deserialize::<_, Foo>(foo).unwrap_err());
    let expected =
      ValidationError::new("Parsing failed because of invalid type: boolean `true`, expected i32".to_string())
        .trace("a");
    assert_eq!(actual, expected);
  }
}
