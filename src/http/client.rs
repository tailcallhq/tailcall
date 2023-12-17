use std::time::Duration;

#[cfg(feature = "default")]
use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions, MokaManager};
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

use super::Response;
use crate::config::Upstream;

#[async_trait::async_trait]
pub trait HttpClient: Sync + Send {
  async fn execute(&self, req: reqwest::Request) -> anyhow::Result<Response>;
}

#[async_trait::async_trait]
impl HttpClient for DefaultHttpClient {
  async fn execute(&self, req: reqwest::Request) -> anyhow::Result<Response> {
    return self.execute(req).await;
  }
}

#[derive(Clone)]
pub struct DefaultHttpClient {
  client: ClientWithMiddleware,
}

impl Default for DefaultHttpClient {
  fn default() -> Self {
    let upstream = Upstream::default();
    //TODO: default is used only in tests. Drop default and move it to test.
    DefaultHttpClient::new(&upstream)
  }
}

impl DefaultHttpClient {
  pub fn new(_upstream: &Upstream) -> Self {
    let mut _builder = Client::builder();
    #[cfg(feature = "default")]
    let mut _builder = Client::builder()
      .tcp_keepalive(Some(Duration::from_secs(_upstream.get_tcp_keep_alive())))
      .timeout(Duration::from_secs(_upstream.get_timeout()))
      .connect_timeout(Duration::from_secs(_upstream.get_connect_timeout()))
      .http2_keep_alive_interval(Some(Duration::from_secs(_upstream.get_keep_alive_interval())))
      .http2_keep_alive_timeout(Duration::from_secs(_upstream.get_keep_alive_timeout()))
      .http2_keep_alive_while_idle(_upstream.get_keep_alive_while_idle())
      .pool_idle_timeout(Some(Duration::from_secs(_upstream.get_pool_idle_timeout())))
      .pool_max_idle_per_host(_upstream.get_pool_max_idle_per_host())
      .user_agent(_upstream.get_user_agent());

    #[cfg(feature = "default")]
    if let Some(ref proxy) = _upstream.proxy {
      _builder = _builder.proxy(reqwest::Proxy::http(proxy.url.clone()).expect("Failed to set proxy in http client"));
    }

    let mut client = ClientBuilder::new(_builder.build().expect("Failed to build client"));
    #[cfg(feature = "default")]
    if _upstream.get_enable_http_cache() {
      client = client.with(Cache(HttpCache {
        mode: CacheMode::Default,
        manager: MokaManager::default(),
        options: HttpCacheOptions::default(),
      }))
    }

    DefaultHttpClient { client: client.build() }
  }

  #[inline(always)]
  pub async fn execute(&self, request: reqwest::Request) -> anyhow::Result<Response> {
    log::info!("{} {} ", request.method(), request.url());
    #[cfg(feature = "worker-sep")]
    let _response = async_std::task::spawn_local(Self::tc_execute_client(self.client.clone(), request)).await?;
    #[cfg(feature = "default")]
    let _response = self.tc_execute(request).await?;
    Ok(_response)
  }
  async fn tc_execute(&self, request: reqwest::Request) -> anyhow::Result<Response> {
    let response = self.client.execute(request).await?;
    Response::from_response(response).await
  }
  #[allow(dead_code)]
  async fn tc_execute_client(client: ClientWithMiddleware, request: reqwest::Request) -> anyhow::Result<Response> {
    let response = client.execute(request).await?;
    Response::from_response(response).await
  }
}
