use async_graphql::futures_util::TryStreamExt;
use log::Level::Error;
use mini_v8::Error::Value;

use crate::blueprint::foldrs::enum_type::EnumFold;
use crate::blueprint::foldrs::objects::ObjectsFold;
use crate::blueprint::foldrs::scalar_type::ScalarFold;
use crate::blueprint::foldrs::union_type::UnionTransFold;
use crate::blueprint::Blueprint;
use crate::config::Config;
use crate::lambda::Expression::Input;
use crate::try_fold::TryFolding;
use crate::valid;
use crate::valid::{Valid, ValidExtensions, VectorExtension};

/// Transform the config blueprint definitions
pub struct DefinitionsFold;

impl TryFolding for DefinitionsFold {
  type Input = Config;
  type Value = Blueprint;
  type Error = String;

  fn try_fold(self, cfg: &Self::Input, blueprint: Self::Value) -> Valid<Self::Value, Self::Error> {
    let input_types = cfg.input_types();
    let output_types = cfg.output_types();
    let mut foldrs: Vec<Valid<dyn TryFolding<Input = Self::Input, Value = Self::Value, Error = Self::Error>, _>> =
      cfg.graphql.types.iter().validate_all(|(name, type_)| {
        let dbl_usage = input_types.contains(name) && output_types.contains(name);
        if let Some(variants) = &type_.variants {
          if !variants.is_empty() {
            Ok(EnumFold { type_of: type_.clone(), name: name.to_string() }.into())
          } else {
            Valid::fail("No variants found for enum".to_string())
          }
        } else if type_.scalar {
          Ok(ScalarFold { name: name.to_string() }.into())
        } else if dbl_usage {
          Valid::fail("type is used in input and output".to_string()).trace(name)
        } else {
          Ok(ObjectsFold { type_of: type_.clone(), name: name.to_string() }.into())
        }
      })?;

    let unions = cfg
      .graphql
      .unions
      .iter()
      .map(|(n, u)| UnionTransFold { name: n.to_owned(), union: u.to_owned() });
    foldrs.extend(unions);
    let Some(Ok(foldr)) = foldrs.into_iter().reduce(|t1, t2| t1.and(t2)) else {
      return Valid::fail("No foldrs".to_string());
    };
    foldr.try_fold(cfg, blueprint)
  }
}
