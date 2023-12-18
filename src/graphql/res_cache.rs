use std::sync::{Arc, RwLock};
use std::time::Duration;

use async_graphql_value::ConstValue;
use ttl_cache::TtlCache;

use crate::config::CacheRules;
use crate::lambda::{EvaluationContext, Expression, ResolverContextLike};

const DEFAULT_MAX_AGE: u64 = 30;

#[derive(Clone)]
pub struct ResCache {
  cache_rules: CacheRules,
  data: Arc<RwLock<TtlCache<u64, ConstValue>>>,
}

impl std::fmt::Debug for ResCache {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "ResCache {{cache_rules: {:?}, capacity: {:?}}}",
      self.cache_rules,
      self.data.read().unwrap().capacity()
    )
  }
}

impl ResCache {
  pub fn new(cache_rules: CacheRules) -> Self {
    ResCache { cache_rules, data: Arc::new(RwLock::new(TtlCache::new(1000))) }
  }

  fn insert(&self, key: u64, value: &ConstValue) -> Option<ConstValue> {
    let ttl = Duration::from_secs(self.cache_rules.max_age.unwrap_or(DEFAULT_MAX_AGE));
    self.data.write().unwrap().insert(key, value.clone(), ttl)
  }

  fn get(&self, key: &u64) -> Option<ConstValue> {
    self.data.read().unwrap().get(key).cloned()
  }

  #[allow(dead_code)]
  fn remove(&self, key: &u64) -> Option<ConstValue> {
    self.data.write().unwrap().remove(key)
  }

  #[allow(clippy::too_many_arguments)]
  pub async fn fetch<'a, Ctx: ResolverContextLike<'a> + Sync + Send>(
    &'a self,
    ctx: &'a EvaluationContext<'a, Ctx>,
    expr: &'a Expression,
    key: u64,
  ) -> anyhow::Result<async_graphql::Value> {
    if let Some(value) = self.get(&key) {
      Ok(value)
    } else {
      let value = expr.eval(ctx).await?;
      self.insert(key, &value);
      Ok(value)
    }
  }
}

#[cfg(test)]
mod tests {
  use std::thread;
  use std::time::Duration;

  use async_graphql_value::ConstValue;

  use super::ResCache;
  use crate::config::CacheRules;

  #[test]
  fn test_res_cache_insert() {
    let max_age = Some(1);
    let cache = ResCache::new(CacheRules { max_age });
    let key = 10;
    let value: ConstValue = "value".into();
    cache.insert(key, &value);
    assert_eq!(cache.get(&key), Some(value));
  }

  #[test]
  fn test_res_cache_ttl() {
    let max_age = Some(1);
    let cache = ResCache::new(CacheRules { max_age });
    let key = 10;
    let value: ConstValue = "value".into();
    cache.insert(key, &value);
    assert_eq!(cache.get(&key), Some(value));
    thread::sleep(Duration::from_secs(1));
    assert_eq!(
      cache.get(&key),
      None,
      "cache shouldn't contain the value after `CacheRules.max_age` secs have passed"
    );
  }

  #[test]
  fn test_res_cache_remove() {
    let max_age = Some(100);
    let cache = ResCache::new(CacheRules { max_age });
    let key = 10;
    let value: ConstValue = "value".into();
    cache.insert(key, &value);
    assert_eq!(cache.get(&key), Some(value));
    cache.remove(&key);
    assert_eq!(
      cache.get(&key),
      None,
      "cache shouldn't contain the value after `CacheRules.max_age` secs have passed"
    );
  }
}
