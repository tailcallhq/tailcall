use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::pin::Pin;
use std::sync::{Arc, Mutex as StdMutex, RwLock};
use std::task::{Context, Poll};

use futures_util::{Future, FutureExt};
use tokio::sync::broadcast::Sender;
use tokio::sync::Mutex;

/// A simple async cache that uses a `HashMap` to store the values.
pub struct AsyncCache<Key, Value, Error> {
    cache: Arc<RwLock<HashMap<Key, CacheValue<Value, Error>>>>,
}

#[derive(Clone)]
pub struct AsyncCache1<'a, Key: Clone, Value: Clone, Error: Clone> {
    cache: Arc<
        StdMutex<
            HashMap<
                Key,
                Arc<Mutex<Pin<Box<dyn Future<Output = Result<Value, Error>> + 'a + Send>>>>,
            >,
        >,
    >,
}

#[derive(Clone)]
pub enum CacheValue<Value, Error> {
    Pending(Sender<Result<Value, Error>>),
    Ready(Result<Value, Error>),
}

impl<Key: Eq + Hash + Send + Clone, Value: Debug + Clone + Send, Error: Debug + Clone + Send>
    Default for AsyncCache<Key, Value, Error>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Key: Eq + Hash + Send + Clone, Value: Debug + Clone + Send, Error: Debug + Clone + Send>
    AsyncCache<Key, Value, Error>
{
    pub fn new() -> Self {
        Self { cache: Arc::new(RwLock::new(HashMap::new())) }
    }

    fn get_cache_value(&self, key: &Key) -> Option<CacheValue<Value, Error>> {
        self.cache.read().unwrap().get(key).cloned()
    }

    pub async fn get_or_eval<'a>(
        &self,
        key: Key,
        or_else: impl FnOnce() -> Pin<Box<dyn Future<Output = Result<Value, Error>> + 'a + Send>> + Send,
    ) -> Result<Value, Error> {
        if let Some(cache_value) = self.get_cache_value(&key) {
            match cache_value {
                CacheValue::Pending(tx) => tx.subscribe().recv().await.unwrap(),
                CacheValue::Ready(value) => value,
            }
        } else {
            let (tx, _) = tokio::sync::broadcast::channel(100);
            self.cache
                .write()
                .unwrap()
                .insert(key.clone(), CacheValue::Pending(tx.clone()));
            let result = or_else().await;
            let mut guard = self.cache.write().unwrap();
            if let Some(cache_value) = guard.get_mut(&key) {
                *cache_value = CacheValue::Ready(result.clone())
            }
            tx.send(result.clone()).ok();
            result
        }
    }
}

impl<
        'a,
        Key: Eq + Hash + Send + Clone + Debug + Unpin + 'a,
        Value: Debug + Clone + Send + Unpin + 'a,
        Error: Debug + Clone + Send + Unpin + 'a,
    > Default for AsyncCache1<'a, Key, Value, Error> {
    fn default() -> Self {
        Self::new()
    }
}

impl<
        'a,
        Key: Eq + Hash + Send + Clone + Debug + Unpin + 'a,
        Value: Debug + Clone + Send + Unpin + 'a,
        Error: Debug + Clone + Send + Unpin + 'a,
    > AsyncCache1<'a, Key, Value, Error>
{
    pub fn new() -> Self {
        Self { cache: Arc::new(StdMutex::new(HashMap::new())) }
    }

    pub fn get_or_eval(
        &self,
        key: Key,
        or_else: Pin<Box<dyn Future<Output = Result<Value, Error>> + 'a + Send>>,
    ) -> Pin<Box<dyn Future<Output = Result<Value, Error>> + 'a + Send>> {
        let fut = if let Some(fut) = self.cache.lock().unwrap().get(&key).cloned() {
            fut
        } else {
            let fut = Arc::new(Mutex::new(or_else));
            self.cache.lock().unwrap().insert(key.clone(), fut.clone());
            fut
        };
        Box::pin(GetOrEvalFuture { inner: GetOrEvalFutureInner::Pending { key, fut } })
    }
}

pub struct GetOrEvalFuture<'a, Key, Value> {
    inner: GetOrEvalFutureInner<'a, Key, Value>,
}

enum GetOrEvalFutureInner<'a, Key, Value> {
    Pending {
        key: Key,
        fut: Arc<Mutex<Pin<Box<dyn Future<Output = Value> + 'a + Send>>>>,
    },
    Ready(Value),
}

impl<'a, Key: Clone + Unpin + Debug, Value: Clone + Unpin> Future
    for GetOrEvalFuture<'a, Key, Value>
{
    type Output = Value;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        println!("Polling for key");
        let fut = match &self.inner {
            GetOrEvalFutureInner::Pending { key: _, fut } => fut.clone(),
            GetOrEvalFutureInner::Ready(value) => return Poll::Ready(value.clone()),
        };

        let result = std::pin::pin!(fut.lock()).poll(cx).map(|mut guard| {
            let result = std::pin::pin!(&mut *guard).poll(cx);
            drop(guard);
            result
        });

        match result {
            Poll::Ready(Poll::Ready(value)) => {
                self.get_mut().inner = GetOrEvalFutureInner::Ready(value.clone());
                Poll::Ready(value.clone())
            }
            Poll::Ready(Poll::Pending) => Poll::Pending,
            Poll::Pending => Poll::Pending,
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
            .unwrap();
        assert_eq!(actual, 1);
    }

    #[tokio::test]
    async fn test_with_key() {
        let cache = AsyncCache::<i32, i32, String>::new();
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
        let cache = AsyncCache::<i32, i32, String>::new();

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
}
