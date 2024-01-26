use std::any::TypeId;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_channel::oneshot;
use futures_timer::Delay;
#[cfg(feature = "tracing")]
use tracing::{info_span, instrument, Instrument};
#[cfg(feature = "tracing")]
use tracinglib as tracing;

pub use super::cache::NoCache;
pub use super::factory::CacheFactory;
pub use super::loader::Loader;
pub use super::storage::CacheStorage;

/// Data loader.
///
/// Reference: <https://github.com/facebook/dataloader>
pub struct DataLoader<
  K: Send + Sync + Eq + Clone + Hash + 'static,
  T: Loader<K>,
  C: CacheFactory<K, T::Value> = NoCache,
> {
  inner: Arc<DataLoaderInner<K, T, C>>,
  delay: Duration,
  max_batch_size: usize,
  disable_cache: AtomicBool,
}

impl<K, T> DataLoader<K, T, NoCache>
where
  K: Send + Sync + Hash + Eq + Clone + 'static,
  T: Loader<K>,
{
  /// Use `Loader` to create a [DataLoader] that does not cache records.
  pub fn new(loader: T) -> Self {
    Self {
      inner: Arc::new(DataLoaderInner { requests: Mutex::new(Requests::new(&NoCache)), loader }),
      delay: Duration::from_millis(1),
      max_batch_size: 1000,
      disable_cache: false.into(),
    }
  }
}

