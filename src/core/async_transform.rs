use super::valid::{Valid, Validator};

/// A configuration transformer that allows us to perform various
/// transformations on the configuration before it's further processed for
/// blueprint creation.
pub trait AsyncTransform {
    type Value;
    type Error;
    async fn transform(&self, value: Self::Value) -> Valid<Self::Value, Self::Error>;
}

/// A suite of common operators that are available for all transformers.
pub trait TransformerOps: Sized + AsyncTransform {
    fn pipe<Other: AsyncTransform>(self, other: Other) -> Pipe<Self, Other> {
        Pipe(self, other)
    }
    async fn generate(&self) -> Valid<Self::Value, Self::Error>
    where
        Self::Value: std::default::Default,
    {
        self.transform(Self::Value::default()).await
    }

    fn when(self, cond: bool) -> When<Self> {
        When(self, cond)
    }
}

impl<T: AsyncTransform> TransformerOps for T {}

pub struct When<A>(A, bool);
impl<A: AsyncTransform> AsyncTransform for When<A> {
    type Value = A::Value;
    type Error = A::Error;

    async fn transform(&self, value: Self::Value) -> Valid<Self::Value, Self::Error> {
        if self.1 {
            self.0.transform(value).await
        } else {
            Valid::succeed(value)
        }
    }
}

/// Represents a composition of two transformers.
pub struct Pipe<A, B>(A, B);

impl<A, E, X, Y> AsyncTransform for Pipe<X, Y>
where
    X: AsyncTransform<Value = A, Error = E>,
    Y: AsyncTransform<Value = A, Error = E>,
{
    type Value = A;
    type Error = E;
    async fn transform(&self, value: Self::Value) -> Valid<Self::Value, Self::Error> {
        let result = self.0.transform(value).await;
        match result.to_result() {
            Ok(result) => self.1.transform(result).await,
            Err(err) => Valid::from_validation_err(err),
        }
    }
}

/// Represents an empty transformer.
pub struct Default<A, E>(std::marker::PhantomData<(A, E)>);

impl<A, E> AsyncTransform for Default<A, E> {
    type Value = A;
    type Error = E;
    async fn transform(&self, value: Self::Value) -> Valid<Self::Value, Self::Error> {
        Valid::succeed(value)
    }
}

pub fn default<A, E>() -> Default<A, E> {
    Default(std::marker::PhantomData)
}
