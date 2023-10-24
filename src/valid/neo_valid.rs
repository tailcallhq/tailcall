use super::ValidationError;
use crate::valid::Cause;

#[derive(Debug, PartialEq)]
pub struct NeoValid<A, E>(pub Result<A, ValidationError<E>>);

impl<A, E> NeoValid<A, E> {
  pub fn fail(e: E) -> NeoValid<A, E> {
    NeoValid(Err((vec![Cause::new(e)]).into()))
  }

  pub fn from_validation_err(error: ValidationError<E>) -> Self {
    NeoValid(Err(error))
  }

  pub fn from_vec_cause(error: Vec<Cause<E>>) -> Self {
    NeoValid(Err(error.into()))
  }

  pub fn map<A1>(self, f: impl FnOnce(A) -> A1) -> NeoValid<A1, E> {
    NeoValid(self.0.map(f))
  }

  pub fn succeed(a: A) -> NeoValid<A, E> {
    NeoValid(Ok(a))
  }

  pub fn and<A1>(self, other: NeoValid<A1, E>) -> NeoValid<A1, E> {
    match self.0 {
      Ok(_) => other,
      Err(e1) => match other.0 {
        Err(e2) => NeoValid(Err(e1.combine(e2))),
        _ => NeoValid(Err(e1)),
      },
    }
  }

  pub fn zip<A1>(self, other: NeoValid<A1, E>) -> NeoValid<(A, A1), E> {
    match self.0 {
      Ok(a) => match other.0 {
        Ok(a1) => NeoValid(Ok((a, a1))),
        Err(e1) => NeoValid(Err(e1)),
      },
      Err(e1) => match other.0 {
        Ok(_) => NeoValid(Err(e1)),
        Err(e2) => NeoValid(Err(e1.combine(e2))),
      },
    }
  }

  pub fn trace(self, message: &str) -> NeoValid<A, E> {
    let valid = self.0;
    if let Err(error) = valid {
      return NeoValid(Err(error.trace(message)));
    }

    NeoValid(valid)
  }

  pub fn fold<A1>(self, ok: impl Fn(A) -> NeoValid<A1, E>, err: NeoValid<A1, E>) -> NeoValid<A1, E> {
    match self.0 {
      Ok(a) => ok(a),
      Err(e) => NeoValid::<A1, E>(Err(e)).and(err),
    }
  }

  pub fn from_iter<B>(iter: impl IntoIterator<Item = A>, f: impl Fn(A) -> NeoValid<B, E>) -> NeoValid<Vec<B>, E> {
    let mut values: Vec<B> = Vec::new();
    let mut errors: ValidationError<E> = ValidationError::empty();
    for a in iter.into_iter() {
      match f(a).to_result() {
        Ok(b) => {
          values.push(b);
        }
        Err(err) => {
          errors = errors.combine(err);
        }
      }
    }

    if errors.is_empty() {
      NeoValid::succeed(values)
    } else {
      NeoValid::from_validation_err(errors)
    }
  }

  pub fn from_option(option: Option<A>, e: E) -> NeoValid<A, E> {
    match option {
      Some(a) => NeoValid::succeed(a),
      None => NeoValid::fail(e),
    }
  }

  pub fn to_result(self) -> Result<A, ValidationError<E>> {
    self.0
  }

  pub fn and_then<B>(self, f: impl FnOnce(A) -> NeoValid<B, E>) -> NeoValid<B, E> {
    match self.0 {
      Ok(a) => f(a),
      Err(e) => NeoValid(Err(e)),
    }
  }

  pub fn unit(self) -> NeoValid<(), E> {
    self.map(|_| ())
  }

  pub fn some(self) -> NeoValid<Option<A>, E> {
    self.map(Some)
  }

  pub fn none() -> NeoValid<Option<A>, E> {
    NeoValid::succeed(None)
  }
}

