use std::collections::HashMap;
use std::hash::Hash;
use std::num::NonZeroU64;
use std::sync::Arc;

use tailcall::Cache;
use tokio::sync::RwLock;

/// tailcall Cache for Lambda
///
/// Lambda has no great way to access a KV cache, so this implementation just puts everything in a HashMap (without considering the ttl), under the assumption that the Lambda will not live long enough for the cache to get too big.
#[derive(Clone)]
pub struct LambdaCache<K, V> {
    data: Arc<RwLock<HashMap<K, V>>>,
}

impl<K: Hash + Eq + Send + Sync, V: Clone + Send + Sync> Default for LambdaCache<K, V> {
    fn default() -> Self {
        LambdaCache { data: Arc::new(RwLock::new(HashMap::new())) }
    }
}

impl<K: Hash + Eq + Send + Sync, V: Clone + Send + Sync> LambdaCache<K, V> {
    pub fn new() -> Self {
        Default::default()
    }
}

#[async_trait::async_trait]
impl<K: Hash + Eq + Send + Sync, V: Clone + Send + Sync> Cache for LambdaCache<K, V> {
    type Key = K;
    type Value = V;
    #[allow(clippy::too_many_arguments)]
    async fn set<'a>(&'a self, key: K, value: V, _ttl: NonZeroU64) -> anyhow::Result<()> {
        self.data.write().await.insert(key, value);

        Ok(())
    }

    async fn get<'a>(&'a self, key: &'a K) -> anyhow::Result<Option<Self::Value>> {
        Ok(self.data.read().await.get(key).cloned())
    }
}
