use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::pin::Pin;
use std::sync::{Arc, RwLock};

use futures_util::Future;
use tokio::sync::broadcast;

pub trait CachingBehavior: Clone + Send + Sync + 'static {
    fn should_cache() -> bool;
}

#[derive(Clone)]
pub struct Cache;

impl CachingBehavior for Cache {
    fn should_cache() -> bool {
        true
    }
}

#[derive(Clone)]
pub struct NoCache;

impl CachingBehavior for NoCache {
    fn should_cache() -> bool {
        false
    }
}

/// A simple async cache that uses a `DashMap` to store the values.
pub struct AsyncCache<Key, Value, Error, Behavior>
where
    Behavior: CachingBehavior,
{
    cache: Arc<RwLock<HashMap<Key, CacheValue<Value, Error>>>>,
    _behavior: std::marker::PhantomData<Behavior>,
}

#[derive(Debug, Clone)]
pub enum CacheValue<Value, Error> {
    Pending(broadcast::Sender<Arc<Result<Value, Error>>>),
    Ready(Arc<Result<Value, Error>>),
}

impl<
        Key: Eq + Hash + Clone + Send + Sync,
        Value: Debug + Clone + Send + Sync,
        Error: Debug + Clone + Send + Sync,
        Behavior: CachingBehavior,
    > Default for AsyncCache<Key, Value, Error, Behavior>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<
        Key: Eq + Hash + Clone + Send + Sync,
        Value: Debug + Clone + Send + Sync,
        Error: Debug + Clone + Send + Sync,
        Behavior: CachingBehavior,
    > AsyncCache<Key, Value, Error, Behavior>
{
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            _behavior: std::marker::PhantomData,
        }
    }

    fn get_cache_value(&self, key: &Key) -> Option<CacheValue<Value, Error>> {
        self.cache.read().unwrap().get(key).cloned()
    }

    pub async fn get_or_eval<'a>(
        &self,
        key: Key,
        or_else: impl FnOnce() -> Pin<Box<dyn Future<Output = Result<Value, Error>> + 'a + Send>> + Send,
    ) -> Arc<Result<Value, Error>> {
        if Behavior::should_cache() {
            if let Some(cache_value) = self.get_cache_value(&key) {
                match cache_value {
                    CacheValue::Pending(tx) => tx.subscribe().recv().await.unwrap(),
                    CacheValue::Ready(value) => value,
                }
            } else {
                let (tx, _) = broadcast::channel(100);
                self.cache
                    .write()
                    .unwrap()
                    .insert(key.clone(), CacheValue::Pending(tx.clone()));
                let result = Arc::new(or_else().await);
                let mut guard = self.cache.write().unwrap();
                if let Some(cache_value) = guard.get_mut(&key) {
                    *cache_value = CacheValue::Ready(result.clone())
                }
                tx.send(result.clone()).ok();
                result
            }
        } else {
            self.load_without_cache(key, or_else).await
        }
    }

    async fn load_without_cache<'a>(
        &self,
        key: Key,
        func: impl FnOnce() -> Pin<Box<dyn Future<Output = Result<Value, Error>> + 'a + Send>> + Send,
    ) -> Arc<Result<Value, Error>> {
        if let Some(cache_value) = self.get_cache_value(&key) {
            match cache_value {
                CacheValue::Pending(tx) => tx.subscribe().recv().await.unwrap(),
                CacheValue::Ready(value) => {
                    self.cache.write().unwrap().remove(&key);
                    value
                }
            }
        } else {
            let (tx, _) = broadcast::channel(100);
            self.cache
                .write()
                .unwrap()
                .insert(key.clone(), CacheValue::Pending(tx.clone()));
            let result = Arc::new(func().await);
            self.cache.write().unwrap().remove(&key);
            tx.send(result.clone()).ok();
            result
        }
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
        let cache = AsyncCache::<i32, i32, String, Cache>::new();
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
        let cache = AsyncCache::<i32, i32, String, Cache>::new();
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
        let cache = AsyncCache::<i32, i32, String, Cache>::new();

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
        let cache = AsyncCache::<i32, String, String, Cache>::new();
        let actual = cache
            .get_or_eval(1, || Box::pin(async { Err("error".into()) }))
            .await;
        assert!(actual.is_err());
    }

    #[tokio::test]
    async fn test_with_multi_get_failure() {
        let cache = AsyncCache::<i32, i32, String, Cache>::new();
        let _ = cache
            .get_or_eval(1, || Box::pin(async { Err("error".into()) }))
            .await;

        let actual = cache.get_or_eval(1, || Box::pin(async { Ok(2) })).await;

        assert!(actual.is_err());
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let cache = Arc::new(AsyncCache::<i32, i32, String, Cache>::new());
        let key = 1;
        let value = 42;
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

        let results: Vec<_> = futures_util::future::join_all(handles)
            .await
            .into_iter()
            .map(|res| res.unwrap().as_ref().clone().unwrap())
            .collect();

        assert!(results.iter().all(|&v| v == value));
    }

    #[tokio::test]
    async fn test_no_cache_behavior() {
        let cache: Arc<AsyncCache<i32, String, String, NoCache>> = Arc::new(AsyncCache::new());

        // Test case where cache is empty
        let key1 = 1;
        let result1 = cache
            .get_or_eval(key1, || {
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
        let cache: Arc<AsyncCache<i32, String, String, NoCache>> = Arc::new(AsyncCache::new());
        let key = 1;

        // Spawning multiple tasks to simulate race conditions
        let mut handles = vec![];
        for i in 0..10 {
            let cache_clone = cache.clone();
            let key_clone = key;
            handles.push(tokio::spawn(async move {
                cache_clone
                    .get_or_eval(key_clone, || {
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
    #[tokio::test]
    async fn test_deadlock_scenario() {
        let cache = Arc::new(AsyncCache::<u64, String, String, NoCache>::new());
        let key = 1;

        let mut handles = Vec::new();

        // Spawn multiple tasks to simulate concurrent access
        for i in 0..100 {
            let cache = cache.clone();
            handles.push(tokio::spawn(async move {
                cache
                    .get_or_eval(key, || {
                        Box::pin(async move {
                            sleep(Duration::from_nanos(100)).await;
                            Ok(format!("value{}", i))
                        })
                    })
                    .await
            }));
        }

        // Wait for all tasks to complete
        let results = futures_util::future::join_all(handles).await;

        // Check results for any potential errors or deadlocks
        for (i, result) in results.into_iter().enumerate() {
            match result {
                Ok(res) => {
                    assert!(res.is_ok());
                }
                Err(e) => panic!("Task {}: Error: {:?}", i, e),
            }
        }
    }
}
