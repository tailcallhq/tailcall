use std::time::Duration;

use async_graphql_value::ConstValue;
use ttl_cache::TtlCache;

pub struct GQLCache {
  data: TtlCache<u64, ConstValue>,
}

impl Default for GQLCache {
  fn default() -> Self {
    Self::new()
  }
}

impl GQLCache {
  pub fn new() -> Self {
    GQLCache { data: TtlCache::new(10000) }
  }

  #[allow(clippy::too_many_arguments)]
  pub fn insert(&mut self, key: u64, value: ConstValue, ttl: Duration) -> Option<ConstValue> {
    self.data.insert(key, value, ttl)
  }

  pub fn get(&self, key: &u64) -> Option<ConstValue> {
    self.data.get(key).cloned()
  }

  pub fn remove(&mut self, key: &u64) -> Option<ConstValue> {
    self.data.remove(key)
  }
}
