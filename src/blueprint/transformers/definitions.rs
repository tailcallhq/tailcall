use crate::blueprint::transform::Transform;
use crate::blueprint::transformers::enum_type::EnumTransform;
use crate::blueprint::transformers::objects::ObjectTransform;
use crate::blueprint::transformers::scalar_type::ScalarTransform;
use crate::blueprint::transformers::union_type::UnionTransform;
use crate::blueprint::transformers::Valid;
use crate::blueprint::Blueprint;
use crate::config::Config;
use crate::valid::{ValidExtensions, VectorExtension};

/// Transform the config blueprint definitions
pub struct DefinitionsTransform;

impl From<DefinitionsTransform> for Transform<Config, Blueprint, String> {
  fn from(_value: DefinitionsTransform) -> Self {
    Transform::new(move |config: &Config, blueprint: Blueprint| {
      let input_types = config.input_types();
      let output_types = config.output_types();
      let mut transformers = config.graphql.types.iter().validate_all(|(name, type_)| {
        let dbl_usage = input_types.contains(name) && output_types.contains(name);
        if let Some(variants) = &type_.variants {
          if !variants.is_empty() {
            Ok(Transform::from(EnumTransform {
              type_of: type_.clone(),
              name: name.to_string(),
            }))
          } else {
            Valid::fail("No variants found for enum".to_string())
          }
        } else if type_.scalar {
          Ok(Transform::from(ScalarTransform { name: name.to_string() }))
        } else if dbl_usage {
          Valid::fail("type is used in input and output".to_string()).trace(name)
        } else {
          Ok(Transform::from(ObjectTransform {
            type_of: type_.clone(),
            name: name.to_string(),
          }))
        }
      })?;

      let unions = config
        .graphql
        .unions
        .iter()
        .map(|(n, u)| Transform::from(UnionTransform { name: n.to_owned(), union: u.to_owned() }));
      transformers.extend(unions);
      let Some(transformer) = transformers.into_iter().reduce(|t1, t2| t1 + t2) else {
        return Valid::fail("No transformers".to_string());
      };
      transformer.transform(config, blueprint)
    })
  }
}