impl<A, E> From<super::Valid<A, E>> for NeoValid<A, E> {
  fn from(value: super::Valid<A, E>) -> Self {
    match value {
      Ok(a) => NeoValid::succeed(a),
      Err(e) => NeoValid::from_validation_err(e),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::{Cause, ValidationError};
  use crate::valid::neo_valid::NeoValid;

  #[test]
  fn test_ok() {
    let result = NeoValid::<i32, ()>::succeed(1);
    assert_eq!(result, NeoValid::succeed(1));
  }

  #[test]
  fn test_fail() {
    let result = NeoValid::<(), i32>::fail(1);
    assert_eq!(result, NeoValid::fail(1));
  }

  #[test]
  fn test_validate_or_both_ok() {
    let result1 = NeoValid::<bool, i32>::succeed(true);
    let result2 = NeoValid::<u8, i32>::succeed(3);

    assert_eq!(result1.and(result2), NeoValid::succeed(3u8));
  }

  #[test]
  fn test_validate_or_first_fail() {
    let result1 = NeoValid::<bool, i32>::fail(-1);
    let result2 = NeoValid::<u8, i32>::succeed(3);

    assert_eq!(result1.and(result2), NeoValid::fail(-1));
  }

  #[test]
  fn test_validate_or_second_fail() {
    let result1 = NeoValid::<bool, i32>::succeed(true);
    let result2 = NeoValid::<u8, i32>::fail(-2);

    assert_eq!(result1.and(result2), NeoValid::fail(-2));
  }

  #[test]
  fn test_validate_all() {
    let input: Vec<i32> = [1, 2, 3].to_vec();
    let result: NeoValid<Vec<i32>, i32> = NeoValid::from_iter(input, |a| NeoValid::fail(a * 2));
    assert_eq!(
      result,
      NeoValid::from_vec_cause(vec![Cause::new(2), Cause::new(4), Cause::new(6)])
    );
  }

  #[test]
  fn test_validate_all_ques() {
    let input: Vec<i32> = [1, 2, 3].to_vec();
    let result: NeoValid<Vec<i32>, i32> = NeoValid::from_iter(input, |a| NeoValid::fail(a * 2));
    assert_eq!(
      result,
      NeoValid::from_vec_cause(vec![Cause::new(2), Cause::new(4), Cause::new(6)])
    );
  }

  #[test]
  fn test_ok_ok_cause() {
    let option: Option<i32> = None;
    let result = NeoValid::from_option(option, 1);
    assert_eq!(result, NeoValid::from_vec_cause(vec![Cause::new(1)]));
  }

  #[test]
  fn test_trace() {
    let result = NeoValid::<(), i32>::fail(1).trace("A").trace("B").trace("C");
    let expected = NeoValid::from_vec_cause(vec![Cause {
      message: 1,
      description: None,
      trace: vec!["C".to_string(), "B".to_string(), "A".to_string()].into(),
    }]);
    assert_eq!(result, expected);
  }

  #[test]
  fn test_validate_fold_err() {
    let valid = NeoValid::<(), i32>::fail(1);
    let result = valid.fold(|_| NeoValid::<(), i32>::fail(2), NeoValid::<(), i32>::fail(3));
    assert_eq!(result, NeoValid::from_vec_cause(vec![Cause::new(1), Cause::new(3)]));
  }

  #[test]
  fn test_validate_fold_ok() {
    let valid = NeoValid::<i32, i32>::succeed(1);
    let result = valid.fold(NeoValid::<i32, i32>::fail, NeoValid::<i32, i32>::fail(2));
    assert_eq!(result, NeoValid::fail(1));
  }

  #[test]
  fn test_to_result() {
    let result = NeoValid::<(), i32>::fail(1).to_result().unwrap_err();
    assert_eq!(result, ValidationError::new(1));
  }

  #[test]
  fn test_validate_both_ok() {
    let result1 = NeoValid::<bool, i32>::succeed(true);
    let result2 = NeoValid::<u8, i32>::succeed(3);

    assert_eq!(result1.zip(result2), NeoValid::succeed((true, 3u8)));
  }
  #[test]
  fn test_validate_both_first_fail() {
    let result1 = NeoValid::<bool, i32>::fail(-1);
    let result2 = NeoValid::<u8, i32>::succeed(3);

    assert_eq!(result1.zip(result2), NeoValid::fail(-1));
  }
  #[test]
  fn test_validate_both_second_fail() {
    let result1 = NeoValid::<bool, i32>::succeed(true);
    let result2 = NeoValid::<u8, i32>::fail(-2);

    assert_eq!(result1.zip(result2), NeoValid::fail(-2));
  }

  #[test]
  fn test_validate_both_both_fail() {
    let result1 = NeoValid::<bool, i32>::fail(-1);
    let result2 = NeoValid::<u8, i32>::fail(-2);

    assert_eq!(
      result1.zip(result2),
      NeoValid::from_vec_cause(vec![Cause::new(-1), Cause::new(-2)])
    );
  }

  #[test]
  fn test_and_then_success() {
    let result = NeoValid::<i32, i32>::succeed(1).and_then(|a| NeoValid::succeed(a + 1));
    assert_eq!(result, NeoValid::succeed(2));
  }

  #[test]
  fn test_and_then_fail() {
    let result = NeoValid::<i32, i32>::succeed(1).and_then(|a| NeoValid::<i32, i32>::fail(a + 1));
    assert_eq!(result, NeoValid::fail(2));
  }
}