impl<K, T, C> DataLoader<K, T, C>
where
  K: Send + Sync + Hash + Eq + Clone + 'static,
  T: Loader<K>,
  C: CacheFactory<K, T::Value>,
{
  /// Use `Loader` to create a [DataLoader] with a cache factory.
  pub fn with_cache(loader: T, cache_factory: C) -> Self {
    Self {
      inner: Arc::new(DataLoaderInner { requests: Mutex::new(Requests::new(&cache_factory)), loader }),
      delay: Duration::from_millis(1),
      max_batch_size: 1000,
      disable_cache: false.into(),
    }
  }

  /// Specify the delay time for loading data, the default is `1ms`.
  #[must_use]
  pub fn delay(self, delay: Duration) -> Self {
    Self { delay, ..self }
  }

  /// pub fn Specify the max batch size for loading data, the default is
  /// `1000`.
  ///
  /// If the keys waiting to be loaded reach the threshold, they are loaded
  /// immediately.
  #[must_use]
  pub fn max_batch_size(self, max_batch_size: usize) -> Self {
    Self { max_batch_size, ..self }
  }

  /// Get the loader.
  #[inline]
  pub fn loader(&self) -> &T {
    &self.inner.loader
  }

  /// Enable/Disable cache of all loaders.
  pub fn enable_all_cache(&self, enable: bool) {
    self.disable_cache.store(!enable, Ordering::SeqCst);
  }

  /// Enable/Disable cache of specified loader.
  pub fn enable_cache(&self, enable: bool)
  where
    K: Send + Sync + Hash + Eq + Clone + 'static,
    T: Loader<K>,
  {
    let mut requests = self.inner.requests.lock().unwrap();
    requests.disable_cache = !enable;
  }

  /// Use this `DataLoader` load a data.
  #[cfg_attr(feature = "tracing", instrument(skip_all))]
  pub async fn load_one(&self, key: K) -> Result<Option<T::Value>, T::Error>
  where
    K: Send + Sync + Hash + Eq + Clone + 'static,
    T: Loader<K>,
  {
    let mut values = self.load_many(std::iter::once(key.clone())).await?;
    Ok(values.remove(&key))
  }

  /// Use this `DataLoader` to load some data.
  #[cfg_attr(feature = "tracing", instrument(skip_all))]
  pub async fn load_many<I>(&self, keys: I) -> Result<HashMap<K, T::Value>, T::Error>
  where
    K: Send + Sync + Hash + Eq + Clone + 'static,
    I: IntoIterator<Item = K>,
    T: Loader<K>,
  {
    enum Action<K: Send + Sync + Hash + Eq + Clone + 'static, T: Loader<K>> {
      ImmediateLoad(KeysAndSender<K, T>),
      StartFetch,
      Delay,
    }

    let (action, rx) = {
      let mut requests = self.inner.requests.lock().unwrap();
      let prev_count = requests.keys.len();
      let mut keys_set = HashSet::new();
      let mut use_cache_values = HashMap::new();

      if requests.disable_cache || self.disable_cache.load(Ordering::SeqCst) {
        keys_set = keys.into_iter().collect();
      } else {
        for key in keys {
          if let Some(value) = requests.cache_storage.get(&key) {
            // Already in cache
            use_cache_values.insert(key.clone(), value.clone());
          } else {
            keys_set.insert(key);
          }
        }
      }

      if !use_cache_values.is_empty() && keys_set.is_empty() {
        return Ok(use_cache_values);
      } else if use_cache_values.is_empty() && keys_set.is_empty() {
        return Ok(Default::default());
      }

      requests.keys.extend(keys_set.clone());
      let (tx, rx) = oneshot::channel();
      requests.pending.push((keys_set, ResSender { use_cache_values, tx }));

      if requests.keys.len() >= self.max_batch_size {
        (Action::ImmediateLoad(requests.take()), rx)
      } else {
        (
          if !requests.keys.is_empty() && prev_count == 0 {
            Action::StartFetch
          } else {
            Action::Delay
          },
          rx,
        )
      }
    };

    match action {
      Action::ImmediateLoad(keys) => {
        let inner = self.inner.clone();
        let disable_cache = self.disable_cache.load(Ordering::SeqCst);
        let task = async move { inner.do_load(disable_cache, keys).await };
        #[cfg(feature = "tracing")]
        let task = task.instrument(info_span!("immediate_load")).in_current_span();

        #[cfg(not(target_arch = "wasm32"))]
        tokio::spawn(Box::pin(task));
        #[cfg(target_arch = "wasm32")]
        async_std::task::spawn_local(Box::pin(task));
      }
      Action::StartFetch => {
        let inner = self.inner.clone();
        let disable_cache = self.disable_cache.load(Ordering::SeqCst);
        let delay = self.delay;

        let task = async move {
          Delay::new(delay).await;

          let keys = {
            let mut requests = inner.requests.lock().unwrap();
            requests.take()
          };

          if !keys.0.is_empty() {
            inner.do_load(disable_cache, keys).await
          }
        };
        #[cfg(feature = "tracing")]
        let task = task.instrument(info_span!("start_fetch")).in_current_span();
        #[cfg(not(target_arch = "wasm32"))]
        tokio::spawn(Box::pin(task));
        #[cfg(target_arch = "wasm32")]
        async_std::task::spawn_local(Box::pin(task));
      }
      Action::Delay => {}
    }

    rx.await.unwrap()
  }

  /// Feed some data into the cache.
  ///
  /// **NOTE: If the cache type is [NoCache], this function will not take
  /// effect. **
  #[cfg_attr(feature = "tracing", instrument(skip_all))]
  pub async fn feed_many<I>(&self, values: I)
  where
    K: Send + Sync + Hash + Eq + Clone + 'static,
    I: IntoIterator<Item = (K, T::Value)>,
    T: Loader<K>,
  {
    let mut requests = self.inner.requests.lock().unwrap();
    for (key, value) in values {
      requests.cache_storage.insert(Cow::Owned(key), Cow::Owned(value));
    }
  }

  /// Feed some data into the cache.
  ///
  /// **NOTE: If the cache type is [NoCache], this function will not take
  /// effect. **
  #[cfg_attr(feature = "tracing", instrument(skip_all))]
  pub async fn feed_one(&self, key: K, value: T::Value)
  where
    K: Send + Sync + Hash + Eq + Clone + 'static,
    T: Loader<K>,
  {
    self.feed_many(std::iter::once((key, value))).await;
  }

  /// Clears the cache.
  ///
  /// **NOTE: If the cache type is [NoCache], this function will not take
  /// effect. **
  #[cfg_attr(feature = "tracing", instrument(skip_all))]
  pub fn clear(&self)
  where
    K: Send + Sync + Hash + Eq + Clone + 'static,
    T: Loader<K>,
  {
    let _tid = TypeId::of::<K>();
    let mut requests = self.inner.requests.lock().unwrap();
    requests.cache_storage.clear();
  }

  /// Gets all values in the cache.
  pub fn get_cached_values(&self) -> HashMap<K, T::Value>
  where
    K: Send + Sync + Hash + Eq + Clone + 'static,
    T: Loader<K>,
  {
    let _tid = TypeId::of::<K>();
    let requests = self.inner.requests.lock().unwrap();
    requests
      .cache_storage
      .iter()
      .map(|(k, v)| (k.clone(), v.clone()))
      .collect()
  }
}

#[allow(clippy::type_complexity)]
struct ResSender<K: Send + Sync + Hash + Eq + Clone + 'static, T: Loader<K>> {
  use_cache_values: HashMap<K, T::Value>,
  tx: oneshot::Sender<Result<HashMap<K, T::Value>, T::Error>>,
}

