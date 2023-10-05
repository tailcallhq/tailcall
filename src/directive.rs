use async_graphql::parser::types::ConstDirective;
use async_graphql::{Name, Pos, Positioned};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_path_to_error::deserialize;

use crate::valid::{Valid, ValidationError, VectorExtension};

fn pos<A>(a: A) -> Positioned<A> {
  Positioned::new(a, Pos::default())
}

fn from_directive<'a, A: Deserialize<'a>>(directive: &'a ConstDirective) -> Valid<A, String> {
  let mut map = Map::new();
  let p = directive.arguments.iter().validate_all(|(k, v)| {
    Valid::Ok((
      k.node.as_str().to_string(),
      serde_json::to_value(&v.node)
        .map_err(|e| ValidationError::new(e.to_string()).trace(directive.name.node.as_str()))?,
    ))
  })?;
  p.into_iter().for_each(|(k, v)| {
    map.insert(k, v);
  });

  deserialize(Value::Object(map)).map_err(|e| ValidationError::from(e).trace(directive.name.node.as_str()))
}

fn to_directive<A: Serialize>(a: &A, name: String) -> ConstDirective {
  let value = serde_json::to_value(a).unwrap();
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

pub trait DirectiveCodec<'a, A> {
  fn from_directive(directive: &'a ConstDirective) -> Valid<A, String>;
  fn to_directive(&'a self, name: String) -> ConstDirective;
}

impl<'a, A: Deserialize<'a> + Serialize> DirectiveCodec<'a, A> for A {
  fn from_directive(directive: &'a ConstDirective) -> Valid<A, String> {
    from_directive(directive)
  }

  fn to_directive(&'a self, name: String) -> ConstDirective {
    to_directive(self, name)
  }
}
