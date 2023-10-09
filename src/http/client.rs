use std::time::Duration;

use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions, MokaManager};
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

use super::Response;
use crate::config::{Server, Upstream};

#[derive(Clone)]
pub struct HttpClient {
  client: ClientWithMiddleware,
}

impl Default for HttpClient {
  fn default() -> Self {
    HttpClient::new(Default::default())
  }
}

impl HttpClient {
  pub fn new(server: Server) -> Self {
    let upstream_settings_serilaised: String =
      serde_json::to_string(&server.upstream.clone().unwrap_or_default()).unwrap();
    let upstream_settings: Upstream = serde_json::from_str(&upstream_settings_serilaised).unwrap();

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

    HttpClient { client: client.build() }
  }

  pub async fn execute(&self, request: reqwest::Request) -> reqwest_middleware::Result<Response> {
    let response = self.client.execute(request).await?;
    let response = Response::from_response(response).await?;
    Ok(response)
  }
}
