use std::fmt::Display;

use convert_case::{Case, Casing};
pub(super) static DEFAULT_SEPARATOR: &str = "__";
static PACKAGE_SEPARATOR: &str = ".";

fn normalize_name(name: &str) -> String {
    name.replace(PACKAGE_SEPARATOR, "_")
}

/// A helper to infer and build the name of a GraphQL type from raw list of
/// strings separated by a special character. It can be used to represent any
/// kind of GraphQL entity like an enum, an object type, a field or a method. It
/// can also be used to represent a package or a namespace. In unparsed form it
/// is just a list of strings.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphQLType<A>(A);

/// Represents a parsed GraphQL name where the actual name, the namespace and
/// the type of the entity is known.
#[derive(Debug, Clone, PartialEq)]
pub struct Parsed {
    namespace: Namespace,
    name: String,
    entity: Entity,
}

/// Represents an unparsed GraphQL name with just a list of strings.
/// Keeping the head separated ensures that we don't have an empty list of
/// strings.
#[derive(Debug, Clone)]
pub struct Unparsed {
    /// The head of the GraphQL type name.
    head: String,

    /// The tail of the GraphQL type name.
    tail: Vec<String>,
}

/// Represents a package or a namespace for the type, a feature typically found
/// in protobuf.
#[derive(Debug, Default, Clone, PartialEq)]
struct Namespace(Vec<String>);

impl Namespace {
    fn id(&self) -> String {
        self.0.join(PACKAGE_SEPARATOR)
    }
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[cfg(test)]
    fn new<A: AsRef<str> + Clone>(s: &[A]) -> Self {
        Self(s.iter().map(|a| a.as_ref().to_string()).collect())
    }
}

impl Display for Namespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.join(DEFAULT_SEPARATOR).as_str())
    }
}

impl GraphQLType<Unparsed> {
    pub fn new(name: &str) -> Self {
        Self(Unparsed { head: name.to_string(), tail: Vec::new() })
    }

    pub fn extend<S: AsRef<str>>(mut self, items: &[S]) -> Self {
        for item in items {
            self.0.tail.push(item.as_ref().to_string());
        }
        self
    }

    pub fn push<S: AsRef<str>>(mut self, text: S) -> Self {
        self.0.tail.push(text.as_ref().to_string());

        self
    }

    /// Parses list of string and extracts the entity's name, its package or
    /// namespace and it's type.
    fn parse(self, entity: Entity) -> GraphQLType<Parsed> {
        let unparsed = self.0;
        let path = unparsed
            .tail
            .iter()
            // TODO: separate should be passed as an argument.
            // Parser can not assume that the separator is always a dot. For eg: in Rust itself the
            // types are separated by `::`
            .flat_map(|c| c.split(PACKAGE_SEPARATOR))
            .map(|a| a.trim().to_string())
            .filter(|a| !a.is_empty())
            .collect::<Vec<_>>();

        let name = normalize_name(&unparsed.head);

        GraphQLType(Parsed { name, namespace: Namespace(path), entity })
    }

    pub fn into_enum(self) -> GraphQLType<Parsed> {
        self.parse(Entity::Enum)
    }

    pub fn into_enum_variant(self) -> GraphQLType<Parsed> {
        self.parse(Entity::EnumVariant)
    }

    pub fn into_object_type(self) -> GraphQLType<Parsed> {
        self.parse(Entity::ObjectType)
    }

    pub fn into_method(self) -> GraphQLType<Parsed> {
        self.parse(Entity::Method)
    }

    pub fn into_field(self) -> GraphQLType<Parsed> {
        self.parse(Entity::Field)
    }
}

impl GraphQLType<Parsed> {
    pub fn id(&self) -> String {
        let namespace = &self.0.namespace;
        if !namespace.is_empty() {
            format!("{}.{}", namespace.id(), self.0.name)
        } else {
            self.0.name.clone()
        }
    }
}

