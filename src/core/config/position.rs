use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::core::merge_right::MergeRight;
use crate::core::valid::SourcePos;

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

    pub fn to_positioned_err(&self) -> Option<SourcePos> {
        if self.pos_trace_is_unsupported() {
            return None;
        }

        Some(SourcePos::from(self))
    }

    fn pos_trace_is_unsupported(&self) -> bool {
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
