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
    fn when<B: Transform>(
        self,
        other: B,
        f: impl FnOnce() -> bool,
    ) -> Pipe<Self, ConditionalTransform<B>>;
    fn generate(&self) -> Valid<Self::Value, Self::Error>
    where
        Self::Value: std::default::Default;
}

impl<A> TransformerOps for A
where
    A: Transform,
{
    fn pipe<B: Transform>(self, other: B) -> Pipe<A, B> {
        Pipe(self, other)
    }

    fn when<B>(self, other: B, f: impl FnOnce() -> bool) -> Pipe<Self, ConditionalTransform<B>>
    where
        B: Transform,
    {
        if f() {
            Pipe(self, ConditionalTransform::Actual(other))
        } else {
            Pipe(
                self,
                ConditionalTransform::NoOp(Default(std::marker::PhantomData)),
            )
        }
    }

    fn generate(&self) -> Valid<Self::Value, Self::Error>
    where
        A::Value: std::default::Default,
    {
        self.transform(A::Value::default())
    }
}

/// helper struct for conditional pipe.
pub enum ConditionalTransform<B: Transform> {
    Actual(B),
    NoOp(Default<B::Value, B::Error>),
}

impl<B> Transform for ConditionalTransform<B>
where
    B: Transform,
{
    type Value = B::Value;
    type Error = B::Error;

    fn transform(&self, input: Self::Value) -> Valid<Self::Value, Self::Error> {
        match self {
            ConditionalTransform::Actual(b) => b.transform(input),
            ConditionalTransform::NoOp(no_op) => no_op.transform(input),
        }
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
pub struct Default<A, E>(std::marker::PhantomData<(A, E)>);

impl<A, E> Transform for Default<A, E> {
    type Value = A;
    type Error = E;
    fn transform(&self, value: Self::Value) -> Valid<Self::Value, Self::Error> {
        Valid::succeed(value)
    }
}

pub fn default<A, E>() -> Default<A, E> {
    Default(std::marker::PhantomData)
}
