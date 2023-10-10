use std::time::Duration;

use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions, MokaManager};
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

use super::Response;
use crate::config::Server;

#[async_trait::async_trait]
pub trait HttpClient {
  async fn execute(&self, req: reqwest::Request) -> anyhow::Result<Response>;
}

#[async_trait::async_trait]
impl HttpClient for DefaultHttpClient {
  async fn execute(&self, req: reqwest::Request) -> anyhow::Result<Response> {
    let response = self.execute(req).await?;
    Ok(response)
  }
}

#[derive(Clone)]
pub struct DefaultHttpClient {
  client: ClientWithMiddleware,
}

impl Default for DefaultHttpClient {
  fn default() -> Self {
    DefaultHttpClient::new(Default::default())
  }
}

impl DefaultHttpClient {
  pub fn new(server: Server) -> Self {
    let upstream_settings = &server.upstream;

    let mut builder = Client::builder()
      .tcp_keepalive(Some(Duration::from_secs(upstream_settings.tcp_keep_alive)))
      .timeout(Duration::from_secs(upstream_settings.timeout))
      .connect_timeout(Duration::from_secs(upstream_settings.connect_timeout))
      .http2_keep_alive_interval(Some(Duration::from_secs(upstream_settings.keep_alive_interval)))
      .http2_keep_alive_timeout(Duration::from_secs(upstream_settings.keep_alive_timeout))
      .http2_keep_alive_while_idle(upstream_settings.keep_alive_while_idle)
      .pool_idle_timeout(Some(Duration::from_secs(upstream_settings.pool_idle_timeout)))
      .pool_max_idle_per_host(upstream_settings.pool_max_idle_per_host)
      .user_agent(upstream_settings.user_agent.clone());

    if let Some(ref proxy) = upstream_settings.proxy {
      builder = builder.proxy(reqwest::Proxy::http(proxy.url.clone()).expect("Failed to set proxy in http client"));
    }

    let mut client = ClientBuilder::new(builder.build().expect("Failed to build client"));

    if server.enable_http_cache() {
      client = client.with(Cache(HttpCache {
        mode: CacheMode::Default,
        manager: MokaManager::default(),
        options: HttpCacheOptions::default(),
      }))
    }

    DefaultHttpClient { client: client.build() }
  }

  pub async fn execute(&self, request: reqwest::Request) -> reqwest_middleware::Result<Response> {
    log::info!("{} {} ", request.method(), request.url());
    let response = self.client.execute(request).await?;
    let response = Response::from_response(response).await?;
    Ok(response)
  }
}
