use std::collections::HashMap;
use std::hash::Hash;

use crate::core::http;

/// Trait for batch loading.
#[async_trait::async_trait]
pub trait Loader<K: Send + Sync + Hash + Eq + Clone + 'static>: Send + Sync + 'static {
    /// type of value.
    type Value: Send + Sync + Clone + 'static;

    /// Type of error.
    type Error: Send + Clone + 'static;

    /// Load the data set specified by the `keys`.
    async fn load(
        &self,
        keys: &[K],
        http_filter: http::HttpFilter,
    ) -> Result<HashMap<K, Self::Value>, Self::Error>;
}
