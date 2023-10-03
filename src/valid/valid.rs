use hyper::header::InvalidHeaderName;
use super::ValidationError;
use crate::valid::Cause;

pub type Valid<A, E> = Result<A, ValidationError<E>>;

pub trait ValidExtensions<A, E>:
  Sized + From<Result<A, ValidationError<E>>> + Into<Result<A, ValidationError<E>>>
{
  fn to_valid(self) -> Valid<A, E>;

  fn fail(e: E) -> Valid<A, E> {
    Err((vec![Cause::new(e)]).into())
  }

  fn fail_cause(cause: Vec<Cause<E>>) -> Valid<A, E> {
    Err(cause.into())
  }

  fn succeed(a: A) -> Valid<A, E> {
    Ok(a)
  }

  fn validate_or<A1>(self, other: Result<A1, ValidationError<E>>) -> Valid<A1, E> {
    match self.to_valid() {
      Ok(_) => other,
      Err(e1) => match other {
        Err(e2) => Err(e1.combine(e2)),
        _ => Err(e1),
      },
    }
  }
  fn trace(self, message: &str) -> Valid<A, E> {
    let valid = self.to_valid();
    if let Err(error) = valid {
      return Err(error.trace(message));
    }

    valid
  }
  fn validate_fold<A1>(self, ok: impl Fn(A) -> Valid<A1, E>, err: Valid<A1, E>) -> Valid<A1, E> {
    match self.to_valid() {
      Ok(a) => ok(a),
      Err(e) => Err::<A1, ValidationError<E>>(e).validate_or(err),
    }
  }
}

pub trait ValidConstructor<A, E> {
  fn to_valid(self) -> Valid<A, E>;
}

impl<A, E> ValidConstructor<A, E> for Result<A, E> {
  fn to_valid(self) -> Valid<A, E> {
    self.map_err(|e| ValidationError::new(e))
  }
}

impl<A, E> ValidExtensions<A, E> for Result<A, ValidationError<E>> {
  fn to_valid(self) -> Valid<A, E> {
    self
  }
}

pub trait OptionExtension<A> {
  fn validate_some<E>(self, e: E) -> Valid<A, E>;
}

pub trait VectorExtension<A, E> {
  fn validate_all<B>(self, f: impl Fn(A) -> Valid<B, E>) -> Valid<Vec<B>, E>;
}

impl<A, E, I> VectorExtension<A, E> for I
where
  I: IntoIterator<Item = A>,
{
  fn validate_all<B>(self, f: impl Fn(A) -> Valid<B, E>) -> Valid<Vec<B>, E> {
    let mut values: Vec<B> = Vec::new();
    let mut errors: ValidationError<E> = ValidationError::empty();
    for a in self {
      match f(a) {
        Ok(b) => {
          values.push(b);
        }
        Err(err) => {
          errors = errors.combine(err);
        }
      }
    }

    if errors.is_empty() {
      Ok(values)
    } else {
      Err(errors)
    }
  }
}

impl<A> OptionExtension<A> for Option<A> {
  fn validate_some<E>(self, e: E) -> Valid<A, E> {
    match self {
      Some(a) => Ok(a),
      None => Valid::fail(e),
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::valid::{
    Cause, OptionExtension, Valid, ValidConstructor, ValidExtensions, ValidationError, VectorExtension,
  };

  #[test]
  fn test_ok() {
    let result = Valid::<i32, ()>::succeed(1);
    assert_eq!(result, Ok(1));
  }

  #[test]
  fn test_fail() {
    let result = Valid::<(), i32>::fail(1);
    assert_eq!(result, Valid::fail(1));
  }

  #[test]
  fn test_validate_or_both_ok() {
    let result1 = Valid::<bool, i32>::succeed(true);
    let result2 = Valid::<u8, i32>::succeed(3);

    assert_eq!(result1.validate_or(result2), Ok(3u8));
  }

  #[test]
  fn test_validate_or_first_fail() {
    let result1 = Valid::<bool, i32>::fail(-1);
    let result2 = Valid::<u8, i32>::succeed(3);

    assert_eq!(result1.validate_or(result2), Valid::fail(-1));
  }

  #[test]
  fn test_validate_or_second_fail() {
    let result1 = Valid::<bool, i32>::succeed(true);
    let result2 = Valid::<u8, i32>::fail(-2);

    assert_eq!(result1.validate_or(result2), Valid::fail(-2));
  }

  #[test]
  fn test_validate_all() {
    let input: Vec<i32> = [1, 2, 3].to_vec();
    let result: Valid<Vec<i32>, i32> = input.validate_all(|a| Valid::fail(a * 2));
    assert_eq!(result, Err(vec![Cause::new(2), Cause::new(4), Cause::new(6)].into()));
  }

  #[test]
  fn test_validate_all_ques() {
    let input: Vec<i32> = [1, 2, 3].to_vec();
    let result: Valid<Vec<i32>, i32> = input.validate_all(|a| {
      let a = Valid::fail(a * 2)?;
      Ok(a)
    });
    assert_eq!(result, Err(vec![Cause::new(2), Cause::new(4), Cause::new(6)].into()));
  }

  #[test]
  fn test_ok_ok_cause() {
    let option: Option<i32> = None;
    let result = option.validate_some(1);
    assert_eq!(result, Err(vec![Cause::new(1)].into()));
  }

  #[test]
  fn test_trace() {
    let result = Valid::<(), i32>::fail(1).trace("A").trace("B").trace("C");
    let expected = Err(
      vec![Cause {
        message: 1,
        description: None,
        trace: vec!["C".to_string(), "B".to_string(), "A".to_string()].into(),
      }]
      .into(),
    );
    assert_eq!(result, expected);
  }

  #[test]
  fn test_validate_fold_err() {
    let valid = Valid::<(), i32>::fail(1);
    let result = valid.validate_fold(|_| Valid::<(), i32>::fail(2), Valid::<(), i32>::fail(3));
    assert_eq!(result, Valid::fail_cause(vec![Cause::new(1), Cause::new(3)]));
  }

  #[test]
  fn test_validate_fold_ok() {
    let valid = Valid::<i32, i32>::succeed(1);
    let result = valid.validate_fold(Valid::<i32, i32>::fail, Valid::<i32, i32>::fail(2));
    assert_eq!(result, Valid::fail(1));
  }

  #[test]
  fn test_to_valid() {
    let result = Err::<(), i32>(1).to_valid().unwrap_err();
    assert_eq!(result, ValidationError::new(1));
  }
}
