use std::hash::Hash;
use std::num::NonZeroU64;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use ttl_cache::TtlCache;

const CACHE_CAPACITY: usize = 100000;

#[derive(Clone)]
pub struct ChronoCache<K: Hash + Eq, V> {
  data: Arc<RwLock<TtlCache<K, V>>>,
}

impl<K: Hash + Eq, V> std::fmt::Debug for ChronoCache<K, V> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "ResCache {{capacity: {:?}}}", self.data.read().unwrap().capacity())
  }
}

impl<K: Hash + Eq, V: Clone> Default for ChronoCache<K, V> {
  fn default() -> Self {
    Self::new()
  }
}

impl<K: Hash + Eq, V: Clone> ChronoCache<K, V> {
  pub fn new() -> Self {
    ChronoCache { data: Arc::new(RwLock::new(TtlCache::new(CACHE_CAPACITY))) }
  }

  #[allow(clippy::too_many_arguments)]
  pub fn insert(&self, key: K, value: V, ttl: NonZeroU64) -> Option<V> {
    let ttl = Duration::from_secs(ttl.get());
    self.data.write().unwrap().insert(key, value, ttl)
  }

  pub fn get(&self, key: &K) -> Option<V> {
    self.data.read().unwrap().get(key).cloned()
  }

  #[allow(dead_code)]
  pub fn remove(&self, key: &K) -> Option<V> {
    self.data.write().unwrap().remove(key)
  }
}
