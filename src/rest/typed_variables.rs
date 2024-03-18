use async_graphql::parser::types::{BaseType, Type};
use async_graphql_value::ConstValue;

#[derive(Clone, Debug, PartialEq)]
pub enum UrlParamType {
    String,
    Number(N),
    Boolean,
}

#[derive(Clone, Debug, PartialEq)]
pub enum N {
    Int,
    Float,
}

impl N {
    fn to_value(&self, value: &str) -> anyhow::Result<ConstValue> {
        Ok(match self {
            Self::Int => ConstValue::from(value.parse::<i64>()?),
            Self::Float => ConstValue::from(value.parse::<f64>()?),
        })
    }
}

impl UrlParamType {
    fn to_value(&self, value: &str) -> anyhow::Result<ConstValue> {
        Ok(match self {
            Self::String => ConstValue::String(value.to_string()),
            Self::Number(n) => n.to_value(value)?,
            Self::Boolean => ConstValue::Boolean(value.parse()?),
        })
    }
}

impl TryFrom<&Type> for UrlParamType {
    type Error = anyhow::Error;
    fn try_from(value: &Type) -> anyhow::Result<Self> {
        match &value.base {
            BaseType::Named(name) => match name.as_str() {
                "String" => Ok(Self::String),
                "Int" => Ok(Self::Number(N::Int)),
                "Boolean" => Ok(Self::Boolean),
                "Float" => Ok(Self::Number(N::Float)),
                _ => Err(anyhow::anyhow!("unsupported type: {}", name)),
            },
            // TODO: support for list types
            _ => Err(anyhow::anyhow!("unsupported type: {:?}", value)),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct TypedVariable {
    type_of: UrlParamType,
    name: String,
    // TODO: validate types for query
    nullable: bool,
}

impl TypedVariable {
    fn new(tpe: UrlParamType, name: &str) -> Self {
        Self { type_of: tpe, name: name.to_string(), nullable: false }
    }

    pub fn try_from(type_of: &Type, name: &str) -> anyhow::Result<Self> {
        let tpe = UrlParamType::try_from(type_of)?;
        Ok(Self::new(tpe, name))
    }

    pub fn to_value(&self, value: &str) -> anyhow::Result<ConstValue> {
        self.type_of.to_value(value)
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn nullable(&self) -> bool {
        self.nullable
    }
    pub fn ty(&self) -> UrlParamType {
        self.type_of.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl TypedVariable {
        pub fn string(name: &str) -> Self {
            Self::new(UrlParamType::String, name)
        }

        pub fn float(name: &str) -> Self {
            Self::new(UrlParamType::Number(N::Float), name)
        }

        pub fn boolean(name: &str) -> Self {
            Self::new(UrlParamType::Boolean, name)
        }

        pub fn int(name: &str) -> Self {
            Self::new(UrlParamType::Number(N::Int), name)
        }
    }
}
