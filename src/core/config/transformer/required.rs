use crate::core::config::Config;
use crate::core::transform::{self, Transform, TransformerOps};

/// Defines a set of required transformers that must be applied to every
/// configuration to make it work with GraphQL.
#[derive(Debug, PartialEq, Default)]
pub struct Required;

impl Transform for Required {
    type Value = Config;
    type Error = String;

    fn transform(
        &self,
        config: Self::Value,
    ) -> crate::core::valid::Valid<Self::Value, Self::Error> {
        transform::default()
            .pipe(super::NestedUnions)
            .pipe(super::UnionInputType)
            .pipe(super::AmbiguousType::default())
            .transform(config)
    }
}
