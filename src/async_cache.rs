
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::pin::Pin;
use std::sync::{Arc, RwLock};

use futures_util::Future;
use tokio::sync::broadcast::{Receiver, Sender};

/// A simple async cache that uses a `HashMap` to store the values.
pub struct AsyncCache<Key, Value> {
    cache: Arc<RwLock<HashMap<Key, (Arc<RwLock<Option<Value>>>, Sender<Value>, Receiver<Value>)>>>,
}

impl<Key: Eq + Hash + Send + Clone, Value: Debug + Clone + Send> Default for AsyncCache<Key, Value> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Key: Eq + Hash + Send + Clone, Value: Debug + Clone + Send> AsyncCache<Key, Value> {
    pub fn new() -> Self {
        Self { cache: Arc::new(RwLock::new(HashMap::new())) }
    }

    fn get_value(&self, key: &Key) -> Option<Value> {
        let guard = self.cache.read().unwrap();
        if let Some((value, _tx, _rx)) = guard.get(key) {
            let value = value.read().unwrap().clone();
            value
        } else {
            None
        }
    }

    fn get_tx(&self, key: &Key) -> Option<Sender<Value>> {
        let guard = self.cache.read().unwrap();
        if let Some((_value, tx, _rx)) = guard.get(key) {
            Some(tx.clone())
        } else {
            None
        }
    }

    fn set_key(&self, key: Key) {
        let (tx, rx) = tokio::sync::broadcast::channel(100);
        let last_value = Arc::new(RwLock::new(None));
        let mut guard = self.cache.write().unwrap();
        guard.insert(key, (last_value.clone(), tx, rx));
    }

    fn set_value(&self, key: &Key, value: Value) {
        let mut guard = self.cache.write().unwrap();
        if let Some((last_value, _, _)) = guard.get_mut(key) {
            last_value.write().unwrap().replace(value);
        }
    }

    pub async fn get_or_eval<'a>(
        &'a self,
        key: Key,
        or_else: impl FnOnce() -> Pin<Box<dyn Future<Output = anyhow::Result<Value>> + 'a + Send>>
            + Send,
    ) -> anyhow::Result<Value> {
        if let Some(value) = self.get_value(&key) {
            Ok(value.clone())
        } else if let Some(tx) = self.get_tx(&key) {
            let value = tx.subscribe().recv().await?;
            Ok(value.clone())
        } else {
            self.set_key(key.clone());

            let value = or_else().await?;
            self.set_value(&key, value.clone());
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
}
