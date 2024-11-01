use std::collections::HashMap;

use async_graphql::parser::types::ConstDirective;
use async_graphql::Name;
use serde_json::Value;

use crate::core::valid::{Valid, ValidationError, Validator};
use crate::core::{config, pos};

#[derive(Clone, Debug)]
pub struct Directive {
    pub name: String,
    pub arguments: HashMap<String, Value>,
}

pub fn to_directive(const_directive: ConstDirective) -> Valid<Directive, miette::MietteDiagnostic> {
    const_directive
        .arguments
        .into_iter()
        .map(|(k, v)| {
            let value = v.node.into_json();

            value.map(|value| (k.node.to_string(), value))
        })
        .collect::<Result<_, _>>()
        .map_err(|e| ValidationError::new(miette::diagnostic!("{}", e)))
        .map(|arguments| Directive { name: const_directive.name.node.to_string(), arguments })
        .into()
}

pub fn to_const_directive(
    directive: &Directive,
) -> Valid<ConstDirective, miette::MietteDiagnostic> {
    Valid::from_iter(directive.arguments.iter(), |(k, v)| {
        let name = pos(Name::new(k));
        Valid::from(serde_json::from_value(v.clone()).map(pos).map_err(|e| {
            ValidationError::new(miette::diagnostic!("{}", e))
                .trace(format!("@{}", directive.name).as_str())
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
