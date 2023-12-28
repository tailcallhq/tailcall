use std::time::Duration;

use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions, MokaManager};
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

use super::Response;
use crate::config::{self, Upstream};

#[async_trait::async_trait]
pub trait HttpClient: Sync + Send {
  async fn execute(&self, req: reqwest::Request) -> anyhow::Result<Response>;
  async fn execute_raw(&self, req: reqwest::Request) -> anyhow::Result<reqwest::Response>;
}

#[async_trait::async_trait]
impl HttpClient for DefaultHttpClient {
  async fn execute(&self, request: reqwest::Request) -> anyhow::Result<Response> {
    let response = self.execute_raw(request).await?;
    let response = Response::from_response(response).await?;
    Ok(response)
  }

  async fn execute_raw(&self, request: reqwest::Request) -> anyhow::Result<reqwest::Response> {
    log::info!("{} {} {:?}", request.method(), request.url(), request.version());
    Ok(self.client.execute(request).await?.error_for_status()?)
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
    let mut builder = Client::builder()
      .tcp_keepalive(Some(Duration::from_secs(upstream.get_tcp_keep_alive())))
      .timeout(Duration::from_secs(upstream.get_timeout()))
      .connect_timeout(Duration::from_secs(upstream.get_connect_timeout()))
      .http2_keep_alive_interval(Some(Duration::from_secs(upstream.get_keep_alive_interval())))
      .http2_keep_alive_timeout(Duration::from_secs(upstream.get_keep_alive_timeout()))
      .http2_keep_alive_while_idle(upstream.get_keep_alive_while_idle())
      .pool_idle_timeout(Some(Duration::from_secs(upstream.get_pool_idle_timeout())))
      .pool_max_idle_per_host(upstream.get_pool_max_idle_per_host())
      .use_rustls_tls()
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

    DefaultHttpClient { client: client.build() }
  }
}
