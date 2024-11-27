use std::collections::HashMap;

use async_graphql::parser::types::ConstDirective;
use async_graphql::Name;
use serde_json::Value;
use tailcall_valid::{Valid, ValidationError, Validator};

use super::BlueprintError;
use crate::core::{config, pos};

#[derive(Clone, Debug)]
pub struct Directive {
    pub name: String,
    pub arguments: HashMap<String, Value>,
}

pub fn to_directive(const_directive: ConstDirective) -> Valid<Directive, BlueprintError> {
    match const_directive
        .arguments
        .into_iter()
        .map(|(k, v)| {
            let value = v.node.into_json();

            value.map(|value| (k.node.to_string(), value))
        })
        .collect::<Result<_, _>>()
        .map_err(|e| ValidationError::new(e.to_string()))
        .map(|arguments| Directive { name: const_directive.name.node.to_string(), arguments })
    {
        Ok(data) => Valid::succeed(data),
        Err(e) => Valid::from_validation_err(BlueprintError::from_validation_string(e)),
    }
}

pub fn to_const_directive(directive: &Directive) -> Valid<ConstDirective, String> {
    Valid::from_iter(directive.arguments.iter(), |(k, v)| {
        let name = pos(Name::new(k));
        Valid::from(serde_json::from_value(v.clone()).map(pos).map_err(|e| {
            ValidationError::new(e.to_string()).trace(format!("@{}", directive.name).as_str())
        }))
        .map(|value| (name, value))
    })
    .map(|arguments| ConstDirective { name: pos(Name::new(&directive.name)), arguments })
}

impl From<config::Directive> for Directive {
    fn from(value: config::Directive) -> Self {
        Self {
            name: value.name,
            arguments: value.arguments.into_iter().collect(),
        }
    }
}
