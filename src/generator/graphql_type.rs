use std::fmt::Display;

use convert_case::{Case, Casing};
pub(super) static DEFAULT_SEPARATOR: &str = "__";
static PACKAGE_SEPARATOR: &str = ".";

fn normalize_name(name: &str) -> String {
    name.replace(PACKAGE_SEPARATOR, "_")
}

/// A struct to represent the name of a GraphQL type.
#[derive(Debug, Clone)]
pub struct GraphQLType<A>(A);

#[derive(Debug, Clone)]
pub struct Parsed {
    namespace: Option<Namespace>,
    name: String,
    entity: Entity,
}

#[derive(Debug, Clone)]
pub struct Unparsed {
    namespace: Option<Namespace>,
    name: String,
}

#[derive(Debug, Clone)]
struct Namespace {
    path: Vec<String>,
}

impl Namespace {
    fn parse(input: &str) -> Self {
        let path = input
            .split(PACKAGE_SEPARATOR)
            .map(String::from)
            .collect::<Vec<_>>();

        Self { path }
    }

    fn combine(mut self, other: Namespace) -> Self {
        self.path.extend(other.path);

        self
    }
}

impl Display for Namespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.path.join(DEFAULT_SEPARATOR).as_str())
    }
}

impl GraphQLType<Unparsed> {
    pub fn new(name: &str) -> Self {
        Self(Unparsed { namespace: None, name: name.to_string() })
    }

    pub fn append_namespace(mut self, namespace: &str) -> Self {
        if namespace.is_empty() {
            return self;
        }

        let additional = Namespace::parse(namespace);

        self.0.namespace = if let Some(namespace) = self.0.namespace {
            Some(namespace.combine(additional))
        } else {
            Some(additional)
        };

        self
    }

    fn parse(self, entity: Entity) -> GraphQLType<Parsed> {
        let unparsed = self.0;
        let name = normalize_name(&unparsed.name);
        let namespace = unparsed.namespace;

        GraphQLType(Parsed { name, namespace, entity })
    }

    pub fn as_enum(self) -> GraphQLType<Parsed> {
        self.parse(Entity::Enum)
    }

    pub fn as_enum_variant(self) -> GraphQLType<Parsed> {
        self.parse(Entity::EnumVariant)
    }

    pub fn as_object_type(self) -> GraphQLType<Parsed> {
        self.parse(Entity::ObjectType)
    }

    pub fn as_method(self) -> GraphQLType<Parsed> {
        self.parse(Entity::Method)
    }

    pub fn as_field(self) -> GraphQLType<Parsed> {
        self.parse(Entity::Field)
    }
}

impl GraphQLType<Parsed> {
    pub fn id(&self) -> String {
        if let Some(ref namespace) = self.0.namespace {
            format!("{}.{}", namespace, self.0.name)
        } else {
            self.0.name.clone()
        }
    }
}

// FIXME: make it private
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
            Entity::EnumVariant => f.write_str(parsed.name.as_str())?,
            Entity::Field => f.write_str(parsed.name.to_case(Case::Camel).as_str())?,
            Entity::Method => {
                if let Some(package) = &parsed.namespace {
                    f.write_str(package.to_string().as_str())?;
                    f.write_str(DEFAULT_SEPARATOR)?;
                };
                f.write_str(parsed.name.as_str())?
            }
            Entity::Enum | Entity::ObjectType => {
                if let Some(package) = &parsed.namespace {
                    f.write_str(package.to_string().as_str())?;
                    f.write_str(DEFAULT_SEPARATOR)?;
                };
                f.write_str(parsed.name.as_str())?
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
            ((Entity::Enum, None, "foo"), "foo"),
            ((Entity::Enum, None, "a.b.c.foo"), "a_b_c_foo"),
            ((Entity::Enum, Some("a.b.c"), "foo"), "a__b__c__foo"),
            (
                (Entity::Enum, Some("a.b.c"), "d.e.f.foo"),
                "a__b__c__d_e_f_foo",
            ),
            ((Entity::Enum, Some(""), "a.b.c.foo"), "a_b_c_foo"),
            ((Entity::Enum, None, "a_b_c_foo"), "a_b_c_foo"),
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
            ((Entity::Method, Some("a.b.c"), "foo"), "a_b_c__foo"),
            ((Entity::Method, Some("a.b"), "d.e.foo"), "a_b__d_e_foo"),
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
        for ((entity, namespace, name), expected) in input {
            let mut g = GraphQLType::new(name);
            if let Some(namespace) = namespace {
                g = g.append_namespace(namespace);
            }

            let actual = g.parse(entity).to_string();
            assert_eq!(actual, expected);
        }
    }
}
