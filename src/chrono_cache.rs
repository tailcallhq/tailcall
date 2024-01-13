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
    let ttl = Duration::from_millis(ttl.get());
    self.data.write().unwrap().insert(key, value, ttl)
  }

  pub fn get(&self, key: &K) -> Option<V> {
    self.data.read().unwrap().get(key).cloned()
  }

  pub fn hit_rate(&self) -> Option<f64> {
    let cache = self.data.read().unwrap();
    let hits = cache.hit_count();
    let misses = cache.miss_count();
    drop(cache);

    if hits + misses > 0 {
      return Some(hits as f64 / (hits + misses) as f64);
    }

    None
  }
}
