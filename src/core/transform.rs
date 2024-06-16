use super::valid::{Valid, Validator};

/// A configuration transformer that allows us to perform various
/// transformations on the configuration before it's further processed for
/// blueprint creation.
pub trait Transform {
    type Value;
    type Error;
    fn transform(&self, value: Self::Value) -> Valid<Self::Value, Self::Error>;
}

/// A suite of common operators that are available for all transformers.
pub trait TransformerOps: Sized + Transform {
    fn pipe<B: Transform>(self, other: B) -> Pipe<Self, B>;
}

impl<A> TransformerOps for A
where
    A: Transform,
{
    fn pipe<B: Transform>(self, other: B) -> Pipe<A, B> {
        Pipe(self, other)
    }
}

/// Represents a composition of two transformers.
pub struct Pipe<A, B>(A, B);

impl<A, E, X, Y> Transform for Pipe<X, Y>
where
    X: Transform<Value = A, Error = E>,
    Y: Transform<Value = A, Error = E>,
{
    type Value = A;
    type Error = E;
    fn transform(&self, value: Self::Value) -> Valid<Self::Value, Self::Error> {
        self.0.transform(value).and_then(|v| self.1.transform(v))
    }
}

/// Represents an empty transformer.
pub struct Empty<A, E>(std::marker::PhantomData<(A, E)>);

impl<A, E> Transform for Empty<A, E> {
    type Value = A;
    type Error = E;
    fn transform(&self, value: Self::Value) -> Valid<Self::Value, Self::Error> {
        Valid::succeed(value)
    }
}
