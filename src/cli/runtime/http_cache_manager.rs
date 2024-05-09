use http_cache_reqwest::{CacheManager, HttpResponse};
use http_cache_semantics::CachePolicy;
use moka::future::Cache;
use serde::{Deserialize, Serialize};
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, BoxError>;

use std::sync::Arc;

pub struct MokaManager {
    pub cache: Arc<Cache<String, Store>>,
}

impl Default for MokaManager {
    fn default() -> Self {
        Self::new(Cache::new(42))
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Store {
    response: HttpResponse,
    policy: CachePolicy,
}

impl MokaManager {
    pub fn new(cache: Cache<String, Store>) -> Self {
        Self { cache: Arc::new(cache) }
    }
}

#[async_trait::async_trait]
impl CacheManager for MokaManager {
    async fn get(&self, cache_key: &str) -> Result<Option<(HttpResponse, CachePolicy)>> {
        let store: Store = match self.cache.get(cache_key).await {
            Some(d) => d,
            None => return Ok(None),
        };
        Ok(Some((store.response, store.policy)))
    }

    async fn put(
        &self,
        cache_key: String,
        response: HttpResponse,
        policy: CachePolicy,
    ) -> Result<HttpResponse> {
        let data = Store { response: response.clone(), policy };
        self.cache.insert(cache_key, data).await;
        self.cache.run_pending_tasks().await;
        Ok(response)
    }

    async fn delete(&self, cache_key: &str) -> Result<()> {
        self.cache.invalidate(cache_key).await;
        self.cache.run_pending_tasks().await;
        Ok(())
    }
}
