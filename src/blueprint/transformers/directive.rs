use std::collections::HashMap;

use crate::blueprint::transform::Transform;
use crate::blueprint::{Blueprint, Directive};
use crate::config::Config;
use crate::directive::DirectiveCodec;
use crate::valid::ValidationError;

pub struct DirectiveTransform;

impl From<DirectiveTransform> for Transform<Config, Blueprint, String> {
  fn from(_value: DirectiveTransform) -> Self {
    Transform::new(move |config: &Config, mut blueprint: Blueprint| {
      let const_directive = config.server.to_directive("server".to_string());
      let arguments = const_directive
        .arguments
        .into_iter()
        .map(|(k, v)| {
          let value = v.node.into_json();
          if let Ok(value) = value {
            return Ok((k.node.to_string(), value));
          }
          Err(value.unwrap_err())
        })
        .collect::<Result<HashMap<String, serde_json::Value>, _>>()
        .map_err(|e| ValidationError::new(e.to_string()))?;

      blueprint.schema.directives =
        vec![Directive { name: const_directive.name.node.clone().to_string(), arguments, index: 0 }];

      Ok(blueprint)
    })
  }
}
