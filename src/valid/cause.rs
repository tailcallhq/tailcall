use std::collections::VecDeque;

use derive_setters::Setters;
use thiserror::Error;

#[derive(Clone, PartialEq, Debug, Setters, Error)]
pub struct Cause<E> {
  pub message: E,
  #[setters(strip_option)]
  pub description: Option<E>,
  pub trace: VecDeque<String>,
}

impl<E> Cause<E> {
  pub fn new(e: E) -> Self {
    Cause { message: e, description: None, trace: VecDeque::new() }
  }
}
