use std::fmt::{Debug, Display};

use crate::cli::CLIError;

use super::Cause;

#[derive(Debug, PartialEq, Default)]
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

impl<'a> From<ValidationError<&'a str>> for CLIError {
    fn from(error: ValidationError<&'a str>) -> Self {
        CLIError::new("Invalid Configuration").caused_by(
            error
                .as_vec()
                .iter()
                .map(|cause| CLIError::new(cause.message).trace(Vec::from(cause.trace.clone())))
                .collect(),
        )
    }
}

impl<E> ValidationError<E> {
    pub fn map<E1, F: Fn(E) -> E1>(self, f: F) -> ValidationError<E1> {
        ValidationError(self.0.into_iter().map(|e| e.map(&f)).collect())
    }

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
}

impl<E: Display + Debug> std::error::Error for ValidationError<E> {}

impl From<Cause<String>> for ValidationError<String> {
    fn from(value: Cause<String>) -> Self {
        ValidationError(vec![value])
    }
}

impl<E> From<Vec<Cause<E>>> for ValidationError<E> {
    fn from(value: Vec<Cause<E>>) -> Self {
        ValidationError(value)
    }
}
