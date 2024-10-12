use std::collections::HashMap;

use async_graphql::parser::types::ConstDirective;
use async_graphql::Name;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tailcall_valid::{Valid, ValidationError, Validator};

use crate::core::{is_default, pos};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
pub struct Directive {
    pub name: String,
    #[serde(default, skip_serializing_if = "is_default")]
    #[schemars(with = "HashMap::<String, Value>")]
    pub arguments: IndexMap<String, Value>,
}

pub fn to_const_directive(directive: &Directive) -> Valid<ConstDirective, String> {
    Valid::from_iter(directive.arguments.iter(), |(k, v)| {
        let name = pos(Name::new(k.clone()));
        Valid::from(serde_json::from_value(v.clone()).map(pos).map_err(|e| {
            ValidationError::new(e.to_string()).trace(format!("@{}", directive.name).as_str())
        }))
        .map(|value| (name, value))
    })
    .map(|arguments| ConstDirective { name: pos(Name::new(directive.name.clone())), arguments })
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

#[cfg(test)]
mod tests {

    use async_graphql::parser::types::ConstDirective;
    use async_graphql_value::Name;
    use pretty_assertions::assert_eq;
    use tailcall_valid::Validator;

    use super::*;

    #[test]
    fn test_to_const_directive() {
        let directive = Directive {
            name: "test".to_string(),
            arguments: vec![("a".to_string(), serde_json::json!(1.0))]
                .into_iter()
                .collect(),
        };

        let const_directive: ConstDirective = to_const_directive(&directive).to_result().unwrap();
        let expected_directive: ConstDirective = ConstDirective {
            name: pos(Name::new("test")),
            arguments: vec![(pos(Name::new("a")), pos(async_graphql::Value::from(1.0)))]
                .into_iter()
                .collect(),
        };

        assert_eq!(
            format!("{:?}", const_directive),
            format!("{:?}", expected_directive)
        );
    }
}
