use std::cmp::max;

use super::error::Error;

///
/// Represents the result of the auth verification process. It can either
/// succeed or fail with an Error.
#[derive(Clone, PartialEq, Debug)]
pub enum Verification {
    Succeed,
    Fail(Error),
}

impl Verification {
    pub fn fail(error: Error) -> Self {
        Verification::Fail(error)
    }

    pub fn succeed() -> Self {
        Verification::Succeed
    }

    pub fn fold(self, on_success: Self, on_error: impl Fn(Error) -> Self) -> Self {
        match self {
            Verification::Succeed => on_success,
            Verification::Fail(err) => on_error(err),
        }
    }

    pub fn or(&self, other: Self) -> Self {
        match self {
            Verification::Succeed => Verification::Succeed,
            Verification::Fail(this) => other.fold(Verification::succeed(), |that| {
                Verification::Fail(max(this.clone(), that))
            }),
        }
    }

    pub fn and(self, other: Self) -> Self {
        match self {
            Verification::Succeed => other,
            Verification::Fail(_) => self,
        }
    }

    pub fn from_result<A, E>(
        result: Result<A, E>,
        on_success: impl FnOnce(A) -> Verification,
        on_err: impl FnOnce(E) -> Verification,
    ) -> Self {
        match result {
            Ok(data) => on_success(data),
            Err(err) => on_err(err),
        }
    }

    pub fn to_result(&self) -> Result<(), Error> {
        match self {
            Verification::Succeed => Ok(()),
            Verification::Fail(err) => Err(err.clone()),
        }
    }
}
