use serde::{Deserialize, Serialize};

use crate::core::merge_right::MergeRight;

#[derive(
    Serialize, Deserialize, PartialEq, Eq, Clone, Debug, schemars::JsonSchema, Ord, PartialOrd,
)]
#[serde(transparent)]
pub struct Pos<T> {
    #[serde(skip_serializing, skip_deserializing)]
    pub line: usize,
    #[serde(skip_serializing, skip_deserializing)]
    pub column: usize,

    pub inner: T,
}

impl<T> Pos<T> {
    pub fn new(line: usize, column: usize, inner: T) -> Self {
        Self { line, column, inner }
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    pub fn set_position(&mut self, line: usize, column: usize) {
        self.line = line;
        self.column = column;
    }
}

impl<T: Default> Default for Pos<T> {
    fn default() -> Self {
        Pos { line: 0, column: 0, inner: T::default() }
    }
}

impl<T: std::fmt::Debug + MergeRight> MergeRight for Pos<T> {
    fn merge_right(mut self, other: Self) -> Self {
        self.line = other.line;
        self.column = other.column;
        self.inner = self.inner.merge_right(other.inner);
        self
    }
}

