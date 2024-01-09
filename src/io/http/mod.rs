#[cfg(not(feature = "default"))]
pub mod cloudflare;
#[cfg(feature = "default")]
pub mod native;

use crate::config::Upstream;
#[cfg(not(feature = "default"))]
pub use cloudflare::*;
#[cfg(feature = "default")]
pub use native::*;

use crate::http::{HttpClientOptions, Response};

// TODO: there is no method to change the version in reqwest::wasm
#[cfg(feature = "default")]
pub fn set_req_version(req: &mut reqwest::Request) {
  *req.version_mut() = reqwest::Version::HTTP_2;
}

#[async_trait::async_trait]
pub trait HttpIO: Sync + Send {
  async fn execute(&self, request: reqwest::Request) -> anyhow::Result<Response<async_graphql::Value>> {
    self.execute_raw(request).await?.to_json()
  }
  async fn execute_raw(&self, request: reqwest::Request) -> anyhow::Result<Response<Vec<u8>>>;
}

#[cfg(feature = "default")]
pub fn init_http_native(upstream: &Upstream, http_client_options: &HttpClientOptions) -> impl HttpIO + Default + Clone {
  HttpNative::init(upstream, http_client_options)
}

#[cfg(not(feature = "default"))]
pub fn init_http_cloudflare() -> impl HttpIO {
  HttpCloudflare::init()
}
