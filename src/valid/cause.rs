use std::{collections::VecDeque, fmt::Display};

use derive_setters::Setters;
use thiserror::Error;

#[derive(Clone, PartialEq, Debug, Setters, Error)]
pub struct Cause<E> {
  pub message: E,
  pub trace: VecDeque<String>,
}

impl<E: Display> Display for Cause<E> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "[")?;
    for (i, entry) in self.trace.iter().rev().enumerate() {
      if i > 0 {
        write!(f, ", ")?;
      }
      write!(f, "{}", entry)?;
    }
    write!(f, "] {}", self.message)
  }
}

impl<E> Cause<E> {
  pub fn new(e: E) -> Self {
    Cause { message: e, trace: VecDeque::new() }
  }

  pub fn map<E1, F: Fn(E) -> E1>(self, f: F) -> Cause<E1> {
    Cause { message: f(self.message), trace: self.trace }
  }
}
