use super::ValidationError;
use crate::valid::Cause;

pub type Valid<A, E> = Result<A, super::error::ValidationError<E>>;

pub trait ValidExtensions<A, E>:
  Sized + From<Result<A, super::error::ValidationError<E>>> + Into<Result<A, super::error::ValidationError<E>>>
{
  fn fail(e: E) -> Self;
  fn succeed(a: A) -> Self;
  fn validate_or(self, other: Self) -> Self;
  fn trace(self, message: &str) -> Self;
}

pub trait ValidConstructor<A, E> {
  fn validate(self) -> Valid<A, E>;
}

impl<A, E> ValidConstructor<A, E> for Result<A, E> {
  fn validate(self) -> Valid<A, E> {
    self.map_err(|e| ValidationError::new(e))
  }
}

impl<A, E> ValidExtensions<A, E> for Result<A, super::error::ValidationError<E>> {
  fn fail(e: E) -> Self {
    Err((vec![Cause::new(e)]).into())
  }

  fn succeed(a: A) -> Self {
    Ok(a)
  }

  fn validate_or(self, other: Self) -> Self {
    match self {
      Ok(_) => other,
      Err(e1) => match other {
        Err(e2) => Err(e1.combine(e2)),
        ok => ok,
      },
    }
  }

  fn trace(self, message: &str) -> Self {
    if let Err(error) = self {
      return Err(error.trace(message));
    }

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
  use crate::valid::{Cause, OptionExtension, Valid, ValidExtensions, VectorExtension};

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
    assert_eq!(
      result,
      Err(vec![Cause { message: 1, trace: vec!["C".to_string(), "B".to_string(), "A".to_string()].into() }].into())
    );
  }
}
