use std::hash::Hash;
use std::num::NonZeroU64;
use std::sync::Arc;

use anyhow::Result;
use tailcall::ChronoCache;

use crate::to_anyhow;

pub struct CloudflareChronoCache<K: Hash + Eq, V> {
  env: Arc<worker::Env>,
}

impl<K: Hash + Eq, V: Clone> CloudflareChronoCache<K, V> {
  fn get_kv(&self) -> Result<worker::kv::KvStore> {
    self.env.kv("TMP_KV").map_err(to_anyhow)
  }
}

impl<K: Hash + Eq, V: Clone> ChronoCache<K, V> for CloudflareChronoCache<K, V> {
  fn insert(&self, key: K, value: V, ttl: NonZeroU64) -> Result<V> {
    unimplemented!()
  }

  fn get(&self, key: &K) -> Result<V> {
    unimplemented!()
  }
}
