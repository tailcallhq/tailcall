mod append;
mod cause;
mod error;
mod valid;

pub use cause::*;
pub use error::*;
pub use valid::*;

/// Moral equivalent of TryFrom for validation purposes
pub trait ValidateFrom<T>: Sized {
    type Error;
    fn validate_from(a: T) -> Valid<Self, Self::Error>;
}

/// Moral equivalent of TryInto for validation purposes
pub trait ValidateInto<T> {
    type Error;
    fn validate_into(self) -> Valid<T, Self::Error>;
}

/// A blanket implementation for ValidateInto
impl<S, T: ValidateFrom<S>> ValidateInto<T> for S {
    type Error = T::Error;

    fn validate_into(self) -> Valid<T, Self::Error> {
        T::validate_from(self)
    }
}
