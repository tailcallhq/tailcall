use std::path::{Path, PathBuf};

use async_trait::async_trait;
use crypto_hash::{hex_digest, Algorithm};
use http_cache_reqwest::{CacheManager, HttpResponse};
use http_cache_semantics::CachePolicy;
use serde::{Deserialize, Serialize};
use tokio::fs;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, BoxError>;

pub struct FsCacheManager {
    cache_dir: PathBuf,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Store {
    response: HttpResponse,
    policy: CachePolicy,
}

impl Default for FsCacheManager {
    fn default() -> Self {
        Self { cache_dir: PathBuf::from("./cache") }
    }
}

impl FsCacheManager {
    pub fn new(cache_dir: impl AsRef<Path>) -> Self {
        let cache_dir = PathBuf::from(cache_dir.as_ref());
        Self { cache_dir }
    }

    fn get_cache_file_path(&self, cache_key: &str) -> PathBuf {
        self.cache_dir.join(cache_key)
    }

    fn encode(&self, key: &str) -> String {
        hex_digest(Algorithm::SHA256, key.as_bytes())
    }

    pub async fn clear(&self) -> Result<()> {
        fs::remove_dir_all(&self.cache_dir).await?;
        fs::create_dir_all(&self.cache_dir).await?;
        Ok(())
    }
}

#[async_trait]
impl CacheManager for FsCacheManager {
    async fn get(&self, cache_key: &str) -> Result<Option<(HttpResponse, CachePolicy)>> {
        let cache_file_path = self.get_cache_file_path(&self.encode(cache_key));
        if cache_file_path.exists() {
            let file_content = fs::read(&cache_file_path).await?;
            let store: Store = serde_json::from_slice(&file_content)?;
            Ok(Some((store.response, store.policy)))
        } else {
            Ok(None)
        }
    }

    async fn put(
        &self,
        cache_key: String,
        response: HttpResponse,
        policy: CachePolicy,
    ) -> Result<HttpResponse> {
        let store = Store { response: response.clone(), policy };
        let cache_file_path = self.get_cache_file_path(&self.encode(&cache_key));
        let file_content = serde_json::to_vec(&store)?;
        if !self.cache_dir.exists() {
            fs::create_dir(&self.cache_dir).await?;
        }

        if !cache_file_path.exists() {
            // only write to the file if file path not exists.
            fs::write(cache_file_path, file_content).await?;
        }

        Ok(response)
    }

    async fn delete(&self, cache_key: &str) -> Result<()> {
        let cache_file_path = self.get_cache_file_path(cache_key);
        if cache_file_path.exists() {
            fs::remove_file(cache_file_path).await?;
        }
        Ok(())
    }
}