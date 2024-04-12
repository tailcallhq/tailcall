use convert_case::{Case, Casing};
use derive_setters::Setters;
use strum_macros::Display;
pub(super) static DEFAULT_SEPARATOR: &str = "__";
pub(super) static DEFAULT_PACKAGE_SEPARATOR: &str = "_";

#[derive(Setters)]
pub struct Name<A> {
    phantom: std::marker::PhantomData<A>,
    package: Option<String>,
    name: String,
    convertor: NameConvertor,
}

impl Name<Proto> {
    fn new(name: &str, convertor: NameConvertor) -> Self {
        Self {
            package: None,
            phantom: std::marker::PhantomData,
            name: name.to_string(),
            convertor,
        }
    }

    pub fn enum_value(name: &str) -> Self {
        Self::new(name, NameConvertor::Enum)
    }

    pub fn enum_variant(name: &str) -> Self {
        Self::new(name, NameConvertor::EnumVariant)
    }

    pub fn message(name: &str) -> Self {
        Self::new(name, NameConvertor::Message)
    }

    pub fn method(name: &str) -> Self {
        Self::new(name, NameConvertor::Method)
    }

    pub fn field(name: &str) -> Self {
        Self::new(name, NameConvertor::Field)
    }

    pub fn convert(self) -> Name<GraphQL> {
        Name {
            package: self.package.clone(),
            phantom: std::marker::PhantomData,
            name: self.convertor.convert(self.package, &self.name),
            convertor: self.convertor,
        }
    }
}

pub struct GraphQL {}
pub struct Proto {}

/// Used to convert proto type names to GraphQL formatted names.
/// Enum to represent the type of the descriptor
#[derive(Display, Clone)]
enum NameConvertor {
    Enum,
    EnumVariant,
    Message,
    Method,
    Field,
}

impl NameConvertor {
    /// Takes in a name and returns a GraphQL name based on the descriptor type.
    fn convert(&self, package: Option<String>, name: &str) -> String {
        let package = package.unwrap_or_default();
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

        let package_prefix = if package.is_empty() {
            "".to_string()
        } else {
            package + DEFAULT_SEPARATOR
        };

        match self {
            NameConvertor::EnumVariant => name.to_case(Case::UpperSnake),
            NameConvertor::Method | NameConvertor::Field => name.to_case(Case::Camel),
            NameConvertor::Enum | NameConvertor::Message => {
                package_prefix + &name.to_case(Case::Pascal)
            }
        }
    }
}
