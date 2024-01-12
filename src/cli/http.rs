use std::time::Duration;

use anyhow::Result;
use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions, MokaManager};
use reqwest::Client;
use reqwest_middleware::ClientBuilder;

use super::HttpIO;
use crate::config::Upstream;
use crate::http::{HttpService, Response};

#[derive(Clone)]
pub struct HttpNative {
  service: HttpService,
  http2_only: bool,
}

impl Default for HttpNative {
  fn default() -> Self {
    let client = ClientBuilder::new(Client::new()).build();
    let service = HttpService::simple(client);
    Self { service, http2_only: false }
  }
}

impl HttpNative {
  pub fn init(upstream: &Upstream) -> Self {
    let mut builder = Client::builder()
      .tcp_keepalive(Some(Duration::from_secs(upstream.get_tcp_keep_alive())))
      .timeout(Duration::from_secs(upstream.get_timeout()))
      .connect_timeout(Duration::from_secs(upstream.get_connect_timeout()))
      .http2_keep_alive_interval(Some(Duration::from_secs(upstream.get_keep_alive_interval())))
      .http2_keep_alive_timeout(Duration::from_secs(upstream.get_keep_alive_timeout()))
      .http2_keep_alive_while_idle(upstream.get_keep_alive_while_idle())
      .pool_idle_timeout(Some(Duration::from_secs(upstream.get_pool_idle_timeout())))
      .pool_max_idle_per_host(upstream.get_pool_max_idle_per_host())
      .user_agent(upstream.get_user_agent());

    // Add Http2 Prior Knowledge
    if upstream.get_http_2_only() {
      log::info!("Enabled Http2 prior knowledge");
      builder = builder.http2_prior_knowledge();
    }

    // Add Http Proxy
    if let Some(ref proxy) = upstream.proxy {
      builder = builder.proxy(reqwest::Proxy::http(proxy.url.clone()).expect("Failed to set proxy in http client"));
    }

    let mut client = ClientBuilder::new(builder.build().expect("Failed to build client"));

    if upstream.get_enable_http_cache() {
      client = client.with(Cache(HttpCache {
        mode: CacheMode::Default,
        manager: MokaManager::default(),
        options: HttpCacheOptions::default(),
      }))
    }
    let client = client.build();

    let service = match upstream.rate_limit.as_ref() {
      Some(rate_limit) => {
        let num = rate_limit.requests_per_unit.get();
        let secs = rate_limit.unit.into_secs();
        let per = Duration::from_secs(secs);
        HttpService::rate_limited(client, num, per)
      }
      None => HttpService::simple(client),
    };

    Self { service, http2_only: upstream.get_http_2_only() }
  }
}

#[async_trait::async_trait]
impl HttpIO for HttpNative {
  async fn execute_raw(&self, mut request: reqwest::Request) -> Result<Response<Vec<u8>>> {
    if self.http2_only {
      *request.version_mut() = reqwest::Version::HTTP_2;
    }
    log::info!("{} {} {:?}", request.method(), request.url(), request.version());
    let response = self.service.call(request).await?;
    Ok(Response::from_reqwest(response).await?)
  }
}
