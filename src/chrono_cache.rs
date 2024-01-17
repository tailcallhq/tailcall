use std::hash::Hash;
use std::num::NonZeroU64;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use anyhow::{anyhow, Result};
use ttl_cache::TtlCache;

use crate::ChronoCache;

const CACHE_CAPACITY: usize = 100000;

pub struct NativeChronoCache<K: Hash + Eq, V> {
  data: Arc<RwLock<TtlCache<K, V>>>,
}

impl<K: Hash + Eq, V: Clone> Default for NativeChronoCache<K, V> {
  fn default() -> Self {
    Self::new()
  }
}

impl<K: Hash + Eq, V: Clone> NativeChronoCache<K, V> {
  pub fn new() -> Self {
    NativeChronoCache { data: Arc::new(RwLock::new(TtlCache::new(CACHE_CAPACITY))) }
  }
}
impl<K: Hash + Eq + Send + Sync, V: Clone + Send + Sync> ChronoCache<K, V> for NativeChronoCache<K, V> {
  #[allow(clippy::too_many_arguments)]
  fn insert(&self, key: K, value: V, ttl: NonZeroU64) -> Result<V> {
    let ttl = Duration::from_millis(ttl.get());
    self
      .data
      .write()
      .unwrap()
      .insert(key, value, ttl)
      .ok_or(anyhow!("unable to insert value"))
  }

  fn get(&self, key: &K) -> Result<V> {
    self
      .data
      .read()
      .unwrap()
      .get(key)
      .cloned()
      .ok_or(anyhow!("key not found"))
  }
}
