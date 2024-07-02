use std::io::{Read, Write};
use std::path::PathBuf;

use flate2::write::GzEncoder;
use flate2::Compression;
use http_cache_reqwest::{CacheManager, HttpResponse};
use http_cache_semantics::CachePolicy;
use serde::{Deserialize, Serialize};

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, BoxError>;

pub struct CaCacheManager {
    path: PathBuf,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Store {
    response: HttpResponse,
    policy: CachePolicy,
}

impl Default for CaCacheManager {
    fn default() -> Self {
        Self { path: PathBuf::from("./.cache") }
    }
}

#[async_trait::async_trait]
impl CacheManager for CaCacheManager {
    async fn put(
        &self,
        cache_key: String,
        response: HttpResponse,
        policy: CachePolicy,
    ) -> Result<HttpResponse> {
        let data = Store { response: response.clone(), policy };
        let bytes = bincode::serialize(&data)?;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&bytes)?;
        let compressed_bytes = encoder.finish()?;

        cacache::write(&self.path, cache_key, compressed_bytes).await?;
        Ok(response)
    }

    async fn get(&self, cache_key: &str) -> Result<Option<(HttpResponse, CachePolicy)>> {
        match cacache::read(&self.path, cache_key).await {
            Ok(compressed_data) => {
                let mut decoder = flate2::read::GzDecoder::new(compressed_data.as_slice());
                let mut serialized_data = Vec::new();
                decoder.read_to_end(&mut serialized_data)?;
                let store: Store = bincode::deserialize(&serialized_data)?;
                Ok(Some((store.response, store.policy)))
            }
            Err(_) => Ok(None),
        }
    }

    async fn delete(&self, cache_key: &str) -> Result<()> {
        Ok(cacache::remove(&self.path, cache_key).await?)
    }
}
