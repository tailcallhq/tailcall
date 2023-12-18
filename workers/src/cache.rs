use std::num::NonZeroUsize;
use std::ops::Add;
use std::time::Duration;

use lru::LruCache;
use wasm_timer::SystemTime;

struct CacheEntry {
  value: String,
  expire_time: SystemTime,
}

pub struct TTLCache {
  inner_cache: LruCache<String, CacheEntry>,
  default_ttl: Duration,
}

impl TTLCache {
  pub fn new(capacity: usize, default_ttl: u64) -> Self {
    TTLCache {
      inner_cache: LruCache::new(NonZeroUsize::new(capacity).unwrap()),
      default_ttl: Duration::from_secs(default_ttl),
    }
  }

  pub fn put(&mut self, key: String, value: String) {
    let expire_time = SystemTime::now().add(self.default_ttl);
    let entry = CacheEntry { value, expire_time };
    self.inner_cache.put(key, entry);
  }

  pub fn get(&mut self, key: &str) -> Option<String> {
    if let Some(entry) = self.inner_cache.get(key) {
      if entry.expire_time > SystemTime::now() {
        return Some(entry.value.clone());
      } else {
        self.inner_cache.pop(key);
      }
    }
    None
  }
}
