use std::ops::{Deref, DerefMut};

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
