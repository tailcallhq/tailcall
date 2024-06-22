use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::core::merge_right::MergeRight;

#[derive(
    Serialize, Deserialize, PartialEq, Eq, Clone, Debug, schemars::JsonSchema, Ord, PartialOrd,
)]
#[serde(transparent)]
pub struct Pos<T> {
    pub inner: T,
    #[serde(skip_serializing, skip_deserializing)]
    pub line: usize,
    #[serde(skip_serializing, skip_deserializing)]
    pub column: usize,
    #[serde(skip_serializing, skip_deserializing)]
    pub file_path: Option<String>,
}

impl<T> Pos<T> {
    pub fn new(line: usize, column: usize, file_path: Option<String>, inner: T) -> Self {
        Self { line, column, file_path, inner }
    }

    pub fn set_position(&mut self, position: (usize, usize, &str)) {
        self.line = position.0;
        self.column = position.1;
        self.file_path = Some(position.2.to_owned());
    }

    // This method can be called for trace errors that needs to be recorded for
    // formats such as YAML and JSON only, becuase positional traces are not
    // supported for those formats.
    pub fn to_trace_err<'a>(&self, default: &'a str) -> Option<&'a str> {
        if self.pos_trace_is_supported() {
            return None;
        }

        Some(default)
    }

    pub fn to_pos_trace_err(&self, default: String) -> Option<String> {
        // in case positional tracing error messages are not supported for the source we
        // record the trace with the default value provided
        if self.pos_trace_is_unsupported() {
            return Some(default);
        }

        Some(format!(
            "{} {}#{}",
            self.file_path.as_ref().unwrap().as_str(),
            self.line,
            self.column
        ))
    }

    // if file path exist we know that we read the positional details from
    // that source
    pub fn pos_trace_is_supported(&self) -> bool {
        self.file_path.is_some()
    }

    pub fn pos_trace_is_unsupported(&self) -> bool {
        self.file_path.is_none()
    }
}

impl<T: Default> Default for Pos<T> {
    fn default() -> Self {
        Pos { line: 0, column: 0, file_path: None, inner: T::default() }
    }
}

impl<T: std::fmt::Debug + MergeRight> MergeRight for Pos<T> {
    fn merge_right(mut self, other: Self) -> Self {
        self.line = other.line;
        self.column = other.column;
        self.file_path = other.file_path;
        self.inner = self.inner.merge_right(other.inner);
        self
    }
}

impl<T> Deref for Pos<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for Pos<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
