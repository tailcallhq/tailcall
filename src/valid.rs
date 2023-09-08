use crate::cause::Cause;

pub type Valid<A, E> = Result<A, Vec<Cause<E>>>;

pub trait ValidExtensions<A, E>: Sized + From<Result<A, Vec<Cause<E>>>> + Into<Result<A, Vec<Cause<E>>>> {
    fn fail(e: E) -> Valid<A, E> {
        Err(vec![Cause::new(e)])
    }

    fn succeed(a: A) -> Valid<A, E> {
        Ok(a)
    }

    fn validate_or(self, other: Self) -> Self {
        match self.into().as_mut() {
            Ok(_) => other,
            Err(e1) => match other.into() {
                Err(mut e2) => {
                    e2.append(e1);
                    (Err(e2)).into()
                }
                ok => (ok).into(),
            },
        }
    }

    fn trace(self, message: &str) -> Self {
        match self.into() {
            Ok(a) => Ok(a).into(),
            Err(mut causes) => {
                for cause in &mut causes {
                    cause.trace.insert(0, message.to_owned());
                }
                (Err(causes)).into()
            }
        }
    }
}

impl<A, E> ValidExtensions<A, E> for Result<A, Vec<Cause<E>>> {}

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
        let mut errors: Vec<Cause<E>> = Vec::new();
        for a in self {
            match f(a) {
                Ok(b) => values.push(b),
                Err(err) => errors.extend(err),
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
    use crate::{
        cause::Cause,
        valid::{OptionExtension, Valid, ValidExtensions, VectorExtension},
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
    fn test_validate_all() {
        let input: Vec<i32> = [1, 2, 3].to_vec();
        let result: Valid<Vec<i32>, i32> = input.validate_all(|a| Valid::fail(a * 2));
        assert_eq!(result, Err(vec![Cause::new(2), Cause::new(4), Cause::new(6)]));
    }

    #[test]
    fn test_validate_all_ques() {
        let input: Vec<i32> = [1, 2, 3].to_vec();
        let result: Valid<Vec<i32>, i32> = input.validate_all(|a| {
            let a = Valid::fail(a * 2)?;
            Ok(a)
        });
        assert_eq!(result, Err(vec![Cause::new(2), Cause::new(4), Cause::new(6)]));
    }

    #[test]
    fn test_ok_ok_cause() {
        let option: Option<i32> = None;
        let result = option.validate_some(1);
        assert_eq!(result, Err(vec![Cause::new(1)]));
    }

    #[test]
    fn test_trace() {
        let result = Valid::<(), i32>::fail(1).trace("A").trace("B").trace("C");
        assert_eq!(
            result,
            Err(vec![Cause {
                message: 1,
                trace: vec!["C".to_string(), "B".to_string(), "A".to_string()].into()
            }])
        );
    }
}
