use super::append::Append;
use super::ValidationError;
use crate::valid::Cause;

#[derive(Debug, PartialEq)]
pub struct Valid<A, E>(Result<A, ValidationError<E>>);

pub trait Validator<A, E>: Sized {
    fn map<A1>(self, f: impl FnOnce(A) -> A1) -> Valid<A1, E> {
        Valid(self.to_result().map(f))
    }

    fn foreach(self, mut f: impl FnMut(A)) -> Valid<A, E>
    where
        A: Clone,
    {
        match self.to_result() {
            Ok(a) => {
                f(a.clone());
                Valid::succeed(a)
            }
            Err(e) => Valid(Err(e)),
        }
    }

    fn is_succeed(&self) -> bool;

    fn and<A1>(self, other: Valid<A1, E>) -> Valid<A1, E> {
        self.zip(other).map(|(_, a1)| a1)
    }

    fn zip<A1>(self, other: Valid<A1, E>) -> Valid<(A, A1), E> {
        match self.to_result() {
            Ok(a) => match other.0 {
                Ok(a1) => Valid(Ok((a, a1))),
                Err(e1) => Valid(Err(e1)),
            },
            Err(e1) => match other.0 {
                Ok(_) => Valid(Err(e1)),
                Err(e2) => Valid(Err(e1.combine(e2))),
            },
        }
    }

    fn fuse<A1>(self, other: Valid<A1, E>) -> Fusion<(A, A1), E> {
        Fusion(self.zip(other))
    }

    fn trace(self, message: &str) -> Valid<A, E> {
        let valid = self.to_result();
        if let Err(error) = valid {
            return Valid(Err(error.trace(message)));
        }

        Valid(valid)
    }

    fn fold<A1>(
        self,
        ok: impl FnOnce(A) -> Valid<A1, E>,
        err: impl FnOnce() -> Valid<A1, E>,
    ) -> Valid<A1, E> {
        match self.to_result() {
            Ok(a) => ok(a),
            Err(e) => Valid::<A1, E>(Err(e)).and(err()),
        }
    }

    fn to_result(self) -> Result<A, ValidationError<E>>;

    fn and_then<B>(self, f: impl FnOnce(A) -> Valid<B, E>) -> Valid<B, E> {
        match self.to_result() {
            Ok(a) => f(a),
            Err(e) => Valid(Err(e)),
        }
    }

    fn unit(self) -> Valid<(), E> {
        self.map(|_| ())
    }

    fn some(self) -> Valid<Option<A>, E> {
        self.map(Some)
    }

    fn map_to<B>(self, b: B) -> Valid<B, E> {
        self.map(|_| b)
    }
    fn when(self, f: impl FnOnce() -> bool) -> Valid<(), E> {
        if f() {
            self.unit()
        } else {
            Valid::succeed(())
        }
    }
}

impl<A, E> Valid<A, E> {
    pub fn fail(e: E) -> Valid<A, E> {
        Valid(Err((vec![Cause::new(e)]).into()))
    }

    pub fn fail_with(message: E, description: E) -> Valid<A, E>
    where
        E: std::fmt::Debug,
    {
        Valid(Err(
            (vec![Cause::new(message).description(description)]).into()
        ))
    }

    pub fn from_validation_err(error: ValidationError<E>) -> Self {
        Valid(Err(error))
    }

    pub fn from_vec_cause(error: Vec<Cause<E>>) -> Self {
        Valid(Err(error.into()))
    }

    pub fn succeed(a: A) -> Valid<A, E> {
        Valid(Ok(a))
    }

    pub fn from_iter<B>(
        iter: impl IntoIterator<Item = A>,
        f: impl Fn(A) -> Valid<B, E>,
    ) -> Valid<Vec<B>, E> {
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
            Valid::succeed(values)
        } else {
            Valid::from_validation_err(errors)
        }
    }

    pub fn from_option(option: Option<A>, e: E) -> Valid<A, E> {
        match option {
            Some(a) => Valid::succeed(a),
            None => Valid::fail(e),
        }
    }

    pub fn none() -> Valid<Option<A>, E> {
        Valid::succeed(None)
    }
}

impl<A, E> Validator<A, E> for Valid<A, E> {
    fn to_result(self) -> Result<A, ValidationError<E>> {
        self.0
    }

    fn is_succeed(&self) -> bool {
        self.0.is_ok()
    }
}

pub struct Fusion<A, E>(Valid<A, E>);
impl<A, E> Fusion<A, E> {
    pub fn fuse<A1>(self, other: Valid<A1, E>) -> Fusion<A::Out, E>
    where
        A: Append<A1>,
    {
        Fusion(self.0.zip(other).map(|(a, a1)| a.append(a1)))
    }
}

impl<A, E> Validator<A, E> for Fusion<A, E> {
    fn to_result(self) -> Result<A, ValidationError<E>> {
        self.0.to_result()
    }
    fn is_succeed(&self) -> bool {
        self.0.is_succeed()
    }
}

