use std::fmt::Display;

use convert_case::{Case, Casing};
use derive_setters::Setters;
use strum_macros::Display;
pub(super) static DEFAULT_SEPARATOR: &str = "_";
pub(super) static DEFAULT_PACKAGE_SEPARATOR: &str = "_";

/// A struct to represent the name of a GraphQL type.
#[derive(Debug)]
pub struct GraphQLType {
    package: Option<String>,
    name: String,
    convertor: Entity,
}

impl GraphQLType {
    fn parse(name: &str, convertor: Entity) -> Self {
        let mut package = None;
        let mut name = name.to_string();
        if let Some(index) = name.rfind('.') {
            package = Some(name[..index].to_string());
            name = name[index + 1..].to_string();
        }
        Self { package, name: name.to_string(), convertor }
    }

    pub fn parse_enum(name: &str) -> Self {
        Self::parse(name, Entity::Enum)
    }

    pub fn parse_enum_variant(name: &str) -> Self {
        Self::parse(name, Entity::EnumVariant)
    }

    pub fn parse_object_type(name: &str) -> Self {
        Self::parse(name, Entity::ObjectType)
    }

    pub fn parse_method(name: &str) -> Self {
        Self::parse(name, Entity::Method)
    }

    pub fn parse_field(name: &str) -> Self {
        Self::parse(name, Entity::Field)
    }

    pub fn id(&self) -> String {
        match &self.package {
            Some(package) => format!("{}.{}", package, self.name),
            None => self.name.clone(),
        }
    }

    pub fn package(self, package: &str) -> Self {
        if package.is_empty() {
            self
        } else {
            Self::parse(&format!("{}.{}", package, self.name), self.convertor)
        }
    }
}

// FIXME: make it private
/// Used to convert proto type names to GraphQL formatted names.
/// Enum to represent the type of the descriptor
#[derive(Clone, Debug)]
pub enum Entity {
    Enum,
    EnumVariant,
    ObjectType,
    Method,
    Field,
}

impl Display for GraphQLType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.convertor {
            Entity::EnumVariant => f.write_str(self.name.to_case(Case::ScreamingSnake).as_str())?,
            Entity::Field => f.write_str(self.name.to_case(Case::Snake).as_str())?,
            Entity::Method => {
                if let Some(package) = &self.package {
                    f.write_str(package.replace(".", "_").to_case(Case::Snake).as_str())?;
                    f.write_str(DEFAULT_SEPARATOR)?;
                };
                f.write_str(self.name.to_case(Case::Snake).as_str())?
            }
            Entity::Enum | Entity::ObjectType => {
                if let Some(package) = &self.package {
                    f.write_str(package.replace(".", "_").to_case(Case::UpperSnake).as_str())?;
                    f.write_str(DEFAULT_SEPARATOR)?;
                };
                f.write_str(self.name.to_case(Case::UpperCamel).as_str())?
            }
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    type TestParams = ((Entity, Option<&'static str>, &'static str), &'static str);

    #[test]
    fn test_from_enum() {
        let input: Vec<TestParams> = vec![
            // Enums
            ((Entity::Enum, None, "foo"), "Foo"),
            ((Entity::Enum, None, "a.b.c.foo"), "A_B_C_Foo"),
            ((Entity::Enum, Some("a.b.c"), "foo"), "A_B_C_Foo"),
            ((Entity::Enum, Some("a.b.c"), "d.e.f.foo"), "A_B_C_Foo"),
            ((Entity::Enum, Some(""), "a.b.c.foo"), "A_B_C_Foo"),
            ((Entity::Enum, None, "a_b_c_foo"), "ABCFoo"),
        ];

        assert_type_names(input);
    }

    #[test]
    fn test_from_enum_variant() {
        let input: Vec<TestParams> = vec![
            // Enum variants
            ((Entity::EnumVariant, None, "foo"), "FOO"),
            ((Entity::EnumVariant, None, "a.b.c.foo"), "FOO"),
            ((Entity::EnumVariant, Some("a.b.c"), "foo"), "FOO"),
            ((Entity::EnumVariant, Some("a.b"), "d.e.foo"), "FOO"),
            ((Entity::EnumVariant, Some(""), "a.b.c.foo"), "FOO"),
            ((Entity::EnumVariant, None, "a_b_c_foo"), "A_B_C_FOO"),
        ];

        assert_type_names(input);
    }

    #[test]
    fn test_from_object_type() {
        let input: Vec<TestParams> = vec![
            // Object types
            ((Entity::ObjectType, None, "foo"), "Foo"),
            ((Entity::ObjectType, None, "a.b.c.foo"), "A_B_C_Foo"),
            ((Entity::ObjectType, Some("a.b.c"), "foo"), "A_B_C_Foo"),
            ((Entity::ObjectType, Some("a.b"), "d.e.foo"), "A_B_Foo"),
            ((Entity::ObjectType, Some(""), "a.b.c.foo"), "A_B_C_Foo"),
            ((Entity::ObjectType, None, "a_b_c_foo"), "ABCFoo"),
        ];

        assert_type_names(input);
    }

    #[test]
    fn test_from_method() {
        let input: Vec<TestParams> = vec![
            // Methods
            ((Entity::Method, None, "foo"), "foo"),
            ((Entity::Method, None, "a.b.c.foo"), "a_b_c_foo"),
            ((Entity::Method, Some("a.b.c"), "foo"), "a_b_c_foo"),
            ((Entity::Method, Some("a.b"), "d.e.foo"), "a_b_foo"),
            ((Entity::Method, Some(""), "a.b.c.foo"), "a_b_c_foo"),
            ((Entity::Method, None, "a_b_c_foo"), "a_b_c_foo"),
        ];

        assert_type_names(input);
    }

    #[test]
    fn test_from_field() {
        let input: Vec<TestParams> = vec![
            // Fields
            ((Entity::Field, None, "foo"), "foo"),
            ((Entity::Field, None, "a.b.c.foo"), "foo"),
            ((Entity::Field, Some("a.b.c"), "foo"), "foo"),
            ((Entity::Field, Some("a.b"), "d.e.foo"), "foo"),
            ((Entity::Field, Some(""), "a.b.c.foo"), "foo"),
            ((Entity::Field, None, "a_b_c_foo"), "a_b_c_foo"),
        ];

        assert_type_names(input);
    }

    fn assert_type_names(input: Vec<((Entity, Option<&str>, &str), &str)>) {
        for ((entity, package, name), expected) in input {
            let mut g = GraphQLType::parse(name, entity);
            if let Some(package) = package {
                g = g.package(package);
            }
            let actual = g.to_string();
            assert_eq!(actual, expected, "Given: {:?}", g);
        }
    }
}
