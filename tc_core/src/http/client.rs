#[cfg(feature = "default")]
use std::time::Duration;

#[cfg(feature = "default")]
use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions, MokaManager};
use reqwest::{Client, Request};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

use super::Response;
use crate::config::{self, Upstream};
use crate::grpc::protobuf::ProtobufOperation;

#[async_trait::async_trait]
pub trait HttpClient: Sync + Send {
  async fn execute(&self, req: Request, operation: Option<&ProtobufOperation>) -> anyhow::Result<Response>;
  #[cfg(feature = "default")]
  async fn execute_raw(&self, req: Request) -> anyhow::Result<reqwest::Response>;
}

#[async_trait::async_trait]
impl HttpClient for DefaultHttpClient {
  async fn execute(&self, req: Request, operation: Option<&ProtobufOperation>) -> anyhow::Result<Response> {
    return match operation {
      None => {
        #[cfg(feature = "default")]
        return self.tc_execute(req).await;
        async_std::task::spawn_local(Self::wasm_execute(self.client.clone(), req)).await
      }
      Some(operation) => {
        #[cfg(feature = "default")]
        return self.tc_execute_grpc(req, operation).await;
        async_std::task::spawn_local(Self::wasm_execute_grpc(self.client.clone(), req, operation.clone())).await
      }
    };
  }

  async fn execute_raw(&self, request: Request) -> anyhow::Result<reqwest::Response> {
    log::info!("{} {} {:?} {:?}", request.method(), request.url(), request.version(), request.headers());
    Ok(self.client.execute(request).await?.error_for_status()?)
  }
}

impl DefaultHttpClient {
  #[cfg(feature = "default")]
  async fn tc_execute(&self, request: Request) -> anyhow::Result<Response> {
    log::info!("{} {} {:?}", request.method(), request.url(), request.version());
    let response = self.client.execute(request).await?;
    Response::from_response(response, None).await
  }
  async fn wasm_execute(client: ClientWithMiddleware, request: Request) -> anyhow::Result<Response> {
    let response = client.execute(request).await?.error_for_status()?;
    Response::from_response(response, None).await
  }
  #[cfg(feature = "default")]
  async fn tc_execute_grpc(&self, request: Request, operation: &ProtobufOperation) -> anyhow::Result<Response> {
    log::info!("{} {} {:?}", request.method(), request.url(), request.version());
    let response = self.client.execute(request).await?;
    Response::from_response(response, Some(operation.clone())).await
  }
  async fn wasm_execute_grpc(
    client: ClientWithMiddleware,
    request: Request,
    operation: ProtobufOperation,
  ) -> anyhow::Result<Response> {
    let response = client.execute(request).await?.error_for_status()?;
    Response::from_response(response, Some(operation)).await
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
  pub fn new(_upstream: &Upstream) -> Self {
    #[cfg(target_arch = "wasm32")]
    return Self::wasm_client();
    #[cfg(feature = "default")]
    Self::with_options(_upstream, HttpClientOptions::default())
  }
  #[cfg(target_arch = "wasm32")]
  pub fn wasm_client() -> Self {
    let builder = Client::builder();
    let client = ClientBuilder::new(builder.build().expect("Failed to build client"));
    DefaultHttpClient { client: client.build() }
  }
  #[cfg(feature = "default")]
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
