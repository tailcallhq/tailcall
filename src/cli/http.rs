use std::time::Duration;

use anyhow::Result;
use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions, MokaManager};
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

use super::HttpIO;
use crate::config::Upstream;
use crate::http::{HttpClientOptions, Response};

#[derive(Clone)]
pub struct HttpNative {
  client: ClientWithMiddleware,
}

impl Default for HttpNative {
  fn default() -> Self {
    Self { client: ClientBuilder::new(Client::new()).build() }
  }
}

impl HttpNative {
  pub fn init(upstream: &Upstream, options: &HttpClientOptions) -> Self {
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
    if options.http2_only {
      builder = builder.http2_prior_knowledge();
    }
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
    Self { client: client.build() }
  }
}

#[async_trait::async_trait]
impl HttpIO for HttpNative {
  async fn execute_raw(&self, mut request: reqwest::Request, option: HttpClientOptions) -> Result<Response<Vec<u8>>> {
    log::info!("{} {} {:?}", request.method(), request.url(), request.version());
    if option.http2_only {
      *request.version_mut() = reqwest::Version::HTTP_2;
    }
    let response = self.client.execute(request).await?;
    Ok(Response::from_reqwest(response).await?)
  }
}
