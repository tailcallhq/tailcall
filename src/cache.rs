use std::hash::Hash;
use std::num::NonZeroU64;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use ttl_cache::TtlCache;

pub struct InMemoryCache<K: Hash + Eq, V> {
    data: Arc<RwLock<TtlCache<K, V>>>,
}

// TODO: take this from the user instead of hardcoding it
const CACHE_CAPACITY: usize = 100000;

impl<K: Hash + Eq, V: Clone> Default for InMemoryCache<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Hash + Eq, V: Clone> InMemoryCache<K, V> {
    pub fn new() -> Self {
        InMemoryCache { data: Arc::new(RwLock::new(TtlCache::new(CACHE_CAPACITY))) }
    }
}

#[async_trait::async_trait]
impl<K: Hash + Eq + Send + Sync, V: Clone + Send + Sync> crate::Cache for InMemoryCache<K, V> {
    type Key = K;
    type Value = V;
    #[allow(clippy::too_many_arguments)]
    async fn set<'a>(&'a self, key: K, value: V, ttl: NonZeroU64) -> anyhow::Result<()> {
        let ttl = Duration::from_millis(ttl.get());
        self.data.write().unwrap().insert(key, value, ttl);
        Ok(())
    }

    async fn get<'a>(&'a self, key: &'a K) -> anyhow::Result<Option<Self::Value>> {
        Ok(self.data.read().unwrap().get(key).cloned())
    }

    fn hit_rate(&self) -> Option<f64> {
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

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;
    use std::time::Duration;

    use crate::Cache;

    #[tokio::test]
    async fn test_native_chrono_cache_set_get() {
        let cache: crate::cache::InMemoryCache<u64, String> =
            crate::cache::InMemoryCache::default();
        let ttl = NonZeroU64::new(100).unwrap();
        assert_eq!(cache.get(&10).await.ok(), Some(None));

        cache.set(10, "hello".into(), ttl).await.unwrap();
        assert_eq!(cache.get(&10).await.ok(), Some(Some("hello".into())));

        cache.set(10, "bye".into(), ttl).await.ok();
        tokio::time::sleep(Duration::from_millis(ttl.get())).await;
        assert_eq!(cache.get(&10).await.ok(), Some(None));
    }
}