/// Used to convert proto type names to GraphQL formatted names.
/// Enum to represent the type of the descriptor
#[derive(Clone, Debug, PartialEq)]
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
                if !parsed.namespace.is_empty() {
                    f.write_str(parsed.namespace.to_string().as_str())?;
                    f.write_str(DEFAULT_SEPARATOR)?;
                };
                f.write_str(parsed.name.as_str())?
            }
            Entity::Enum | Entity::ObjectType => {
                if !parsed.namespace.is_empty() {
                    f.write_str(parsed.namespace.to_string().as_str())?;
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

    type TestParams = (
        (
            Entity, // Entity type for example Enum, EnumVariant, ObjectType, Method, Field
            &'static [&'static str], // Namespace or package
            &'static str, // Name of the entity
        ),
        &'static str,
    );

    #[test]
    fn test_from_enum() {
        let input: Vec<TestParams> = vec![
            // Enums
            ((Entity::Enum, &[], "foo"), "foo"),
            ((Entity::Enum, &[], "Foo"), "Foo"),
            ((Entity::Enum, &["a", "b.c"], "foo_bar"), "a__b__c__foo_bar"),
            ((Entity::Enum, &[], "a.b.c.foo"), "a_b_c_foo"),
            ((Entity::Enum, &["a.b.c"], "foo"), "a__b__c__foo"),
            (
                (Entity::Enum, &["a.b.c"], "d.e.f.foo"),
                "a__b__c__d_e_f_foo",
            ),
            ((Entity::Enum, &[""], "a.b.c.foo"), "a_b_c_foo"),
            ((Entity::Enum, &[], "a_b_c_foo"), "a_b_c_foo"),
        ];

        assert_type_names(input);
    }

    #[test]
    fn test_from_enum_variant() {
        let input: Vec<TestParams> = vec![
            // Enum variants
            ((Entity::EnumVariant, &[], "foo"), "foo"),
            ((Entity::EnumVariant, &[], "FOO_VAR"), "FOO_VAR"),
            ((Entity::EnumVariant, &[], "a.b.c.foo"), "a_b_c_foo"),
            ((Entity::EnumVariant, &["a.b.c"], "foo"), "foo"),
            ((Entity::EnumVariant, &["a.b"], "d.e.foo"), "d_e_foo"),
            ((Entity::EnumVariant, &[""], "a.b.c.foo"), "a_b_c_foo"),
            ((Entity::EnumVariant, &[], "a_b_c_foo"), "a_b_c_foo"),
        ];

        assert_type_names(input);
    }

    #[test]
    fn test_from_object_type() {
        let input: Vec<TestParams> = vec![
            // Object types
            ((Entity::ObjectType, &[], "foo"), "foo"),
            (
                (Entity::ObjectType, &["a", "b.c"], "fooBar"),
                "a__b__c__fooBar",
            ),
            ((Entity::ObjectType, &[], "a.b.c.foo"), "a_b_c_foo"),
            ((Entity::ObjectType, &["a.b.c"], "foo"), "a__b__c__foo"),
            ((Entity::ObjectType, &["a.b"], "d.e.foo"), "a__b__d_e_foo"),
            ((Entity::ObjectType, &[""], "a.b.c.foo"), "a_b_c_foo"),
            ((Entity::ObjectType, &[], "a_b_c_foo"), "a_b_c_foo"),
            ((Entity::ObjectType, &[], "foo.bar.Baz"), "foo_bar_Baz"),
        ];

        assert_type_names(input);
    }

    #[test]
    fn test_from_method() {
        let input: Vec<TestParams> = vec![
            // Methods
            ((Entity::Method, &[], "foo"), "foo"),
            ((Entity::Method, &["a.b.c"], "fooBar"), "a__b__c__fooBar"),
            (
                (Entity::Method, &["a.b", "c"], "foo_bar"),
                "a__b__c__foo_bar",
            ),
            ((Entity::Method, &[], "a.b.c.foo"), "a_b_c_foo"),
            ((Entity::Method, &["a.b.c"], "foo"), "a__b__c__foo"),
            ((Entity::Method, &["a.b"], "d.e.foo"), "a__b__d_e_foo"),
            ((Entity::Method, &[""], "a.b.c.foo"), "a_b_c_foo"),
            ((Entity::Method, &[], "a_b_c_foo"), "a_b_c_foo"),
        ];

        assert_type_names(input);
    }

    #[test]
    fn test_from_field() {
        let input: Vec<TestParams> = vec![
            // Fields
            ((Entity::Field, &[], "foo"), "foo"),
            ((Entity::Field, &["a.b.c"], "fooBar"), "fooBar"),
            ((Entity::Field, &["a.b", "c"], "foo_bar"), "fooBar"),
            ((Entity::Field, &[], "a.b.c.foo"), "aBCFoo"),
            ((Entity::Field, &["a.b.c"], "foo"), "foo"),
            ((Entity::Field, &["a.b"], "d.e.foo"), "dEFoo"),
            ((Entity::Field, &[""], "a.b.c.foo"), "aBCFoo"),
            ((Entity::Field, &[], "a_bC_foo"), "aBCFoo"),
        ];

        assert_type_names(input);
    }

    #[test]
    fn test_parse_name() {
        let actual = GraphQLType::new("foo").into_enum();
        let expected = GraphQLType(Parsed {
            name: "foo".to_string(),
            namespace: Namespace::default(),
            entity: Entity::Enum,
        });
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_namespace() {
        let actual = GraphQLType::new("foo").push("bar").push("baz").into_enum();
        let expected = GraphQLType(Parsed {
            name: "foo".to_string(),
            namespace: Namespace::new(&["bar", "baz"]),
            entity: Entity::Enum,
        });
        assert_eq!(actual, expected);
    }

    fn assert_type_names(input: Vec<TestParams>) {
        for ((entity, namespaces, name), expected) in input {
            let mut g = GraphQLType::new(name);
            for namespace in namespaces {
                g = g.push(namespace);
            }

            let actual = g.parse(entity).to_string();
            assert_eq!(actual, expected);
        }
    }
}
