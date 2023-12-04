use std::collections::HashSet;
use std::hash::Hash;

use async_graphql_value::ConstValue;
use indexmap::IndexMap;

use crate::lambda::Expression;

type Field = String;

pub struct ResolvePlan {
  resolver: Box<dyn Resolver>,
  fields: IndexMap<String, ResolvePlan>,
}


#[async_trait::async_trait]
trait Resolver {
  async fn resolve(&self, fields: HashSet<Field>) -> ConstValue;
}

struct ResolverWrapper<R: Resolver> {
  resolver: R,
  fields: HashSet<Field>
}

impl<R: Resolver> ResolverWrapper<R> {
  fn extend(&mut self, other: ResolverWrapper<R>) {
    self.fields.extend(other.fields);
  }
}

#[cfg(test)]
mod tests {
  use serde_json::Value;

  use super::*;

  #[test]
  fn test() {
    let user_resolve_plan = ResolvePlan {
      resolver: Resolver::Empty,
      fields: IndexMap::from_iter([(
        "user".to_owned(),
        ResolvePlan {
          // resolver: Resolver::Expression(Expression::Literal(Value::String("user_name".to_owned()))),
          fields: IndexMap::from_iter([(
            "name".to_owned(),
            ResolvePlan { resolver: Resolver::Parent, fields: IndexMap::new() },
          )]),
        },
      )]),
    };

    /// ```graphql
    /// user {
    ///   id
    ///   name
    /// }
    /// ```
  }
}
