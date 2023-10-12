use async_graphql::dynamic::ResolverContext;
use async_graphql::{Name, Value};
use indexmap::IndexMap;

pub trait ResolverContextLike<'a> {
  fn value(&'a self) -> Option<&'a Value>;
  fn args(&'a self) -> Option<&'a IndexMap<Name, Value>>;
}

pub struct EmptyResolverContext;

impl<'a> ResolverContextLike<'a> for EmptyResolverContext {
  fn value(&'a self) -> Option<&'a Value> {
    None
  }

  fn args(&'a self) -> Option<&'a IndexMap<Name, Value>> {
    None
  }
}

impl<'a> ResolverContextLike<'a> for ResolverContext<'a> {
  fn value(&'a self) -> Option<&'a Value> {
    self.parent_value.as_value()
  }

  fn args(&'a self) -> Option<&'a IndexMap<Name, Value>> {
    Some(self.args.as_index_map())
  }
}
