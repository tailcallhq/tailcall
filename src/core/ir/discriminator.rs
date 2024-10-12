mod keyed_discriminator;
mod probability_discriminator;
mod type_field_discriminator;

use anyhow::{bail, Result};
use probability_discriminator::ProbabilityDiscriminator;
use async_graphql::Value;

use crate::core::{
    config::Type,
    json::{JsonLike, JsonObjectLike},
    valid::{Valid, Validator},
};

/// Resolver for type member of a union or interface.
#[derive(Debug, Clone)]
pub enum Discriminator {
    Probability(ProbabilityDiscriminator),
}

impl Discriminator {
    pub fn new(
        union_name: &str,
        union_types: &[(&str, &Type)],
        mode: DiscriminatorMode,
    ) -> Valid<Self, String> {
        match mode {
            DiscriminatorMode::Probability => {
                ProbabilityDiscriminator::new(union_name, union_types).map(|d| Self::Probability(d))
            }
        }
    }

    pub fn resolve_type(&self, value: &Value) -> Result<&str> {
        match self {
            Discriminator::Probability(probability_discriminator) => probability_discriminator.resolve_type(value),
        }
    }
}

pub enum DiscriminatorMode {
    Probability,
}

pub trait TypedValue<'a> {
    type Error;

    fn get_type_name(&'a self) -> Option<&'a str>;
    fn set_type_name(&'a mut self, type_name: String) -> Result<(), Self::Error>;
}

const TYPENAME_FIELD: &str = "__typename";

impl<'json, T> TypedValue<'json> for T
where
    T: JsonLike<'json>,
{
    type Error = anyhow::Error;

    fn get_type_name(&'json self) -> Option<&'json str> {
        self.as_object()
            .and_then(|obj| obj.get_key(TYPENAME_FIELD))
            .and_then(|val| val.as_str())
    }

    fn set_type_name(&'json mut self, type_name: String) -> Result<(), Self::Error> {
        if let Some(obj) = self.as_object_mut() {
            obj.insert_key(TYPENAME_FIELD, T::string(type_name.into()));

            Ok(())
        } else {
            bail!("Expected object")
        }
    }
}
