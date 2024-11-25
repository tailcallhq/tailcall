use std::hash::Hash;
use std::num::NonZeroU64;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use ttl_cache::TtlCache;

use super::error::Result;

pub struct InMemoryCache<K: Hash + Eq, V> {
    data: Arc<RwLock<TtlCache<K, V>>>,
    hits: AtomicUsize,
    miss: AtomicUsize,
}

impl<K: Hash + Eq, V: Clone> Default for InMemoryCache<K, V> {
    fn default() -> Self {
        Self::new(100000)
    }
}

impl<K: Hash + Eq, V: Clone> InMemoryCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        InMemoryCache {
            data: Arc::new(RwLock::new(TtlCache::new(capacity))),
            hits: AtomicUsize::new(0),
            miss: AtomicUsize::new(0),
        }
    }
}

#[async_trait::async_trait]
impl<K: Hash + Eq + Send + Sync, V: Clone + Send + Sync> crate::core::Cache
    for InMemoryCache<K, V>
{
    type Key = K;
    type Value = V;
    #[allow(clippy::too_many_arguments)]
    async fn set<'a>(&'a self, key: K, value: V, ttl: NonZeroU64) -> Result<()> {
        let ttl = Duration::from_millis(ttl.get());
        self.data.write().unwrap().insert(key, value, ttl);
        Ok(())
    }

    async fn get<'a>(&'a self, key: &'a K) -> Result<Option<Self::Value>> {
        let val = self.data.read().unwrap().get(key).cloned();
        if val.is_some() {
            self.hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.miss.fetch_add(1, Ordering::Relaxed);
        }
        Ok(val)
    }

    fn hit_rate(&self) -> Option<f64> {
        let cache = self.data.read().unwrap();
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.miss.load(Ordering::Relaxed);

        drop(cache);

        if hits + misses > 0 {
            return Some(hits as f64 / (hits + misses) as f64);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;
    use std::time::Duration;

    use crate::core::Cache;

    #[tokio::test]
    async fn test_native_chrono_cache_set_get() {
        let cache: crate::core::cache::InMemoryCache<u64, String> =
            crate::core::cache::InMemoryCache::default();
        let ttl = NonZeroU64::new(100).unwrap();
        assert_eq!(cache.get(&10).await.ok(), Some(None));

        cache.set(10, "hello".into(), ttl).await.unwrap();
        assert_eq!(cache.get(&10).await.ok(), Some(Some("hello".into())));

        cache.set(10, "bye".into(), ttl).await.ok();
        tokio::time::sleep(Duration::from_millis(ttl.get())).await;
        assert_eq!(cache.get(&10).await.ok(), Some(None));
    }
}
