use std::fmt::Debug;
use std::hash::Hash;
use std::pin::Pin;
use std::sync::Arc;

use dashmap::DashMap;
use futures_util::Future;
use tokio::sync::broadcast;

/// A simple async cache that uses a `DashMap` to store the values.
pub struct AsyncCache<Key, Value, Error> {
    cache: Arc<DashMap<Key, CacheValue<Value, Error>>>,
}

#[derive(Clone)]
pub enum CacheValue<Value, Error> {
    Pending(broadcast::Sender<Arc<Result<Value, Error>>>),
    Ready(Arc<Result<Value, Error>>),
}

impl<
        Key: Eq + Hash + Clone + Send + Sync,
        Value: Debug + Clone + Send + Sync,
        Error: Debug + Clone + Send + Sync,
    > Default for AsyncCache<Key, Value, Error>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<
        Key: Eq + Hash + Clone + Send + Sync,
        Value: Debug + Clone + Send + Sync,
        Error: Debug + Clone + Send + Sync,
    > AsyncCache<Key, Value, Error>
{
    pub fn new() -> Self {
        Self { cache: Arc::new(DashMap::new()) }
    }

    fn get_cache_value(&self, key: &Key) -> Option<CacheValue<Value, Error>> {
        self.cache.get(key).map(|v| v.clone())
    }

    pub async fn get_or_eval<'a>(
        &self,
        key: Key,
        or_else: impl FnOnce() -> Pin<Box<dyn Future<Output = Result<Value, Error>> + 'a + Send>> + Send,
    ) -> Arc<Result<Value, Error>> {
        if let Some(cache_value) = self.get_cache_value(&key) {
            match cache_value {
                CacheValue::Pending(tx) => tx.subscribe().recv().await.unwrap(),
                CacheValue::Ready(value) => value,
            }
        } else {
            let (tx, _) = broadcast::channel(100);
            self.cache
                .insert(key.clone(), CacheValue::Pending(tx.clone()));
            let result = Arc::new(or_else().await);
            let mut guard = self
                .cache
                .entry(key)
                .or_insert(CacheValue::Pending(tx.clone()));
            *guard = CacheValue::Ready(result.clone());
            tx.send(result.clone()).ok();
            result
        }
    }

    pub async fn read_aside<'a>(
        &self,
        key: Key,
        func: impl FnOnce() -> Pin<Box<dyn Future<Output = Result<Value, Error>> + 'a + Send>>
            + 'a
            + Send,
    ) -> Arc<Result<Value, Error>> {
        if let Some(cache_value) = self.get_cache_value(&key) {
            match cache_value {
                CacheValue::Pending(tx) => tx.subscribe().recv().await.unwrap(),
                CacheValue::Ready(_) => {
                    let (tx, mut rx) = broadcast::channel(100);
                    self.cache
                        .insert(key.clone(), CacheValue::Pending(tx.clone()));
                    let result = Arc::new(func().await);
                    let mut guard = self
                        .cache
                        .entry(key)
                        .or_insert(CacheValue::Pending(tx.clone()));
                    *guard = CacheValue::Ready(result.clone());
                    tx.send(result.clone()).ok();
                    rx.recv().await.unwrap()
                }
            }
        } else {
            let (tx, mut rx) = broadcast::channel(100);
            self.cache
                .insert(key.clone(), CacheValue::Pending(tx.clone()));
            let result = Arc::new(func().await);
            let mut guard = self
                .cache
                .entry(key)
                .or_insert(CacheValue::Pending(tx.clone()));
            *guard = CacheValue::Ready(result.clone());
            tx.send(result.clone()).ok();
            rx.recv().await.unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn test_no_key() {
        let cache = AsyncCache::<i32, i32, String>::new();
        let actual = cache
            .get_or_eval(1, || Box::pin(async { Ok(1) }))
            .await
            .as_ref()
            .clone()
            .unwrap();
        assert_eq!(actual, 1);
    }

    #[tokio::test]
    async fn test_with_key() {
        let cache = AsyncCache::<i32, i32, String>::new();
        cache
            .get_or_eval(1, || Box::pin(async { Ok(1) }))
            .await
            .as_ref()
            .clone()
            .unwrap();

        let actual = cache
            .get_or_eval(1, || Box::pin(async { Ok(2) }))
            .await
            .as_ref()
            .clone()
            .unwrap();
        assert_eq!(actual, 1);
    }

    #[tokio::test]
    async fn test_with_multi_get() {
        let cache = AsyncCache::<i32, i32, String>::new();

        for i in 0..100 {
            cache
                .get_or_eval(1, || Box::pin(async move { Ok(i) }))
                .await
                .as_ref()
                .clone()
                .unwrap();
        }

        let actual = cache
            .get_or_eval(1, || Box::pin(async { Ok(2) }))
            .await
            .as_ref()
            .clone()
            .unwrap();
        assert_eq!(actual, 0);
    }

    #[tokio::test]
    async fn test_with_failure() {
        let cache = AsyncCache::<i32, String, String>::new();
        let actual = cache
            .get_or_eval(1, || Box::pin(async { Err("error".into()) }))
            .await;
        assert!(actual.is_err());
    }

    #[tokio::test]
    async fn test_with_multi_get_failure() {
        let cache = AsyncCache::<i32, i32, String>::new();
        let _ = cache
            .get_or_eval(1, || Box::pin(async { Err("error".into()) }))
            .await;

        let actual = cache.get_or_eval(1, || Box::pin(async { Ok(2) })).await;

        assert!(actual.is_err());
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let cache = Arc::new(AsyncCache::<i32, i32, String>::new());
        let key = 1;
        let value = 42;
        // Simulate concurrent access by spawning multiple tasks.
        let handles: Vec<_> = (0..100)
            .map(|_| {
                let cache = cache.clone();
                tokio::spawn(async move {
                    cache
                        .get_or_eval(key, || Box::pin(async { Ok(value) }))
                        .await
                })
            })
            .collect();

        // Await all spawned tasks and collect their results.
        let results: Vec<_> = futures_util::future::join_all(handles)
            .await
            .into_iter()
            .map(|res| res.unwrap().as_ref().clone().unwrap()) // Unwrap the Result from the join, and the Result from get_or_eval
            .collect();

        // Check that all tasks received the correct value.
        assert!(results.iter().all(|&v| v == value));

        // Optionally, verify that the value was computed only once.
        // This might require additional instrumentation in the cache or the
        // computation function.
    }
}
