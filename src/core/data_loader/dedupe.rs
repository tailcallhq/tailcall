use std::collections::HashMap;
use std::hash::Hash;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use futures_util::Future;
use tokio::sync::broadcast;

pub struct Dedupe<Key, Value> {
    cache: Arc<Mutex<HashMap<Key, State<Value>>>>,
    size: usize,
    persist: bool,
}

enum State<Value> {
    Value(Value),
    Send(broadcast::Sender<Value>),
}

enum Step<Value> {
    Value(Value),
    Recv(broadcast::Receiver<Value>),
    Send(broadcast::Sender<Value>),
}

// TODO: Use Arc or something to make cloning faster
impl<K: Send + Sync + Eq + Hash + Clone, V: Send + Sync + Clone> Dedupe<K, V> {
    pub fn new(size: usize, persist: bool) -> Self {
        Self { cache: Arc::new(Mutex::new(HashMap::new())), size, persist }
    }

    pub async fn dedupe(
        &self,
        key: &K,
        or_else: impl FnOnce() -> Pin<Box<dyn Future<Output = V> + Send>> + Send,
    ) -> V {
        match self.step(key) {
            Step::Value(value) => value,
            Step::Recv(mut rx) => rx.recv().await.unwrap(),
            Step::Send(tx) => {
                let value = or_else().await;
                let mut guard = self.cache.lock().unwrap();
                if self.persist {
                    guard.insert(key.to_owned(), State::Value(value.clone()));
                } else {
                    guard.remove(key);
                }
                let _ = tx.send(value.clone());
                value
            }
        }
    }

    fn step(&self, key: &K) -> Step<V> {
        let mut this = self.cache.lock().unwrap();
        match this.get(key) {
            Some(state) => match state {
                State::Value(value) => Step::Value(value.clone()),
                State::Send(tx) => Step::Recv(tx.subscribe()),
            },
            None => {
                let (tx, _) = broadcast::channel(self.size);
                this.insert(key.to_owned(), State::Send(tx.clone()));
                Step::Send(tx.clone())
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use std::time::Duration;

    use tokio::join;
    use tokio::time::sleep;

    use super::*;

    // FIXME: Migrate tests from async_cache

    #[tokio::test]
    async fn test_no_key() {
        let cache = Arc::new(Dedupe::<u64, u64>::new(1000, true));
        let actual = cache.dedupe(&1, || Box::pin(async { 1 })).await;
        pretty_assertions::assert_eq!(actual, 1);
    }

    #[tokio::test]
    async fn test_with_key() {
        let cache = Arc::new(Dedupe::<u64, u64>::new(1000, true));
        cache.dedupe(&1, || Box::pin(async { 1 })).await;

        let actual = cache.dedupe(&1, || Box::pin(async { 2 })).await;
        pretty_assertions::assert_eq!(actual, 1);
    }

    #[tokio::test]
    async fn test_with_multi_get() {
        let cache = Arc::new(Dedupe::<u64, u64>::new(1000, true));

        for i in 0..100 {
            cache.dedupe(&1, || Box::pin(async move { i })).await;
        }

        let actual = cache.dedupe(&1, || Box::pin(async { 2 })).await;
        pretty_assertions::assert_eq!(actual, 0);
    }

    #[tokio::test]
    async fn test_with_multi_async_get() {
        let cache = Arc::new(Dedupe::<u64, u64>::new(1000, true));

        let a = cache.dedupe(&1, || {
            Box::pin(async move {
                sleep(Duration::from_millis(1)).await;
                1
            })
        });
        let b = cache.dedupe(&1, || {
            Box::pin(async move {
                sleep(Duration::from_millis(1)).await;
                2
            })
        });
        let (a, b) = join!(a, b);

        pretty_assertions::assert_eq!(a, b);
    }

    async fn compute_value(i: usize) -> String {
        sleep(Duration::from_millis(1)).await;
        format!("value_{}", i)
    }

    #[tokio::test(worker_threads = 16, flavor = "multi_thread")]
    async fn test_deadlock_scenario() {
        let _ = tracing_subscriber::fmt();
        let cache = Arc::new(Dedupe::<u64, String>::new(1000, false));
        let key = 1;

        let mut handles = Vec::new();

        // Spawn multiple tasks to simulate concurrent access
        for i in 0..100000 {
            let cache = cache.clone();
            let handle = tokio::task::spawn(async move {
                let result = cache.dedupe(&key, || Box::pin(compute_value(i))).await;
                (i, result)
            });
            handles.push(handle);
        }

        // Await each task and check results for any potential errors or deadlocks
        for (i, handle) in handles.into_iter().enumerate() {
            let (_, actual) = handle.await.unwrap();
            let expected = format!("value_{}", i);

            // FIXME: Insert a proper assertion test
            assert_eq!(actual, expected);
        }
    }
}
