use async_graphql::parser::types::ConstDirective;
use async_graphql::{Name, Pos, Positioned};
use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_path_to_error::deserialize;

use crate::blueprint;
use crate::valid::{Valid, ValidationError};

fn pos<A>(a: A) -> Positioned<A> {
  Positioned::new(a, Pos::default())
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

fn to_const_directive(directive: &blueprint::Directive) -> Valid<ConstDirective, String> {
  Valid::from_iter(directive.arguments.iter(), |(k, v)| {
    let name = pos(Name::new(k.clone()));
    Valid::from(
      serde_json::from_value(v.clone())
        .map(pos)
        .map_err(|e| ValidationError::new(e.to_string()).trace(format!("@{}", directive.name).as_str())),
    )
    .map(|value| (name, value))
  })
  .map(|arguments| ConstDirective { name: pos(Name::new(directive.name.clone())), arguments })
}

pub trait DirectiveCodec<A> {
  fn directive_name() -> String;
  fn from_directive(directive: &ConstDirective) -> Valid<A, String>;
  fn from_blueprint_directive(directive: &blueprint::Directive) -> Valid<A, String> {
    to_const_directive(directive).and_then(|a| Self::from_directive(&a))
  }
  fn to_directive(&self) -> ConstDirective;
}

impl<'a, A: Deserialize<'a> + Serialize + 'a> DirectiveCodec<A> for A {
  fn directive_name() -> String {
    std::any::type_name::<A>()
      .split("::")
      .last()
      .unwrap_or_default()
      .to_string()
      .to_case(Case::Camel)
  }

  fn from_directive(directive: &ConstDirective) -> Valid<A, String> {
    Valid::from_iter(directive.arguments.iter(), |(k, v)| {
      Valid::from(
        serde_json::to_value(&v.node)
          .map_err(|e| ValidationError::new(e.to_string()).trace(format!("@{}", directive.name.node).as_str())),
      )
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
      Err(e) => {
        Valid::from_validation_err(ValidationError::from(e).trace(format!("@{}", directive.name.node).as_str()))
      }
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
