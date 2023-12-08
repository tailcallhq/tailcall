use std::borrow::Cow;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};
use std::marker::PhantomData;
use std::num::NonZeroUsize;

/// Factory for creating cache storage.
pub trait CacheFactory<K, V>: Send + Sync + 'static
where
  K: Send + Sync + Clone + Eq + Hash + 'static,
  V: Send + Sync + Clone + 'static,
{
  type Storage: CacheStorage<Key = K, Value = V>;

  /// Create a cache storage.
  ///
  fn create(&self) -> Self::Storage;
}

/// Cache storage for [DataLoader](crate::dataloader::DataLoader).
pub trait CacheStorage: Send + Sync + 'static {
  /// The key type of the record.
  type Key: Send + Sync + Clone + Eq + Hash + 'static;

  /// The value type of the record.
  type Value: Send + Sync + Clone + 'static;

  /// Returns a reference to the value of the key in the cache or None if it
  /// is not present in the cache.
  fn get(&mut self, key: &Self::Key) -> Option<&Self::Value>;

  /// Puts a key-value pair into the cache. If the key already exists in the
  /// cache, then it updates the key's value.
  fn insert(&mut self, key: Cow<'_, Self::Key>, val: Cow<'_, Self::Value>);

  /// Removes the value corresponding to the key from the cache.
  fn remove(&mut self, key: &Self::Key);

  /// Clears the cache, removing all key-value pairs.
  fn clear(&mut self);

  /// Returns an iterator over the key-value pairs in the cache.
  fn iter(&self) -> Box<dyn Iterator<Item = (&'_ Self::Key, &'_ Self::Value)> + '_>;
}

/// No cache.
pub struct NoCache;

impl<K, V> CacheFactory<K, V> for NoCache
where
  K: Send + Sync + Clone + Eq + Hash + 'static,
  V: Send + Sync + Clone + 'static,
{
  type Storage = NoCacheImpl<K, V>;

  fn create(&self) -> Self::Storage {
    NoCacheImpl { _mark1: PhantomData, _mark2: PhantomData }
  }
}

pub struct NoCacheImpl<K, V> {
  _mark1: PhantomData<K>,
  _mark2: PhantomData<V>,
}

impl<K, V> CacheStorage for NoCacheImpl<K, V>
where
  K: Send + Sync + Clone + Eq + Hash + 'static,
  V: Send + Sync + Clone + 'static,
{
  type Key = K;
  type Value = V;

  #[inline]
  fn get(&mut self, _key: &K) -> Option<&V> {
    None
  }

  #[inline]
  fn insert(&mut self, _key: Cow<'_, Self::Key>, _val: Cow<'_, Self::Value>) {}

  #[inline]
  fn remove(&mut self, _key: &K) {}

  #[inline]
  fn clear(&mut self) {}

  fn iter(&self) -> Box<dyn Iterator<Item = (&'_ Self::Key, &'_ Self::Value)> + '_> {
    Box::new(std::iter::empty())
  }
}

/// [std::collections::HashMap] cache.
pub struct HashMapCache<S = RandomState> {
  _mark: PhantomData<S>,
}

impl<S: Send + Sync + BuildHasher + Default + 'static> HashMapCache<S> {
  /// Use specified `S: BuildHasher` to create a `HashMap` cache.
  pub fn new() -> Self {
    Self { _mark: PhantomData }
  }
}

impl Default for HashMapCache<RandomState> {
  fn default() -> Self {
    Self { _mark: PhantomData }
  }
}

impl<K, V, S: Send + Sync + BuildHasher + Default + 'static> CacheFactory<K, V> for HashMapCache<S>
where
  K: Send + Sync + Clone + Eq + Hash + 'static,
  V: Send + Sync + Clone + 'static,
{
  type Storage = HashMapCacheImpl<K, V, S>;

  fn create(&self) -> Self::Storage {
    HashMapCacheImpl(HashMap::default())
  }
}

pub struct HashMapCacheImpl<K, V, S>(HashMap<K, V, S>);

impl<K, V, S> CacheStorage for HashMapCacheImpl<K, V, S>
where
  K: Send + Sync + Clone + Eq + Hash + 'static,
  V: Send + Sync + Clone + 'static,
  S: Send + Sync + BuildHasher + 'static,
{
  type Key = K;
  type Value = V;

  #[inline]
  fn get(&mut self, key: &Self::Key) -> Option<&Self::Value> {
    self.0.get(key)
  }

  #[inline]
  fn insert(&mut self, key: Cow<'_, Self::Key>, val: Cow<'_, Self::Value>) {
    self.0.insert(key.into_owned(), val.into_owned());
  }

  #[inline]
  fn remove(&mut self, key: &Self::Key) {
    self.0.remove(key);
  }

  #[inline]
  fn clear(&mut self) {
    self.0.clear();
  }

  fn iter(&self) -> Box<dyn Iterator<Item = (&'_ Self::Key, &'_ Self::Value)> + '_> {
    Box::new(self.0.iter())
  }
}

/// LRU cache.
pub struct LruCache {
  cap: usize,
}

impl LruCache {
  /// Creates a new LRU Cache that holds at most `cap` items.
  pub fn new(cap: usize) -> Self {
    Self { cap }
  }
}

impl<K, V> CacheFactory<K, V> for LruCache
where
  K: Send + Sync + Clone + Eq + Hash + 'static,
  V: Send + Sync + Clone + 'static,
{
  type Storage = LruCacheImpl<K, V>;

  fn create(&self) -> Self::Storage {
    LruCacheImpl(lru::LruCache::new(NonZeroUsize::new(self.cap).unwrap()))
  }
}

pub struct LruCacheImpl<K, V>(lru::LruCache<K, V>);

impl<K, V> CacheStorage for LruCacheImpl<K, V>
where
  K: Send + Sync + Clone + Eq + Hash + 'static,
  V: Send + Sync + Clone + 'static,
{
  type Key = K;
  type Value = V;

  #[inline]
  fn get(&mut self, key: &Self::Key) -> Option<&Self::Value> {
    self.0.get(key)
  }

  #[inline]
  fn insert(&mut self, key: Cow<'_, Self::Key>, val: Cow<'_, Self::Value>) {
    self.0.put(key.into_owned(), val.into_owned());
  }

  #[inline]
  fn remove(&mut self, key: &Self::Key) {
    self.0.pop(key);
  }

  #[inline]
  fn clear(&mut self) {
    self.0.clear();
  }

  fn iter(&self) -> Box<dyn Iterator<Item = (&'_ Self::Key, &'_ Self::Value)> + '_> {
    Box::new(self.0.iter())
  }
}
