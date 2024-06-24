use anyhow::Result;
use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions};
use hyper::body::Bytes;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use tailcall::core::http::Response;
use tailcall::core::HttpIO;

use crate::fs_cache::FsCacheManager;

#[derive(Clone)]
pub struct NativeHttpTest {
    client: ClientWithMiddleware,
}

impl Default for NativeHttpTest {
    fn default() -> Self {
        let mut client = ClientBuilder::new(Client::new());
        client = client.with(Cache(HttpCache {
            mode: CacheMode::Default,
            manager: FsCacheManager::default(),
            options: HttpCacheOptions::default(),
        }));
        Self { client: client.build() }
    }
}

#[async_trait::async_trait]
impl HttpIO for NativeHttpTest {
    #[allow(clippy::blocks_in_conditions)]
    async fn execute(&self, request: reqwest::Request) -> Result<Response<Bytes>> {
        let response = self.client.execute(request).await;
        Ok(Response::from_reqwest(
            response?
                .error_for_status()
                .map_err(|err| err.without_url())?,
        )
        .await?)
    }
}
