use std::any::TypeId;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_channel::oneshot;
use futures_timer::Delay;
use futures_util::future::BoxFuture;
#[cfg(feature = "tracing")]
use tracing::{info_span, instrument, Instrument};
#[cfg(feature = "tracing")]
use tracinglib as tracing;

use super::cache::NoCache;
use super::traits::{CacheFactory, CacheStorage, Loader};
use super::{DataLoaderInner, KeysAndSender, Requests, ResSender};

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
  spawner: Box<dyn Fn(BoxFuture<'static, ()>) + Send + Sync>,
}

impl<K, T> DataLoader<K, T, NoCache>
where
  K: Send + Sync + Hash + Eq + Clone + 'static,
  T: Loader<K>,
{
  /// Use `Loader` to create a [DataLoader] that does not cache records.
  pub fn new<S, R>(loader: T, spawner: S) -> Self
  where
    S: Fn(BoxFuture<'static, ()>) -> R + Send + Sync + 'static,
  {
    Self {
      inner: Arc::new(DataLoaderInner { requests: Mutex::new(Requests::new(&NoCache)), loader }),
      delay: Duration::from_millis(1),
      max_batch_size: 1000,
      disable_cache: false.into(),
      spawner: Box::new(move |fut| {
        spawner(fut);
      }),
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
  pub fn with_cache<S, R>(loader: T, spawner: S, cache_factory: C) -> Self
  where
    S: Fn(BoxFuture<'static, ()>) -> R + Send + Sync + 'static,
  {
    Self {
      inner: Arc::new(DataLoaderInner { requests: Mutex::new(Requests::new(&cache_factory)), loader }),
      delay: Duration::from_millis(1),
      max_batch_size: 1000,
      disable_cache: false.into(),
      spawner: Box::new(move |fut| {
        spawner(fut);
      }),
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

        (self.spawner)(Box::pin(task));
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
        (self.spawner)(Box::pin(task))
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
