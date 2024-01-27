use std::collections::HashMap;
use std::hash::Hash;
use std::num::NonZeroU64;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use ttl_cache::TtlCache;

pub struct Cache<K, V>(Mutex<HashMap<K, V>>);

impl<K, V> Cache<K, V>
where
    K: std::cmp::Eq,
    K: PartialEq,
    K: core::hash::Hash,
    V: std::clone::Clone,
{
    pub fn get(&self, key: &K) -> Option<V> {
        self.0.lock().unwrap().get(key).cloned()
    }

    pub fn insert(&self, key: K, value: V) {
        self.0.lock().unwrap().insert(key, value);
    }

    pub fn empty() -> Self {
        Self(Mutex::new(HashMap::new()))
    }
}

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

#[async_trait::async_trait]
impl<K: Hash + Eq + Send + Sync, V: Clone + Send + Sync> crate::Cache for NativeChronoCache<K, V> {
    type Key = K;
    type Value = V;
    #[allow(clippy::too_many_arguments)]
    async fn set<'a>(
        &'a self,
        key: K,
        value: V,
        ttl: NonZeroU64,
    ) -> anyhow::Result<Option<Self::Value>> {
        let ttl = Duration::from_millis(ttl.get());
        Ok(self.data.write().unwrap().insert(key, value, ttl))
    }

    async fn get<'a>(&'a self, key: &'a K) -> anyhow::Result<Option<Self::Value>> {
        Ok(self.data.read().unwrap().get(key).cloned())
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;
    use std::time::Duration;

    use crate::Cache;

    #[tokio::test]
    async fn test_native_chrono_cache_set_get() {
        let cache: crate::cache::NativeChronoCache<u64, String> =
            crate::cache::NativeChronoCache::new();
        let ttl = NonZeroU64::new(100).unwrap();
        assert_eq!(cache.get(&10).await.ok(), Some(None));
        assert_eq!(cache.set(10, "hello".into(), ttl).await.ok(), Some(None));
        assert_eq!(cache.get(&10).await.ok(), Some(Some("hello".into())));
        assert_eq!(
            cache.set(10, "bye".into(), ttl).await.ok(),
            Some(Some("hello".into()))
        );
        tokio::time::sleep(Duration::from_millis(ttl.get())).await;
        assert_eq!(cache.get(&10).await.ok(), Some(None));
    }
}
