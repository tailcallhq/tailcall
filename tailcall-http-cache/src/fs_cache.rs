use std::path::{Path, PathBuf};

use async_trait::async_trait;
use http_cache_reqwest::{CacheManager, HttpResponse};
use http_cache_semantics::CachePolicy;
use serde::{Deserialize, Serialize};

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, BoxError>;

pub struct FsCacheManager {
    path: PathBuf,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Store {
    response: HttpResponse,
    policy: CachePolicy,
}

impl Default for FsCacheManager {
    fn default() -> Self {
        Self { path: PathBuf::from("./.cache") }
    }
}

impl FsCacheManager {
    pub fn new(cache_dir: impl AsRef<Path>) -> Self {
        let cache_dir = PathBuf::from(cache_dir.as_ref());
        Self { path: cache_dir }
    }

    pub async fn clear(&self) -> Result<()> {
        cacache::clear(&self.path).await?;
        Ok(())
    }
}

#[async_trait]
impl CacheManager for FsCacheManager {
    async fn get(&self, cache_key: &str) -> Result<Option<(HttpResponse, CachePolicy)>> {
        let store: Store = match cacache::read(&self.path, cache_key).await {
            Ok(d) => serde_json::from_slice(&d)?,
            Err(_e) => {
                return Ok(None);
            }
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
        let bytes = serde_json::to_vec(&data)?;
        cacache::write(&self.path, cache_key, bytes).await?;
        Ok(response)
    }

    async fn delete(&self, cache_key: &str) -> Result<()> {
        Ok(cacache::remove(&self.path, cache_key).await?)
    }
}
