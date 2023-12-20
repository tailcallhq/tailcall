use std::hash::Hash;
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
  pub fn insert(&self, key: K, value: V, ttl: u64) -> Option<V> {
    let ttl = Duration::from_secs(ttl);
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

#[cfg(test)]
mod tests {
  use std::thread;
  use std::time::Duration;

  use crate::chrono_cache::ChronoCache;

  #[test]
  fn test_res_cache_insert() {
    let ttl = 1;
    let cache = ChronoCache::new();
    let key = 10;
    let value = "value";
    cache.insert(key, value, ttl);
    assert_eq!(cache.get(&key), Some("value"));
  }

  #[test]
  fn test_res_cache_ttl() {
    let ttl = 1;
    let cache = ChronoCache::new();
    let key = 10;
    let value = "value";
    cache.insert(key, value, ttl);
    assert_eq!(cache.get(&key), Some("value"));
    thread::sleep(Duration::from_secs(1));
    assert_eq!(
      cache.get(&key),
      None,
      "cache shouldn't contain the value after `CacheRules.max_age` secs have passed"
    );
  }

  #[test]
  fn test_res_cache_remove() {
    let ttl = 100;
    let cache = ChronoCache::new();
    let key = 10;
    let value = "value";
    cache.insert(key, value, ttl);
    assert_eq!(cache.get(&key), Some("value"));
    cache.remove(&key);
    assert_eq!(
      cache.get(&key),
      None,
      "cache shouldn't contain the value after `CacheRules.max_age` secs have passed"
    );
  }
}
