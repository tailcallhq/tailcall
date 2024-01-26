use std::hash::Hash;
use std::num::NonZeroU64;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use ttl_cache::TtlCache;

use crate::Cache;

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
impl<K: Hash + Eq + Send + Sync, V: Clone + Send + Sync> Cache for NativeChronoCache<K, V> {
    type Key = K;
    type Value = V;
    #[allow(clippy::too_many_arguments)]
    async fn set<'a>(&'a self, key: K, value: V, ttl: NonZeroU64) -> anyhow::Result<()> {
        let ttl = Duration::from_millis(ttl.get());
        self.data.write().unwrap().insert(key, value, ttl);
        Ok(())
    }

    async fn get<'a>(&'a self, key: &'a K) -> anyhow::Result<V> {
        self.data
            .read()
            .unwrap()
            .get(key)
            .cloned()
            .ok_or(anyhow::anyhow!("Key not found"))
    }
}