impl<A, E> From<Result<A, ValidationError<E>>> for Valid<A, E> {
    fn from(value: Result<A, ValidationError<E>>) -> Self {
        match value {
            Ok(a) => Valid::succeed(a),
            Err(e) => Valid::from_validation_err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Cause, ValidationError};
    use crate::valid::valid::Valid;
    use crate::valid::Validator;

    #[test]
    fn test_ok() {
        let result = Valid::<i32, ()>::succeed(1);
        assert_eq!(result, Valid::succeed(1));
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

        assert_eq!(result1.and(result2), Valid::succeed(3u8));
    }

    #[test]
    fn test_validate_or_first_fail() {
        let result1 = Valid::<bool, i32>::fail(-1);
        let result2 = Valid::<u8, i32>::succeed(3);

        assert_eq!(result1.and(result2), Valid::fail(-1));
    }

    #[test]
    fn test_validate_or_second_fail() {
        let result1 = Valid::<bool, i32>::succeed(true);
        let result2 = Valid::<u8, i32>::fail(-2);

        assert_eq!(result1.and(result2), Valid::fail(-2));
    }

    #[test]
    fn test_validate_all() {
        let input: Vec<i32> = [1, 2, 3].to_vec();
        let result: Valid<Vec<i32>, i32> = Valid::from_iter(input, |a| Valid::fail(a * 2));
        assert_eq!(
            result,
            Valid::from_vec_cause(vec![Cause::new(2), Cause::new(4), Cause::new(6)])
        );
    }

    #[test]
    fn test_validate_all_ques() {
        let input: Vec<i32> = [1, 2, 3].to_vec();
        let result: Valid<Vec<i32>, i32> = Valid::from_iter(input, |a| Valid::fail(a * 2));
        assert_eq!(
            result,
            Valid::from_vec_cause(vec![Cause::new(2), Cause::new(4), Cause::new(6)])
        );
    }

    #[test]
    fn test_ok_ok_cause() {
        let option: Option<i32> = None;
        let result = Valid::from_option(option, 1);
        assert_eq!(result, Valid::from_vec_cause(vec![Cause::new(1)]));
    }

    #[test]
    fn test_trace() {
        let result = Valid::<(), i32>::fail(1).trace("A").trace("B").trace("C");
        let expected = Valid::from_vec_cause(vec![Cause {
            message: 1,
            description: None,
            trace: vec!["C".to_string(), "B".to_string(), "A".to_string()].into(),
        }]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_validate_fold_err() {
        let valid = Valid::<(), i32>::fail(1);
        let result = valid.fold(|_| Valid::<(), i32>::fail(2), || Valid::<(), i32>::fail(3));
        assert_eq!(
            result,
            Valid::from_vec_cause(vec![Cause::new(1), Cause::new(3)])
        );
    }

    #[test]
    fn test_validate_fold_ok() {
        let valid = Valid::<i32, i32>::succeed(1);
        let result = valid.fold(Valid::<i32, i32>::fail, || Valid::<i32, i32>::fail(2));
        assert_eq!(result, Valid::fail(1));
    }

    #[test]
    fn test_to_result() {
        let result = Valid::<(), i32>::fail(1).to_result().unwrap_err();
        assert_eq!(result, ValidationError::new(1));
    }

    #[test]
    fn test_validate_both_ok() {
        let result1 = Valid::<bool, i32>::succeed(true);
        let result2 = Valid::<u8, i32>::succeed(3);

        assert_eq!(result1.zip(result2), Valid::succeed((true, 3u8)));
    }
    #[test]
    fn test_validate_both_first_fail() {
        let result1 = Valid::<bool, i32>::fail(-1);
        let result2 = Valid::<u8, i32>::succeed(3);

        assert_eq!(result1.zip(result2), Valid::fail(-1));
    }
    #[test]
    fn test_validate_both_second_fail() {
        let result1 = Valid::<bool, i32>::succeed(true);
        let result2 = Valid::<u8, i32>::fail(-2);

        assert_eq!(result1.zip(result2), Valid::fail(-2));
    }

    #[test]
    fn test_validate_both_both_fail() {
        let result1 = Valid::<bool, i32>::fail(-1);
        let result2 = Valid::<u8, i32>::fail(-2);

        assert_eq!(
            result1.zip(result2),
            Valid::from_vec_cause(vec![Cause::new(-1), Cause::new(-2)])
        );
    }

    #[test]
    fn test_and_then_success() {
        let result = Valid::<i32, i32>::succeed(1).and_then(|a| Valid::succeed(a + 1));
        assert_eq!(result, Valid::succeed(2));
    }

    #[test]
    fn test_and_then_fail() {
        let result = Valid::<i32, i32>::succeed(1).and_then(|a| Valid::<i32, i32>::fail(a + 1));
        assert_eq!(result, Valid::fail(2));
    }

    #[test]
    fn test_foreach_succeed() {
        let mut a = 0;
        let result = Valid::<i32, i32>::succeed(1).foreach(|v| a = v);
        assert_eq!(result, Valid::succeed(1));
        assert_eq!(a, 1);
    }

    #[test]
    fn test_foreach_fail() {
        let mut a = 0;
        let result = Valid::<i32, i32>::fail(1).foreach(|v| a = v);
        assert_eq!(result, Valid::fail(1));
        assert_eq!(a, 0);
    }
}
