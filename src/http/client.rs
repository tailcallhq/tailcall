#[cfg(feature = "default")]
use std::time::Duration;

#[cfg(feature = "default")]
use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions, MokaManager};
use reqwest::{Client, Request};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

use super::Response;
use crate::config::{self, Upstream};

#[async_trait::async_trait]
pub trait HttpClient: Sync + Send {
  async fn execute(&self, req: reqwest::Request) -> anyhow::Result<Response<async_graphql::Value>>;
  async fn execute_raw(&self, req: reqwest::Request) -> anyhow::Result<Response<Vec<u8>>>;
}

#[async_trait::async_trait]
impl HttpClient for DefaultHttpClient {
  async fn execute(&self, request: reqwest::Request) -> anyhow::Result<Response<async_graphql::Value>> {
    #[cfg(feature = "default")]
    log::info!("{} {} {:?}", request.method(), request.url(), request.version());
    #[cfg(not(feature = "default"))]
    return async_std::task::spawn_local(execute(self.client.clone(), request)).await;
    #[cfg(feature = "default")]
    Response::from_response_to_val(self.client.execute(request).await?).await
  }

  async fn execute_raw(&self, request: Request) -> anyhow::Result<Response<Vec<u8>>> {
    #[cfg(feature = "default")]
    log::info!("{} {} {:?}", request.method(), request.url(), request.version());
    #[cfg(not(feature = "default"))]
    return async_std::task::spawn_local(execute_vec(self.client.clone(), request)).await;
    #[cfg(feature = "default")]
    Response::from_response_to_vec(self.client.execute(request).await?).await
  }
}

#[derive(Clone)]
pub struct DefaultHttpClient {
  client: ClientWithMiddleware,
}

impl Default for DefaultHttpClient {
  fn default() -> Self {
    let upstream = config::Upstream::default();
    //TODO: default is used only in tests. Drop default and move it to test.
    DefaultHttpClient::new(&upstream)
  }
}

#[derive(Default)]
pub struct HttpClientOptions {
  pub http2_only: bool,
}

impl DefaultHttpClient {
  pub fn new(upstream: &Upstream) -> Self {
    Self::with_options(upstream, HttpClientOptions::default())
  }

  #[allow(unused_mut)]
  pub fn with_options(_upstream: &Upstream, _options: HttpClientOptions) -> Self {
    let builder = Client::builder();
    #[cfg(feature = "default")]
    let mut builder = builder
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
    if _options.http2_only {
      builder = builder.http2_prior_knowledge();
    }
    #[cfg(feature = "default")]
    if let Some(ref proxy) = _upstream.proxy {
      builder = builder.proxy(reqwest::Proxy::http(proxy.url.clone()).expect("Failed to set proxy in http client"));
    }

    let mut client = ClientBuilder::new(builder.build().expect("Failed to build client"));
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
}
#[cfg(not(feature = "default"))]
async fn execute(client: ClientWithMiddleware, request: Request) -> anyhow::Result<Response<async_graphql::Value>> {
  let response = client.execute(request).await?.error_for_status()?;
  let response = Response::from_response_to_val(response).await?;
  Ok(response)
}
#[cfg(not(feature = "default"))]
async fn execute_vec(client: ClientWithMiddleware, request: Request) -> anyhow::Result<Response<Vec<u8>>> {
  let response = client.execute(request).await?.error_for_status()?;
  let response = Response::from_response_to_vec(response).await?;
  Ok(response)
}
