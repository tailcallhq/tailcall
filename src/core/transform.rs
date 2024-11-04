use tailcall_valid::{Valid, Validator};

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
    fn pipe<Other: Transform>(self, other: Other) -> Pipe<Self, Other> {
        Pipe(self, other)
    }
    fn generate(&self) -> Valid<Self::Value, Self::Error>
    where
        Self::Value: std::default::Default,
    {
        self.transform(Self::Value::default())
    }

    fn when(self, cond: bool) -> When<Self> {
        When(self, cond)
    }
}

impl<T: Transform> TransformerOps for T {}

pub struct When<A>(A, bool);
impl<A: Transform> Transform for When<A> {
    type Value = A::Value;
    type Error = A::Error;

    fn transform(&self, value: Self::Value) -> Valid<Self::Value, Self::Error> {
        if self.1 {
            self.0.transform(value)
        } else {
            Valid::succeed(value)
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
