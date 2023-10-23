use std::collections::VecDeque;

use crate::valid::NeoValid;

/// Trait for types that support a "try fold" operation.
///
/// `TryFolding` describes a composable folding operation that can potentially fail.
/// It can optionally consume an input to transform the provided value.
pub trait TryFolding: Sized {
  type Input;
  type Value: Clone;
  type Error;

  /// Try to fold the value with the input.
  ///
  /// # Parameters
  /// - `input`: The input used in the folding operation.
  /// - `value`: The value to be folded.
  ///
  /// # Returns
  /// Returns a `NeoValid` value, which can be either a success with the folded value
  /// or an error.
  fn try_fold(self, input: &Self::Input, value: Self::Value) -> NeoValid<Self::Value, Self::Error>;

  /// Combine two `TryFolding` implementors into a sequential operation.
  ///
  /// This method allows for chaining two `TryFolding` operations, where the result of the first operation
  /// (if successful) will be used as the input for the second operation.
  ///
  /// # Parameters
  /// - `other`: Another `TryFolding` implementor.
  ///
  /// # Returns
  /// Returns a combined `And` structure that represents the sequential folding operation.
  fn and<R: TryFolding>(self, other: R) -> And<Self, R> {
    And { left: self, right: other }
  }
}

/// Represents a custom folding operation.
///
/// `TryFold` is a structure that wraps a closure to perform a custom fold operation
/// with the possibility of failure.
pub struct TryFold<'a, I, O, E>(Box<dyn Fn(&I, O) -> NeoValid<O, E> + 'a>);

impl<'a, I, O, E> TryFold<'a, I, O, E> {
  /// Create a new `TryFold` with a specified folding function.
  ///
  /// # Parameters
  /// - `f`: The folding function.
  ///
  /// # Returns
  /// Returns a new `TryFold` instance.
  pub fn new(f: impl Fn(&I, O) -> NeoValid<O, E> + 'static) -> Self {
    Self(Box::new(f))
  }

  /// Tries to fold all items in the provided list.
  ///
  /// # Parameters
  /// - `list`: A list of items implementing `TryFolding`.
  ///
  /// # Returns
  /// Returns a `Collect` instance that can be used to perform a folding operation
  /// over all the items in the list.

  pub fn try_all<A: TryFolding<Input = I, Value = O, Error = E>>(list: Vec<A>) -> Collect<A> {
    Collect(VecDeque::from(list))
  }
}

impl<I, O: Clone, E> TryFolding for TryFold<'static, I, O, E> {
  type Input = I;
  type Value = O;
  type Error = E;

  fn try_fold(self, input: &Self::Input, value: Self::Value) -> NeoValid<Self::Value, Self::Error> {
    (self.0)(input, value)
  }
}

///
/// Represents a sequential folding operation combining two `TryFolding` implementors.
///
pub struct And<L, R> {
  left: L,
  right: R,
}

impl<L: TryFolding<Input = R::Input, Value = R::Value, Error = R::Error>, R: TryFolding> TryFolding for And<L, R> {
  type Input = L::Input;
  type Value = L::Value;
  type Error = L::Error;

  fn try_fold(self, input: &Self::Input, value: Self::Value) -> NeoValid<Self::Value, Self::Error> {
    match self.left.try_fold(input, value.clone()) {
      Ok(value) => self.right.try_fold(input, value),
      err => err.validate_or(self.right.try_fold(input, value)),
    }
  }
}

/// Represents a folding operation that applies to many items.
///
/// `Collect` is used to perform a fold operation over a list of items,
/// with each item being processed sequentially.
pub struct Collect<A>(VecDeque<A>);
impl<A: TryFolding> TryFolding for Collect<A> {
  type Input = A::Input;
  type Value = A::Value;
  type Error = A::Error;

