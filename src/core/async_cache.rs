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
            let (tx, _) = broadcast::channel(10000);
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

    pub async fn load_without_cache<'a>(
        &self,
        key: Key,
        func: impl FnOnce() -> Pin<Box<dyn Future<Output = Result<Value, Error>> + 'a + Send>>
            + 'a
            + Send,
    ) -> Arc<Result<Value, Error>> {
        if let Some(CacheValue::Pending(tx)) = self.get_cache_value(&key) {
            // Subscribe to the broadcast channel and wait for the result
            return tx.subscribe().recv().await.unwrap();
        }

        // Create a new broadcast channel
        let (tx, _) = broadcast::channel(10000);
        // Insert a pending state with the broadcast sender into the cache
        self.cache
            .insert(key.clone(), CacheValue::Pending(tx.clone()));

        // Perform the async operation
        let result = Arc::new(func().await);
        // Remove the pending state from the cache
        self.cache.remove(&key);
        // Notify all subscribers of the result
        tx.send(result.clone()).ok();

        result
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures_util::future;
    use pretty_assertions::assert_eq;
    use tokio::time::sleep;

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

    #[tokio::test]
    async fn test_load_without_cache() {
        let cache = Arc::new(AsyncCache::new());

        // Test case where cache is empty
        let key1 = 1;
        let result1 = cache
            .load_without_cache(key1, || {
                Box::pin(async { Ok("value1".to_string()) })
                    as Pin<Box<dyn Future<Output = Result<String, String>> + Send>>
            })
            .await;

        assert_eq!(*result1, Ok("value1".to_string()));

        // Test case where cache already has a pending value
        let key2 = 2;
        let cache_clone = cache.clone();
        let handle1 = tokio::spawn(async move {
            cache_clone
                .load_without_cache(key2, || {
                    Box::pin(async {
                        sleep(Duration::from_millis(100)).await;
                        Ok("value2".to_string())
                    })
                        as Pin<Box<dyn Future<Output = Result<String, String>> + Send>>
                })
                .await
        });

        let cache_clone = cache.clone();
        let handle2 = tokio::spawn(async move {
            cache_clone
                .load_without_cache(key2, || {
                    Box::pin(async {
                        sleep(Duration::from_millis(200)).await;
                        Ok("value2".to_string())
                    })
                        as Pin<Box<dyn Future<Output = Result<String, String>> + Send>>
                })
                .await
        });

        let result2_1 = handle1.await.unwrap();
        let result2_2 = handle2.await.unwrap();

        assert_eq!(result2_1, result2_2);
        assert_eq!(*result2_1, Ok("value2".to_string()));

        // Test case where the async function returns an error
        let key3 = 3;
        let result3 = cache
            .load_without_cache(key3, || {
                Box::pin(async { Err("failed".to_string()) })
                    as Pin<Box<dyn Future<Output = Result<String, String>> + Send>>
            })
            .await;

        assert_eq!(*result3, Err("failed".to_string()));
    }

    #[tokio::test]
    async fn test_load_without_cache_race_condition() {
        let cache = Arc::new(AsyncCache::new());
        let key = 1;

        // Spawning multiple tasks to simulate race conditions
        let mut handles = vec![];
        for i in 0..10 {
            let cache_clone = cache.clone();
            let key_clone = key;
            handles.push(tokio::spawn(async move {
                cache_clone
                    .load_without_cache(key_clone, || {
                        Box::pin(async {
                            sleep(Duration::from_millis(50)).await;
                            Ok(format!("value{}", i))
                        })
                            as Pin<Box<dyn Future<Output = Result<String, String>> + Send>>
                    })
                    .await
            }));
        }

        // Collect all results
        let results: Vec<_> = future::join_all(handles)
            .await
            .into_iter()
            .map(|h| h.unwrap())
            .collect();
        let first_result = results.first().unwrap().clone();

        // Ensure all results are the same to confirm the cache behavior
        for result in results {
            assert_eq!(result, first_result);
        }
    }
}
