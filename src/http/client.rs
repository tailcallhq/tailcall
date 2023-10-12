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
    let upstream = &server.upstream;

    let mut builder = Client::builder()
      .tcp_keepalive(Some(Duration::from_secs(upstream.tcp_keep_alive.unwrap_or(5))))
      .timeout(Duration::from_secs(upstream.timeout.unwrap_or(60)))
      .connect_timeout(Duration::from_secs(upstream.connect_timeout.unwrap_or(60)))
      .http2_keep_alive_interval(Some(Duration::from_secs(upstream.keep_alive_interval.unwrap_or(60))))
      .http2_keep_alive_timeout(Duration::from_secs(upstream.keep_alive_timeout.unwrap_or(60)))
      .http2_keep_alive_while_idle(upstream.keep_alive_while_idle.unwrap_or(false))
      .pool_idle_timeout(Some(Duration::from_secs(upstream.pool_idle_timeout.unwrap_or(60))))
      .pool_max_idle_per_host(upstream.pool_max_idle_per_host.unwrap_or(200))
      .user_agent(upstream.user_agent.clone().unwrap_or("Tailcall/1.0".to_string()));

    if let Some(ref proxy) = upstream.proxy {
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
