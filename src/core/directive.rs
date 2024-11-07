use async_graphql::parser::types::ConstDirective;
use async_graphql::{Name, Positioned};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_path_to_error::deserialize;
use tailcall_valid::{Valid, ValidationError, Validator};

use super::pos;

pub trait DirectiveCodec: Sized {
    fn directive_name() -> String;
    fn from_directive(directive: &ConstDirective) -> Valid<Self, String>;
    fn to_directive(&self) -> ConstDirective;
    fn trace_name() -> String {
        format!("@{}", Self::directive_name())
    }
    fn from_directives<'a>(
        directives: impl Iterator<Item = &'a Positioned<ConstDirective>>,
    ) -> Valid<Option<Self>, String> {
        for directive in directives {
            if directive.node.name.node == Self::directive_name() {
                return Self::from_directive(&directive.node).map(Some);
            }
        }
        Valid::succeed(None)
    }
}
fn lower_case_first_letter(s: &str) -> String {
    if s.len() <= 2 {
        s.to_lowercase()
    } else if let Some(first_char) = s.chars().next() {
        first_char.to_string().to_lowercase() + &s[first_char.len_utf8()..]
    } else {
        s.to_string()
    }
}

impl<'a, A: Deserialize<'a> + Serialize + 'a> DirectiveCodec for A {
    fn directive_name() -> String {
        lower_case_first_letter(
            std::any::type_name::<A>()
                .split("::")
                .last()
                .unwrap_or_default(),
        )
    }

    fn from_directive(directive: &ConstDirective) -> Valid<A, String> {
        Valid::from_iter(directive.arguments.iter(), |(k, v)| {
            Valid::from(serde_json::to_value(&v.node).map_err(|e| {
                ValidationError::new(e.to_string())
                    .trace(format!("@{}", directive.name.node).as_str())
            }))
            .map(|v| (k.node.as_str().to_string(), v))
        })
        .map(|items| {
            items.iter().fold(Map::new(), |mut map, (k, v)| {
                map.insert(k.clone(), v.clone());
                map
            })
        })
        .and_then(|map| match deserialize(Value::Object(map)) {
            Ok(a) => Valid::succeed(a),
            Err(e) => Valid::from_validation_err(
                ValidationError::from(e).trace(format!("@{}", directive.name.node).as_str()),
            ),
        })
    }

    fn to_directive(&self) -> ConstDirective {
        let name = Self::directive_name();
        let value = serde_json::to_value(self).unwrap();
        let default_map = &Map::new();
        let map = value.as_object().unwrap_or(default_map);

        let mut arguments = Vec::new();
        for (k, v) in map {
            arguments.push((
                pos(Name::new(k.clone())),
                pos(serde_json::from_value(v.to_owned()).unwrap()),
            ));
        }

        ConstDirective { name: pos(Name::new(name)), arguments }
    }
}
