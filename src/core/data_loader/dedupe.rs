use std::collections::HashMap;
use std::hash::Hash;
use std::pin::Pin;
use std::sync::{Arc, RwLock};

use futures_util::Future;
use tokio::sync::broadcast;

pub struct Dedupe<Key, Value> {
    cache: Arc<RwLock<HashMap<Key, Step<Value>>>>,
    size: usize,
    persist: bool,
}

// TODO: Use Arc or something to make cloning faster
#[derive(Clone)]
enum Step<Value> {
    AwaitIO(broadcast::Sender<Value>),
    RunIO(broadcast::Sender<Value>),
    Ready(Value),
}

impl<K: Send + Sync + Eq + Hash + Clone, V: Send + Sync + Clone> Dedupe<K, V> {
    pub fn new(size: usize, persist: bool) -> Self {
        Self { cache: Arc::new(RwLock::new(HashMap::new())), size, persist }
    }

    fn cache_value(&self, key: &K) -> Step<V> {
        let guard = self.cache.read().unwrap();
        let cache_value = guard.get(key).cloned();
        drop(guard);
        if let Some(cache_value) = cache_value {
            cache_value
        } else {
            let mut guard = self.cache.write().unwrap();
            if let Some(cache_value) = guard.get(key) {
                cache_value.clone()
            } else {
                let (tx, _) = broadcast::channel(self.size);
                let cache_value = Step::AwaitIO(tx.clone());
                guard.insert(key.to_owned(), cache_value.clone());
                Step::RunIO(tx)
            }
        }
    }

    fn set_ready(&self, key: &K, value: V) {
        let mut guard = self.cache.write().unwrap();
        guard.insert(key.to_owned(), Step::Ready(value));
    }

    pub async fn dedupe(
        &self,
        key: &K,
        or_else: impl FnOnce() -> Pin<Box<dyn Future<Output = V> + Send>> + Send,
    ) -> V {
        match self.cache_value(key) {
            Step::RunIO(tx) => {
                let value = or_else().await;
                if self.persist {
                    self.set_ready(key, value.clone());
                }
                let _ = tx.send(value.clone());
                value
            }
            Step::AwaitIO(tx) => tx.subscribe().recv().await.unwrap(),
            Step::Ready(value) => value,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    // FIXME: Migrate tests from async_cache

    async fn compute_value(i: usize) -> String {
        println!("Should happen only once");
        format!("value_{}", i)
    }

    #[tokio::test(worker_threads = 16, flavor = "multi_thread")]
    async fn test_deadlock_scenario() {
        let cache = Arc::new(Dedupe::<u64, String>::new(1000, true));
        let key = 1;

        let mut handles = Vec::new();

        // Spawn multiple tasks to simulate concurrent access
        for i in 0..1000000 {
            let cache = cache.clone();
            let handle = tokio::task::spawn(async move {
                cache.dedupe(&key, || Box::pin(compute_value(i))).await
            });
            handles.push(handle);
        }

        // Await each task and check results for any potential errors or deadlocks
        for (i, handle) in handles.into_iter().enumerate() {
            let _ = handle.await.unwrap();
            // let expected = format!("value_{}", i);

            // assert_eq!(actual, expected);
        }
    }
}
