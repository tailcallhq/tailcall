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
            .user_agent("Tailcall/1.0");

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
    pub async fn execute(&self, request: reqwest::Request) -> reqwest_middleware::Result<Response> {
        let cached_req: Request = Request::from(&request);
        let response = self.client.execute(request).await?;
        let mut cached_response = Response::from(&response);
        cached_response.body = response.json().await?;
        if self.enable_cache_control {
            let cache_ttl = CachePolicy::new(&cached_req, &cached_response)
                .time_to_live(SystemTime::now())
                .as_secs();
            Ok(cached_response.ttl(Option::from(cache_ttl)))
        } else {
            Ok(cached_response)
        }
    }

    pub async fn get<T>(
        &self,
        url: T,
        forwarded_headers: BTreeMap<String, String>,
    ) -> reqwest_middleware::Result<Response>
    where
        T: IntoUrl,
    {
        let mut headers = reqwest::header::HeaderMap::new();
        // for (key, value) in forwarded_headers.iter() {
        //     headers.insert(key.parse::<HeaderName>().unwrap(), value.parse().unwrap());
        // }
        let request = self.client.get(url).headers(headers).build()?;
        self.execute(request).await
    }
}
