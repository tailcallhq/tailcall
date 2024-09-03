use std::future::Future;

use super::valid::{Valid, Validator};

/// A configuration transformer that allows us to perform various
/// transformations on the configuration before it's further processed for
/// blueprint creation.
pub trait AsyncTransform {
    type Value: Send;
    type Error;

    fn transform(
        &self,
        value: Self::Value,
    ) -> impl Future<Output = Valid<Self::Value, Self::Error>> + Send;
}

/// A suite of common operators that are available for all transformers.
pub trait AsyncTransformerOps: AsyncTransform + Send {
    fn pipe<Other: AsyncTransform<Value = Self::Value, Error = Self::Error>>(
        self,
        other: Other,
    ) -> Pipe<Self, Other>
    where
        Self: Sized,
    {
        Pipe(self, other)
    }

    fn generate(&self) -> impl Future<Output = Valid<Self::Value, Self::Error>> + Send
    where
        Self: Send + Sync,
        Self::Value: std::default::Default,
    {
        async move { self.transform(Self::Value::default()).await }
    }

    fn when(self, cond: bool) -> When<Self>
    where
        Self: Sized,
    {
        When(self, cond)
    }
}

impl<T: AsyncTransform + Sync + Send> AsyncTransformerOps for T {}

pub struct When<A>(A, bool);
impl<A: AsyncTransform + Sync + Send> AsyncTransform for When<A> {
    type Value = A::Value;
    type Error = A::Error;

    async fn transform(
        &self,
        value: Self::Value,
    ) -> Valid<Self::Value, Self::Error> {
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
    X: AsyncTransform<Value = A, Error = E> + Send + Sync,
    Y: AsyncTransform<Value = A, Error = E> + Send + Sync,
    A: Sync + Send,
    E: Sync + Send,
{
    type Value = A;
    type Error = E;

    async fn transform(
        &self,
        value: Self::Value,
    ) -> Valid<Self::Value, Self::Error> {
        let result = self.0.transform(value).await;
        match result.to_result() {
            Ok(result) => self.1.transform(result).await,
            Err(err) => Valid::from_validation_err(err),
        }
    }
}

/// Represents an empty transformer.
pub struct Default<A, E>(std::marker::PhantomData<(A, E)>);

impl<A: Send + Sync, E: Send + Sync> AsyncTransform for Default<A, E> {
    type Value = A;
    type Error = E;
    async fn transform(
        &self,
        value: Self::Value,
    ) -> Valid<Self::Value, Self::Error> { Valid::succeed(value) }
}

// pub fn default<A, E>() -> Default<A, E> {
//     Default(std::marker::PhantomData)
// }
