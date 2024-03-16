use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use futures_util::Future;
use tokio::sync::oneshot::Receiver;

/// A simple async cache that uses a `HashMap` to store the values.
pub struct AsyncCache<Key, Value> {
    cache: Arc<Mutex<HashMap<Key, (Arc<Mutex<Option<Value>>>, Receiver<Value>)>>>,
}

impl<Key: Eq + Hash, Value: Debug + Clone> AsyncCache<Key, Value> {
    pub fn new() -> Self {
        Self { cache: Arc::new(Mutex::new(HashMap::new())) }
    }

    pub async fn get_or_else(
        &self,
        key: Key,
        or_else: impl FnOnce() -> Pin<Box<dyn Future<Output = anyhow::Result<Value>>>>,
    ) -> anyhow::Result<Value> {
        let mut cache = self.cache.lock().unwrap();
        if let Some((value, rx)) = cache.get_mut(&key) {
            if let Some(value) = value.lock().unwrap().as_ref() {
                Ok(value.clone())
            } else {
                let value = rx.await?;
                Ok(value.clone())
            }
        } else {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let last_value = Arc::new(Mutex::new(None));
            cache.insert(key, (last_value.clone(), rx));
            let value = or_else().await?;
            last_value.lock().unwrap().replace(value.clone());
            tx.send(value.clone()).unwrap();
            Ok(value)
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
            .get_or_else(1, || Box::pin(async { Ok(1) }))
            .await
            .unwrap();
        assert_eq!(actual, 1);
    }

    #[tokio::test]
    async fn test_with_key() {
        let cache = AsyncCache::new();
        cache
            .get_or_else(1, || Box::pin(async { Ok(1) }))
            .await
            .unwrap();

        let actual = cache
            .get_or_else(1, || Box::pin(async { Ok(2) }))
            .await
            .unwrap();
        assert_eq!(actual, 1);
    }

    #[tokio::test]
    async fn test_with_multi_get() {
        let cache = AsyncCache::new();

        for i in 0..100 {
            cache
                .get_or_else(1, || Box::pin(async move { Ok(i) }))
                .await
                .unwrap();
        }

        let actual = cache
            .get_or_else(1, || Box::pin(async { Ok(2) }))
            .await
            .unwrap();
        assert_eq!(actual, 0);
    }
}
