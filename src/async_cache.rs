use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::pin::Pin;
use std::sync::{Arc, RwLock};

use futures_util::Future;
use tokio::sync::broadcast::Sender;

/// A simple async cache that uses a `HashMap` to store the values.
pub struct AsyncCache<Key, Value> {
    cache: Arc<RwLock<HashMap<Key, CacheValue<Value>>>>,
}

#[derive(Clone)]
pub enum CacheValue<Value> {
    Pending(Sender<Result<Value, String>>),
    Ready(Result<Value, String>),
}

impl<Key: Eq + Hash + Send + Clone, Value: Debug + Clone + Send> Default
    for AsyncCache<Key, Value>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Key: Eq + Hash + Send + Clone, Value: Debug + Clone + Send> AsyncCache<Key, Value> {
    pub fn new() -> Self {
        Self { cache: Arc::new(RwLock::new(HashMap::new())) }
    }

    fn get_cache_value(&self, key: &Key) -> Option<CacheValue<Value>> {
        self.cache.read().unwrap().get(key).cloned()
    }

    pub async fn get_or_eval<'a>(
        &self,
        key: Key,
        or_else: impl FnOnce() -> Pin<Box<dyn Future<Output = anyhow::Result<Value>> + 'a + Send>>
            + Send,
    ) -> anyhow::Result<Value> {
        if let Some(cache_value) = self.get_cache_value(&key) {
            match cache_value {
                CacheValue::Pending(tx) => tx.subscribe().recv().await?,
                CacheValue::Ready(value) => value,
            }
            .map_err(|err| anyhow::anyhow!(err))
        } else {
            let (tx, _) = tokio::sync::broadcast::channel(100);
            self.cache
                .write()
                .unwrap()
                .insert(key.clone(), CacheValue::Pending(tx.clone()));
            let result = or_else().await;
            let (cloned, original) = match result {
                Ok(value) => (Ok(value.clone()), Ok(value)),
                Err(err) => (Err(err.to_string()), Err(err)),
            };
            let mut guard = self.cache.write().unwrap();
            if let Some(cache_value) = guard.get_mut(&key) {
                *cache_value = CacheValue::Ready(cloned.clone())
            }
            tx.send(cloned.clone()).ok();
            original
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn test_no_key() {
        let cache = AsyncCache::new();
        let actual = cache
            .get_or_eval(1, || Box::pin(async { Ok(1) }))
            .await
            .unwrap();
        assert_eq!(actual, 1);
    }

    #[tokio::test]
    async fn test_with_key() {
        let cache = AsyncCache::new();
        cache
            .get_or_eval(1, || Box::pin(async { Ok(1) }))
            .await
            .unwrap();

        let actual = cache
            .get_or_eval(1, || Box::pin(async { Ok(2) }))
            .await
            .unwrap();
        assert_eq!(actual, 1);
    }

    #[tokio::test]
    async fn test_with_multi_get() {
        let cache = AsyncCache::new();

        for i in 0..100 {
            cache
                .get_or_eval(1, || Box::pin(async move { Ok(i) }))
                .await
                .unwrap();
        }

        let actual = cache
            .get_or_eval(1, || Box::pin(async { Ok(2) }))
            .await
            .unwrap();
        assert_eq!(actual, 0);
    }

    #[tokio::test]
    async fn test_with_failure() {
        let cache = AsyncCache::<i32, String>::new();
        let actual = cache
            .get_or_eval(1, || Box::pin(async { Err(anyhow::anyhow!("error")) }))
            .await;
        assert!(actual.is_err());
    }

    #[tokio::test]
    async fn test_with_multi_get_failure() {
        let cache = AsyncCache::<i32, i32>::new();
        let _ = cache
            .get_or_eval(1, || Box::pin(async { Err(anyhow::anyhow!("error")) }))
            .await;

        let actual = cache.get_or_eval(1, || Box::pin(async { Ok(2) })).await;

        assert!(actual.is_err());
    }
}
