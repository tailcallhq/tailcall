use std::collections::HashMap;

use async_graphql::parser::types::ConstDirective;
use serde_json::Value;

use crate::core::valid::{Valid, ValidationError};

#[derive(Clone, Debug)]
pub struct Directive {
    pub name: String,
    pub arguments: HashMap<String, Value>,
}

pub fn to_directive(const_directive: ConstDirective) -> Valid<Directive, String> {
    const_directive
        .arguments
        .into_iter()
        .map(|(k, v)| {
            let value = v.node.into_json();

            value.map(|value| (k.node.to_string(), value))
        })
        .collect::<Result<_, _>>()
        .map_err(|e| ValidationError::new(e.to_string()))
        .map(|arguments| Directive { name: const_directive.name.node.to_string(), arguments })
        .into()
}
