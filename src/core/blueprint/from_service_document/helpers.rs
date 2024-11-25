use std::slice::Iter;
use async_graphql::parser::types::ConstDirective;
use async_graphql::Positioned;
use tailcall_valid::{Valid, ValidationError};
use crate::core::blueprint;
use tailcall_valid::Validator;

pub fn extract_directives(iter: Iter<Positioned<ConstDirective>>) -> Valid<Vec<blueprint::directive::Directive>, super::Error> {
    Valid::from_iter(iter, |directive| {
        let directives = Valid::from_iter(directive.node.arguments.iter(), |(k, v)| {
            let value = v.clone().node.into_json();
            let value = value.map_err(|e| ValidationError::new(e.to_string()));
            Valid::from(value.map(|value| (k.node.to_string(), value)))
        }).map(|arguments| {
            blueprint::directive::Directive {
                name: directive.node.name.node.to_string(),
                arguments: arguments.into_iter().collect(),
            }
        });
        directives
    })
}
