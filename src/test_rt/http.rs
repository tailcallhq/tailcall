use std::time::Duration;

use anyhow::Result;
use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions, MokaManager};
use hyper::body::Bytes;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

use crate::blueprint::Upstream;
use crate::http::Response;
use crate::HttpIO;

#[derive(Clone)]
pub struct TestHttp {
    client: ClientWithMiddleware,
    http2_only: bool,
}

impl Default for TestHttp {
    fn default() -> Self {
        Self {
            client: ClientBuilder::new(Client::new()).build(),
            http2_only: false,
        }
    }
}

impl TestHttp {
    pub fn init(upstream: &Upstream) -> Self {
        let mut builder = Client::builder()
            .tcp_keepalive(Some(Duration::from_secs(upstream.tcp_keep_alive)))
            .timeout(Duration::from_secs(upstream.timeout))
            .connect_timeout(Duration::from_secs(upstream.connect_timeout))
            .http2_keep_alive_interval(Some(Duration::from_secs(upstream.keep_alive_interval)))
            .http2_keep_alive_timeout(Duration::from_secs(upstream.keep_alive_timeout))
            .http2_keep_alive_while_idle(upstream.keep_alive_while_idle)
            .pool_idle_timeout(Some(Duration::from_secs(upstream.pool_idle_timeout)))
            .pool_max_idle_per_host(upstream.pool_max_idle_per_host)
            .user_agent(upstream.user_agent.clone());

        // Add Http2 Prior Knowledge
        if upstream.http2_only {
            builder = builder.http2_prior_knowledge();
        }

        // Add Http Proxy
        if let Some(ref proxy) = upstream.proxy {
            builder = builder.proxy(
                reqwest::Proxy::http(proxy.url.clone())
                    .expect("Failed to set proxy in http client"),
            );
        }

        let mut client = ClientBuilder::new(builder.build().expect("Failed to build client"));

        if upstream.http_cache {
            client = client.with(Cache(HttpCache {
                mode: CacheMode::Default,
                manager: MokaManager::default(),
                options: HttpCacheOptions::default(),
            }))
        }
        Self { client: client.build(), http2_only: upstream.http2_only }
    }
}

#[async_trait::async_trait]
impl HttpIO for TestHttp {
    async fn execute(&self, mut request: reqwest::Request) -> Result<Response<Bytes>> {
        if self.http2_only {
            *request.version_mut() = reqwest::Version::HTTP_2;
        }
        log::info!(
            "{} {} {:?}",
            request.method(),
            request.url(),
            request.version()
        );
        log::debug!("request: {:?}", request);
        let response = self.client.execute(request).await;
        log::debug!("response: {:?}", response);
        Ok(Response::from_reqwest(response?.error_for_status()?).await?)
    }
}
