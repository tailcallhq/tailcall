use std::collections::BTreeMap;

use std::time::Duration;

use derive_setters::Setters;
use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions, MokaManager};

use reqwest::Client;

use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

use crate::config::Proxy;

use super::{GetRequest, Response};

#[derive(Clone, Setters)]
pub struct HttpClient {
    client: ClientWithMiddleware,
    // TODO: may be we can save `server` here instead of enable-cache-control
    pub enable_cache_control: bool,

    // TODO: forwarded headers isn't the client's responsibility
    pub forwarded_headers: BTreeMap<String, String>,
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
            .user_agent("Tailcall/1.0");

        if let Some(proxy) = proxy {
            builder = builder.proxy(reqwest::Proxy::http(proxy.url).expect("Failed to set proxy in http client"));
        }

        let mut client = ClientBuilder::new(builder.build().expect("Failed to build client"));

        if enable_http_cache {
            client = client.with(Cache(HttpCache {
                mode: CacheMode::Default,
                manager: MokaManager::default(),
                options: HttpCacheOptions::default(),
            }))
        }

        HttpClient { client: client.build(), enable_cache_control, forwarded_headers: BTreeMap::new() }
    }

    pub async fn execute(&self, request: reqwest::Request) -> reqwest_middleware::Result<Response> {
        if request.method() == reqwest::Method::GET {
            let get_request = GetRequest::from(&request);
            let response = self.client.execute(request).await?;
            let response = Response::from_response(response).await?;

            // TTL inference should happen for GET requests only
            if self.enable_cache_control {
                Ok(response.set_min_ttl(get_request))
            } else {
                Ok(response)
            }
        } else {
            let response = self.client.execute(request).await?;
            Ok(Response::from_response(response).await?)
        }
    }
}
