use strum_macros::Display;
use convert_case::{Case, Casing};
pub(super) static DEFAULT_SEPARATOR: &str = "__";
pub(super) static DEFAULT_PACKAGE_SEPARATOR: &str = "_";

pub struct Name<A> {
    phantom: std::marker::PhantomData<A>,
    package: String,
    name: String,
    convertor: NameConvertor,
}

impl Name<Proto> {
    fn new(package: &str, name: &str, convertor: NameConvertor) -> Self {
        Self {
            package: package.to_string(),
            phantom: std::marker::PhantomData,
            name: name.to_string(),
            convertor,
        }
    }
    pub fn enumeration(package: &str, name: &str) -> Self {
        Self::new(package, name, NameConvertor::Enum)
    }
    pub fn message(package: &str, name: &str) -> Self {
        Self::new(package, name, NameConvertor::Message)
    }
    pub fn method(package: &str, name: &str) -> Self {
        Self::new(package, name, NameConvertor::Method)
    }
    pub fn arg(package: &str, name: &str) -> Self {
        Self::new(package, name, NameConvertor::Arg)
    }

    pub fn convert(self) -> Name<GraphQL> {
        Name {
            package: self.package.clone(),
            phantom: std::marker::PhantomData,
            name: self.convertor.convert(self.package.as_str(), &self.name),
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
    Message,
    Method,
    Arg,
}

impl NameConvertor {
    /// Takes in a name and returns a GraphQL name based on the descriptor type.
    fn convert(&self, package: &str, name: &str) -> String {
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
            NameConvertor::Method | NameConvertor::Arg => name.to_case(Case::Camel),
            NameConvertor::Enum | NameConvertor::Message => {
                package_prefix + &name.to_case(Case::Pascal)
            }
        }
    }
}