  fn try_fold(self, input: &Self::Input, value: Self::Value) -> NeoValid<Self::Value, Self::Error> {
    let mut items = self.0;
    let head = items.pop_front();
    let tail = items;

    if let Some(head) = head {
      head
        .and(TryFold::<Self::Input, Self::Value, Self::Error>::try_all(tail.into()))
        .try_fold(input, value)
    } else {
      NeoValid::succeed(value)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::TryFolding;
  use crate::try_fold::TryFold;
  use crate::valid::{NeoValid, NeoValidExtensions, NeoValidationError};

  #[test]
  fn test_combine_ok() {
    let t1 = TryFold::<i32, i32, ()>::new(|a: &i32, b: i32| NeoValid::succeed(a + b));
    let t2 = TryFold::<i32, i32, ()>::new(|a: &i32, b: i32| NeoValid::succeed(a * b));
    let t = t1.and(t2);

    let actual = t.try_fold(&2, 3).unwrap();
    let expected = 10;

    assert_eq!(actual, expected)
  }

  #[test]
  fn test_one_failure() {
    let t1 = TryFold::new(|a: &i32, b: i32| NeoValid::fail(a + b));
    let t2 = TryFold::new(|a: &i32, b: i32| NeoValid::succeed(a * b));
    let t = t1.and(t2);

    let actual = t.try_fold(&2, 3).unwrap_err();
    let expected = NeoValidationError::new(5);

    assert_eq!(actual, expected)
  }

  #[test]
  fn test_both_failure() {
    let t1 = TryFold::new(|a: &i32, b: i32| NeoValid::fail(a + b));
    let t2 = TryFold::new(|a: &i32, b: i32| NeoValid::fail(a * b));
    let t = t1.and(t2);

    let actual = t.try_fold(&2, 3).unwrap_err();
    let expected = NeoValidationError::new(5).combine(NeoValidationError::new(6));

    assert_eq!(actual, expected)
  }

  #[test]
  fn test_1_3_failure_left() {
    let t1 = TryFold::new(|a: &i32, b: i32| NeoValid::fail(a + b)); // 2 + 3
    let t2 = TryFold::new(|a: &i32, b: i32| NeoValid::succeed(a * b)); // 2 * 3
    let t3 = TryFold::new(|a: &i32, b: i32| NeoValid::fail(a * b * 100)); // 2 * 6
    let t = t1.and(t2).and(t3);

    let actual = t.try_fold(&2, 3).unwrap_err();
    let expected = NeoValidationError::new(5).combine(NeoValidationError::new(600));

    assert_eq!(actual, expected)
  }

  #[test]
  fn test_1_3_failure_right() {
    let t1 = TryFold::new(|a: &i32, b: i32| NeoValid::fail(a + b)); // 2 + 3
    let t2 = TryFold::new(|a: &i32, b: i32| NeoValid::succeed(a * b)); // 2 * 3
    let t3 = TryFold::new(|a: &i32, b: i32| NeoValid::fail(a * b * 100)); // 2 * 6
    let t = t1.and(t2.and(t3));

    let actual = t.try_fold(&2, 3).unwrap_err();
    let expected = NeoValidationError::new(5).combine(NeoValidationError::new(1200));

    assert_eq!(actual, expected)
  }

  #[test]
  fn test_2_3_failure() {
    let t1 = TryFold::new(|a: &i32, b: i32| NeoValid::succeed(a + b));
    let t2 = TryFold::new(|a: &i32, b: i32| NeoValid::fail(a * b));
    let t3 = TryFold::new(|a: &i32, b: i32| NeoValid::fail(a * b * 100));
    let t = t1.and(t2.and(t3));

    let actual = t.try_fold(&2, 3).unwrap_err();
    let expected = NeoValidationError::new(10).combine(NeoValidationError::new(1000));

    assert_eq!(actual, expected)
  }

  #[test]
  fn test_try_all() {
    let t1 = TryFold::new(|a: &i32, b: i32| NeoValid::succeed(a + b));
    let t2 = TryFold::new(|a: &i32, b: i32| NeoValid::fail(a * b));
    let t3 = TryFold::new(|a: &i32, b: i32| NeoValid::fail(a * b * 100));
    let t = TryFold::try_all(vec![t1, t2, t3]);

    let actual = t.try_fold(&2, 3).unwrap_err();
    let expected = NeoValidationError::new(10).combine(NeoValidationError::new(1000));

    assert_eq!(actual, expected)
  }

  #[test]
  fn test_try_all_1_3_fail() {
    let t1 = TryFold::new(|a: &i32, b: i32| NeoValid::fail(a + b));
    let t2 = TryFold::new(|a: &i32, b: i32| NeoValid::succeed(a * b));
    let t3 = TryFold::new(|a: &i32, b: i32| NeoValid::fail(a * b * 100));
    let t = TryFold::try_all(vec![t1, t2, t3]);

    let actual = t.try_fold(&2, 3).unwrap_err();
    let expected = NeoValidationError::new(5).combine(NeoValidationError::new(1200));

    assert_eq!(actual, expected)
  }
}
