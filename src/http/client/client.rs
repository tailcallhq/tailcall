#[cfg(feature = "default")]
pub use super::client_cli::*;
#[cfg(not(feature = "default"))]
pub use super::client_wasm::*;
use crate::grpc::protobuf::ProtobufOperation;
use crate::http::Response;

#[async_trait::async_trait]
pub trait HttpClient: Sync + Send {
  async fn execute(&self, req: reqwest::Request, operation: Option<ProtobufOperation>) -> anyhow::Result<Response>;
}
