mod ambiguous_type;
mod remove_unused;
mod type_merger;

pub use ambiguous_type::{AmbiguousType, Resolution};
pub use remove_unused::RemoveUnused;
pub use type_merger::TypeMerger;

use super::Config;
use crate::core::valid::{Valid, Validator};

/// A configuration transformer that allows us to perform various
/// transformations on the configuration before it's further processed for
/// blueprint creation.
pub trait Transform {
    fn transform(&self, value: Config) -> Valid<Config, String>;
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

impl<A: Transform, B: Transform> Transform for Pipe<A, B> {
    fn transform(&self, value: Config) -> Valid<Config, String> {
        self.0.transform(value).and_then(|v| self.1.transform(v))
    }
}

/// Represents an empty transformer.
pub struct Empty;

impl Transform for Empty {
    fn transform(&self, value: Config) -> Valid<Config, String> {
        Valid::succeed(value)
    }
}

/// A helper struct that allows us to easily create and compose transformers.
pub struct Transformer;
impl Transformer {
    /// Creates an empty transformer
    pub fn empty() -> Empty {
        Empty
    }

    /// Combine two transformers into a single transformer.
    pub fn pipe<A: Transform, B: Transform>(a: A, b: B) -> Pipe<A, B> {
        Pipe(a, b)
    }
}
