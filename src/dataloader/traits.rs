use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::Hash;

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

/// Trait for batch loading.
#[async_trait::async_trait]
pub trait Loader<K: Send + Sync + Hash + Eq + Clone + 'static>: Send + Sync + 'static {
  /// type of value.
  type Value: Send + Sync + Clone + 'static;

  /// Type of error.
  type Error: Send + Clone + 'static;

  /// Load the data set specified by the `keys`.
  async fn load(&self, keys: &[K]) -> Result<HashMap<K, Self::Value>, Self::Error>;
}
