use std::time::Duration;

use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions, MokaManager};

use reqwest::Client;

use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

use crate::config::Server;

use super::Response;

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
    let mut builder = Client::builder()
      .pool_max_idle_per_host(200)
      .tcp_keepalive(Some(Duration::from_secs(5)))
      .timeout(Duration::from_secs(60))
      .connect_timeout(Duration::from_secs(60))
      .user_agent("Tailcall/1.0");

    if let Some(ref proxy) = server.proxy {
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
