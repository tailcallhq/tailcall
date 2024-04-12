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
    convertor: NameConvertor,
}

impl GraphQLType {
    fn new(name: &str, convertor: NameConvertor) -> Self {
        Self { package: None, name: name.to_string(), convertor }
    }

    pub fn from_enum(name: &str) -> Self {
        Self::new(name, NameConvertor::Enum)
    }

    pub fn from_enum_variant(name: &str) -> Self {
        Self::new(name, NameConvertor::EnumVariant)
    }

    pub fn from_object_type(name: &str) -> Self {
        Self::new(name, NameConvertor::ObjectType)
    }

    pub fn from_method(name: &str) -> Self {
        Self::new(name, NameConvertor::Method)
    }

    pub fn from_field(name: &str) -> Self {
        Self::new(name, NameConvertor::Field)
    }

    pub fn id(&self) -> String {
        match self.package {
            Some(package) => format!("{}.{}", package, self.name),
            None => self.name.clone(),
        }
    }
}

/// Used to convert proto type names to GraphQL formatted names.
/// Enum to represent the type of the descriptor
#[derive(Clone)]
enum NameConvertor {
    Enum,
    EnumVariant,
    ObjectType,
    Method,
    Field,
}

impl Display for GraphQLType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let package = self.package.unwrap_or_default();
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
            NameConvertor::EnumVariant => {
                f.write_str(self.name.to_case(Case::UpperSnake).as_str())?
            }
            NameConvertor::Method | NameConvertor::Field => {
                f.write_str(self.name.to_case(Case::Snake).as_str())?
            }
            NameConvertor::Enum | NameConvertor::ObjectType => {
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
