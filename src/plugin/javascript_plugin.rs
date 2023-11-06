use mini_v8::MiniV8;
use serde::{Deserialize, Serialize};

use super::Plugin;
use crate::blueprint::Definition;
use crate::directive::DirectiveCodec;
use crate::lambda::Expression;
use crate::valid::Valid;

pub struct JSPlugin {
  mini_v8: MiniV8,
}

impl JSPlugin {
  fn new(mini_v8: MiniV8) -> Self {
    Self { mini_v8 }
  }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct JavascriptDirective {
  source: String,
}

impl JavascriptDirective {
  fn to_js_resolver(self) -> Expression {
    todo!()
  }
}

impl<'a> Plugin<'a, String> for JSPlugin {
  fn run(
    _: &'a crate::config::Config,
    blueprint: crate::blueprint::Blueprint,
  ) -> Result<crate::blueprint::Blueprint, crate::valid::ValidationError<String>> {
    Valid::from_iter(blueprint.definitions.clone(), |def| match def.clone() {
      Definition::ObjectTypeDefinition(object_type_definition) => {
        Valid::from_iter(object_type_definition.fields.clone(), |mut field| {
          let find = field
            .directives
            .iter()
            .find(|a| a.name == JavascriptDirective::directive_name());

          if let Some(directive) = find {
            JavascriptDirective::from_blueprint_directive(directive).foreach(|js| {
              field.resolver = Some(js.to_js_resolver());
            });
          }
          Valid::succeed(field)
        })
      }
      .map_to(def),
      def => Valid::succeed(def),
    })
    .map(|definitions| blueprint.definitions(definitions))
    .to_result()
  }
}
