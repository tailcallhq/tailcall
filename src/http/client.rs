use std::collections::BTreeMap;
use std::time::{Duration, SystemTime};

use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions, MokaManager};
use http_cache_semantics::CachePolicy;
use reqwest::header::HeaderName;
use reqwest::Client;
use reqwest::IntoUrl;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

use crate::config::Proxy;

use super::{Request, Response};

#[derive(Clone)]
pub struct HttpClient {
    client: ClientWithMiddleware,
    pub enable_cache_control: bool,
}

impl Default for HttpClient {
    fn default() -> Self {
        HttpClient::new(false, None, false)
    }
}

impl HttpClient {
    pub fn new(enable_http_cache: bool, proxy: Option<Proxy>, enable_cache_control: bool) -> Self {
        let mut builder = Client::builder()
            .pool_max_idle_per_host(200)
            .tcp_keepalive(Some(Duration::from_secs(5)))
            .timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(60))
            .user_agent("tailcall/1.0");

        if let Some(proxy) = proxy {
            builder = builder.proxy(reqwest::Proxy::http(proxy.url).unwrap());
        }

        let mut client = ClientBuilder::new(builder.build().expect("Failed to build client"));

        if enable_http_cache {
            client = client.with(Cache(HttpCache {
                mode: CacheMode::Default,
                manager: MokaManager::default(),
                options: HttpCacheOptions::default(),
            }))
        }

        HttpClient { client: client.build(), enable_cache_control }
    }
    pub async fn execute(&self, request: reqwest::Request) -> reqwest_middleware::Result<reqwest::Response> {
        Ok(self.client.execute(request).await?)
    }
}
