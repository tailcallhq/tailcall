use std::fmt::Formatter;

#[derive(Clone)]
pub enum WrappingType {
    NamedType { name: String, non_null: bool },
    ListType { of_type: Box<WrappingType>, non_null: bool },
}

impl std::fmt::Debug for WrappingType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WrappingType::NamedType { name, non_null } => {
                if *non_null {
                    write!(f, "{}!", name)
                } else {
                    write!(f, "{}", name)
                }
            }
            WrappingType::ListType { of_type, non_null } => {
                if *non_null {
                    write!(f, "[{:?}]!", of_type)
                } else {
                    write!(f, "[{:?}]", of_type)
                }
            }
        }
    }
}

impl Default for WrappingType {
    fn default() -> Self {
        WrappingType::NamedType { name: "JSON".to_string(), non_null: false }
    }
}

impl WrappingType {
    /// gets the name of the type
    pub fn name(&self) -> &str {
        match self {
            WrappingType::NamedType { name, .. } => name,
            WrappingType::ListType { of_type, .. } => of_type.name(),
        }
    }

    /// checks if the type is nullable
    pub fn is_nullable(&self) -> bool {
        !match self {
            WrappingType::NamedType { non_null, .. } => *non_null,
            WrappingType::ListType { non_null, .. } => *non_null,
        }
    }
    /// checks if the type is a list
    pub fn is_list(&self) -> bool {
        matches!(self, WrappingType::ListType { .. })
    }
}
