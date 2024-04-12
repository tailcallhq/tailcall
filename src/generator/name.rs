use std::fmt::Display;

use convert_case::{Case, Casing};
pub(super) static DEFAULT_SEPARATOR: &str = "_";

/// A struct to represent the name of a GraphQL type.
#[derive(Debug, Clone)]
pub struct GraphQLType {
    package: Option<Package>,
    name: String,
    entity: Entity,
}

#[derive(Debug, Clone)]
struct Package {
    path: Vec<String>,
    input: String,
}

impl Package {
    fn parse(input: &str, separator: &str) -> Option<Self> {
        let path = input.split(separator).map(String::from).collect::<Vec<_>>();
        if path.is_empty() | input.is_empty() {
            None
        } else {
            Some(Self { path, input: input.to_string() })
        }
    }

    fn source(&self) -> &str {
        &self.input
    }
}

impl Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            self.path
                .iter()
                .map(|a| a.to_case(Case::Snake))
                .collect::<Vec<_>>()
                .join(DEFAULT_SEPARATOR)
                .as_str(),
        )
    }
}

impl GraphQLType {
    // FIXME: this can fail, should return a Result
    // FIXME: separator should be taken as an input
    fn parse(input: &str, convertor: Entity) -> Option<Self> {
        const SEPARATOR: &str = ".";
        if input.contains(SEPARATOR) {
            if let Some((package, name)) = input.rsplit_once(SEPARATOR) {
                let package = Package::parse(package, SEPARATOR);
                Some(Self { package, name: name.to_string(), entity: convertor })
            } else {
                None
            }
        } else {
            Some(Self { package: None, name: input.to_string(), entity: convertor })
        }
    }

    pub fn parse_enum(name: &str) -> Option<Self> {
        Self::parse(name, Entity::Enum)
    }

    pub fn parse_enum_variant(name: &str) -> Option<Self> {
        Self::parse(name, Entity::EnumVariant)
    }

    pub fn parse_object_type(name: &str) -> Option<Self> {
        Self::parse(name, Entity::ObjectType)
    }

    pub fn parse_method(name: &str) -> Option<Self> {
        Self::parse(name, Entity::Method)
    }

    pub fn parse_field(name: &str) -> Option<Self> {
        Self::parse(name, Entity::Field)
    }

    pub fn id(&self) -> String {
        if let Some(ref package) = self.package {
            format!("{}.{}", package.source(), self.name)
        } else {
            self.name.clone()
        }
    }

    pub fn package(mut self, package: &str) -> Option<Self> {
        let package = Package::parse(package, ".")?;
        self.package = Some(package);
        Some(self)
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
        match self.entity {
            Entity::EnumVariant => f.write_str(self.name.to_case(Case::ScreamingSnake).as_str())?,
            Entity::Field => f.write_str(self.name.to_case(Case::Snake).as_str())?,
            Entity::Method => {
                if let Some(package) = &self.package {
                    f.write_str(package.to_string().to_case(Case::Snake).as_str())?;
                    f.write_str(DEFAULT_SEPARATOR)?;
                };
                f.write_str(self.name.to_case(Case::Snake).as_str())?
            }
            Entity::Enum | Entity::ObjectType => {
                if let Some(package) = &self.package {
                    f.write_str(package.to_string().to_case(Case::ScreamingSnake).as_str())?;
                    f.write_str(DEFAULT_SEPARATOR)?;
                };
                f.write_str(self.name.to_case(Case::ScreamingSnake).as_str())?
            }
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    type TestParams = ((Entity, Option<&'static str>, &'static str), &'static str);

    #[test]
    fn test_from_enum() {
        let input: Vec<TestParams> = vec![
            // Enums
            ((Entity::Enum, None, "foo"), "FOO"),
            ((Entity::Enum, None, "a.b.c.foo"), "A_B_C_FOO"),
            ((Entity::Enum, Some("a.b.c"), "foo"), "A_B_C_FOO"),
            ((Entity::Enum, Some("a.b.c"), "d.e.f.foo"), "A_B_C_FOO"),
            ((Entity::Enum, Some(""), "a.b.c.foo"), "A_B_C_FOO"),
            ((Entity::Enum, None, "a_b_c_foo"), "A_B_C_FOO"),
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
            ((Entity::ObjectType, None, "foo"), "FOO"),
            ((Entity::ObjectType, None, "a.b.c.foo"), "A_B_C_FOO"),
            ((Entity::ObjectType, Some("a.b.c"), "foo"), "A_B_C_FOO"),
            ((Entity::ObjectType, Some("a.b"), "d.e.foo"), "A_B_FOO"),
            ((Entity::ObjectType, Some(""), "a.b.c.foo"), "A_B_C_FOO"),
            ((Entity::ObjectType, None, "a_b_c_foo"), "A_B_C_FOO"),
            // FIXME: failing
            ((Entity::ObjectType, None, "foo.bar.Baz"), "FOO_BAR_BAZ"),
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
            ((Entity::Method, None, "a_bC_foo"), "a_b_c_foo"),
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
            ((Entity::Field, None, "a_bC_foo"), "a_b_c_foo"),
        ];

        assert_type_names(input);
    }

    fn assert_type_names(input: Vec<TestParams>) {
        for ((entity, package, name), expected) in input {
            let mut g = GraphQLType::parse(name, entity).unwrap();
            if let Some(package) = package {
                g = g.clone().package(package).unwrap_or(g);
            }

            let actual = g.to_string();
            assert_eq!(actual, expected, "Given: {:?}", g);
        }
    }
}
