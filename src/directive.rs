use async_graphql::parser::types::ConstDirective;
use async_graphql::{Name, Pos, Positioned};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_path_to_error::deserialize;

use anyhow::Result;

use crate::valid::ValidationError;

fn pos<A>(a: A) -> Positioned<A> {
    Positioned::new(a, Pos::default())
}

fn from_directive<'a, A: Deserialize<'a>>(directive: &'a ConstDirective) -> Result<A> {
    let mut map = Map::new();
    for (k, v) in directive.arguments.clone() {
        map.insert(k.node.as_str().to_string(), serde_json::to_value(&v.node)?);
    }

    Ok(deserialize(Value::Object(map)).map_err(|e| ValidationError::from(e).trace(directive.name.node.as_str()))?)
}

fn to_directive<A: Serialize>(a: &A, name: String) -> Result<ConstDirective> {
    let value = serde_json::to_value(a)?;
    let default_map = &Map::new();
    let map = value.as_object().unwrap_or(default_map);

    let mut arguments = Vec::new();
    for (k, v) in map {
        arguments.push((pos(Name::new(k.clone())), pos(serde_json::from_value(v.to_owned())?)));
    }

    Ok(ConstDirective { name: pos(Name::new(name)), arguments })
}

pub trait DirectiveCodec<'a, A> {
    fn from_directive(directive: &'a ConstDirective) -> Result<A>;
    fn to_directive(&'a self, name: String) -> Result<ConstDirective>;
}

impl<'a, A: Deserialize<'a> + Serialize> DirectiveCodec<'a, A> for A {
    fn from_directive(directive: &'a ConstDirective) -> Result<A> {
        from_directive(directive)
    }

    fn to_directive(&'a self, name: String) -> Result<ConstDirective> {
        to_directive(self, name)
    }
}
