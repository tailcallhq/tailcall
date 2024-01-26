use std::num::NonZeroU64;

use anyhow::anyhow;
use async_graphql_value::ConstValue;
use concurrent_lru::sharded::LruCache;
use tailcall::Cache;

pub struct WasmCache {
  cache: LruCache<u64, ConstValue>,
}
impl WasmCache {
  pub fn init() -> Self {
    Self { cache: LruCache::new(999) }
  }
}

#[async_trait::async_trait]
impl Cache for WasmCache {
  type Key = u64;
  type Value = ConstValue;

  async fn set<'a>(&'a self, key: Self::Key, value: Self::Value, ttl: NonZeroU64) -> anyhow::Result<Self::Value> {
    Ok(self.cache.get_or_init(key, 1, |_| value).value().clone())
  }

  async fn get<'a>(&'a self, key: &'a Self::Key) -> anyhow::Result<Self::Value> {
    Ok(
      self
        .cache
        .get(key.clone())
        .ok_or(anyhow!("No such key found"))?
        .value()
        .clone(),
    )
  }
}
