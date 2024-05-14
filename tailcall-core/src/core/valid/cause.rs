use std::collections::VecDeque;
use std::fmt::Display;

use derive_setters::Setters;
use thiserror::Error;

#[derive(Clone, PartialEq, Debug, Setters, Error)]
pub struct Cause<E> {
    pub message: E,
    #[setters(strip_option)]
    pub description: Option<E>,
    #[setters(skip)]
    pub trace: VecDeque<String>,
}

impl<E: Display> Display for Cause<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (i, entry) in self.trace.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", entry)?;
        }
        write!(f, "] {}", self.message)?;
        if let Some(desc) = self.description.as_ref() {
            write!(f, ": {}", desc)?;
        }
        Ok(())
    }
}

impl<E> Cause<E> {
    pub fn new(e: E) -> Self {
        Cause { message: e, description: None, trace: VecDeque::new() }
    }

    pub fn transform<E1>(self, e: impl Fn(E) -> E1) -> Cause<E1> {
        Cause {
            message: e(self.message),
            description: self.description.map(e),
            trace: self.trace,
        }
    }

    pub fn trace<T: Display>(mut self, trace: Vec<T>) -> Self {
        self.trace = trace
            .iter()
            .map(|t| t.to_string())
            .collect::<VecDeque<String>>();
        self
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_display() {
        use super::Cause;
        let cause = Cause::new("error")
            .trace(vec!["trace0", "trace1"])
            .description("description");
        assert_eq!(cause.to_string(), "[trace0, trace1] error: description");
    }
}
