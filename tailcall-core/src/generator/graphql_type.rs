use std::fmt::Display;

use convert_case::{Case, Casing};
pub(super) static DEFAULT_SEPARATOR: &str = "_";
static PACKAGE_SEPARATOR: &str = ".";

/// A struct to represent the name of a GraphQL type.
#[derive(Debug, Clone)]
pub struct GraphQLType<A>(A);

#[derive(Debug, Clone)]
pub struct Parsed {
    package: Option<Package>,
    name: String,
    entity: Entity,
}

#[derive(Debug, Clone)]
pub struct Unparsed {
    package: Option<String>,
    name: String,
}

#[derive(Debug, Clone)]
struct Package {
    path: Vec<String>,
    input: String,
}

impl Package {
    fn parse(input: &str) -> Option<Self> {
        let separator = PACKAGE_SEPARATOR;
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

impl GraphQLType<Unparsed> {
    pub fn new(input: &str) -> Self {
        Self(Unparsed { package: None, name: input.to_string() })
    }

    fn parse(&self, entity: Entity) -> Option<GraphQLType<Parsed>> {
        let unparsed = &self.0;
        let parsed_package = unparsed.package.as_deref().and_then(Package::parse);

        // Name contains package
        if unparsed.name.contains(PACKAGE_SEPARATOR) {
            if let Some((package, name)) = unparsed.name.rsplit_once(PACKAGE_SEPARATOR) {
                Some(GraphQLType(Parsed {
                    name: name.to_string(),
                    package: parsed_package.or(Package::parse(package)),
                    entity,
                }))
            } else {
                None
            }
        }
        // Name doesn't contain package
        else {
            Some(GraphQLType(Parsed {
                package: parsed_package,
                name: unparsed.name.to_string(),
                entity,
            }))
        }
    }

    pub fn as_enum(&self) -> Option<GraphQLType<Parsed>> {
        self.parse(Entity::Enum)
    }

    pub fn as_enum_variant(&self) -> Option<GraphQLType<Parsed>> {
        self.parse(Entity::EnumVariant)
    }

    pub fn as_object_type(&self) -> Option<GraphQLType<Parsed>> {
        self.parse(Entity::ObjectType)
    }

    pub fn as_method(&self) -> Option<GraphQLType<Parsed>> {
        self.parse(Entity::Method)
    }

    pub fn as_field(&self) -> Option<GraphQLType<Parsed>> {
        self.parse(Entity::Field)
    }

    pub fn package(mut self, package: &str) -> Self {
        self.0.package = Some(package.to_string());
        self
    }
}

impl GraphQLType<Parsed> {
    pub fn id(&self) -> String {
        if let Some(ref package) = self.0.package {
            format!("{}.{}", package.source(), self.0.name)
        } else {
            self.0.name.clone()
        }
    }
}

/// Used to convert proto type names to GraphQL formatted names.
/// Enum to represent the type of the descriptor
#[derive(Clone, Debug)]
enum Entity {
    Enum,
    EnumVariant,
    ObjectType,
    Method,
    Field,
}

impl Display for GraphQLType<Parsed> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let parsed = &self.0;
        match parsed.entity {
            Entity::EnumVariant => {
                f.write_str(parsed.name.to_case(Case::ScreamingSnake).as_str())?
            }
            Entity::Field => f.write_str(parsed.name.to_case(Case::Snake).as_str())?,
            Entity::Method => {
                if let Some(package) = &parsed.package {
                    f.write_str(package.to_string().to_case(Case::Snake).as_str())?;
                    f.write_str(DEFAULT_SEPARATOR)?;
                };
                f.write_str(parsed.name.to_case(Case::Snake).as_str())?
            }
            Entity::Enum | Entity::ObjectType => {
                if let Some(package) = &parsed.package {
                    f.write_str(package.to_string().to_case(Case::ScreamingSnake).as_str())?;
                    f.write_str(DEFAULT_SEPARATOR)?;
                };
                f.write_str(parsed.name.to_case(Case::ScreamingSnake).as_str())?
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
            let mut g = GraphQLType::new(name);
            if let Some(package) = package {
                g = g.package(package);
            }

            let actual = g.parse(entity).unwrap().to_string();
            assert_eq!(actual, expected, "Given: {:?}", g);
        }
    }
}
