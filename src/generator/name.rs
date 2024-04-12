use std::fmt::Display;

use convert_case::{Case, Casing};
use derive_setters::Setters;
pub(super) static DEFAULT_SEPARATOR: &str = "__";
pub(super) static DEFAULT_PACKAGE_SEPARATOR: &str = "_";

/// A struct to represent the name of a GraphQL type.
#[derive(Setters)]
pub struct GraphQLType {
    #[setters(strip_option)]
    package: Option<String>,
    name: String,
    convertor: Entity,
}

impl GraphQLType {
    fn new(name: &str, convertor: Entity) -> Self {
        Self { package: None, name: name.to_string(), convertor }
    }

    pub fn from_enum(name: &str) -> Self {
        Self::new(name, Entity::Enum)
    }

    pub fn from_enum_variant(name: &str) -> Self {
        Self::new(name, Entity::EnumVariant)
    }

    pub fn from_object_type(name: &str) -> Self {
        Self::new(name, Entity::ObjectType)
    }

    pub fn from_method(name: &str) -> Self {
        Self::new(name, Entity::Method)
    }

    pub fn from_field(name: &str) -> Self {
        Self::new(name, Entity::Field)
    }

    pub fn id(&self) -> String {
        match &self.package {
            Some(package) => format!("{}.{}", package, self.name),
            None => self.name.clone(),
        }
    }
}

// FIXME: make it private
/// Used to convert proto type names to GraphQL formatted names.
/// Enum to represent the type of the descriptor
#[derive(Clone)]
pub enum Entity {
    Enum,
    EnumVariant,
    ObjectType,
    Method,
    Field,
}

impl Display for GraphQLType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let package = self.package.clone().unwrap_or_default();
        let package = package
            .split('.')
            .map(|word| {
                word.split('_')
                    .map(|name| name.to_case(Case::Pascal))
                    .reduce(|acc, x| acc + "_" + &x)
                    .unwrap_or(word.to_string())
            })
            .reduce(|acc, x| acc + DEFAULT_PACKAGE_SEPARATOR + &x)
            .unwrap_or(package.to_string());

        match self.convertor {
            Entity::EnumVariant => f.write_str(self.name.to_case(Case::UpperSnake).as_str())?,
            Entity::Method | Entity::Field => {
                f.write_str(self.name.to_case(Case::Snake).as_str())?
            }
            Entity::Enum | Entity::ObjectType => {
                if !package.is_empty() {
                    f.write_str(package.as_str())?;
                    f.write_str(DEFAULT_SEPARATOR)?;
                };
                f.write_str(self.name.to_case(Case::Pascal).as_str())?
            }
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_new() {
        let g = GraphQLType::from_enum("test");
        let actual = g.to_string();
        let expected = "Test";
        assert_eq!(actual, expected);
    }
}
