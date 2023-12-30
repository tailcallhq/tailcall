use crate::http::Response;
#[cfg(feature = "default")]
pub use super::client_cli::*;
#[cfg(not(feature = "default"))]
pub use super::client_wasm::*;

#[async_trait::async_trait]
pub trait HttpClient: Sync + Send {
  async fn execute(&self, req: reqwest::Request) -> anyhow::Result<Response>;
  async fn execute_raw(&self, req: reqwest::Request) -> anyhow::Result<reqwest::Response>;
}
