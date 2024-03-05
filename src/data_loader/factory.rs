use std::hash::Hash;

use super::storage::CacheStorage;

/// Factory for creating cache storage.
pub trait CacheFactory<K, V>: Send + Sync + 'static
where
    K: Send + Sync + Clone + Eq + Hash + 'static,
    V: Send + Sync + Clone + 'static,
{
    type Storage: CacheStorage<Key = K, Value = V>;

    /// Create a cache storage.
    fn create(&self) -> Self::Storage;
}
