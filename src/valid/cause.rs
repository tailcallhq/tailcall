use std::collections::VecDeque;
use std::fmt::Display;

use async_graphql::{ErrorExtensionValues, ServerError};
use async_graphql_value::ConstValue;
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

impl From<Cause<String>> for ServerError {
    fn from(value: Cause<String>) -> Self {
        let mut error = ServerError::new(value.message.to_owned(), None);

        let mut ext: ErrorExtensionValues = ErrorExtensionValues::default();

        if let Some(description) = &value.description {
            ext.set("description", ConstValue::String(description.to_owned()));
        }

        if !value.trace.is_empty() {
            ext.set(
                "trace",
                ConstValue::List(value.trace.iter().map(|x| x.into()).collect()),
            );
        }

        error.extensions = Some(ext);
        error
    }
}

#[cfg(test)]
mod tests {
    use async_graphql::ServerError;
    use async_graphql_value::ConstValue;

    use super::Cause;

    #[test]
    fn test_display() {
        let cause = Cause::new("error")
            .trace(vec!["trace0", "trace1"])
            .description("description");
        assert_eq!(cause.to_string(), "[trace0, trace1] error: description");
    }

    #[test]
    fn test_server_error() {
        let cause: Cause<String> = Cause::new("Error".to_string())
            .description("This is a fake error".to_string())
            .trace(vec!["fake".to_string(), "trace".to_string()]);
        let err = ServerError::from(cause.clone());

        assert_eq!(err.message, cause.message);

        let ext = err.extensions.unwrap();
        assert_eq!(
            ext.get("description"),
            cause.description.map(ConstValue::String).as_ref()
        );

        let trace = ext.get("trace").unwrap().to_owned();
        if let ConstValue::List(trace) = trace {
            for (i, x) in trace.into_iter().enumerate() {
                if let ConstValue::String(s) = x {
                    assert_eq!(s, cause.trace[i]);
                } else {
                    panic!("Element {} of trace wasn't a string", i);
                }
            }
        } else {
            panic!("trace was not a list");
        }
    }
}
