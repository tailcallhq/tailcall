use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions};
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use tailcall_http_cache::FsCacheManager;

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