struct Requests<K: Send + Sync + Hash + Eq + Clone + 'static, T: Loader<K>, C: CacheFactory<K, T::Value>> {
  keys: HashSet<K>,
  pending: Vec<(HashSet<K>, ResSender<K, T>)>,
  cache_storage: C::Storage,
  disable_cache: bool,
}

type KeysAndSender<K, T> = (HashSet<K>, Vec<(HashSet<K>, ResSender<K, T>)>);

impl<K: Send + Sync + Hash + Eq + Clone + 'static, T: Loader<K>, C: CacheFactory<K, T::Value>> Requests<K, T, C> {
  fn new(cache_factory: &C) -> Self {
    Self { keys: Default::default(), pending: Vec::new(), cache_storage: cache_factory.create(), disable_cache: false }
  }

  fn take(&mut self) -> KeysAndSender<K, T> {
    (std::mem::take(&mut self.keys), std::mem::take(&mut self.pending))
  }
}

struct DataLoaderInner<K: Send + Sync + Hash + Eq + Clone + 'static, T: Loader<K>, C: CacheFactory<K, T::Value>> {
  requests: Mutex<Requests<K, T, C>>,
  loader: T,
}

impl<K, T, C> DataLoaderInner<K, T, C>
where
  K: Send + Sync + Hash + Eq + Clone + 'static,
  T: Loader<K>,
  C: CacheFactory<K, T::Value>,
{
  #[cfg_attr(feature = "tracing", instrument(skip_all))]
  async fn do_load(&self, disable_cache: bool, (keys, senders): KeysAndSender<K, T>)
  where
    K: Send + Sync + Hash + Eq + Clone + 'static,
    T: Loader<K>,
  {
    let keys = keys.into_iter().collect::<Vec<_>>();

    match self.loader.load(&keys).await {
      Ok(values) => {
        // update cache
        let mut requests = self.requests.lock().unwrap();
        let disable_cache = requests.disable_cache || disable_cache;
        if !disable_cache {
          for (key, value) in &values {
            requests.cache_storage.insert(Cow::Borrowed(key), Cow::Borrowed(value));
          }
        }

        // send response
        for (keys, sender) in senders {
          let mut res = HashMap::new();
          res.extend(sender.use_cache_values);
          for key in &keys {
            res.extend(values.get(key).map(|value| (key.clone(), value.clone())));
          }
          sender.tx.send(Ok(res)).ok();
        }
      }
      Err(err) => {
        for (_, sender) in senders {
          sender.tx.send(Err(err.clone())).ok();
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;
  use std::time::Duration;

  use fnv::FnvBuildHasher;

  use super::*;
  use crate::data_loader::HashMapCache;

  struct MyLoader;

  #[async_trait::async_trait]
  impl Loader<i32> for MyLoader {
    type Value = i32;
    type Error = ();

    async fn load(&self, keys: &[i32]) -> Result<HashMap<i32, Self::Value>, Self::Error> {
      assert!(keys.len() <= 10);
      Ok(keys.iter().copied().map(|k| (k, k)).collect())
    }
  }

  #[async_trait::async_trait]
  impl Loader<i64> for MyLoader {
    type Value = i64;
    type Error = ();

    async fn load(&self, keys: &[i64]) -> Result<HashMap<i64, Self::Value>, Self::Error> {
      assert!(keys.len() <= 10);
      Ok(keys.iter().copied().map(|k| (k, k)).collect())
    }
  }

  #[tokio::test]
  async fn test_dataloader() {
    let loader = Arc::new(DataLoader::new(MyLoader).max_batch_size(10));
    assert_eq!(
      futures_util::future::try_join_all((0..100i32).map({
        let loader = loader.clone();
        move |n| {
          let loader = loader.clone();
          async move { loader.load_one(n).await }
        }
      }))
      .await
      .unwrap(),
      (0..100).map(Option::Some).collect::<Vec<_>>()
    );
  }

  #[tokio::test]
  async fn test_duplicate_keys() {
    let loader = Arc::new(DataLoader::new(MyLoader).max_batch_size(10));
    assert_eq!(
      futures_util::future::try_join_all([1, 3, 5, 1, 7, 8, 3, 7].iter().copied().map({
        let loader = loader.clone();
        move |n| {
          let loader = loader.clone();
          async move { loader.load_one(n).await }
        }
      }))
      .await
      .unwrap(),
      [1, 3, 5, 1, 7, 8, 3, 7]
        .iter()
        .copied()
        .map(Option::Some)
        .collect::<Vec<_>>()
    );
  }

  #[tokio::test]
  async fn test_dataloader_load_empty() {
    let loader = DataLoader::new(MyLoader);
    assert!(loader.load_many::<Vec<i32>>(vec![]).await.unwrap().is_empty());
  }

  #[tokio::test]
  async fn test_dataloader_with_cache() {
    let loader = DataLoader::with_cache(MyLoader, HashMapCache::default());
    loader.feed_many(vec![(1, 10), (2, 20), (3, 30)]).await;

    // All from the cache
    assert_eq!(
      loader.load_many(vec![1, 2, 3]).await.unwrap(),
      vec![(1, 10), (2, 20), (3, 30)].into_iter().collect()
    );

    // Part from the cache
    assert_eq!(
      loader.load_many(vec![1, 5, 6]).await.unwrap(),
      vec![(1, 10), (5, 5), (6, 6)].into_iter().collect()
    );

    // All from the loader
    assert_eq!(
      loader.load_many(vec![8, 9, 10]).await.unwrap(),
      vec![(8, 8), (9, 9), (10, 10)].into_iter().collect()
    );

    // Clear cache
    loader.clear();
    assert_eq!(
      loader.load_many(vec![1, 2, 3]).await.unwrap(),
      vec![(1, 1), (2, 2), (3, 3)].into_iter().collect()
    );
  }

  #[tokio::test]
  async fn test_dataloader_with_cache_hashmap_fnv() {
    let loader = DataLoader::with_cache(MyLoader, HashMapCache::<FnvBuildHasher>::new());
    loader.feed_many(vec![(1, 10), (2, 20), (3, 30)]).await;

    // All from the cache
    assert_eq!(
      loader.load_many(vec![1, 2, 3]).await.unwrap(),
      vec![(1, 10), (2, 20), (3, 30)].into_iter().collect()
    );

    // Part from the cache
    assert_eq!(
      loader.load_many(vec![1, 5, 6]).await.unwrap(),
      vec![(1, 10), (5, 5), (6, 6)].into_iter().collect()
    );

    // All from the loader
    assert_eq!(
      loader.load_many(vec![8, 9, 10]).await.unwrap(),
      vec![(8, 8), (9, 9), (10, 10)].into_iter().collect()
    );

    // Clear cache
    loader.clear();
    assert_eq!(
      loader.load_many(vec![1, 2, 3]).await.unwrap(),
      vec![(1, 1), (2, 2), (3, 3)].into_iter().collect()
    );
  }

  #[tokio::test]
  async fn test_dataloader_disable_all_cache() {
    let loader = DataLoader::with_cache(MyLoader, HashMapCache::default());
    loader.feed_many(vec![(1, 10), (2, 20), (3, 30)]).await;

    // All from the loader
    loader.enable_all_cache(false);
    assert_eq!(
      loader.load_many(vec![1, 2, 3]).await.unwrap(),
      vec![(1, 1), (2, 2), (3, 3)].into_iter().collect()
    );

    // All from the cache
    loader.enable_all_cache(true);
    assert_eq!(
      loader.load_many(vec![1, 2, 3]).await.unwrap(),
      vec![(1, 10), (2, 20), (3, 30)].into_iter().collect()
    );
  }

  #[tokio::test]
  async fn test_dataloader_disable_cache() {
    let loader = DataLoader::with_cache(MyLoader, HashMapCache::default());
    loader.feed_many(vec![(1, 10), (2, 20), (3, 30)]).await;

    // All from the loader
    loader.enable_cache(false);
    assert_eq!(
      loader.load_many(vec![1, 2, 3]).await.unwrap(),
      vec![(1, 1), (2, 2), (3, 3)].into_iter().collect()
    );

    // All from the cache
    loader.enable_cache(true);
    assert_eq!(
      loader.load_many(vec![1, 2, 3]).await.unwrap(),
      vec![(1, 10), (2, 20), (3, 30)].into_iter().collect()
    );
  }

  #[tokio::test]
  async fn test_dataloader_dead_lock() {
    struct MyDelayLoader;

    #[async_trait::async_trait]
    impl Loader<i32> for MyDelayLoader {
      type Value = i32;
      type Error = ();

      async fn load(&self, keys: &[i32]) -> Result<HashMap<i32, Self::Value>, Self::Error> {
        tokio::time::sleep(Duration::from_secs(1)).await;
        Ok(keys.iter().copied().map(|k| (k, k)).collect())
      }
    }

    let loader = Arc::new(DataLoader::with_cache(MyDelayLoader, NoCache).delay(Duration::from_secs(1)));
    let handle = tokio::spawn({
      let loader = loader.clone();
      async move {
        loader.load_many(vec![1, 2, 3]).await.unwrap();
      }
    });

    tokio::time::sleep(Duration::from_millis(500)).await;
    handle.abort();
    loader.load_many(vec![4, 5, 6]).await.unwrap();
  }
}
