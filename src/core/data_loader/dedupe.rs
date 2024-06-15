use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

use futures_util::Future;
use tokio::sync::broadcast;

pub trait Key: Send + Sync + Eq + Hash + Clone {}
impl<A: Send + Sync + Eq + Hash + Clone> Key for A {}

pub trait Value: Send + Sync + Clone {}
impl<A: Send + Sync + Clone> Value for A {}

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

impl<K: Key, V: Value> Dedupe<K, V> {
    pub fn new(size: usize, persist: bool) -> Self {
        Self { cache: Arc::new(Mutex::new(HashMap::new())), size, persist }
    }

    pub async fn dedupe<'a, Fn, Fut>(&'a self, key: &'a K, or_else: Fn) -> V
    where
        Fn: FnOnce() -> Fut,
        Fut: Future<Output = V>,
    {
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

pub struct DedupeResult<K, V, E>(Dedupe<K, Result<V, E>>);

impl<K: Key, V: Value, E: Value> DedupeResult<K, V, E> {
    pub fn new(persist: bool) -> Self {
        Self(Dedupe::new(1, persist))
    }
}

impl<K: Key, V: Value, E: Value> DedupeResult<K, V, E> {
    pub async fn dedupe<'a, Fn, Fut>(&'a self, key: &'a K, or_else: Fn) -> Result<V, E>
    where
        Fn: FnOnce() -> Fut,
        Fut: Future<Output = Result<V, E>>,
    {
        self.0.dedupe(key, or_else).await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use tokio::join;
    use tokio::time::sleep;

    use super::*;

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

    async fn compute_value(counter: Arc<AtomicUsize>) -> String {
        counter.fetch_add(1, Ordering::SeqCst);
        sleep(Duration::from_millis(1)).await;
        format!("value_{}", counter.load(Ordering::SeqCst))
    }

    #[tokio::test(worker_threads = 16, flavor = "multi_thread")]
    async fn test_deadlock_scenario() {
        let _ = tracing_subscriber::fmt();
        let cache = Arc::new(Dedupe::<u64, String>::new(1000, true));
        let key = 1;
        let counter = Arc::new(AtomicUsize::new(0));
        let mut handles = Vec::new();

        // Spawn multiple tasks to simulate concurrent access
        for i in 0..1000000 {
            let cache = cache.clone();
            let counter = counter.clone();
            let handle = tokio::task::spawn(async move {
                let result = cache
                    .dedupe(&key, || Box::pin(compute_value(counter)))
                    .await;
                (i, result)
            });
            handles.push(handle);
        }
        // Await each task for any potential deadlocks
        for handle in handles.into_iter() {
            let _ = handle.await.unwrap();
        }
        // Check that compute_value was called exactly once
        assert_eq!(
            counter.load(Ordering::SeqCst),
            1,
            "compute_value was called more than once"
        );
    }
}
