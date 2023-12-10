mod cache;
mod dataloader_impl;
mod traits;

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::Mutex;

pub use cache::{HashMapCache, LruCache, NoCache};
pub use dataloader_impl::DataLoader;
use futures_channel::oneshot;
#[cfg(feature = "tracing")]
use tracing::{info_span, instrument, Instrument};
#[cfg(feature = "tracing")]
use tracinglib as tracing;
pub use traits::{CacheFactory, CacheStorage, Loader};

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

  use dataloader_impl::DataLoader;
  use fnv::FnvBuildHasher;

  use super::*;

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
    let loader = Arc::new(DataLoader::new(MyLoader, tokio::spawn).max_batch_size(10));
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
    let loader = Arc::new(DataLoader::new(MyLoader, tokio::spawn).max_batch_size(10));
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
    let loader = DataLoader::new(MyLoader, tokio::spawn);
    assert!(loader.load_many::<Vec<i32>>(vec![]).await.unwrap().is_empty());
  }

  #[tokio::test]
  async fn test_dataloader_with_cache() {
    let loader = DataLoader::with_cache(MyLoader, tokio::spawn, HashMapCache::default());
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
    let loader = DataLoader::with_cache(MyLoader, tokio::spawn, HashMapCache::<FnvBuildHasher>::new());
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
    let loader = DataLoader::with_cache(MyLoader, tokio::spawn, HashMapCache::default());
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
    let loader = DataLoader::with_cache(MyLoader, tokio::spawn, HashMapCache::default());
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

    let loader = Arc::new(DataLoader::with_cache(MyDelayLoader, tokio::spawn, NoCache).delay(Duration::from_secs(1)));
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
