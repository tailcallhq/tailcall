#[cfg(feature = "default")]
use std::time::Duration;
#[cfg(feature = "default")]
use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions, MokaManager};
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

use super::Response;
use crate::config::{self, Upstream};

#[async_trait::async_trait]
pub trait HttpClient: Sync + Send {
  async fn execute<T: Default + Clone>(&self, req: reqwest::Request) -> anyhow::Result<Response<T>>;
}

#[async_trait::async_trait]
impl HttpClient for DefaultHttpClient {
  async fn execute<T: Default + Clone>(&self, request: reqwest::Request) -> anyhow::Result<Response<T>> {
    #[cfg(feature = "default")]
    log::info!("{} {} {:?}", request.method(), request.url(), request.version());
    #[cfg(not(feature = "default"))]
    return async_std::task::spawn_local(execute(self.client.clone(), request)).await;
    execute(self.client.clone(), request).await
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

  pub fn with_options(upstream: &Upstream, options: HttpClientOptions) -> Self {
    let builder = Client::builder();
    #[cfg(feature = "default")]
        let mut builder = builder.tcp_keepalive(Some(Duration::from_secs(upstream.get_tcp_keep_alive())))
        .timeout(Duration::from_secs(upstream.get_timeout()))
        .connect_timeout(Duration::from_secs(upstream.get_connect_timeout()))
        .http2_keep_alive_interval(Some(Duration::from_secs(upstream.get_keep_alive_interval())))
        .http2_keep_alive_timeout(Duration::from_secs(upstream.get_keep_alive_timeout()))
        .http2_keep_alive_while_idle(upstream.get_keep_alive_while_idle())
        .pool_idle_timeout(Some(Duration::from_secs(upstream.get_pool_idle_timeout())))
        .pool_max_idle_per_host(upstream.get_pool_max_idle_per_host())
        .user_agent(upstream.get_user_agent());
    #[cfg(feature = "default")]
    if options.http2_only {
      builder = builder.http2_prior_knowledge();
    }
    #[cfg(feature = "default")]
    if let Some(ref proxy) = upstream.proxy {
      builder = builder.proxy(reqwest::Proxy::http(proxy.url.clone()).expect("Failed to set proxy in http client"));
    }

    let mut client = ClientBuilder::new(builder.build().expect("Failed to build client"));
    #[cfg(feature = "default")]
    if upstream.get_enable_http_cache() {
      client = client.with(Cache(HttpCache {
        mode: CacheMode::Default,
        manager: MokaManager::default(),
        options: HttpCacheOptions::default(),
      }))
    }

    DefaultHttpClient { client: client.build() }
  }
}

async fn execute<T: Default + Clone>(client: ClientWithMiddleware, request: reqwest::Request) -> anyhow::Result<Response<T>>{
  let response = client.execute(request).await?.error_for_status()?;
  let response = Response::from_response(response).await?;
  Ok(response)
}